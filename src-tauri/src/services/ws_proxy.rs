use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use crate::utils::error::AppError;

/// Native Rust WebSocket-to-TCP proxy for VNC/SPICE connections
/// This replaces the external websockify dependency
pub struct WsProxyService {
    /// Map of VM ID to proxy info (port and shutdown channel)
    proxies: Arc<RwLock<HashMap<String, ProxyHandle>>>,
    /// Base port for WebSocket proxies
    base_port: u16,
}

struct ProxyHandle {
    ws_port: u16,
    vnc_port: u16,
    shutdown_tx: mpsc::Sender<()>,
}

impl WsProxyService {
    pub fn new() -> Self {
        Self {
            proxies: Arc::new(RwLock::new(HashMap::new())),
            base_port: 6080,
        }
    }

    /// Start a WebSocket proxy for a VM's VNC/SPICE connection
    /// Returns the WebSocket port to connect to
    pub async fn start_proxy(&self, vm_id: &str, vnc_port: u16) -> Result<u16, AppError> {
        // Check if proxy already exists
        {
            let proxies = self.proxies.read().await;
            if let Some(handle) = proxies.get(vm_id) {
                if handle.vnc_port == vnc_port {
                    tracing::debug!("Reusing existing proxy for VM {} on port {}", vm_id, handle.ws_port);
                    return Ok(handle.ws_port);
                }
            }
        }

        // Stop existing proxy if VNC port changed
        self.stop_proxy(vm_id).await?;

        // Find available port
        let ws_port = self.find_available_port().await?;

        // Create shutdown channel
        let (shutdown_tx, shutdown_rx) = mpsc::channel::<()>(1);

        // Start the proxy server
        let listener = TcpListener::bind(format!("127.0.0.1:{}", ws_port))
            .await
            .map_err(|e| AppError::Other(format!("Failed to bind to port {}: {}", ws_port, e)))?;

        tracing::info!("Starting WebSocket proxy for VM {} on port {} -> VNC {}", vm_id, ws_port, vnc_port);

        // Spawn the proxy task
        let vm_id_clone = vm_id.to_string();
        tokio::spawn(async move {
            run_proxy_server(listener, vnc_port, shutdown_rx, vm_id_clone).await;
        });

        // Store the handle
        {
            let mut proxies = self.proxies.write().await;
            proxies.insert(vm_id.to_string(), ProxyHandle {
                ws_port,
                vnc_port,
                shutdown_tx,
            });
        }

        Ok(ws_port)
    }

    /// Stop the proxy for a specific VM
    pub async fn stop_proxy(&self, vm_id: &str) -> Result<(), AppError> {
        let mut proxies = self.proxies.write().await;
        if let Some(handle) = proxies.remove(vm_id) {
            // Send shutdown signal (ignore if receiver is gone)
            let _ = handle.shutdown_tx.send(()).await;
            tracing::info!("Stopped WebSocket proxy for VM {}", vm_id);
        }
        Ok(())
    }

    /// Stop all proxies
    #[allow(dead_code)]
    pub async fn stop_all(&self) {
        let mut proxies = self.proxies.write().await;
        for (vm_id, handle) in proxies.drain() {
            let _ = handle.shutdown_tx.send(()).await;
            tracing::info!("Stopped WebSocket proxy for VM {}", vm_id);
        }
    }

    /// Find an available port
    async fn find_available_port(&self) -> Result<u16, AppError> {
        let proxies = self.proxies.read().await;
        let used_ports: Vec<u16> = proxies.values().map(|h| h.ws_port).collect();

        for port in self.base_port..self.base_port + 100 {
            if !used_ports.contains(&port) {
                // Try to bind to verify it's available
                if TcpListener::bind(format!("127.0.0.1:{}", port)).await.is_ok() {
                    return Ok(port);
                }
            }
        }

        Err(AppError::Other("No available ports for WebSocket proxy".to_string()))
    }

    /// Get proxy port for a VM if running
    #[allow(dead_code)]
    pub async fn get_proxy_port(&self, vm_id: &str) -> Option<u16> {
        let proxies = self.proxies.read().await;
        proxies.get(vm_id).map(|h| h.ws_port)
    }
}

