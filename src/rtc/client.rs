use anyhow::Result;
use clap::{App, AppSettings, Arg};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client, Method, Request, Response, Server, StatusCode};
use interceptor::registry::Registry;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::Duration;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::APIBuilder;
use webrtc::data::data_channel::data_channel_message::DataChannelMessage;
use webrtc::data::data_channel::DataChannel;
use webrtc::peer::configuration::Configuration;
use webrtc::peer::ice::ice_candidate::{ICECandidate, ICECandidateInit};
use webrtc::peer::ice::ice_server::ICEServer;
use webrtc::peer::peer_connection::PeerConnection;
use webrtc::peer::peer_connection_state::PeerConnectionState;
use webrtc::peer::sdp::session_description::{SessionDescription, SessionDescriptionSerde};

lazy_static! {
    static ref PEER_CONNECTION_MUTEX: Arc<Mutex<Option<Arc<PeerConnection>>>> =
        Arc::new(Mutex::new(None));
    static ref PENDING_CANDIDATES: Arc<Mutex<Vec<ICECandidate>>> = Arc::new(Mutex::new(vec![]));
    static ref ADDRESS: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    static ref CLIENT: RTCClient = RTCClient::new("127.0.0.1".to_string(), "default".to_string(), 60000);
}

struct RTCClient {
    remote_data: Vec<u8>,
    ip: String,
    channel: String,
    port: u16
}

async fn start_client() -> Result<()> {

    let offer_addr = CLIENT.ip.clone();
    let answer_addr = CLIENT.ip.clone();

    {
        let mut oa = ADDRESS.lock().await;
        *oa = offer_addr.clone();
    }

    let config = Configuration {
        ice_servers: vec![ICEServer {
            ..Default::default()
        }],
        ..Default::default()
    };

    let mut registry = Registry::new();

    let api = APIBuilder::new().with_interceptor_registry(registry).build();

    let remote_connection  = Arc::new(api.new_peer_connection(config).await?);

    let remote_connection2 = Arc::clone(&remote_connection);
    let pending_candidates2 = Arc::clone(&PENDING_CANDIDATES);
    let addr2 = offer_addr.clone();

    remote_connection.on_ice_candidate(Box::new(move | c: Option<ICECandidate>| {
        println!("on_ice_candidate {:?}",c);

        let remote_connection3 = Arc::clone(&remote_connection2);
        let pending_candidates3 = Arc::clone(&pending_candidates2);

        let addr3 = addr2.clone();

        Box::pin(async move {
            if let Some(c) = c {
                let desc = remote_connection3.remote_description().await;
                if desc.is_none() {
                    let mut cs = pending_candidates3.lock().await;
                    cs.push(c);
                } else if let Err(err) = CLIENT.signal_candidate(&addr3, &c).await {
                    assert!(false, "{}", err);
                }
            }
        })
    })).await;

    println!("Listening on {}", CLIENT.ip);
    {
        let mut pcm = PEER_CONNECTION_MUTEX.lock().await;
        *pcm = Some(Arc::clone(&remote_connection));
    }

    tokio::spawn(async move {
        let addr = SocketAddr::from_str(&answer_addr).unwrap();
        let service = make_service_fn(|_| async {Ok::<_, hyper::Error>(service_fn(|req: Request<Body>| async move {
            CLIENT.remote_handler(req).await
        }))});

        let server = Server::bind(&addr).serve(service);

        if let Err(e) = server.await {
            eprintln!("Server error: {}", e);
        }
    });

    remote_connection.on_data_channel(Box::new(move |d:Arc<DataChannel>| {
        let d_label = d.label().to_owned();
        let d_id = d.id();
        print!("New DataChannel {} {}\n", d_label, d_id);

        Box::pin(async move{
            // Register channel opening handling
            let d2 =  Arc::clone(&d);
            let d_label2 = d_label.clone();
            let d_id2 = d_id.clone();
            d.on_open(Box::new(move || {
                print!("Data channel '{}'-'{}' open. Random messages will now be sent to any connected DataChannels every 5 seconds\n", d_label2, d_id2);
                Box::pin(async move {
                    let mut result = Result::<usize>::Ok(0);
                    let mut i = 0;
                    while result.is_ok() {
                        let timeout = tokio::time::sleep(Duration::from_secs(5));
                        tokio::pin!(timeout);

                        tokio::select! {
                            _ = timeout.as_mut() =>{
                                let message = format!("Sending '{}'", i);
                                println!("on_data_channel - on_open: {}", message);
                                i += 1;
                                result = d2.send_text(message).await;
                            }
                        };
                    }
                })
            })).await;

            println!("after on_data_channel - on_open");

            // Register text message handling
            d.on_message(Box::new(move |msg: DataChannelMessage| {
               let msg_str = String::from_utf8(msg.data.to_vec()).unwrap();
               print!("Message from DataChannel '{}': '{}'\n", d_label, msg_str);
               Box::pin(async{})
           })).await;
        })
    })).await;


    Ok(())
}

