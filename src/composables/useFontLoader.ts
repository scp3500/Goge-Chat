import { ref } from 'vue';
import { convertFileSrc } from '@tauri-apps/api/core';

// Singleton style manager to avoid duplicating style tags
let styleElement: HTMLStyleElement | null = null;

const getStyleElement = () => {
    if (!styleElement) {
        styleElement = document.createElement('style');
        styleElement.id = 'dynamic-font-loader';
        document.head.appendChild(styleElement);
    }
    return styleElement;
};

export function useFontLoader() {

    /**
     * Loads a font from a local file path or returns the system font name.
     * @param fontInput The font name or absolute file path
     * @param type 'english' | 'chinese' - used to generate unique font-family names
     * @returns The CSS font-family string to use
     */
    const loadFont = (fontInput: string, type: 'english' | 'chinese'): string => {
        if (!fontInput || !fontInput.trim()) return '';

        const isFilePath = /\.(ttf|otf|woff|woff2)$/i.test(fontInput);

        if (isFilePath) {
            try {
                const assetUrl = convertFileSrc(fontInput);
                const fontHash = btoa(fontInput).substring(0, 8).replace(/[^a-zA-Z0-9]/g, '');
                const familyName = `CustomFont${type === 'english' ? 'En' : 'Zh'}_${fontHash}`;

                const rule = `
                    @font-face {
                        font-family: '${familyName}';
                        src: url('${assetUrl}');
                        font-display: swap;
                    }
                `;

                updateGlobalStyle(type, rule);

                return `'${familyName}'`;
            } catch (e) {
                console.error("Failed to load local font:", e);
                return 'sans-serif';
            }
        } else {
            // Treat as system font name, just return it quoted if necessary
            // If it contains spaces and isn't quoted, quote it.
            if (fontInput.includes(' ') && !fontInput.startsWith('"') && !fontInput.startsWith("'")) {
                return `"${fontInput}"`;
            }
            return fontInput;
        }
    };

    return {
        loadFont
    };
}

// Global state to hold active font rules
const activeRules = {
    english: '',
    chinese: ''
};

function updateGlobalStyle(type: 'english' | 'chinese', rule: string) {
    activeRules[type] = rule;
    const styleEl = getStyleElement();
    styleEl.innerHTML = `${activeRules.english}\n${activeRules.chinese}`;
}
