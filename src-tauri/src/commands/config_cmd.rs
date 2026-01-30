use base64::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{path::BaseDirectory, AppHandle, Manager};

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
    "nord".into()
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
        }
    }
}

// ==================================================================================
// logic
// ==================================================================================

/// Resolves the config directory relative to the executable.
/// Usually: <exe_dir>/config/
fn get_local_config_dir() -> PathBuf {
    let exe_path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    // If we are in a dev environment (e.g. target/debug/), this still places it next to the exe
    let exe_dir = exe_path.parent().unwrap_or_else(|| Path::new("."));
    exe_dir.join("config")
}

/// Helper to ensure the config directory exists
fn ensure_config_dir(dir: &Path) -> Result<(), String> {
    if !dir.exists() {
        fs::create_dir_all(dir).map_err(|e| format!("Failed to create config directory: {}", e))?;
    }
    Ok(())
}

fn encode_key(key: &str) -> String {
    BASE64_STANDARD.encode(key)
}

fn decode_key(encoded: &str) -> String {
    String::from_utf8(BASE64_STANDARD.decode(encoded).unwrap_or_default()).unwrap_or_default()
}

#[tauri::command]
pub async fn load_config(app: AppHandle) -> Result<AppConfig, String> {
    let config_dir = get_local_config_dir();
    let settings_path = config_dir.join("settings.json");
    let providers_path = config_dir.join("providers.json");
    let presets_path = config_dir.join("presets.json");
    let secrets_path = config_dir.join("secrets.json");

    // Strategy:
    // 1. Try to load from new split files (portable/local mode).
    // 2. If not found, try to load from legacy standard location (AppData).
    // 3. If legacy exists, MIGRATE it to local mode immediately.
    // 4. Fallback to default.

    // 1. Check if new format exists (checking settings.json is enough as an indicator)
    if settings_path.exists() {
        println!("[Config] Loading split config from {:?}", config_dir);
        let settings_content = fs::read_to_string(&settings_path).unwrap_or_else(|_| "{}".into());
        let providers_content =
            fs::read_to_string(&providers_path).unwrap_or_else(|_| "{\"providers\":[]}".into());
        let presets_content =
            fs::read_to_string(&presets_path).unwrap_or_else(|_| "{\"presets\":[]}".into());
        let secrets_content = fs::read_to_string(&secrets_path).unwrap_or_else(|_| "{}".into());

        let settings: SettingsPart = serde_json::from_str(&settings_content)
            .unwrap_or_else(|_| serde_json::from_str("{}").unwrap());
        let mut providers_part: ProvidersPart = serde_json::from_str(&providers_content)
            .unwrap_or_else(|_| serde_json::from_str("{\"providers\":[]}").unwrap());
        let presets_part: PresetsPart = serde_json::from_str(&presets_content)
            .unwrap_or_else(|_| serde_json::from_str("{\"presets\":[]}").unwrap());
        let secrets_part: SecretsPart = serde_json::from_str(&secrets_content)
            .unwrap_or_else(|_| serde_json::from_str("{}").unwrap());

        // MERGE SECRETS INTO PROVIDERS
        if let serde_json::Value::Array(ref mut providers) = providers_part.providers {
            for provider in providers {
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

        config.providers = providers_part.providers;
        config.presets = presets_part.presets;

        return Ok(config);
    }

    // 2. Fallback: Check legacy AppData config
    // Note: app.path() requires `tauri::Manager`
    let legacy_config_path = app
        .path()
        .resolve("config.json", BaseDirectory::AppConfig)
        .ok();

    if let Some(path) = legacy_config_path {
        if path.exists() {
            println!("[Config] Found legacy config at {:?}, migrating...", path);
            match fs::read_to_string(&path) {
                Ok(content) => {
                    match serde_json::from_str::<AppConfig>(&content) {
                        Ok(mut old_config) => {
                            // Inject default presets if missing (legacy file won't have it, but Struct default does)
                            // Actually serde default handles it, but let's be safe
                            if old_config.default_preset_id.is_empty() {
                                old_config.default_preset_id = default_preset_id();
                            }

                            // 3. Migrate: Save to new location
                            // We don't delete the old file for safety, just ignore it next time
                            save_config(app.clone(), old_config.clone()).await?;
                            return Ok(old_config);
                        }
                        Err(e) => println!("[Config] Failed to parse legacy config: {}", e),
                    }
                }
                Err(e) => println!("[Config] Failed to read legacy config: {}", e),
            }
        }
    }

    // 4. Default
    Ok(AppConfig::default())
}

#[tauri::command]
pub async fn save_config(_app: AppHandle, mut config: AppConfig) -> Result<(), String> {
    let config_dir = get_local_config_dir();
    ensure_config_dir(&config_dir)?;

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

    // 5. Settings (Remainder)
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
    };
    let settings_json = serde_json::to_string_pretty(&settings_part).map_err(|e| e.to_string())?;
    fs::write(config_dir.join("settings.json"), settings_json).map_err(|e| e.to_string())?;

    Ok(())
}
