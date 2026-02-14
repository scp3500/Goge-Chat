/**
 * Patch for PixiJS v7.3+ url.resolve deprecation
 * This patches the PIXI.utils.url.resolve to use native URL API
 */

export function patchPixiUrlResolve() {
    // Check if PIXI is available
    if (typeof window === 'undefined' || !window.PIXI) {
        console.warn('[PixiURLPatch] PIXI not found, skipping patch');
        return;
    }

    const PIXI = window.PIXI;

    // Patch utils.url if it exists
    if (PIXI.utils && PIXI.utils.url) {
        const originalResolve = PIXI.utils.url.resolve;

        PIXI.utils.url.resolve = function (base, url) {
            try {
                // Use native URL API
                const baseUrl = new URL(base, window.location.href);
                return new URL(url, baseUrl.href).href;
            } catch (e) {
                // Fallback to original if it exists
                if (originalResolve && typeof originalResolve === 'function') {
                    return originalResolve(base, url);
                }
                // Last resort: just return the url
                return url;
            }
        };

        console.log('[PixiURLPatch] Successfully patched PIXI.utils.url.resolve');
    } else {
        console.warn('[PixiURLPatch] PIXI.utils.url not found, no patch needed');
    }
}
