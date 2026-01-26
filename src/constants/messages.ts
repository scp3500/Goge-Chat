// 提示消息常量

/**
 * 成功消息
 */
export const SUCCESS_MESSAGES = {
    // 会话相关
    SESSION_CREATED: '会话创建成功',
    SESSION_DELETED: '会话已删除',
    SESSION_RENAMED: '会话重命名成功',
    
    // 消息相关
    MESSAGE_SENT: '消息发送成功',
    MESSAGE_COPIED: '消息已复制到剪贴板',
    
    // 文件夹相关
    FOLDER_CREATED: '文件夹创建成功',
    FOLDER_DELETED: '文件夹已删除',
    FOLDER_RENAMED: '文件夹重命名成功',
    
    // 配置相关
    SETTINGS_SAVED: '设置已保存',
    API_KEY_SAVED: 'API 密钥已保存',
    
    // 操作相关
    OPERATION_COMPLETED: '操作完成',
    DATA_LOADED: '数据加载完成',
    
    // 导出导入
    EXPORT_SUCCESS: '导出成功',
    IMPORT_SUCCESS: '导入成功',
    BACKUP_SUCCESS: '备份成功',
    RESTORE_SUCCESS: '恢复成功',
} as const;

/**
 * 错误消息
 */
export const ERROR_MESSAGES = {
    // 网络错误
    NETWORK_ERROR: '网络连接失败，请检查网络设置',
    API_ERROR: 'API 请求失败',
    TIMEOUT_ERROR: '请求超时，请稍后重试',
    
    // 会话错误
    SESSION_CREATE_FAILED: '创建会话失败',
    SESSION_DELETE_FAILED: '删除会话失败',
    SESSION_RENAME_FAILED: '重命名会话失败',
    SESSION_NOT_FOUND: '会话不存在',
    
    // 消息错误
    MESSAGE_SEND_FAILED: '发送消息失败',
    MESSAGE_EMPTY: '消息内容不能为空',
    MESSAGE_TOO_LONG: '消息内容过长',
    
    // 文件夹错误
    FOLDER_CREATE_FAILED: '创建文件夹失败',
    FOLDER_DELETE_FAILED: '删除文件夹失败',
    FOLDER_RENAME_FAILED: '重命名文件夹失败',
    FOLDER_NOT_FOUND: '文件夹不存在',
    
    // 配置错误
    SETTINGS_SAVE_FAILED: '保存设置失败',
    API_KEY_INVALID: 'API 密钥无效',
    API_KEY_REQUIRED: '请先设置 API 密钥',
    
    // 数据库错误
    DB_CONNECTION_FAILED: '数据库连接失败',
    DB_QUERY_FAILED: '数据库查询失败',
    DB_BACKUP_FAILED: '备份失败',
    DB_RESTORE_FAILED: '恢复失败',
    
    // 文件错误
    FILE_READ_FAILED: '读取文件失败',
    FILE_WRITE_FAILED: '写入文件失败',
    FILE_NOT_FOUND: '文件不存在',
    FILE_TOO_LARGE: '文件过大',
    
    // 权限错误
    PERMISSION_DENIED: '权限不足',
    STORAGE_PERMISSION_DENIED: '存储权限被拒绝',
    
    // 通用错误
    UNKNOWN_ERROR: '发生未知错误',
    OPERATION_FAILED: '操作失败',
    VALIDATION_FAILED: '验证失败',
} as const;

/**
 * 警告消息
 */
export const WARNING_MESSAGES = {
    // 会话警告
    SESSION_DELETE_CONFIRM: '确定要删除此会话吗？此操作不可撤销。',
    SESSION_UNSAVED_CHANGES: '有未保存的更改，确定要离开吗？',
    
    // 文件夹警告
    FOLDER_DELETE_CONFIRM: '确定要删除此文件夹吗？文件夹内的会话将移动到根目录。',
    FOLDER_NOT_EMPTY: '文件夹不为空，请先清空文件夹内容。',
    
    // 配置警告
    UNSAVED_SETTINGS: '有未保存的设置更改',
    API_KEY_WARNING: 'API 密钥将保存在本地，请确保环境安全',
    
    // 导出导入警告
    EXPORT_CONFIRM: '确定要导出数据吗？',
    IMPORT_CONFIRM: '导入将覆盖现有数据，确定要继续吗？',
    BACKUP_CONFIRM: '确定要创建备份吗？',
    RESTORE_CONFIRM: '恢复将覆盖当前数据，确定要继续吗？',
    
    // 性能警告
    LARGE_HISTORY: '历史记录较大，加载可能需要一些时间',
    MEMORY_WARNING: '内存使用较高，建议清理历史记录',
    
    // 网络警告
    SLOW_CONNECTION: '网络连接较慢',
    OFFLINE_MODE: '当前处于离线模式，部分功能不可用',
} as const;

