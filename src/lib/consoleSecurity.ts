const LOOPBACK_CONSOLE_HOSTS = new Set(['127.0.0.1', 'localhost', '[::1]'])

/** Console viewers may only connect to the local proxy started by the Rust backend. */
export function localConsoleWebSocketUrl(host: string, port: number): string {
  if (!LOOPBACK_CONSOLE_HOSTS.has(host) || !Number.isInteger(port) || port < 1 || port > 65535) {
    throw new Error('The console proxy endpoint is unavailable.')
  }

  return `ws://${host}:${port}`
}
