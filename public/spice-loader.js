// SPICE loader - loads the spice-html5 bundle and signals when ready
// This is loaded as a module and triggers the 'spice-loaded' event when done

// Load the SPICE bundle script
const script = document.createElement('script');
script.src = '/spice-bundle.js';
script.onload = () => {
  console.log('SPICE library loaded and SpiceMainConn exposed to window');
  window.dispatchEvent(new CustomEvent('spice-loaded'));
};
script.onerror = (e) => {
  console.error('Failed to load SPICE bundle:', e);
  window.dispatchEvent(new CustomEvent('spice-error', { detail: 'Failed to load SPICE library' }));
};
document.head.appendChild(script);
