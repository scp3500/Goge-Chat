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
    _app: tauri::AppHandle,
    file_path: String,
) -> Result<String, String> {
    use crate::commands::config_cmd::get_local_config_dir;
    use std::path::Path;

    println!("🖼️ Rust: Uploading avatar from: {}", file_path);

    let config_dir = get_local_config_dir();
    let upload_dir = config_dir.join("upload");

    if !upload_dir.exists() {
        fs::create_dir_all(&upload_dir)
            .map_err(|e| format!("Failed to create upload dir: {}", e))?;
    }

    let source_path = Path::new(&file_path);
    let extension = source_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png");

    // Generate a unique filename or use a fixed one "user_avatar.png"
    // to avoid accumulation? User said "upload", implies we might want to keep it.
    // But for a single user avatar, a fixed name or timestamped name is fine.
    // Let's use timestamp to avoid caching issues.
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let target_filename = format!("avatar_{}.{}", timestamp, extension);
    let target_path = upload_dir.join(&target_filename);

    fs::copy(&source_path, &target_path).map_err(|e| format!("Failed to copy file: {}", e))?;

    // Return the relative path or absolute path?
    // Since we serve static files or read them, returning absolute path might be easier for now,
    // but relative to config dir is more portable.
    // Let's return the absolute path for immediate display use.
    Ok(target_path.to_string_lossy().to_string())
}
