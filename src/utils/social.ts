import { convertFileSrc } from '@tauri-apps/api/core';

// 🗺️ Stable ID to Public Asset Path Map
// Since these are in public/, we use absolute web paths starting with /
export const PRESET_AVATARS_MAP: Record<string, string> = {
    'avatar_1': '/avatars/avatar_1.png',
    'avatar_2': '/avatars/avatar_2.png',
    'avatar_3': '/avatars/avatar_3.png',
    'avatar_4': '/avatars/avatar_4.png',
    'avatar_5': '/avatars/avatar_5.png',
    'avatar_6': '/avatars/avatar_6.png',
    'avatar_7': '/avatars/avatar_7.png',
    'avatar_8': '/avatars/avatar_8.png',
};

// Also map full filenames for backward compatibility or direct usage
export const FILENAME_MAP: Record<string, string> = {
    'avatar_1.png': '/avatars/avatar_1.png',
    'avatar_2.png': '/avatars/avatar_2.png',
    'avatar_3.png': '/avatars/avatar_3.png',
    'avatar_4.png': '/avatars/avatar_4.png',
    'avatar_5.png': '/avatars/avatar_5.png',
    'avatar_6.png': '/avatars/avatar_6.png',
    'avatar_7.png': '/avatars/avatar_7.png',
    'avatar_8.png': '/avatars/avatar_8.png',
};

const DEFAULT_AVATARS = [
    '/avatars/avatar_1.png',
    '/avatars/avatar_2.png',
    '/avatars/avatar_3.png',
    '/avatars/avatar_4.png',
    '/avatars/avatar_5.png',
    '/avatars/avatar_6.png',
    '/avatars/avatar_7.png',
    '/avatars/avatar_8.png',
];

export const getDefaultAvatar = (id: number | string | undefined | null) => {
    if (id === undefined || id === null) return DEFAULT_AVATARS[0];
    const numId = typeof id === 'string' ? (parseInt(id) || 0) : id;
    return DEFAULT_AVATARS[Math.abs(numId) % DEFAULT_AVATARS.length];
};

/**
 * 🛡️ Safely resolve avatar path for Tauri
 * - Keeps base64 / http keys as-is
 * - Maps stable IDs ('avatar_1') to public assets
 * - Automatically handles paths previously pointing to /src/assets
 */
export const resolveSocialAvatar = (path: string | null | undefined) => {
    if (!path) return '';

    // 0. Robust Recovery: Fix potential double-encoding or corrupted URL data
    const decoded = decodeURIComponent(path);

    // 0.1 Check if it's a stable ID (e.g. 'avatar_1') or old filename
    if (PRESET_AVATARS_MAP[path]) return PRESET_AVATARS_MAP[path];
    if (PRESET_AVATARS_MAP[decoded]) return PRESET_AVATARS_MAP[decoded];
    if (FILENAME_MAP[path]) return FILENAME_MAP[path];

    // 0.2 Universal Preset Match: Find 'avatar_N' anywhere in the string
    const match = decoded.match(/avatar_([1-8])/i);
    if (match) {
        const key = `avatar_${match[1]}`;
        if (PRESET_AVATARS_MAP[key]) return PRESET_AVATARS_MAP[key];
    }

    // 1. Base64 or Remote URL -> return as is
    if (path.startsWith('data:') || path.startsWith('http')) return path;

    // 2. Vite Public Assets (new stable path)
    if (path.startsWith('/avatars/')) return path;

    // 3. Absolute OS Paths (e.g. C:\Users\...) -> need convertFileSrc
    if (path.includes(':') || path.startsWith('\\\\') || (path.startsWith('/') && !path.startsWith('/src/') && !path.startsWith('/avatars/'))) {
        try {
            return convertFileSrc(path);
        } catch (e) {
            console.warn('Failed to convert file src:', path, e);
        }
    }

    return path;
};
