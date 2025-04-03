use std::fs;
use tokio::net::TcpStream;
use std::path::Path;
use tokio::io::{BufReader, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};
use std::error::Error;
use std::collections::HashMap;
use async_compression::tokio::write::GzipEncoder;
use async_trait::async_trait;

#[derive(Debug)]

struct HttpRequest {
    method: String,
    path: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}


#[derive(Debug)]

struct HttpResponse {
    status_code: u16,
    status_text: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl HttpResponse {

    fn new(status_code: u16) -> Self {
        let status_text = match status_code {
            200 => "OK",
            400 => "Bad Request",
            404 => "Not Found",
            500 => "Internal Server Error",
            _ => "Unknown",
        }.to_string();

        HttpResponse {
            status_code,
            status_text,
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }


    fn with_body(mut self, body: Vec<u8>) -> Self {
        self.headers.insert("Content-Length".to_string(), body.len().to_string());
        self.body = body;
        self
    }

    fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut response = Vec::new();
        // Status line
        response.extend(format!("HTTP/1.1 {} {}\r\n", self.status_code, self.status_text).as_bytes());

        // Headers
        for (key, value) in &self.headers {
            response.extend(format!("{}: {}\r\n", key, value).as_bytes());
        }

        // Empty line separating headers from body
        response.extend(b"\r\n");

        // Body
        response.extend(&self.body);
        response
    }
}


// Define error types
#[derive(Debug)]
enum HandleError{
    FileNotFound(String),
    InvalidRequest(String),
    IoError(std::io::Error),
    EncodingError(String),
}

impl From<std::io::Error> for HandleError {
    fn from(error: std::io::Error) -> Self {
        HandleError::IoError(error)
    }
}

// Handler trait
#[async_trait]
trait RequestHandler: Send + Sync {
    async fn handle(&self,request: &HttpRequest) -> Result<HttpResponse,HandleError>;
}

// handler implementations
struct EchoHandler;
struct UserAgentHandler;
struct FileHandler{
    base_dir:String,
}
struct RootHandler;

#[async_trait]
impl RequestHandler for EchoHandler {
    async fn handle(&self,request: &HttpRequest) -> Result<HttpResponse,HandleError> {
        let path_segments:Vec<&str> = request.path.split("/").collect();
        let echo_content = path_segments.get(2)
                                            .ok_or_else(|| HandleError::InvalidRequest("No echo content provided".to_string()))?;

        // Check for gzip encoding
        let accepts_gzip = request.headers.get("Accept-Encoding")
                            .map(|encodings| encodings.to_lowercase().contains("gzip"))
                            .unwrap_or(false);
        if accepts_gzip {
            let mut compressed = Vec::new();
            let mut encoder = GzipEncoder::new(Vec::new());
            encoder.write_all(echo_content.as_bytes()).await?;
            encoder.shutdown().await?;
            compressed = encoder.into_inner();

            Ok(HttpResponse::new(200)
                .with_body(compressed)
                .with_header("Content-Type","text/plain")
                .with_header("Cotnent-Encoding","gzip"))
        } else {
            Ok(HttpResponse::new(200)
                .with_body(echo_content.as_bytes().to_vec())
                .with_header("Content-Type","text/plain"))
        }
    }
}

#[async_trait]
impl  RequestHandler for UserAgentHandler {
    async fn handle(&self, request: &HttpRequest) -> Result<HttpResponse,HandleError> {
        let user_agent = request.headers.get("User-Agent")
            .ok_or_else(|| HandleError::InvalidRequest("No User-Agent header".to_string()))?;

        Ok(HttpResponse::new(200)
            .with_body(user_agent.as_bytes().to_vec())
            .with_header("Content-Type","text/plain"))
    }
}

#[async_trait]
impl RequestHandler for FileHandler {
    async fn handle(&self,request: &HttpRequest) -> Result<HttpResponse,HandleError> {
        let path_sefments: Vec<&str> = request.path.split("/").collect();
        let file_name = path_sefments.get(2)
                .ok_or_else(|| HandleError::InvalidRequest("No filename provided".to_string()))?;

        let file_path = Path::new(&self.base_dir).join(file_name);

        match request.method.as_str() {
            "GET" => {
                let content = fs::read(&file_path)
                .map_err(|_| HandleError::FileNotFound(file_name.to_string()))?;

            Ok(HttpResponse::new(200)
                .with_body(content)
                .with_header("Content-Type", "application/octet-stream"))
            },
            "POST" => {
                fs::write(&file_path, &request.body)?;

                Ok(HttpResponse::new(201)
                    .with_header("Content-Type", "text/plain")
                    .with_body(b"File created successfully".to_vec()))
            },
            _ => {Err(HandleError::InvalidRequest("Method not allowed".to_string()))}
        }
    }
}

#[async_trait]
impl RequestHandler for RootHandler {
    async fn handle(&self,_request: &HttpRequest) -> Result<HttpResponse,HandleError> {
        Ok(HttpResponse::new(200)
            .with_body(b"Welcome to the server!".to_vec())
            .with_header("Content-Type", "text/plain"))
    }
}

// Main handler function
pub async fn handle_request(request: HttpRequest) -> HttpResponse {

    let handler: Box<dyn RequestHandler> = match request.path.split('/').nth(1) {
        Some("echo") => Box::new(EchoHandler),
        Some("user-agent") => Box::new(UserAgentHandler),
        Some("files") => Box::new(FileHandler {
            base_dir: std::env::args().nth(2)
                .unwrap_or_else(|| "./".to_string())
        }),
        Some("") | None => Box::new(RootHandler),

        _ => return HttpResponse::new(404)
            .with_body(b"Not Found".to_vec())
            .with_header("Content-Type", "text/plain")
    };


    match handler.handle(&request).await {
        Ok(response) => response,
        Err(error) => match error {
            HandleError::FileNotFound(_) => HttpResponse::new(404)
                .with_body(b"File not found".to_vec())
                .with_header("Content-Type", "text/plain"),

            HandleError::InvalidRequest(msg) => HttpResponse::new(400)
                .with_body(msg.as_bytes().to_vec())
                .with_header("Content-Type", "text/plain"),

            HandleError::IoError(_) => HttpResponse::new(500)
                .with_body(b"Internal Server Error".to_vec())
                .with_header("Content-Type", "text/plain"),

            HandleError::EncodingError(_) => HttpResponse::new(500)
                .with_body(b"Encoding Error".to_vec())
                .with_header("Content-Type", "text/plain"),

        }

    }

}


// Usage in main handler
pub async fn handle_client(stream: TcpStream) -> Result<(), Box<dyn Error>> {

    let mut reader = BufReader::new(stream);
    let request = parse_request(&mut reader).await?;
    let response = handle_request(request).await;

    // Get the inner stream back from the reader
    let mut stream = reader.into_inner();
    send_response(&mut stream, response).await?;
    Ok(())

}


async fn parse_request(reader: &mut BufReader<TcpStream>) -> Result<HttpRequest, Box<dyn Error>> {
    // Read headers
    let mut headers_vec = Vec::new();
    let mut content_length = 0;
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).await?;
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
        reader.read_exact(&mut body).await?;
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
    stream.write_all(&response_bytes).await?;
    stream.flush().await?;
    Ok(())
}
