use base64::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::AppHandle;

// ==================================================================================
// core data structures
// ==================================================================================

// The full configuration object (merged from parts)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    #[serde(default = "default_font_size", rename = "fontSize")]
    pub font_size: u32,
    #[serde(default = "default_line_ratio", rename = "lineRatio")]
    pub line_ratio: f32,
    #[serde(default = "default_theme_color", rename = "themeColor")]
    pub theme_color: String,
    #[serde(default = "default_theme", rename = "theme")]
    pub theme: String,
    #[serde(default = "default_dark_theme_id", rename = "darkThemeId")]
    pub dark_theme_id: String,
    #[serde(default = "default_light_theme_id", rename = "lightThemeId")]
    pub light_theme_id: String,
    #[serde(default = "default_scrollbar_width", rename = "scrollbarWidth")]
    pub scrollbar_width: u32,
    #[serde(default, rename = "apiKey")]
    pub api_key: String,
    #[serde(default = "default_search_url", rename = "searchInstanceUrl")]
    pub search_instance_url: String,
    #[serde(default = "default_search_provider", rename = "defaultSearchProvider")]
    pub default_search_provider: String,

    // Providers (from providers.json + secrets.json)
    #[serde(default = "default_providers", rename = "providers")]
    pub providers: serde_json::Value,
    #[serde(default = "default_provider_id", rename = "defaultProviderId")]
    pub default_provider_id: String,
    #[serde(default = "default_model_id", rename = "selectedModelId")]
    pub selected_model_id: String,
    #[serde(default = "default_model_id", rename = "globalModelId")]
    pub global_model_id: String,

    #[serde(default, rename = "useReasoning")]
    pub use_reasoning: bool,
    #[serde(default, rename = "useSearch")]
    pub use_search: bool,
    #[serde(default = "default_search_provider", rename = "searchProvider")]
    pub search_provider: String,

    // NEW: Presets (from presets.json)
    #[serde(default = "default_presets", rename = "presets")]
    pub presets: serde_json::Value,
    #[serde(default = "default_preset_id", rename = "defaultPresetId")]
    pub default_preset_id: String,
    #[serde(default = "default_preset_id", rename = "globalPresetId")]
    pub global_preset_id: String,

    // NEW: Chat Experience Settings
    #[serde(default = "default_true", rename = "enableStream")]
    pub enable_stream: bool,
    #[serde(default = "default_false", rename = "enableBubble")]
    pub enable_bubble: bool,

    // NEW: User Avatar Settings
    #[serde(default = "default_false", rename = "showUserAvatar")]
    pub show_user_avatar: bool,
    #[serde(default, rename = "userAvatarPath")]
    pub user_avatar_path: String,

    // NEW: Prompt Library (from prompts.json)
    #[serde(default = "default_prompt_library", rename = "promptLibrary")]
    pub prompt_library: serde_json::Value,

    // NEW: Global Typography
    #[serde(default, rename = "fontFamilyEnglish")]
    pub font_family_english: String,
    #[serde(default, rename = "fontFamilyChinese")]
    pub font_family_chinese: String,

    // NEW: Chat Mode
    #[serde(default = "default_chat_mode", rename = "chatMode")]
    pub chat_mode: ChatModeConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatModeConfig {
    #[serde(default = "default_false")]
    pub enabled: bool,
    #[serde(default = "default_weekchat_theme", rename = "dayThemeId")]
    pub day_theme_id: String,
    #[serde(default = "default_dark_plus_theme", rename = "nightThemeId")]
    pub night_theme_id: String,
    #[serde(default = "default_false", rename = "enableStream")]
    pub enable_stream: bool,
    #[serde(default = "default_false", rename = "enableLoadingBar")]
    pub enable_loading_bar: bool,
}

// Partial structs for file separation

