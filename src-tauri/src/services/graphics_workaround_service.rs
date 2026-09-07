use std::path::Path;

/// Applies narrowly targeted WebKitGTK workarounds before Tauri creates a webview.
pub struct GraphicsWorkaroundService;

impl GraphicsWorkaroundService {
    pub fn configure_before_webview() {
        if Self::should_disable_dmabuf(
            cfg!(target_os = "linux"),
            Path::new("/proc/driver/nvidia").exists(),
            std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_some(),
        ) {
            // This runs before Tauri creates threads or a webview. Keep a user-provided value intact.
            unsafe {
                std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
            }
            tracing::info!(
                "Disabled WebKitGTK DMA-BUF rendering for an NVIDIA Linux host; set WEBKIT_DISABLE_DMABUF_RENDERER to override"
            );
        }
    }

    fn should_disable_dmabuf(is_linux: bool, has_nvidia: bool, is_user_configured: bool) -> bool {
        is_linux && has_nvidia && !is_user_configured
    }
}

#[cfg(test)]
mod tests {
    use super::GraphicsWorkaroundService;

    #[test]
    fn enables_workaround_only_for_unconfigured_linux_nvidia_hosts() {
        assert!(GraphicsWorkaroundService::should_disable_dmabuf(
            true, true, false
        ));
        assert!(!GraphicsWorkaroundService::should_disable_dmabuf(
            true, true, true
        ));
        assert!(!GraphicsWorkaroundService::should_disable_dmabuf(
            true, false, false
        ));
        assert!(!GraphicsWorkaroundService::should_disable_dmabuf(
            false, true, false
        ));
    }
}
