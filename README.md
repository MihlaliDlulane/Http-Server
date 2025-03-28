# Rust HTTP Server  

A simple HTTP server built in Rust. Currently, it supports **HTTP/1.1** and provides a few basic endpoints.  

## 🚀 Running the Server  

To start the server, run:  
```bash
cargo run
```  
By default, the server binds to `127.0.0.1:4221`.  

## 📌 Supported Endpoints  

### 1️⃣ `GET /echo/{str}`  
Responds with `{str}` in the response body.  

**Example Request:**  
```bash
curl -v http://localhost:4221/echo/abc
```  

**Example Response:**  
```plaintext
HTTP/1.1 200 OK
Content-Type: text/plain
Content-Length: 3

abc
```

---

### 2️⃣ `GET /user-agent`  
Returns the client's **User-Agent** in the response body.  

**Example Request:**  
```bash
curl -v --header "User-Agent: foobar/1.2.3" http://localhost:4221/user-agent
```  

**Example Response:**  
```plaintext
HTTP/1.1 200 OK
Content-Type: text/plain
Content-Length: 12

foobar/1.2.3
```

---

### 3️⃣ `GET /files/{filename}`  
Reads and returns the contents of a file. The server must be run with the `--directory /path/to/files/` flag to specify the file directory.  

**Example Request:**  
```bash
echo -n 'Hello, World!' > /tmp/foo
curl -i http://localhost:4221/files/foo
```  

**Example Response:**  
```plaintext
HTTP/1.1 200 OK
Content-Type: application/octet-stream
Content-Length: 13

Hello, World!
```

---

### 4️⃣ `POST /files/{filename}`  
Creates a file in the specified directory with the provided content.  

**Example Request:**  
```bash
curl -v --data "12345" -H "Content-Type: application/octet-stream" http://localhost:4221/files/file_123
```  

**Example Response:**  
```plaintext
HTTP/1.1 201 Created
```

---

## 🔥 Additional Features  
✅ Supports **gzip compression**  

---

### 📜 License  
This project is open-source and available under the MIT license.  

