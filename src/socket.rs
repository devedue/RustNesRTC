use std::io::Read;
use std::net::{Shutdown, TcpListener, TcpStream};
// use std::sync::{Arc, Mutex};

pub struct RemoteServer {
    // buf: Arc<Mutex<u8>>,
    pub stream: Option<TcpStream>,
    port: u16,
}

impl RemoteServer {
    pub fn new() -> Self {
        return RemoteServer {
            // buf: Arc::new(Mutex::new(0)),
            stream: None,
            port: 3333,
        };
    }

    fn _handle_client(mut stream: TcpStream) {
        let mut data = [0 as u8; 1]; // using 50 byte buffer
        while match stream.read_exact(&mut data) {
            Ok(_) => true,
            Err(_) => {
                println!(
                    "An error occurred, terminating connection with {}",
                    stream.peer_addr().unwrap()
                );
                stream.shutdown(Shutdown::Both).unwrap();
                false
            }
        } {}
    }

    pub fn connect_or_start(server: &mut RemoteServer) -> bool {
        if !server.connect() {
            server.start_server();
            return true;
        }
        return false;
    }

    pub fn _set_byte(&mut self, _data: u8) {}

    fn start_server(&mut self) {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", self.port)).unwrap();
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("New connection: {}", stream.peer_addr().unwrap());
                    match stream.set_nonblocking(true) {
                        Ok(_) => {}
                        Err(_) => {}
                    }
                    self.stream = Some(stream);
                    return;
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
        }
    }

    fn connect(&mut self) -> bool {
        match TcpStream::connect(format!("localhost:{}", self.port)) {
            Ok(stream) => {
                println!("Connected");
                match stream.set_nonblocking(true) {
                    Ok(_) => {}
                    Err(_) => {}
                }
                self.stream = Some(stream);
                return true;
            }
            Err(e) => {
                println!("Failed to connect: {}", e);
                return false;
            }
        }
        println!("Terminated.");
        return true;
    }
}