#[derive(Serialize, Deserialize)]
struct SettingsPart {
    #[serde(default = "default_font_size", rename = "fontSize")]
    font_size: u32,
    #[serde(default = "default_line_ratio", rename = "lineRatio")]
    line_ratio: f32,
    #[serde(default = "default_theme_color", rename = "themeColor")]
    theme_color: String,
    #[serde(default = "default_theme", rename = "theme")]
    theme: String,
    #[serde(default = "default_dark_theme_id", rename = "darkThemeId")]
    dark_theme_id: String,
    #[serde(default = "default_light_theme_id", rename = "lightThemeId")]
    light_theme_id: String,
    #[serde(default = "default_scrollbar_width", rename = "scrollbarWidth")]
    scrollbar_width: u32,
    #[serde(default, rename = "apiKey", skip_serializing_if = "String::is_empty")]
    api_key: String, // Kept for legacy compatibility
    #[serde(default = "default_search_url", rename = "searchInstanceUrl")]
    search_instance_url: String,
    #[serde(default = "default_search_provider", rename = "defaultSearchProvider")]
    default_search_provider: String,
    #[serde(default = "default_provider_id", rename = "defaultProviderId")]
    default_provider_id: String, // Pointer to default provider
    #[serde(default = "default_model_id", rename = "selectedModelId")]
    selected_model_id: String, // Pointer to selected model
    #[serde(default = "default_model_id", rename = "globalModelId")]
    global_model_id: String,
    #[serde(default, rename = "useReasoning")]
    use_reasoning: bool,
    #[serde(default, rename = "useSearch")]
    use_search: bool,
    #[serde(default = "default_search_provider", rename = "searchProvider")]
    search_provider: String,
    #[serde(default = "default_preset_id", rename = "defaultPresetId")]
    default_preset_id: String, // Pointer to default preset
    #[serde(default = "default_preset_id", rename = "globalPresetId")]
    global_preset_id: String,

    #[serde(default = "default_true", rename = "enableStream")]
    enable_stream: bool,
    #[serde(default = "default_false", rename = "enableBubble")]
    enable_bubble: bool,
    #[serde(default = "default_false", rename = "showUserAvatar")]
    show_user_avatar: bool,
    #[serde(default, rename = "userAvatarPath")]
    user_avatar_path: String,

    #[serde(default, rename = "fontFamilyEnglish")]
    font_family_english: String,
    #[serde(default, rename = "fontFamilyChinese")]
    font_family_chinese: String,

    #[serde(default = "default_chat_mode", rename = "chatMode")]
    chat_mode: ChatModeConfig, // Store chatMode in settings part

