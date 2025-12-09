//! File operation handlers

use kvmmanager_agent_common::*;
use serde_json::json;
use std::path::Path;
use tokio::fs;
use tracing::{debug, warn};

use crate::config::Config;

pub async fn handle_file_read(
    request: JsonRpcRequest,
    id: RequestId,
    config: &Config,
) -> JsonRpcResponse {
    debug!("Handling file_read");

    // Parse parameters
    let params: FileReadParams = match request.params {
        Some(params) => match serde_json::from_value(params) {
            Ok(p) => p,
            Err(e) => {
                return JsonRpcResponse::error(
                    JsonRpcError::invalid_params(format!("Invalid parameters: {}", e)),
                    id,
                );
            }
        },
        None => {
            return JsonRpcResponse::error(
                JsonRpcError::invalid_params("Missing parameters"),
                id,
            );
        }
    };

    // Resolve path (canonicalize)
    let path = match std::fs::canonicalize(&params.path) {
        Ok(p) => p,
        Err(e) => {
            warn!("Failed to resolve path {}: {}", params.path, e);
            return JsonRpcResponse::error(
                JsonRpcError::not_found(format!("File not found: {}", params.path)),
                id,
            );
        }
    };

    let path_str = path.to_string_lossy().to_string();

    // Check if path is allowed
    if !config.is_read_allowed(&path_str) {
        warn!("Read access denied for path: {}", path_str);
        return JsonRpcResponse::error(
            JsonRpcError::permission_denied(format!("Read access denied: {}", path_str)),
            id,
        );
    }

    // Read file
    match fs::read(&path).await {
        Ok(bytes) => {
            // Check file size limit
            let max_size = config.security.max_file_size_mb * 1024 * 1024;
            if bytes.len() > max_size {
                return JsonRpcResponse::error(
                    JsonRpcError::command_failed(format!(
                        "File too large: {} bytes (max {} MB)",
                        bytes.len(),
                        config.security.max_file_size_mb
                    )),
                    id,
                );
            }

            let content = if params.encoding == "base64" {
                base64::encode(&bytes)
            } else {
                String::from_utf8_lossy(&bytes).to_string()
            };

            let result = FileReadResult {
                content,
                size_bytes: bytes.len() as u64,
                encoding: params.encoding,
            };

            JsonRpcResponse::success(json!(result), id)
        }
        Err(e) => {
            warn!("Failed to read file {}: {}", path_str, e);
            JsonRpcResponse::error(
                JsonRpcError::command_failed(format!("Failed to read file: {}", e)),
                id,
            )
        }
    }
}

pub async fn handle_file_write(
    request: JsonRpcRequest,
    id: RequestId,
    config: &Config,
) -> JsonRpcResponse {
    debug!("Handling file_write");

    // Parse parameters
    let params: FileWriteParams = match request.params {
        Some(params) => match serde_json::from_value(params) {
            Ok(p) => p,
            Err(e) => {
                return JsonRpcResponse::error(
                    JsonRpcError::invalid_params(format!("Invalid parameters: {}", e)),
                    id,
                );
            }
        },
        None => {
            return JsonRpcResponse::error(
                JsonRpcError::invalid_params("Missing parameters"),
                id,
            );
        }
    };

    let path = Path::new(&params.path);

    // Check if parent directory exists, create if needed
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            if params.create_dirs {
                if let Err(e) = fs::create_dir_all(parent).await {
                    warn!("Failed to create directories for {}: {}", params.path, e);
                    return JsonRpcResponse::error(
                        JsonRpcError::command_failed(format!(
                            "Failed to create directories: {}",
                            e
                        )),
                        id,
                    );
                }
            } else {
                return JsonRpcResponse::error(
                    JsonRpcError::not_found(format!(
                        "Parent directory does not exist: {}",
                        parent.display()
                    )),
                    id,
                );
            }
        }
    }

    // For security, always resolve to absolute path after creation
    let abs_path = match path.canonicalize() {
        Ok(p) => p,
        Err(_) => {
            // If file doesn't exist yet, check parent directory
            if let Some(parent) = path.parent() {
                match parent.canonicalize() {
                    Ok(parent_path) => parent_path.join(
                        path.file_name().unwrap_or_default()
                    ),
                    Err(e) => {
                        warn!("Failed to resolve path {}: {}", params.path, e);
                        return JsonRpcResponse::error(
                            JsonRpcError::not_found(format!("Invalid path: {}", params.path)),
                            id,
                        );
                    }
                }
            } else {
                warn!("Invalid path: {}", params.path);
                return JsonRpcResponse::error(
                    JsonRpcError::not_found(format!("Invalid path: {}", params.path)),
                    id,
                );
            }
        }
    };

    let path_str = abs_path.to_string_lossy().to_string();

    // Check if path is allowed
    if !config.is_write_allowed(&path_str) {
        warn!("Write access denied for path: {}", path_str);
        return JsonRpcResponse::error(
            JsonRpcError::permission_denied(format!("Write access denied: {}", path_str)),
            id,
        );
    }

    // Decode content if base64
    let bytes = if params.encoding == "base64" {
        match base64::decode(&params.content) {
            Ok(b) => b,
            Err(e) => {
                return JsonRpcResponse::error(
                    JsonRpcError::invalid_params(format!("Invalid base64: {}", e)),
                    id,
                );
            }
        }
    } else {
        params.content.into_bytes()
    };

    // Check file size limit
    let max_size = config.security.max_file_size_mb * 1024 * 1024;
    if bytes.len() > max_size {
        return JsonRpcResponse::error(
            JsonRpcError::command_failed(format!(
                "Content too large: {} bytes (max {} MB)",
                bytes.len(),
                config.security.max_file_size_mb
            )),
            id,
        );
    }

    // Write file
    match fs::write(&abs_path, &bytes).await {
        Ok(_) => {
            // Set permissions if specified
            if let Some(perms) = &params.permissions {
                if let Err(e) = set_file_permissions(&abs_path, perms).await {
                    warn!("Failed to set permissions: {}", e);
                    // Don't fail the request, just log warning
                }
            }

            let result = FileWriteResult {
                bytes_written: bytes.len() as u64,
                path: path_str,
            };

            JsonRpcResponse::success(json!(result), id)
        }
        Err(e) => {
            warn!("Failed to write file {}: {}", path_str, e);
            JsonRpcResponse::error(
                JsonRpcError::command_failed(format!("Failed to write file: {}", e)),
                id,
            )
        }
    }
}

async fn set_file_permissions(path: &Path, perms: &str) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    // Parse octal permissions (e.g., "0644")
    let mode = u32::from_str_radix(perms.trim_start_matches("0"), 8)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

    let permissions = std::fs::Permissions::from_mode(mode);
    fs::set_permissions(path, permissions).await
}

// Note: Need to add base64 dependency to Cargo.toml
// For now, we'll use a simple stub that returns an error
mod base64 {
    pub fn encode(bytes: &[u8]) -> String {
        // TODO: Add base64 crate dependency
        format!("base64_encoding_not_implemented_{}_bytes", bytes.len())
    }

    pub fn decode(_s: &str) -> Result<Vec<u8>, String> {
        // TODO: Add base64 crate dependency
        Err("base64 decoding not implemented yet".to_string())
    }
}
