use crate::rtc::DATA_CHANNEL_RX;
use crate::rtc::DATA_CHANNEL_TX;
use anyhow::Result;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client, Method, Request, Response, Server, StatusCode};
use interceptor::registry::Registry;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::setting_engine::SettingEngine;
use webrtc::api::APIBuilder;
use webrtc::peer::configuration::RTCConfiguration;
use webrtc::peer::ice::ice_candidate::{RTCIceCandidate, RTCIceCandidateInit};
use webrtc::peer::ice::ice_server::RTCIceServer;
use webrtc::peer::peer_connection::RTCPeerConnection;
use webrtc::peer::peer_connection_state::RTCPeerConnectionState;

lazy_static! {
    static ref PEER_CONNECTION_MUTEX: Arc<Mutex<Option<Arc<RTCPeerConnection>>>> =
        Arc::new(Mutex::new(None));
    static ref PENDING_CANDIDATES: Arc<Mutex<Vec<RTCIceCandidate>>> = Arc::new(Mutex::new(vec![]));
    static ref ADDRESS: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    static ref CLIENT: Arc<Mutex<RTCClient>> = Arc::new(Mutex::new(RTCClient::new(
        "0.0.0.0:50000".to_string(),
        "localhost:60000".to_string(),
        "data".to_string()
    )));
}

struct RTCClient {
    answer_address: String,
    offer_address: String,
    channel: String,
}

pub async fn start_client(_address: String) -> Result<()> {
    
    // {
    //     let mut cl = CLIENT.lock().await;
    //     (*cl).answer_address = address;
    // }

    let offer_addr = CLIENT.lock().await.offer_address.clone();
    let answer_addr = CLIENT.lock().await.answer_address.clone();

    {
        let mut oa = ADDRESS.lock().await;
        *oa = answer_addr.clone();
    }

    let config = RTCConfiguration {
        ice_servers: vec![RTCIceServer {
            ..Default::default()
        }],
        ..Default::default()
    };

    let mut m = MediaEngine::default();
    m.register_default_codecs()?;

    let mut registry = Registry::new();

    registry = register_default_interceptors(registry, &mut m)?;

    let mut s = SettingEngine::default();
    s.detach_data_channels();

    let api = APIBuilder::new()
        //TODO: Possible remove media engine
        .with_media_engine(m)
        .with_interceptor_registry(registry)
        .with_setting_engine(s)
        .build();

    let remote_connection = Arc::new(api.new_peer_connection(config).await?);

    let remote_connection2 = Arc::clone(&remote_connection);
    let pending_candidates2 = Arc::clone(&PENDING_CANDIDATES);
    let addr2 = answer_addr.clone();

    remote_connection
        .on_ice_candidate(Box::new(move |c: Option<RTCIceCandidate>| {
            let remote_connection3 = Arc::clone(&remote_connection2);
            let pending_candidates3 = Arc::clone(&pending_candidates2);

            let addr3 = addr2.clone();

            Box::pin(async move {
                if let Some(c) = c {
                    let desc = remote_connection3.remote_description().await;
                    if desc.is_none() {
                        let mut cs = pending_candidates3.lock().await;
                        cs.push(c);
                    } else if let Err(err) = RTCClient::signal_candidate(&addr3, &c).await {
                        assert!(false, "{}", err);
                    }
                }
            })
        }))
        .await;

    println!("Listening on {}", offer_addr);
    {
        let mut pcm = PEER_CONNECTION_MUTEX.lock().await;
        *pcm = Some(Arc::clone(&remote_connection));
    }

    tokio::spawn(async move {
        let addr = SocketAddr::from_str(&offer_addr).unwrap();
        let service = make_service_fn(|_| async {
            Ok::<_, hyper::Error>(service_fn(RTCClient::remote_handler))
        });

        let server = Server::bind(&addr).serve(service);

        if let Err(e) = server.await {
            eprintln!("Client error: {}", e);
        }
    });

    let data_channel = remote_connection
        .create_data_channel(&CLIENT.lock().await.channel.clone(), None)
        .await?;

    remote_connection
        .on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
            println!("Peer Connection State has changed: {}", s);

            if s == RTCPeerConnectionState::Failed {
                // Wait until PeerConnection has had no network activity for 30 seconds or another failure. It may be reconnected using an ICE Restart.
                // Use webrtc.PeerConnectionStateDisconnected if you are interested in detecting faster timeout.
                // Note that the PeerConnection may come back from PeerConnectionStateDisconnected.
                println!("Peer Connection has gone to failed exiting");
                std::process::exit(0);
            }

            Box::pin(async {})
        }))
        .await;

    // Register channel opening handling
    let d1 = Arc::clone(&data_channel);
    data_channel.on_open(Box::new(move || {
        println!("Data channel '{}'-'{}' open. Random messages will now be sent to any connected DataChannels every second", d1.label(), d1.id());

        let d2 = Arc::clone(&d1);
        Box::pin(async move {
            let raw = match d2.detach().await {
                Ok(raw) => raw,
                Err(err) => {
                    println!("data channel detach got err: {}", err);
                    return;
                }
            };

            println!("Trying to set data channel");
            let mut server = DATA_CHANNEL_RX.lock().await;
            *server = Some(Arc::clone(&raw));
            let mut server = DATA_CHANNEL_TX.lock().await;
            *server = Some(Arc::clone(&raw));
            println!("Set data channel");
        })
    })).await;

    // Create an offer to send to the other process
    let offer = remote_connection.create_offer(None).await?;

    // Send our offer to the HTTP server listening in the other process
    let payload = match serde_json::to_string(&offer) {
        Ok(p) => p,
        Err(err) => panic!("{}", err),
    };

    // Sets the LocalDescription, and starts our UDP listeners
    // Note: this will start the gathering of ICE candidates
    remote_connection.set_local_description(offer).await?;

    //println!("Post: {}", format!("http://{}/sdp", answer_addr));
    let req = match Request::builder()
        .method(Method::POST)
        .uri(format!("http://{}/sdp", answer_addr))
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

impl RTCClient {
    pub fn new(offer_address: String, answer_address: String, channel: String) -> Self {
        RTCClient {
            offer_address,
            answer_address,
            channel,
        }
    }

    async fn signal_candidate(addr: &str, c: &RTCIceCandidate) -> Result<()> {
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
    async fn remote_handler(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
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
                    .add_ice_candidate(RTCIceCandidateInit {
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

            // A HTTP handler that processes a RTCSessionDescription given to us from the other WebRTC-rs or Pion process
            (&Method::POST, "/sdp") => {
                println!("remote_handler receive from /sdp");
                let sdp;
                let sdp_str =
                    match std::str::from_utf8(&hyper::body::to_bytes(req.into_body()).await?) {
                        Ok(s) => s.to_owned(),
                        Err(err) => panic!("{}", err),
                    };
                sdp = match serde_json::from_str(&sdp_str) {
                    Ok(s) => s,
                    Err(err) => panic!("{}", err),
                };

                if let Err(err) = pc.set_remote_description(sdp).await {
                    panic!("{}", err);
                }

                {
                    let cs = PENDING_CANDIDATES.lock().await;
                    for c in &*cs {
                        if let Err(err) = RTCClient::signal_candidate(&addr, c).await {
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
