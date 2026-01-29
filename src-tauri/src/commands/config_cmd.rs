use serde::{Deserialize, Serialize};
use std::fs;
// 必须保留 Manager，因为它提供了 app.path() 扩展方法
use tauri::{path::BaseDirectory, AppHandle, Manager};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    #[serde(default = "default_font_size", rename = "fontSize")]
    pub font_size: u32,
    #[serde(default = "default_line_ratio", rename = "lineRatio")]
    pub line_ratio: f32,
    #[serde(default = "default_theme_color", rename = "themeColor")]
    pub theme_color: String,
    #[serde(default = "default_scrollbar_width", rename = "scrollbarWidth")]
    pub scrollbar_width: u32,
    #[serde(default, rename = "apiKey")]
    pub api_key: String,
    #[serde(default = "default_search_url", rename = "searchInstanceUrl")]
    pub search_instance_url: String,
    #[serde(default = "default_search_provider", rename = "defaultSearchProvider")]
    pub default_search_provider: String,
    #[serde(default = "default_providers", rename = "providers")]
    pub providers: serde_json::Value,
    #[serde(default = "default_provider_id", rename = "defaultProviderId")]
    pub default_provider_id: String,
    #[serde(default = "default_model_id", rename = "selectedModelId")]
    pub selected_model_id: String,
    #[serde(default, rename = "useReasoning")]
    pub use_reasoning: bool,
    #[serde(default, rename = "useSearch")]
    pub use_search: bool,
    #[serde(default = "default_search_provider", rename = "searchProvider")]
    pub search_provider: String,
}

fn default_font_size() -> u32 {
    16
}
fn default_line_ratio() -> f32 {
    1.7
}
fn default_theme_color() -> String {
    "#202124".into()
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
    serde_json::json!([])
}
fn default_provider_id() -> String {
    "deepseek".into()
}
fn default_model_id() -> String {
    "deepseek-chat".into()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            font_size: default_font_size(),
            line_ratio: default_line_ratio(),
            theme_color: default_theme_color(),
            scrollbar_width: default_scrollbar_width(),
            api_key: "".into(),
            search_instance_url: default_search_url(),
            default_search_provider: default_search_provider(),
            providers: default_providers(),
            default_provider_id: default_provider_id(),
            selected_model_id: default_model_id(),
            use_reasoning: false,
            use_search: false,
            search_provider: default_search_provider(),
        }
    }
}

// 辅助函数：内部使用，不需要 pub
fn get_config_path(app: &AppHandle) -> std::path::PathBuf {
    // 这里的 .path() 方法必须配合 use tauri::Manager 才能使用
    app.path()
        .resolve("config.json", BaseDirectory::AppConfig)
        .expect("无法获取配置目录")
}

#[tauri::command]
pub async fn load_config(app: AppHandle) -> Result<AppConfig, String> {
    let path = get_config_path(&app);

    if !path.exists() {
        return Ok(AppConfig::default());
    }

    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_config(app: AppHandle, config: AppConfig) -> Result<(), String> {
    let path = get_config_path(&app);

    // 确保目录存在
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let content = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())?;
    Ok(())
}