    // Legacy support for promptLibrary in settings.json (optional)
    #[serde(default, rename = "promptLibrary")]
    prompt_library: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
struct ProvidersPart {
    #[serde(rename = "providers")]
    providers: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
struct PresetsPart {
    #[serde(rename = "presets")]
    presets: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
struct PromptsPart {
    #[serde(rename = "promptLibrary")]
    prompt_library: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
struct SecretsPart {
    // Map: ProviderID -> EncodedApiKey
    #[serde(default, rename = "secrets")]
    secrets: HashMap<String, String>,
}

// defaults
// defaults
fn default_font_size() -> u32 {
    16
}
fn default_line_ratio() -> f32 {
    1.7
}
fn default_theme_color() -> String {
    "#202124".into()
}
fn default_theme() -> String {
    "dark".into()
}
fn default_dark_theme_id() -> String {
    "dark".into()
}
fn default_light_theme_id() -> String {
    "light".into()
}
fn default_scrollbar_width() -> u32 {
    12
}
fn default_search_url() -> String {
    "https://searx.be".into()
}
fn default_search_provider() -> String {
    "all".into()
}
fn default_providers() -> serde_json::Value {
    serde_json::json!([
        {
            "id": "deepseek",
            "name": "DeepSeek",
            "icon": "deepseek",
            "enabled": true,
            "apiKey": "",
            "baseUrl": "https://api.deepseek.com",
            "models": ["deepseek-chat", "deepseek-reasoner"],
            "defaultModel": "deepseek-chat",
            "temperature": 1.0,
            "maxTokens": 8192
        },
        {
            "id": "openai",
            "name": "OpenAI",
            "icon": "openai",
            "enabled": false,
            "apiKey": "",
            "baseUrl": "https://api.openai.com/v1",
            "models": ["gpt-4", "gpt-4-turbo", "gpt-3.5-turbo"],
            "defaultModel": "gpt-4-turbo",
            "temperature": 0.7,
            "maxTokens": 4096
        },
        {
            "id": "claude",
            "name": "Claude",
            "icon": "anthropic",
            "enabled": false,
            "apiKey": "",
            "baseUrl": "https://api.anthropic.com",
            "models": ["claude-3-opus", "claude-3-sonnet", "claude-3-haiku"],
            "defaultModel": "claude-3-sonnet",
            "temperature": 0.7,
            "maxTokens": 4096
        },
        {
            "id": "gemini",
            "name": "Gemini 3",
            "icon": "google",
            "enabled": false,
            "apiKey": "",
            "baseUrl": "https://generativelanguage.googleapis.com",
            "models": [
                "gemini-3-pro-preview",
                "gemini-3-flash-preview",
                "gemini-3-pro-image-preview"
            ],
            "defaultModel": "gemini-3-pro-preview",
            "temperature": 0.7,
            "maxTokens": 4096
        },
        {
            "id": "ollama",
            "name": "Ollama",
            "icon": "ollama",
            "enabled": false,
            "apiKey": "",
            "baseUrl": "http://localhost:11434",
            "models": ["llama2", "mistral", "codellama"],
            "defaultModel": "llama2",
            "temperature": 0.7,
            "maxTokens": 2048
        },
        {
            "id": "qwen",
            "name": "Qwen",
            "icon": "qwen",
            "enabled": false,
            "apiKey": "",
            "baseUrl": "https://dashscope.aliyuncs.com/api",
            "models": ["qwen-turbo", "qwen-plus", "qwen-max"],
            "defaultModel": "qwen-turbo",
            "temperature": 0.7,
            "maxTokens": 4096
        },
        {
            "id": "siliconflow",
            "name": "SiliconFlow",
            "icon": "siliconflow",
            "enabled": false,
            "apiKey": "",
            "baseUrl": "https://api.siliconflow.cn",
            "models": [
                "deepseek-ai/DeepSeek-V3.2"
            ],
            "defaultModel": "deepseek-ai/DeepSeek-V3.2",
            "temperature": 0.7,
            "maxTokens": 4096
        }
    ])
}
fn default_provider_id() -> String {
    "deepseek".into()
}
fn default_model_id() -> String {
    "deepseek-chat".into()
}
fn default_presets() -> serde_json::Value {
    serde_json::json!([
        {
            "id": "default_preset",
            "name": "默认配置",
            "temperature": 0.7,
            "maxTokens": 4096,
            "systemPrompt": "",
            "isDefault": true
        }
    ])
}
fn default_preset_id() -> String {
    "default_preset".into()
}
fn default_true() -> bool {
    true
}
fn default_false() -> bool {
    false
}

fn default_prompt_library() -> serde_json::Value {
    serde_json::json!([])
}

// Chat Mode Defaults
fn default_chat_mode() -> ChatModeConfig {
    ChatModeConfig {
        enabled: false,
        day_theme_id: "wechat".into(),
        night_theme_id: "dark_plus".into(),
        enable_stream: false,
        enable_loading_bar: false,
    }
}
fn default_weekchat_theme() -> String {
    "wechat".into()
}
fn default_dark_plus_theme() -> String {
    "dark_plus".into()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            font_size: default_font_size(),
            line_ratio: default_line_ratio(),
            theme_color: default_theme_color(),
            theme: default_theme(),
            dark_theme_id: default_dark_theme_id(),
            light_theme_id: default_light_theme_id(),
            scrollbar_width: default_scrollbar_width(),
            api_key: "".into(),
            search_instance_url: default_search_url(),
            default_search_provider: default_search_provider(),
            providers: default_providers(),
            default_provider_id: default_provider_id(),
            selected_model_id: default_model_id(),
            global_model_id: default_model_id(),
            use_reasoning: false,
            use_search: false,
            search_provider: default_search_provider(),
            presets: default_presets(),
            default_preset_id: default_preset_id(),
            global_preset_id: default_preset_id(),
            enable_stream: true,
            enable_bubble: false,
            show_user_avatar: false,
            user_avatar_path: "".into(),
            prompt_library: default_prompt_library(),
            chat_mode: default_chat_mode(),
            font_family_english: "".into(),
            font_family_chinese: "".into(),
        }
    }
}

// ==================================================================================
// logic
// ==================================================================================

/// Resolves the config directory. Priority: Portable EXE/data/config > Stable AppConfig
pub fn resolve_config_dir(_app: &AppHandle) -> PathBuf {
    // --- 1. 定位“便携式”数据目录 ---
    let exe_path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    let exe_dir = exe_path.parent().unwrap_or_else(|| Path::new("."));
    let portable_config_dir = exe_dir.join("data").join("config");

    if !portable_config_dir.exists() {
        let _ = fs::create_dir_all(&portable_config_dir);
    }

    // --- 2. 迁移逻辑 (仅用于兼容旧版 AppConfig 路径) ---
    // 如果稳定目录中存在且便携式目录刚创建/为空，可以考虑搬迁
    // 但根据用户要求，现在强制使用 debug/data 逻辑

    portable_config_dir
}

fn encode_key(key: &str) -> String {
    base64::engine::general_purpose::STANDARD.encode(key)
}

fn decode_key(encoded: &str) -> String {
    String::from_utf8(
        base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .unwrap_or_default(),
    )
    .unwrap_or_default()
}

#[tauri::command]
pub async fn load_config(app: AppHandle) -> Result<AppConfig, String> {
    let config_dir = resolve_config_dir(&app);
    let settings_path = config_dir.join("settings.json");
    let providers_path = config_dir.join("providers.json");
    let presets_path = config_dir.join("presets.json");
    let prompts_path = config_dir.join("prompts.json");
    let secrets_path = config_dir.join("secrets.json");

    // Strategy:
    // 1. Try to load from new split files (portable/local mode).
    // 2. If not found, try to load from legacy standard location (AppData).
    // 3. If legacy exists, MIGRATE it to local mode immediately.
    // 4. Fallback to default.

    // 1. Check if new format exists (checking settings.json is enough as an indicator)
    if settings_path.exists() {
        let settings_content = fs::read_to_string(&settings_path).unwrap_or_else(|_| "{}".into());
        let providers_content =
            fs::read_to_string(&providers_path).unwrap_or_else(|_| "{\"providers\":[]}".into());
        let presets_content =
            fs::read_to_string(&presets_path).unwrap_or_else(|_| "{\"presets\":[]}".into());
        let prompts_content =
            fs::read_to_string(&prompts_path).unwrap_or_else(|_| "{\"promptLibrary\":[]}".into());
        let secrets_content = fs::read_to_string(&secrets_path).unwrap_or_else(|_| "{}".into());

        let settings: SettingsPart = serde_json::from_str(&settings_content)
            .unwrap_or_else(|_| serde_json::from_str("{}").unwrap());
        let mut providers_part: ProvidersPart = serde_json::from_str(&providers_content)
            .unwrap_or_else(|_| serde_json::from_str("{\"providers\":[]}").unwrap());
        let presets_part: PresetsPart = serde_json::from_str(&presets_content)
            .unwrap_or_else(|_| serde_json::from_str("{\"presets\":[]}").unwrap());
        let prompts_part: PromptsPart = serde_json::from_str(&prompts_content)
            .unwrap_or_else(|_| serde_json::from_str("{\"promptLibrary\":[]}").unwrap());
        let secrets_part: SecretsPart = serde_json::from_str(&secrets_content)
            .unwrap_or_else(|_| serde_json::from_str("{}").unwrap());

        // MERGE SECRETS INTO PROVIDERS
        if let serde_json::Value::Array(ref mut providers) = providers_part.providers {
            for provider in providers.iter_mut() {
                if let Some(id_val) = provider.get("id") {
                    if let Some(id) = id_val.as_str() {
                        if let Some(encoded_key) = secrets_part.secrets.get(id) {
                            let decoded = decode_key(encoded_key);
                            if !decoded.is_empty() {
                                provider["apiKey"] = serde_json::Value::String(decoded);
                            }
                        }
                    }
                }
            }

            // 🟢 [New Logic] Merge missing default providers into loaded providers
            let existing_ids: Vec<String> = providers
                .iter()
                .filter_map(|p| p.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()))
                .collect();

            let default_providers_val = default_providers();
            if let serde_json::Value::Array(defaults) = default_providers_val {
                for default_p in defaults {
                    if let Some(default_id) = default_p.get("id").and_then(|v| v.as_str()) {
                        if !existing_ids.contains(&default_id.to_string()) {
                            println!(
                                "[Config] Injecting missing default provider: {}",
                                default_id
                            );
                            providers.push(default_p);
                        }
                    }
                }
            }
        }

        // Merge into complete AppConfig
        let mut config = AppConfig::default();
        config.font_size = settings.font_size;
        config.line_ratio = settings.line_ratio;
        config.theme_color = settings.theme_color;
        config.theme = settings.theme;
        config.dark_theme_id = settings.dark_theme_id;
        config.light_theme_id = settings.light_theme_id;
        config.scrollbar_width = settings.scrollbar_width;
        config.api_key = settings.api_key;
        config.search_instance_url = settings.search_instance_url;
        config.default_search_provider = settings.default_search_provider;
        config.default_provider_id = settings.default_provider_id;
        config.selected_model_id = settings.selected_model_id;
        config.global_model_id = settings.global_model_id;
        config.use_reasoning = settings.use_reasoning;
        config.use_search = settings.use_search;
        config.search_provider = settings.search_provider;
        config.default_preset_id = settings.default_preset_id;
        config.global_preset_id = settings.global_preset_id;
        config.enable_stream = settings.enable_stream;
        config.enable_bubble = settings.enable_bubble;
        config.show_user_avatar = settings.show_user_avatar;
        config.user_avatar_path = settings.user_avatar_path;
        config.chat_mode = settings.chat_mode;
        // Map font settings
        config.font_family_english = settings.font_family_english;
        config.font_family_chinese = settings.font_family_chinese;

        config.providers = providers_part.providers;
        config.presets = presets_part.presets;

        // Prefer separate prompts.json, fallback to settings.json embedded
        config.prompt_library = if let Some(p) = settings.prompt_library {
            // If local prompts.json is empty but settings has it (migration scenario)
            if prompts_part
                .prompt_library
                .as_array()
                .map_or(0, |a| a.len())
                == 0
                && p.as_array().map_or(0, |a| a.len()) > 0
            {
                p
            } else {
                prompts_part.prompt_library
            }
        } else {
            prompts_part.prompt_library
        };

        return Ok(config);
    }

