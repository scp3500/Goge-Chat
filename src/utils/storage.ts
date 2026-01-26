// 本地存储封装工具

/**
 * 安全获取 localStorage 项
 * @param key 键名
 * @param defaultValue 默认值
 */
export function getLocalStorage<T>(key: string, defaultValue: T): T {
    try {
        const item = localStorage.getItem(key);
        if (item === null) return defaultValue;
        return JSON.parse(item);
    } catch (error) {
        console.error(`Error reading localStorage key "${key}":`, error);
        return defaultValue;
    }
}

/**
 * 安全设置 localStorage 项
 * @param key 键名
 * @param value 值
 */
export function setLocalStorage<T>(key: string, value: T): void {
    try {
        localStorage.setItem(key, JSON.stringify(value));
    } catch (error) {
        console.error(`Error setting localStorage key "${key}":`, error);
    }
}

/**
 * 安全移除 localStorage 项
 * @param key 键名
 */
export function removeLocalStorage(key: string): void {
    try {
        localStorage.removeItem(key);
    } catch (error) {
        console.error(`Error removing localStorage key "${key}":`, error);
    }
}

/**
 * 安全获取 sessionStorage 项
 * @param key 键名
 * @param defaultValue 默认值
 */
export function getSessionStorage<T>(key: string, defaultValue: T): T {
    try {
        const item = sessionStorage.getItem(key);
        if (item === null) return defaultValue;
        return JSON.parse(item);
    } catch (error) {
        console.error(`Error reading sessionStorage key "${key}":`, error);
        return defaultValue;
    }
}

/**
 * 安全设置 sessionStorage 项
 * @param key 键名
 * @param value 值
 */
export function setSessionStorage<T>(key: string, value: T): void {
    try {
        sessionStorage.setItem(key, JSON.stringify(value));
    } catch (error) {
        console.error(`Error setting sessionStorage key "${key}":`, error);
    }
}

/**
 * 生成带前缀的存储键名
 * @param key 原始键名
 */
export function prefixedKey(key: string): string {
    const appName = 'golechat';
    const version = 'v1';
    return `${appName}:${version}:${key}`;
}

/**
 * 清除所有应用相关的存储
 */
export function clearAppStorage(): void {
    const prefix = 'golechat:';
    for (let i = 0; i < localStorage.length; i++) {
        const key = localStorage.key(i);
        if (key && key.startsWith(prefix)) {
            localStorage.removeItem(key);
        }
    }
    for (let i = 0; i < sessionStorage.length; i++) {
        const key = sessionStorage.key(i);
        if (key && key.startsWith(prefix)) {
            sessionStorage.removeItem(key);
        }
    }
}