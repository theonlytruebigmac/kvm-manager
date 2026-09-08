// Build entry used by KVM Manager. Keep protocol behavior in upstream noVNC while exposing its
// keyboard mapping helper for WebKitGTK's canvas-focus fallback.
export { default } from './novnc/rfb.js'
export { getKeysym } from './novnc/input/util.js'