    // --- 2. 移除 C 盘 AppData 回退逻辑，强制使用便携式目录 ---

    // 4. Default
    Ok(AppConfig::default())
}

#[tauri::command]
pub async fn save_config(app: AppHandle, mut config: AppConfig) -> Result<(), String> {
    let config_dir = resolve_config_dir(&app);
    // ensure_config_dir is redundant now since resolve_config_dir/create_dir_all handles it
    // but we can add a check if we want to be safe
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;
    }

    println!("[Config] Saving to {:?}", config_dir);

    // Split and save

    // 1. EXTRACT & SANITIZE SECRETS
    let mut secrets_map = HashMap::new();

    if let serde_json::Value::Array(ref mut providers) = config.providers {
        for provider in providers {
            if let Some(id_val) = provider.get("id") {
                if let Some(id) = id_val.as_str() {
                    // Extract API Key
                    if let Some(key_val) = provider.get("apiKey") {
                        if let Some(key) = key_val.as_str() {
                            if !key.trim().is_empty() {
                                secrets_map.insert(id.to_string(), encode_key(key));
                            }
                        }
                    }
                    // WIPE API Key from the struct being saved to providers.json
                    provider["apiKey"] = serde_json::Value::String("".to_string());
                }
            }
        }
    }

    // 2. Save Providers (Sanitized)
    let providers_part = ProvidersPart {
        providers: config.providers.clone(),
    };
    let providers_json =
        serde_json::to_string_pretty(&providers_part).map_err(|e| e.to_string())?;
    fs::write(config_dir.join("providers.json"), providers_json).map_err(|e| e.to_string())?;

