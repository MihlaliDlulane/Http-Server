# Rust HTTP Server

A robust, async HTTP server built in Rust using Tokio. This server implements HTTP/1.1 with modern features including connection management, graceful shutdown, and comprehensive error handling.

## ✨ Features

- **Async/Await** - Built on Tokio for high-performance async I/O
- **Connection Management** - Limits concurrent connections
- **Graceful Shutdown** - Handles SIGTERM and Ctrl+C
- **Compression** - Supports gzip encoding
- **Logging** - Structured logging with timestamps
- **Error Handling** - Comprehensive error handling and reporting
- **Configurable** - Environment variable configuration

## 🚀 Running the Server

### Basic Usage
```bash
cargo run
```

### With Custom Configuration
```bash
PORT=8080 MAX_CONNECTIONS=500 CONNECTION_TIMEOUT_SECS=60 cargo run
```

### Environment Variables
- `PORT` - Server port (default: 4221)
- `MAX_CONNECTIONS` - Maximum concurrent connections (default: 1000)
- `CONNECTION_TIMEOUT_SECS` - Connection timeout in seconds (default: 30)

## 📌 Supported Endpoints

### 1️⃣ `GET /echo/{str}`
Returns the provided string with optional gzip compression.

**Example Request:**
```bash
curl -v -H "Accept-Encoding: gzip" http://localhost:4221/echo/hello
```

**Example Response:**
```plaintext
HTTP/1.1 200 OK
Content-Type: text/plain
Content-Encoding: gzip
Content-Length: <length>

<compressed-content>
```

### 2️⃣ `GET /user-agent`
Returns the client's User-Agent.

**Example Request:**
```bash
curl -v --header "User-Agent: foobar/1.2.3" http://localhost:4221/user-agent
```

### 3️⃣ `GET /files/{filename}`
Serves file content from the specified directory.

**Example Request:**
```bash
curl -i http://localhost:4221/files/example.txt
```

### 4️⃣ `POST /files/{filename}`
Creates or updates a file.

**Example Request:**
```bash
curl -v --data "content" http://localhost:4221/files/new.txt
```

## 🔧 Technical Details

### Architecture
- Async runtime using Tokio
- Trait-based request handlers
- Custom error types
- Structured logging
- Connection pooling
- Timeout handling

### Error Handling
- Custom error types for different scenarios
- Proper HTTP status codes
- Detailed error logging

### Performance Features
- Connection limiting
- Async I/O operations
- Efficient request parsing
- Optional compression

## 🛠️ Development

### Prerequisites
- Rust 1.56 or higher
- Cargo

### Dependencies
```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
log = "0.4"
env_logger = "0.9"
futures = "0.3"
async-trait = "0.1"
```

### Building
```bash
cargo build --release
```

### Testing
```bash
cargo test
```

## 🔐 Security Features
- Connection timeouts
- Request size limits
- Proper error handling
- Path traversal prevention

## 📈 Future Improvements
- TLS support
- HTTP/2 support
- WebSocket support
- Request rate limiting
- Cache control
- Authentication middleware
- Metrics collection
- Health check endpoint

