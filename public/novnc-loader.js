// noVNC loader - exposes RFB class to window
import RFB from './novnc-bundle.js';
window.__noVNC_RFB__ = RFB;
window.dispatchEvent(new CustomEvent('novnc-loaded'));
console.log('noVNC RFB loaded and exposed to window.__noVNC_RFB__');
