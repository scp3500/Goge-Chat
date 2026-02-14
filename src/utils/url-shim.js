// Polyfill for Node.js 'url' module in browser environment
// This is needed because pixi-live2d-display tries to use url.resolve

const resolve = (from, to) => {
    try {
        const baseUrl = new URL(from, window.location.href);
        return new URL(to, baseUrl.href).href;
    } catch (e) {
        console.warn('[url-shim] Failed to resolve URL:', from, to, e);
        return to;
    }
};

// Export both named and default
export { resolve };
export default { resolve };
