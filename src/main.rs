#[allow(unused_imports)]
use std::net::{TcpListener,TcpStream};
use std::io::{Write,BufReader,BufRead};
use std::{env,fs};

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

fn handle_client(mut stream:TcpStream) {

    let mut response = String::from("HTTP/1.1 200 OK\r\n\r\n");
    let reader = BufReader::new(&stream);
    
    let http_request: Vec<_> = reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty()) // Stop at the empty line (end of headers)
        .collect();

    let h_request = http_request.clone();

    println!("Entire requst:{:?}",http_request);

    let Some(line) = http_request.get(0) else {panic!("Bad request!")};
    println!("First line of request: {:?}", line);

    let url:Vec<_> = line.split(" ").collect();
    let index:Vec<_> = url[1].split("/").collect();

    println!("Url:{:?} and first: {:?}",url,url[0]);
    println!("index:{:?}",index);

    match url[0] {
        "GET" => {
            response = handle_get(h_request);
        }

        "POST" => {
            response = handle_post(h_request);
        }

        _  => {
            response = "HTTP/1.1 400 INVALID REQUEST\r\n\r\n".to_string();
        }
    }
 
    stream.write_all(response.as_bytes()).unwrap();    

}

fn handle_get(http_request:Vec<String>) -> String {
    let mut response:String = String::from("");

    let Some(line) = http_request.get(0) else {panic!("Bad request!")};
    let url:Vec<_> = line.split(" ").collect();
    let index:Vec<_> = url[1].split("/").collect();

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
        "files" => {
            let Some(file_name) = index.get(2) else {panic!("Need file name!")};
            let env_args: Vec<String> = env::args().collect();
            let mut dir = env_args[2].clone();
            dir.push_str(&file_name);
            println!("Dir: {:?}",dir);

            let file = fs::read(dir);

            match file {
                Ok(content) => {
                    let message = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",content.len(),String::from_utf8(content).expect("file content"));
                    response = message;
                }
                Err(_e) => {
                    response = "HTTP/1.1 400 FILE NOT FOUND\r\n\r\n".to_string();
                }
            }
        }
        _ => {response = "HTTP/1.1 400 NOT FOUND\r\n\r\n".to_string();}
    }

    return response;
}

fn handle_post(http_request:Vec<String>) -> String {
    todo!("Code");
}