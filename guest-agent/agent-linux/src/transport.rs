//! Virtio-serial transport layer

use anyhow::{Context, Result};
use kvmmanager_agent_common::{JsonRpcRequest, JsonRpcResponse};
use std::path::Path;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{debug, trace};

use crate::config::Config;

pub struct VirtioSerialTransport {
    device_path: std::path::PathBuf,
    reader: Option<BufReader<tokio::fs::File>>,
    writer: Option<tokio::fs::File>,
    config: Config,
}

impl VirtioSerialTransport {
    pub fn new(device_path: &Path, config: &Config) -> Result<Self> {
        let device_path = device_path.to_path_buf();

        let mut transport = Self {
            device_path,
            reader: None,
            writer: None,
            config: config.clone(),
        };

        transport.connect()?;
        Ok(transport)
    }

    fn connect(&mut self) -> Result<()> {
        // Open device for reading and writing
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&self.device_path)
            .context("Failed to open virtio-serial device")?;

        // Clone file descriptor for reader and writer (std version)
        let reader_file = file.try_clone().context("Failed to clone file descriptor")?;
        let writer_file = file;

        // Convert to tokio files
        let reader_file = tokio::fs::File::from_std(reader_file);
        let writer_file = tokio::fs::File::from_std(writer_file);

        self.reader = Some(BufReader::new(reader_file));
        self.writer = Some(writer_file);

        debug!("Connected to virtio-serial device: {:?}", self.device_path);
        Ok(())
    }    pub async fn reconnect(&mut self) -> Result<()> {
        debug!("Attempting to reconnect to virtio-serial device");
        self.reader = None;
        self.writer = None;
        self.connect()?;
        Ok(())
    }

    pub async fn receive_request(&mut self) -> Result<JsonRpcRequest> {
        let reader = self.reader.as_mut().context("Transport not connected")?;

        let mut line = String::new();
        let bytes_read = reader
            .read_line(&mut line)
            .await
            .context("Failed to read from virtio-serial")?;

        if bytes_read == 0 {
            anyhow::bail!("Connection closed (EOF)");
        }

        trace!("Received: {}", line.trim());

        let request: JsonRpcRequest = serde_json::from_str(&line)
            .context("Failed to parse JSON-RPC request")?;

        Ok(request)
    }

    pub async fn send_response(&mut self, response: JsonRpcResponse) -> Result<()> {
        let writer = self.writer.as_mut().context("Transport not connected")?;

        let mut json = serde_json::to_string(&response)
            .context("Failed to serialize JSON-RPC response")?;

        json.push('\n');

        trace!("Sending: {}", json.trim());

        writer
            .write_all(json.as_bytes())
            .await
            .context("Failed to write to virtio-serial")?;

        writer
            .flush()
            .await
            .context("Failed to flush virtio-serial")?;

        Ok(())
    }
}
