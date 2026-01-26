// 导出所有类型定义
export * from './chat';
export * from './config';
export * from './tauri';

// 通用工具类型
export type Nullable<T> = T | null;
export type Optional<T> = T | undefined;