use anyhow::Result;
use hyper::service::{make_service_fn, service_fn};
use hyper::Client;
use hyper::{Body, Method, Request, Response, Server, StatusCode, Version};
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::Duration;
use webrtc::api::APIBuilder;
use webrtc::data::data_channel::data_channel_message::DataChannelMessage;
use webrtc::peer::configuration::Configuration;
use webrtc::peer::ice::ice_candidate::{ICECandidate, ICECandidateInit};
use webrtc::peer::ice::ice_server::ICEServer;
use webrtc::peer::peer_connection::PeerConnection;
use webrtc::peer::peer_connection_state::PeerConnectionState;
use webrtc::peer::sdp::session_description::SessionDescription;
use webrtc::peer::sdp::session_description::SessionDescriptionSerde;

lazy_static! {
    static ref PEER_CONNECTION_MUTEX: Arc<Mutex<Option<Arc<PeerConnection>>>> =
        Arc::new(Mutex::new(None));
    static ref PENDING_CANDIDATES: Arc<Mutex<Vec<ICECandidate>>> = Arc::new(Mutex::new(vec![]));
    static ref ADDRESS: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    static ref SER: RTCServer =
        RTCServer::new("127.0.0.1".to_string(), "default".to_string(), 60000);
}

async fn start_server() -> Result<()> {
    let ip2 = SER.ip.clone();

    let config = Configuration {
        ice_servers: vec![ICEServer {
            ..Default::default()
        }],
        ..Default::default()
    };

    let api = APIBuilder::new().build();

    let peer_connection = Arc::new(api.new_peer_connection(config).await?);

    let peer_connection2 = Arc::clone(&peer_connection);
    let pending_candidates2 = Arc::clone(&PENDING_CANDIDATES);
    let addr2 = SER.ip.clone();

    peer_connection
        .on_ice_candidate(Box::new(move |c: Option<ICECandidate>| {
            let peer_connection3 = Arc::clone(&peer_connection2);
            let pending_candidates3 = Arc::clone(&pending_candidates2);
            let addr3 = addr2.clone();
            Box::pin(async move {
                if let Some(c) = c {
                    let desc = peer_connection3.remote_description().await;
                    if desc.is_none() {
                        let mut cs = pending_candidates3.lock().await;
                        cs.push(c);
                    } else if let Err(err) = (&SER).signal_candidate(&addr3, &c).await {
                        assert!(false, "{}", err);
                    }
                }
            })
        }))
        .await;

    tokio::spawn(async move {
        let addr = SocketAddr::from_str(&SER.ip).unwrap();
        let service = make_service_fn(|_| async {
            Ok::<_, hyper::Error>(service_fn(|req: Request<Body>| async move {
                SER.remote_handler(req).await
            }))
        });
        let server = Server::bind(&addr).serve(service);
        // Run this server for... forever!
        if let Err(e) = server.await {
            eprintln!("server error: {}", e);
        }
    });

    // Create a datachannel with label 'data'
    let data_channel = peer_connection
        .create_data_channel(&SER.channel, None)
        .await?;

    peer_connection
        .on_peer_connection_state_change(Box::new(move |s: PeerConnectionState| {
            print!("Peer Connection State has changed: {}\n", s);

            if s == PeerConnectionState::Failed {
                println!("Peer Connection has gone to failed exiting");
                std::process::exit(0);
            }

            Box::pin(async {})
        }))
        .await;

    let d1 = Arc::clone(&data_channel);

    data_channel.on_open(Box::new(move || {
        print!("Data channel '{}'-'{}' open. Random messages will now be sent to any connected DataChannels every 5 seconds\n", d1.label(), d1.id());
        let d2 = Arc::clone(&d1);
        Box::pin(async move {
            let mut result = Result::<usize>::Ok(0);
            while result.is_ok() {
                let timeout = tokio::time::sleep(Duration::from_millis(1));
                tokio::pin!(timeout);

                tokio::select! {
                    _ = timeout.as_mut() =>{
                        result = d2.send_text(SER.description.to_string()).await;
                    }
                };
            }
        })
    })).await;

    // Register text message handling
    let d1 = Arc::clone(&data_channel);
    data_channel
        .on_message(Box::new(move |msg: DataChannelMessage| {
            let msg_str = String::from_utf8(msg.data.to_vec()).unwrap();
            print!("Message from DataChannel '{}': '{}'\n", d1.label(), msg_str);
            SER.on_message(d1.label());
            Box::pin(async {})
        }))
        .await;

    // Create an offer to send to the other process
    let offer = peer_connection.create_offer(None).await?;

    // Send our offer to the HTTP server listening in the other process
    let payload = match serde_json::to_string(&offer.serde) {
        Ok(p) => p,
        Err(err) => panic!("{}", err),
    };
    peer_connection.set_local_description(offer).await?;

    let req = match Request::builder()
        .method(Method::POST)
        .uri(format!("http://{}/sdp", &ip2))
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
    println!("Response: {}", resp.status());

    Ok(())
}

pub struct RTCServer {
    channel: String,
    ip: String,
    port: u16,
    pub players: [u8; 2],
    description: String,
}

impl RTCServer {
    pub fn new(ip: String, channel: String, port: u16) -> Self {
        RTCServer {
            channel: "default".to_string(),
            ip: ip.clone(),
            port: port,
            players: [0; 2],
            description: "".to_string(),
        }
    }

    fn set_description(&mut self, description: String) {
        self.description = description;
    }

    async fn signal_candidate(&self, addr: &String, c: &ICECandidate) -> Result<()> {
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
                let sdp_str =
                    match std::str::from_utf8(&hyper::body::to_bytes(req.into_body()).await?) {
                        Ok(s) => s.to_owned(),
                        Err(err) => panic!("{}", err),
                    };
                // sdp.serde = match serde_json::from_str::<SessionDescriptionSerde>(&sdp_str) {
                //     Ok(s) => s,
                //     Err(err) => panic!("{}", err),
                // };
                if let Err(err) = pc.set_remote_description(sdp).await {
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

    pub fn open() {}

    fn on_message(&self, msg: &str) {
        println!("{}", msg);
    }
}