impl RTCClient {
    pub fn new(ip: String, channel: String, port: u16) -> Self {
        RTCClient {
            remote_data: Vec::new(),
            ip,
            channel,
            port
        }
    }

    async fn signal_candidate(&self, addr: &str, c: &ICECandidate) -> Result<()> {
        println!(
            "signal_candidate Post candidate to {}",
            format!("http://{}/candidate", addr)
        );
        let payload = c.to_json().await?.candidate;
        let req = match Request::builder()
            .method(Method::POST)
            .uri(format!("http://{}/candidate", addr))
            .header("content-type", "application/json; charset=utf-8")
            .body(Body::from(payload))
        {
            Ok(req) => req,
            Err(err) => {
                println!("{}", err);
                return Err(err.into());
            }
        };
    
        let resp = match Client::new().request(req).await {
            Ok(resp) => resp,
            Err(err) => {
                println!("{}", err);
                return Err(err.into());
            }
        };
        println!("signal_candidate Response: {}", resp.status());
    
        Ok(())
    }

    // HTTP Listener to get ICE Credentials/Candidate from remote Peer
async fn remote_handler(&self, req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let pc = {
        let pcm = PEER_CONNECTION_MUTEX.lock().await;
        pcm.clone().unwrap()
    };
    let addr = {
        let addr = ADDRESS.lock().await;
        addr.clone()
    };

    match (req.method(), req.uri().path()) {
        // A HTTP handler that allows the other WebRTC-rs or Pion instance to send us ICE candidates
        // This allows us to add ICE candidates faster, we don't have to wait for STUN or TURN
        // candidates which may be slower
        (&Method::POST, "/candidate") => {
            println!("remote_handler receive from /candidate");
            let candidate =
                match std::str::from_utf8(&hyper::body::to_bytes(req.into_body()).await?) {
                    Ok(s) => s.to_owned(),
                    Err(err) => panic!("{}", err),
                };

            if let Err(err) = pc
                .add_ice_candidate(ICECandidateInit {
                    candidate,
                    ..Default::default()
                })
                .await
            {
                panic!("{}", err);
            }

            let mut response = Response::new(Body::empty());
            *response.status_mut() = StatusCode::OK;
            Ok(response)
        }

        // A HTTP handler that processes a SessionDescription given to us from the other WebRTC-rs or Pion process
        (&Method::POST, "/sdp") => {
            println!("remote_handler receive from /sdp");
            let mut sdp = SessionDescription::default();
            let sdp_str = match std::str::from_utf8(&hyper::body::to_bytes(req.into_body()).await?)
            {
                Ok(s) => s.to_owned(),
                Err(err) => panic!("{}", err),
            };
            sdp.serde = match serde_json::from_str::<SessionDescriptionSerde>(&sdp_str) {
                Ok(s) => s,
                Err(err) => panic!("{}", err),
            };

            if let Err(err) = pc.set_remote_description(sdp).await {
                panic!("{}", err);
            }

            // Create an answer to send to the other process
            let answer = match pc.create_answer(None).await {
                Ok(a) => a,
                Err(err) => panic!("{}", err),
            };

            println!(
                "remote_handler Post answer to {}",
                format!("http://{}/sdp", addr)
            );

            // Send our answer to the HTTP server listening in the other process
            let payload = match serde_json::to_string(&answer.serde) {
                Ok(p) => p,
                Err(err) => panic!("{}", err),
            };

            let req = match Request::builder()
                .method(Method::POST)
                .uri(format!("http://{}/sdp", addr))
                .header("content-type", "application/json; charset=utf-8")
                .body(Body::from(payload))
            {
                Ok(req) => req,
                Err(err) => panic!("{}", err),
            };

            let resp = match Client::new().request(req).await {
                Ok(resp) => resp,
                Err(err) => {
                    println!("{}", err);
                    return Err(err.into());
                }
            };
            println!("remote_handler Response: {}", resp.status());

            // Sets the LocalDescription, and starts our UDP listeners
            if let Err(err) = pc.set_local_description(answer).await {
                panic!("{}", err);
            }

            {
                let cs = PENDING_CANDIDATES.lock().await;
                for c in &*cs {
                    if let Err(err) = self.signal_candidate(&addr, c).await {
                        panic!("{}", err);
                    }
                }
            }

            let mut response = Response::new(Body::empty());
            *response.status_mut() = StatusCode::OK;
            Ok(response)
        }
        // Return the 404 Not Found for other routes.
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}
}
