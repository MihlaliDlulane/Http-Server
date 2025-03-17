#[allow(unused_imports)]
use std::net::{TcpListener,TcpStream};
use std::io::{Write,BufReader,BufRead};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
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

fn handle_client(mut stream:TcpStream) {

    let mut response = String::from("HTTP/1.1 200 OK\r\n\r\n");
    let reader = BufReader::new(&stream);
    
    let http_request: Vec<_> = reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty()) // Stop at the empty line (end of headers)
        .collect();

    println!("Entire requst:{:?}",http_request);

    let Some(line) = http_request.get(0) else {panic!("Bad request!")};
    println!("First line of request: {:?}", line);

    let url:Vec<_> = line.split(" ").collect();
    let index:Vec<_> = url[1].split("/").collect();

    println!("index:{:?}",index);

    match index[1]  {
        "" => {}
        "echo" => {
            let Some(echo) = index.get(2) else {panic!("Need content")};
            let echo_len = echo.len();
            let message = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",echo_len,echo);
            response = message;
        }
        "user-agent" => {
            let user_agent= http_request.iter().find(|item| item.starts_with("User"));
            println!("user agent:{:?}",user_agent.unwrap());
            let user_agent_vec: Vec<_> = user_agent.unwrap().split(":").collect();
            let message = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",user_agent_vec[1].len(),user_agent_vec[1]);
            response = message;
        }
        _ => {response = "HTTP/1.1 400 NOT FOUND\r\n\r\n".to_string();}
    }
    
    stream.write_all(response.as_bytes()).unwrap();    

}