    // 3. Save Secrets
    let secrets_part = SecretsPart {
        secrets: secrets_map,
    };
    let secrets_json = serde_json::to_string_pretty(&secrets_part).map_err(|e| e.to_string())?;
    fs::write(config_dir.join("secrets.json"), secrets_json).map_err(|e| e.to_string())?;

    // 4. Presets
    let presets_part = PresetsPart {
        presets: config.presets.clone(),
    };
    let presets_json = serde_json::to_string_pretty(&presets_part).map_err(|e| e.to_string())?;
    fs::write(config_dir.join("presets.json"), presets_json).map_err(|e| e.to_string())?;

    // 5. Prompts
    let prompts_part = PromptsPart {
        prompt_library: config.prompt_library.clone(),
    };
    let prompts_json = serde_json::to_string_pretty(&prompts_part).map_err(|e| e.to_string())?;
    fs::write(config_dir.join("prompts.json"), prompts_json).map_err(|e| e.to_string())?;

    // 6. Settings (Remainder)
    let settings_part = SettingsPart {
        font_size: config.font_size,
        line_ratio: config.line_ratio,
        theme_color: config.theme_color,
        theme: config.theme.clone(),
        dark_theme_id: config.dark_theme_id.clone(),
        light_theme_id: config.light_theme_id.clone(),
        scrollbar_width: config.scrollbar_width,
        api_key: config.api_key,
        search_instance_url: config.search_instance_url,
        default_search_provider: config.default_search_provider,
        default_provider_id: config.default_provider_id,
        selected_model_id: config.selected_model_id,
        global_model_id: config.global_model_id,
        use_reasoning: config.use_reasoning,
        use_search: config.use_search,
        search_provider: config.search_provider,
        default_preset_id: config.default_preset_id,
        global_preset_id: config.global_preset_id,
        enable_stream: config.enable_stream,
        enable_bubble: config.enable_bubble,
        show_user_avatar: config.show_user_avatar,
        user_avatar_path: config.user_avatar_path,
        chat_mode: config.chat_mode,
        font_family_english: config.font_family_english,
        font_family_chinese: config.font_family_chinese,
        prompt_library: None, // No longer saving here to avoid duplication
    };
    let settings_json = serde_json::to_string_pretty(&settings_part).map_err(|e| e.to_string())?;
    fs::write(config_dir.join("settings.json"), settings_json).map_err(|e| e.to_string())?;

    Ok(())
}
