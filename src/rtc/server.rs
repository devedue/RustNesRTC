use crate::rtc::DATA_CHANNEL_RX;
use crate::rtc::DATA_CHANNEL_TX;
use anyhow::Result;
use hyper::service::{make_service_fn, service_fn};
use hyper::Client;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use interceptor::registry::Registry;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::setting_engine::SettingEngine;
use webrtc::api::APIBuilder;
use webrtc::data::data_channel::RTCDataChannel;
use webrtc::peer::configuration::RTCConfiguration;
use webrtc::peer::ice::ice_candidate::{RTCIceCandidate, RTCIceCandidateInit};
use webrtc::peer::ice::ice_server::RTCIceServer;
use webrtc::peer::peer_connection::RTCPeerConnection;
use webrtc::peer::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer::sdp::session_description::RTCSessionDescription;

lazy_static! {
    static ref PEER_CONNECTION_MUTEX: Arc<Mutex<Option<Arc<RTCPeerConnection>>>> =
        Arc::new(Mutex::new(None));
    static ref PENDING_CANDIDATES: Arc<Mutex<Vec<RTCIceCandidate>>> = Arc::new(Mutex::new(vec![]));
    static ref ADDRESS: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    pub static ref SERVER: Arc<Mutex<RTCServer>> = Arc::new(Mutex::new(RTCServer::new(
        "localhost".to_owned(),
        "0.0.0.0".to_owned(),
        50000,
        60000
    )));
}

pub async fn start_server(address: String) -> Result<()> {
    {
        let mut cl = SERVER.lock().await;
        (*cl).offer_address = address;
    }

    let se = SERVER.lock().await;

    let offer_addr = format!("{}:{}", se.offer_address, se.offer_port);
    let answer_addr = format!("{}:{}", se.answer_address, se.answer_port);
    
    drop(se);

    {
        let mut address_ref = ADDRESS.lock().await;
        *address_ref = offer_addr.clone();
    }

    let config = RTCConfiguration {
        ice_servers: vec![RTCIceServer {
            //TODO: Possible remove stun for local
            urls: vec!["stun:stun.1.google.com:19302".to_owned()],
            ..Default::default()
        }],
        ..Default::default()
    };
    //TODO: Possibly remove media engine
    let mut m = MediaEngine::default();
    m.register_default_codecs()?;

    let mut registry = Registry::new();
    registry = register_default_interceptors(registry, &mut m)?;

    let mut s = SettingEngine::default();
    s.detach_data_channels();

    let api = APIBuilder::new()
        .with_media_engine(m)
        .with_interceptor_registry(registry)
        .with_setting_engine(s)
        .build();

    let peer_connection = Arc::new(api.new_peer_connection(config).await?);

    let peer_connection2 = Arc::clone(&peer_connection);
    let pending_candidates2 = Arc::clone(&PENDING_CANDIDATES);
    let addr2 = offer_addr.clone();

    peer_connection
        .on_ice_candidate(Box::new(move |c: Option<RTCIceCandidate>| {
            let peer_connection3 = Arc::clone(&peer_connection2);
            let pending_candidates3 = Arc::clone(&pending_candidates2);
            let addr3 = addr2.clone();
            Box::pin(async move {
                if let Some(c) = c {
                    let desc = peer_connection3.remote_description().await;
                    if desc.is_none() {
                        let mut cs = pending_candidates3.lock().await;
                        cs.push(c);
                    } else if let Err(err) = RTCServer::signal_candidate(&addr3, &c).await {
                        assert!(false, "{}", err);
                    }
                }
            })
        }))
        .await;

    println!("Listening on {}", answer_addr);
    println!("Connect to {}", offer_addr);
    {
        let mut pcm = PEER_CONNECTION_MUTEX.lock().await;
        *pcm = Some(Arc::clone(&peer_connection));
    }

    tokio::spawn(async move {
        let addr = SocketAddr::from_str(&answer_addr).unwrap();
        
        let service = make_service_fn(|_| async {
            Ok::<_, hyper::Error>(service_fn(RTCServer::remote_handler))
        });
        let server = Server::bind(&addr).serve(service);
        // Run this server for... forever!
        if let Err(e) = server.await {
            eprintln!("server error: {}", e);
        }
    });

    peer_connection
        .on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
            print!("Peer Connection State has changed: {}\n", s);

            if s == RTCPeerConnectionState::Failed {
                println!("Peer Connection has gone to failed exiting");
                std::process::exit(0);
            }

            Box::pin(async {})
        }))
        .await;

    peer_connection
        .on_data_channel(Box::new(move |d: Arc<RTCDataChannel>| {
            let d_label = d.label().to_owned();
            let d_id = d.id();
            println!("New DataChannel {} {}", d_label, d_id);

            Box::pin(async move {
                // Register channel opening handling
                let d2 = Arc::clone(&d);
                let d_label2 = d_label.clone();
                let d_id2 = d_id;
                d.on_open(Box::new(move || {
                    println!("Data channel '{}'-'{}' open. ", d_label2, d_id2);
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
                }))
                .await;
            })
        }))
        .await;

    Ok(())
}

#[derive(Default)]
pub struct RTCServer {
    offer_address: String,
    answer_address: String,
    offer_port: u16,
    answer_port: u16
}

impl RTCServer {
    pub fn new(offer_address: String, answer_address: String, offer_port: u16, answer_port: u16) -> Self {
        RTCServer {
            offer_address,
            answer_address,
            offer_port,
            answer_port
        }
    }

    async fn signal_candidate(addr: &String, c: &RTCIceCandidate) -> Result<()> {
        println!(
            "Signalling candidate on {}",
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
            // A HTTP handler that processes a SessionDescription given to us from the other WebRTC-rs or Pion process
            (&Method::POST, "/sdp") => {
                println!("remote_handler receive from /sdp");
                let sdp_str =
                    match std::str::from_utf8(&hyper::body::to_bytes(req.into_body()).await?) {
                        Ok(s) => s.to_owned(),
                        Err(err) => panic!("{}", err),
                    };
                let sdp = match serde_json::from_str::<RTCSessionDescription>(&sdp_str) {
                    Ok(s) => s,
                    Err(err) => panic!("{}", err),
                };
                if let Err(err) = pc.set_remote_description(sdp).await {
                    panic!("{}", err);
                }

                let answer = match pc.create_answer(None).await {
                    Ok(a) => a,
                    Err(err) => panic!("{}", err),
                };

                println!(
                    "remote_handler Post answer to {}",
                    format!("http://{}/sdp", addr)
                );

                let payload = match serde_json::to_string(&answer) {
                    Ok(p) => p,
                    Err(err) => panic!("{}", err),
                };

                println!("Created Payload {}", payload);

                let req = match Request::builder()
                    .method(Method::POST)
                    .uri(format!("http://{}/sdp", addr))
                    .header("content-type", "application/json; charset=utf-8")
                    .body(Body::from(payload))
                {
                    Ok(req) => req,
                    Err(err) => panic!("{}", err),
                };

                println!("Awaiting response from {}", addr);

                let _resp = match Client::new().request(req).await {
                    Ok(resp) => resp,
                    Err(err) => {
                        println!("{}", err);
                        return Err(err);
                    }
                };

                println!("Response Received");

                if let Err(err) = pc.set_local_description(answer).await {
                    panic!("{}", err);
                }

                {
                    let cs = PENDING_CANDIDATES.lock().await;
                    for c in &*cs {
                        if let Err(err) = RTCServer::signal_candidate(&addr, c).await {
                            panic!("{}", err);
                        }
                    }
                }

                println!("Local Description set");
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