/**
 * 信息消息
 */
export const INFO_MESSAGES = {
    // 操作信息
    PROCESSING: '处理中...',
    LOADING: '加载中...',
    SAVING: '保存中...',
    EXPORTING: '导出中...',
    IMPORTING: '导入中...',
    
    // 状态信息
    CONNECTING: '连接中...',
    SYNCING: '同步中...',
    UPDATING: '更新中...',
    
    // 提示信息
    NO_SESSIONS: '暂无会话，点击"新对话"开始聊天',
    NO_MESSAGES: '暂无消息，开始对话吧',
    NO_FOLDERS: '暂无文件夹',
    NO_RESULTS: '无搜索结果',
    
    // 功能信息
    DRAG_TO_REORDER: '拖拽以重新排序',
    CLICK_TO_EDIT: '点击以编辑',
    HOVER_FOR_MORE: '悬停查看更多选项',
    
    // 版本信息
    NEW_VERSION_AVAILABLE: '有新版本可用',
    UPDATE_AVAILABLE: '有可用更新',
} as const;

/**
 * 确认对话框消息
 */
export const CONFIRM_MESSAGES = {
    DELETE_SESSION: '确定要删除这个会话吗？',
    DELETE_FOLDER: '确定要删除这个文件夹吗？文件夹内的会话将移动到根目录。',
    CLEAR_HISTORY: '确定要清空所有历史记录吗？此操作不可撤销。',
    RESET_SETTINGS: '确定要重置所有设置为默认值吗？',
    LOGOUT: '确定要退出登录吗？',
    EXIT_APP: '确定要退出应用吗？',
} as const;

/**
 * 占位符文本
 */
export const PLACEHOLDERS = {
    // 输入框占位符
    MESSAGE_INPUT: '输入消息... (Shift+Enter 换行，Enter 发送)',
    SEARCH_INPUT: '搜索会话或消息...',
    TITLE_INPUT: '输入标题...',
    NAME_INPUT: '输入名称...',
    API_KEY_INPUT: '输入 API 密钥...',
    
    // 选择器占位符
    SELECT_MODEL: '选择模型...',
    SELECT_PROVIDER: '选择提供商...',
    SELECT_THEME: '选择主题...',
    SELECT_LANGUAGE: '选择语言...',
    
    // 空状态占位符
    EMPTY_SESSIONS: '暂无会话',
    EMPTY_MESSAGES: '暂无消息',
    EMPTY_FOLDERS: '暂无文件夹',
    EMPTY_SEARCH: '无搜索结果',
    
    // 其他占位符
    LOADING_CONTENT: '内容加载中...',
    PROCESSING_REQUEST: '请求处理中...',
    CONNECTING_SERVER: '正在连接服务器...',
} as const;

/**
 * 按钮文本
 */
export const BUTTON_TEXTS = {
    // 操作按钮
    OK: '确定',
    CANCEL: '取消',
    CONFIRM: '确认',
    SAVE: '保存',
    DELETE: '删除',
    EDIT: '编辑',
    RENAME: '重命名',
    CREATE: '创建',
    ADD: '添加',
    REMOVE: '移除',
    IMPORT: '导入',
    EXPORT: '导出',
    BACKUP: '备份',
    RESTORE: '恢复',
    
    // 功能按钮
    NEW_CHAT: '新对话',
    NEW_FOLDER: '新建文件夹',
    SEND_MESSAGE: '发送',
    STOP_GENERATION: '停止',
    COPY: '复制',
    SHARE: '分享',
    DOWNLOAD: '下载',
    UPLOAD: '上传',
    
    // 设置按钮
    APPLY: '应用',
    RESET: '重置',
    TEST: '测试',
    CONNECT: '连接',
    DISCONNECT: '断开',
    
    // 导航按钮
    BACK: '返回',
    NEXT: '下一步',
    PREVIOUS: '上一步',
    FINISH: '完成',
    SKIP: '跳过',
} as const;