/// Run the WebSocket proxy server
async fn run_proxy_server(
    listener: TcpListener,
    vnc_port: u16,
    mut shutdown_rx: mpsc::Receiver<()>,
    vm_id: String,
) {
    loop {
        tokio::select! {
            // Accept new WebSocket connections
            accept_result = listener.accept() => {
                match accept_result {
                    Ok((stream, addr)) => {
                        tracing::debug!("New WebSocket connection from {} for VM {}", addr, vm_id);
                        let vm_id_clone = vm_id.clone();
                        tokio::spawn(async move {
                            if let Err(e) = handle_ws_connection(stream, vnc_port, addr, vm_id_clone).await {
                                tracing::warn!("WebSocket connection error: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        tracing::error!("Failed to accept connection: {}", e);
                    }
                }
            }
            // Shutdown signal
            _ = shutdown_rx.recv() => {
                tracing::debug!("Proxy server for VM {} shutting down", vm_id);
                break;
            }
        }
    }
}

/// Handle a single WebSocket connection, bridging to VNC
async fn handle_ws_connection(
    ws_stream: TcpStream,
    vnc_port: u16,
    addr: SocketAddr,
    vm_id: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    // Upgrade to WebSocket
    let ws = accept_async(ws_stream).await?;
    let (mut ws_write, mut ws_read) = ws.split();

    // Connect to VNC server
    let vnc_stream = TcpStream::connect(format!("127.0.0.1:{}", vnc_port)).await?;
    let (mut vnc_read, mut vnc_write) = vnc_stream.into_split();

    tracing::info!("Bridging WebSocket {} <-> VNC {} for VM {}", addr, vnc_port, vm_id);

    // Use a channel for sending messages to WebSocket
    let (ws_tx, mut ws_rx) = mpsc::channel::<Message>(256);
    let ws_tx_clone = ws_tx.clone();

    // Task: Write messages to WebSocket
    let ws_writer_task = async move {
        while let Some(msg) = ws_rx.recv().await {
            if ws_write.send(msg).await.is_err() {
                break;
            }
        }
    };

    // Task: Read from WebSocket, write to VNC
    let ws_to_vnc = async move {
        while let Some(msg_result) = ws_read.next().await {
            match msg_result {
                Ok(Message::Binary(data)) => {
                    if vnc_write.write_all(&data).await.is_err() {
                        break;
                    }
                    if vnc_write.flush().await.is_err() {
                        break;
                    }
                }
                Ok(Message::Close(_)) => {
                    tracing::debug!("WebSocket closed by client");
                    break;
                }
                Ok(Message::Ping(data)) => {
                    let _ = ws_tx_clone.send(Message::Pong(data)).await;
                }
                Ok(_) => {
                    // Ignore text and other messages
                }
                Err(e) => {
                    tracing::debug!("WebSocket read error: {}", e);
                    break;
                }
            }
        }
    };

    // Task: Read from VNC, write to WebSocket
    let vnc_to_ws = async move {
        let mut buffer = vec![0u8; 65536];
        loop {
            match vnc_read.read(&mut buffer).await {
                Ok(0) => {
                    tracing::debug!("VNC connection closed");
                    break;
                }
                Ok(n) => {
                    if ws_tx.send(Message::Binary(buffer[..n].to_vec())).await.is_err() {
                        break;
                    }
                }
                Err(e) => {
                    tracing::debug!("VNC read error: {}", e);
                    break;
                }
            }
        }
    };

    // Run all tasks concurrently, stop when any ends
    tokio::select! {
        _ = ws_writer_task => {}
        _ = ws_to_vnc => {}
        _ = vnc_to_ws => {}
    }

    tracing::debug!("Connection closed for VM {} from {}", vm_id, addr);
    Ok(())
}

impl Drop for WsProxyService {
    fn drop(&mut self) {
        // Note: Can't do async cleanup in drop, but the shutdown channels
        // will be dropped which signals the proxy tasks to stop
        tracing::debug!("WsProxyService dropped");
    }
}
