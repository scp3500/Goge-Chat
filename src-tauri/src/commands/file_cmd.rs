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

#[tauri::command]
pub async fn upload_user_avatar(
    app: tauri::AppHandle,
    file_path: String,
) -> Result<String, String> {
    use crate::commands::config_cmd::resolve_config_dir;
    use std::path::Path;

    let config_dir = resolve_config_dir(&app);
    let upload_dir = config_dir.join("upload");

    if !upload_dir.exists() {
        fs::create_dir_all(&upload_dir)
            .map_err(|e| format!("Failed to create upload dir: {}", e))?;
    }

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    if file_path.starts_with("data:image/") {
        // Handle base64 data
        println!("🖼️ Rust: Saving avatar from base64 data");
        let parts: Vec<&str> = file_path.split(',').collect();
        if parts.len() < 2 {
            return Err("Invalid base64 data".to_string());
        }

        // Extract extension from "data:image/png;base64"
        let header = parts[0];
        let extension = if header.contains("/jpeg") || header.contains("/jpg") {
            "jpg"
        } else if header.contains("/webp") {
            "webp"
        } else {
            "png"
        };

        let base64_data = parts[1];
        use base64::{engine::general_purpose, Engine as _};
        let bytes = general_purpose::STANDARD
            .decode(base64_data)
            .map_err(|e| format!("Failed to decode base64: {}", e))?;

        let target_filename = format!("avatar_{}.{}", timestamp, extension);
        let target_path = upload_dir.join(&target_filename);

        fs::write(&target_path, bytes).map_err(|e| format!("Failed to write file: {}", e))?;
        Ok(target_path.to_string_lossy().to_string())
    } else {
        // Handle physical file path
        println!("🖼️ Rust: Uploading avatar from: {}", file_path);
        let source_path = Path::new(&file_path);
        let extension = source_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("png");

        let target_filename = format!("avatar_{}.{}", timestamp, extension);
        let target_path = upload_dir.join(&target_filename);

        fs::copy(&source_path, &target_path).map_err(|e| format!("Failed to copy file: {}", e))?;
        Ok(target_path.to_string_lossy().to_string())
    }
}
#[tauri::command]
pub async fn read_file_base64(path: String) -> Result<String, String> {
    use base64::{engine::general_purpose, Engine as _};
    println!("📖 Rust: Reading binary file to base64: {}", path);
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    Ok(general_purpose::STANDARD.encode(bytes))
}
