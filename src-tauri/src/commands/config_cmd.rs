use serde::{Deserialize, Serialize};
use std::fs;
// 必须保留 Manager，因为它提供了 app.path() 扩展方法
use tauri::{path::BaseDirectory, AppHandle, Manager};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    #[serde(rename = "fontSize")]
    pub font_size: u32,
    #[serde(rename = "lineRatio")]
    pub line_ratio: f32,
    #[serde(rename = "themeColor")]
    pub theme_color: String,
    #[serde(rename = "scrollbarWidth")]
    pub scrollbar_width: u32,
    #[serde(rename = "apiKey")]
    pub api_key: String,
    #[serde(rename = "searchInstanceUrl")]
    pub search_instance_url: String,
    #[serde(rename = "defaultSearchProvider")]
    pub default_search_provider: String,
    #[serde(rename = "providers")]
    pub providers: serde_json::Value,
    #[serde(rename = "defaultProviderId")]
    pub default_provider_id: String,
    #[serde(rename = "selectedModelId")]
    pub selected_model_id: String,
    #[serde(rename = "useReasoning")]
    pub use_reasoning: bool,
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
        return Ok(AppConfig {
            font_size: 16,
            line_ratio: 1.7,
            theme_color: "#202124".into(),
            scrollbar_width: 12,
            api_key: "".into(),
            search_instance_url: "https://searx.be".into(),
            default_search_provider: "all".into(),
            providers: serde_json::json!([]),
            default_provider_id: "deepseek".into(),
            selected_model_id: "deepseek-chat".into(),
            use_reasoning: false,
        });
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
