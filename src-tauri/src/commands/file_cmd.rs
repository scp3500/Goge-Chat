use std::fs;
use tauri_plugin_opener::OpenerExt;

#[tauri::command]
pub async fn open_file(app: tauri::AppHandle, path: String) -> Result<(), String> {
    println!("📂 Rust: Opening file: {}", path);
    // 使用 tauri-plugin-opener 的扩展方法
    app.opener()
        .open_path(path.as_str(), None::<&str>)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn read_file_text_content(path: String) -> Result<String, String> {
    println!("📖 Rust: Reading file: {}", path);
    fs::read_to_string(path).map_err(|e| e.to_string())
}
