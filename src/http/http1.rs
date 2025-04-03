use std::net::TcpStream;
use std::io::{Write,BufReader,BufRead,Read};
use std::{env,fs};
use std::fs::File;
use std::collections::HashMap;
use std::time::Duration;
use log::info;
use std::error::Error;
use flate2::{write::GzEncoder,Compression};

#[derive(Debug)]
struct HttpRequest {
    method:String,
    path:String,
    headers: HashMap<String,String>,
    body:Vec<u8>,
}

#[derive(Debug)]
struct HttpResponse {
    status_code: u16,
    status_text: String,
    headers:HashMap<String,String>,
    body:Vec<u8>
}

impl HttpResponse {
    fn new(status_code:u16) -> Self {
        let status_text = match status_code {
            200 => "OK",
            400 => "Bad request",
            404 => "Not found",
            500 => "Internal Server Error",
            _ => "Unknown",
        }.to_string();

        HttpResponse{
            status_code,
            status_text,
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }

    fn with_body(mut self,body: Vec<u8>) -> Self{
        self.headers.insert("Content-Length".to_string(), body.len().to_string());
        self.body = body;
        self
    }

    fn with_header(mut self,key:&str,value:&str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut respose = Vec::new();

        //Status line
        respose.extend(format!("HTTP/1.1 {} {}\r\n",self.status_code,self.status_text).as_bytes());

        // headers
        for (key,value) in &self.headers  {
            respose.extend(format!("{}: {}\r\n",key,value).as_bytes());
        }

        // Emtpy line seperating headers from body
        respose.extend(b"r\n");

        //Body
        respose.extend(&self.body);

        respose

    }

}


pub async fn handle_client(mut stream:TcpStream) -> Result<(), Box<dyn Error>>{

    let peer_addr = stream.peer_addr()?;
    info!("New connection from {}",peer_addr);

    stream.set_read_timeout(Some(Duration::from_secs(30)))?;
    stream.set_write_timeout(Some(Duration::from_secs(30)))?;

    let mut reader = BufReader::new(&stream);
    let request = parse_request(&mut reader).await?;

    let response = match request.method.as_str() {
        "GET" => handle_get(request),
        "POST" => handle_post(request),
        _ => HttpResponse::new(400)
                .with_body(b"Invalid Request".to_vec())
                .with_header("Content-Type","text/plain"),
    };

    send_response(&mut stream,response).await?;

    info!("Successfully handled request from {}",peer_addr);
    Ok(()) 

}


async fn parse_request(reader: &mut BufReader<TcpStream>) -> Result<HttpRequest, Box<dyn Error>> {

    // Read headers
    let mut headers_vec = Vec::new();
    let mut content_length = 0;
    loop {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        if line == "\r\n" || line == "\n" {
            break;
        }
        if line.starts_with("Content-Length: ") {
            content_length = line.trim_start_matches("Content-Length: ")
                               .trim_end_matches(&['\r', '\n'])
                               .parse::<usize>()?;
        }
        headers_vec.push(line.trim_end_matches(&['\r', '\n']).to_string());
    }

    // Parse first line
    let first_line = headers_vec.get(0).ok_or("Empty request")?;
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() != 3 {
        return Err("Invalid request line".into());
    }

    // Parse headers
    let headers = headers_vec[1..]
        .iter()
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(2, ": ").collect();
            if parts.len() == 2 {
                Some((parts[0].to_string(), parts[1].to_string()))
            } else {
                None
            }
        })
        .collect();

    // Read body if present
    let mut body = vec![0; content_length];
    if content_length > 0 {
        reader.read_exact(&mut body)?;
    }

    Ok(HttpRequest {
        method: parts[0].to_string(),
        path: parts[1].to_string(),
        headers,
        body,

    })

}


async fn send_response(stream:&mut TcpStream,response:HttpResponse) -> Result<(), Box<dyn Error>>{
    let response_bytes = response.to_bytes();
    stream.write_all(&response_bytes)?;
    stream.flush()?;
    Ok(())
}



fn handle_get(http_request:Vec<String>) -> String {
    let response:String;

    let Some(line) = http_request.get(0) else {panic!("Bad request!")};
    let url:Vec<_> = line.split(" ").collect();
    let index:Vec<_> = url[1].split("/").collect();

    match index[1]  {
        "" => {
            response = "HTTP/1.1 200 OK\r\n\r\n".to_string();
        }
        "echo" => {
            let encoding = http_request.iter().find(|encode| encode.starts_with("Accept-Encoding"));
            let Some(echo) = index.get(2) else {panic!("Need content")};
            response = handle_echo(encoding, echo);
            
        }
        "user-agent" => {
            let user_agent= http_request.iter().find(|item| item.starts_with("User"));
            // println!("user agent:{:?}",user_agent.unwrap());
            let user_agent_vec: Vec<_> = user_agent.unwrap().split(":").collect();
            let message = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",user_agent_vec[1].len(),user_agent_vec[1]);
            response = message;
        }
        "files" => {
            let Some(file_name) = index.get(2) else {panic!("Need file name!")};
            let env_args: Vec<String> = env::args().collect();
            let mut dir = env_args[2].clone();
            dir.push_str(&file_name);
            //println!("Dir: {:?}",dir);

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

fn handle_post(http_request:Vec<String>,body:&[u8]) -> String {
    
    let Some(line) = http_request.get(0) else {panic!("Bad request!")};
    let url:Vec<_> = line.split(" ").collect();
    let index:Vec<_> = url[1].split("/").collect();

    let Some(file_name) = index.get(2) else {panic!("Need file name!")};
    let env_args: Vec<String> = env::args().collect();
    let mut dir = env_args[2].clone();
    dir.push_str(&file_name);

    let mut file = File::create(dir).expect("Failed to create file");
    file.write_all(body).expect("Failed to write to file");

    return "HTTP/1.1 201 Created\r\n\r\n".to_string();
}

fn handle_echo(encoding:Option<&String>,echo:&&str) -> String {

    let message:String;

    match encoding {
        None => {
            let echo_len = echo.len();
            message = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",echo_len,echo);
        }

        Some(code) => {
            let encode_type:Vec<_> = code.split(":").collect();
            let encode_type:Vec<_> = encode_type[1].split(",").map(|s| s.trim()).collect();
            let encode_type = encode_type.iter().find(|&&item| item.eq_ignore_ascii_case("gzip"));
            match encode_type {
                Some(_result) => {
                    let echo_len = echo.len();
                    let mut comp_body = Vec::new();
                    GzEncoder::new(&mut comp_body,Compression::default()).write_all(echo.as_bytes()).unwrap();
                    message = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Encoding: gzip\r\nContent-Length: {}\r\n\r\n{:?}",echo_len,comp_body);
                }
                _ => {
                    message = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\n{}","...");
                }
            }
        }
    }

    
    return message
}
