import avatar1 from '../assets/avatars/avatar_1.png?inline';
import avatar2 from '../assets/avatars/avatar_2.png?inline';
import avatar3 from '../assets/avatars/avatar_3.png?inline';
import avatar4 from '../assets/avatars/avatar_4.png?inline';
import avatar5 from '../assets/avatars/avatar_5.png?inline';
import avatar6 from '../assets/avatars/avatar_6.png?inline';
import avatar7 from '../assets/avatars/avatar_7.png?inline';
import avatar8 from '../assets/avatars/avatar_8.png?inline';

// 🗺️ Stable ID to Asset Map
export const PRESET_AVATARS_MAP: Record<string, string> = {
    'avatar_1': avatar1,
    'avatar_2': avatar2,
    'avatar_3': avatar3,
    'avatar_4': avatar4,
    'avatar_5': avatar5,
    'avatar_6': avatar6,
    'avatar_7': avatar7,
    'avatar_8': avatar8,
};

// Also map full filenames for backward compatibility or direct usage
export const FILENAME_MAP: Record<string, string> = {
    'avatar_1.png': avatar1,
    'avatar_2.png': avatar2,
    'avatar_3.png': avatar3,
    'avatar_4.png': avatar4,
    'avatar_5.png': avatar5,
    'avatar_6.png': avatar6,
    'avatar_7.png': avatar7,
    'avatar_8.png': avatar8,
};

const DEFAULT_AVATARS = [avatar1, avatar2, avatar3, avatar4, avatar5, avatar6, avatar7, avatar8];

export const getDefaultAvatar = (id: number | string | undefined | null) => {
    if (id === undefined || id === null) return DEFAULT_AVATARS[0];
    const numId = typeof id === 'string' ? (parseInt(id) || 0) : id;
    return DEFAULT_AVATARS[Math.abs(numId) % DEFAULT_AVATARS.length];
};

/**
 * 🛡️ Safely resolve avatar path for Tauri
 * - Keeps base64 / http keys as-is
 * - **NEW:** Maps stable IDs ('avatar_1') to imported assets
 * - Keeps local assets (start with /src, /assets) as-is
 * - Only uses convertFileSrc for absolute OS paths
 */
import { convertFileSrc } from '@tauri-apps/api/core';

export const resolveSocialAvatar = (path: string | null | undefined) => {
    if (!path) return '';

    // 0. Robust Recovery: Fix potential double-encoding or corrupted URL data
    const decoded = decodeURIComponent(path);

    // 0.1 Check if it's a stable ID (e.g. 'avatar_1')
    if (PRESET_AVATARS_MAP[path]) return PRESET_AVATARS_MAP[path];
    if (PRESET_AVATARS_MAP[decoded]) return PRESET_AVATARS_MAP[decoded];

    // 0.2 Universal Preset Match: Find 'avatar_N' anywhere in the string (even corrupted paths)
    const match = decoded.match(/avatar_([1-8])/i);
    if (match) {
        const key = `avatar_${match[1]}`;
        if (PRESET_AVATARS_MAP[key]) return PRESET_AVATARS_MAP[key];
    }

    // 1. Base64 or Remote URL -> return as is (if not already handled by preset match)
    if (path.startsWith('data:') || path.startsWith('http')) return path;

    // 2. Vite Dev / Local Asset Imports
    // We check decoded path for robustness against URL-encoded paths in DB
    if (decoded.startsWith('/src/') || decoded.startsWith('/@') || decoded.includes('/assets/')) {
        return path;
    }

    // 3. Absolute OS Paths (e.g. C:\Users\...) -> need convertFileSrc (asset://)
    // We only do this for things that look like actual disk paths to avoid "connection refused" on virtual paths
    if (path.includes(':') || path.startsWith('\\\\') || (path.startsWith('/') && !path.startsWith('/src/'))) {
        try {
            return convertFileSrc(path);
        } catch (e) {
            console.warn('Failed to convert file src:', path, e);
        }
    }

    return path;
};
