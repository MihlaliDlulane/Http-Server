mod http;

use tokio::net::TcpListener;
use tokio::signal;
use std::error::Error;
use std::sync::Arc;
use log::{info, error, LevelFilter};
use env_logger::Builder;
use tokio::sync::Semaphore;
use futures::future::join_all;

// Config struct
#[derive(Clone)]
struct ServerConfig {
    port: String,
    max_connections: usize,
    connection_timeout: std::time::Duration,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: "4221".to_string(),
            max_connections:1000,
            connection_timeout:std::time::Duration::from_secs(30),
        }
    }
}

async fn shutdown_signal() {

    let ctrl_c = async {
            signal::ctrl_c()
                .await
                .expect("failed to install Ctrl+c handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Shutdown signla received");

}

struct Server {
    config: ServerConfig,
    connection_limiter : Arc<Semaphore>
}

impl Server {
    fn new(config:ServerConfig) -> Self {
        Self {
            connection_limiter: Arc::new(Semaphore::new(config.max_connections)),
            config,
        }
    }

    async fn run(&self) -> Result<(), Box<dyn Error>> {
        let addr = format!("127.0.0.1:{}",self.config.port);
        let listener = TcpListener::bind(&addr).await?;
        info!("Server listenening on {}",addr);

        let mut active_connections = Vec::new();

        loop {
            tokio::select! {
                accept_result = listener.accept() => {
                    match accept_result {
                        Ok((stream, addr)) => {
                            info!("New connection from: {}", addr);


                            // Acquire a permit from the semaphore
                            let permit = self.connection_limiter.clone()
                                .acquire_owned()
                                .await
                                .expect("Failed to acquire connection permit");


                            let handle = tokio::spawn(async move {
                                let _permit = permit; // Will be dropped when the task completes
                                if let Err(e) = http::http1::handle_client(stream).await {
                                    error!("Error handling client {}: {}", addr, e);
                                }
                            });

                            active_connections.push(handle);
                            // Clean up completed connections
                            active_connections.retain(|handle| !handle.is_finished());
                        }
                        Err(e) => {
                            error!("Error accepting connection: {}", e);
                        }
                    }
                }

                _ = shutdown_signal() => {
                    info!("Shutting down server...");
                    // Wait for all active connections to complete
                    join_all(active_connections).await;
                    break;
                }
            }

        }

        Ok(())
    }

}

#[tokio::main]

async fn main() -> Result<(), Box<dyn Error>> {

    // Initialize logger
    Builder::new()
        .filter_level(LevelFilter::Info)
        .format_timestamp_millis()
        .init();


    // Load configuration
    let config = ServerConfig {
        port: std::env::var("PORT").unwrap_or_else(|_| "4221".to_string()),
        max_connections: std::env::var("MAX_CONNECTIONS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1000),

        connection_timeout: std::time::Duration::from_secs(
            std::env::var("CONNECTION_TIMEOUT_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30)
        ),
    };


    // Create and run server
    let server = Server::new(config);
    server.run().await?;


    info!("Server shutdown complete");
    Ok(())

}

