mod http    ;
use std::net::TcpListener;
use http::http1::handle_client;

fn main() {

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("accepted new connection");
                std::thread::spawn(|| handle_client(stream));
                
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }

    
}

