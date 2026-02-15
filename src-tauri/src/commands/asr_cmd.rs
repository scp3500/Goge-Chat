use futures_util::StreamExt;
use once_cell::sync::OnceCell;
// use sherpa_rs::sense_voice::{SenseVoiceConfig, SenseVoiceRecognizer};
use sherpa_rs::paraformer::{ParaformerConfig, ParaformerRecognizer};
use std::fs::File;
use std::io::Write;
use std::sync::Mutex;
use std::time::Instant;
use tauri::{AppHandle, Emitter, Manager};

// --- 1. 定义全局状态 (Switch to ParaformerRecognizer for simplified 0.6.8+ API) ---
static RECOGNIZER: OnceCell<Mutex<ParaformerRecognizer>> = OnceCell::new();

// --- 2. 内部帮助函数：获取或初始化模型 ---
fn get_asr_model_dir(app_handle: &AppHandle) -> Result<std::path::PathBuf, String> {
    let resource_dir = app_handle
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to get resource dir: {}", e))?;

    // 🚀 [路径修正]
    // 在 Tauri v2 Windows 平台，resource_dir() 指向的是包含 .exe 的安装根目录。
    // 为了将自动下载的模型也放在资源目录下，我们需要统一 join("resources")。
    let target_dir = resource_dir.join("resources").join("asr_model");

    Ok(target_dir)
}

fn get_recognizer(app_handle: &AppHandle) -> Result<&'static Mutex<ParaformerRecognizer>, String> {
    RECOGNIZER.get_or_try_init(|| {
        println!("[ASR] Initializing Paraformer-Large model...");
        let start_time = Instant::now();

        // 1. 智能获取资源路径
        let resource_path = get_asr_model_dir(app_handle)?;

        // 2. 检查关键文件是否存在并验证大小
        let model_path = resource_path.join("model.int8.onnx");
        let tokens_path = resource_path.join("tokens.txt");

        if !model_path.exists() || !tokens_path.exists() {
            return Err(format!("Missing ASR files in: {:?}", resource_path));
        }

        // 验证文件大小 (Paraformer Large int8 约 220MB)
        let model_size = std::fs::metadata(&model_path).map(|m| m.len()).unwrap_or(0);
        println!("[ASR] Model file size: {} bytes", model_size);
        if model_size < 100_000_000 {
            return Err(format!(
                "ASR model file is too small ({}), possibly corrupted.",
                model_size
            ));
        }

        let model_path_str = model_path.to_string_lossy().to_string();
        let tokens_path_str = tokens_path.to_string_lossy().to_string();

        // 3. 配置 Paraformer (0.6.8 扁平化 API)
        let config = ParaformerConfig {
            model: model_path_str,
            tokens: tokens_path_str,
            num_threads: Some(1),
            debug: true,
            ..Default::default()
        };

        let recognizer = ParaformerRecognizer::new(config)
            .map_err(|e| format!("Paraformer Load Error: {}", e))?;

        let elapsed = start_time.elapsed();
        println!("[ASR] Paraformer loaded in {:.2?}", elapsed);

        Ok(Mutex::new(recognizer))
    })
}

// --- 3. Tauri Command ---

#[tauri::command]
pub async fn transcribe_pcm(
    app_handle: AppHandle,
    samples: Vec<f32>,
    sample_rate: u32,
) -> Result<String, String> {
    let func_start = Instant::now();
    println!(
        "[ASR] Received {} samples at {}Hz",
        samples.len(),
        sample_rate
    );

    // --- Check Audio Signal Quality ---
    if samples.is_empty() {
        return Err("Received empty audio samples".to_string());
    }

    let max_amp = samples.iter().fold(0.0f32, |max, &x| max.max(x.abs()));
    let avg_amp = samples.iter().map(|x| x.abs()).sum::<f32>() / samples.len() as f32;
    println!(
        "[ASR] Audio Stats: Max Amp: {:.4}, Avg Amp: {:.4}",
        max_amp, avg_amp
    );

    if max_amp < 0.001 {
        println!("[ASR] ⚠️ WARNING: Audio seems silent!");
    }
    // ----------------------------------

    let result = tokio::task::spawn_blocking(move || {
        let recognizer_mutex = get_recognizer(&app_handle)?;

        let mut recognizer = recognizer_mutex
            .lock()
            .map_err(|_| "Failed to lock recognizer")?;

        // 0.6.8 Paraformer API: transcribe (sync in blocking task)
        let decode_start = Instant::now();
        let text = recognizer.transcribe(sample_rate, &samples).text;
        let decode_time = decode_start.elapsed();

        // 打印性能日志
        let total_time = func_start.elapsed();
        let audio_duration = samples.len() as f32 / sample_rate as f32;
        let rtf = total_time.as_secs_f32() / audio_duration;

        println!(
            "[ASR] 🟢 Done! \n\
             \t Audio Duration: {:.2}s\n\
             \t Decode Time:    {:.2?}\n\
             \t Total Time:     {:.2?}\n\
             \t RTF:            {:.4} (Lower is better)\n\
             \t Result:         \"{}\"",
            audio_duration, decode_time, total_time, rtf, text
        );

        Ok::<String, String>(text)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))??;

    Ok(result)
}

#[derive(Clone, serde::Serialize)]
struct DownloadProgress {
    file: String,
    total: u64,
    downloaded: u64,
    percent: f64,
}

#[tauri::command]
pub async fn download_asr_model(app_handle: AppHandle) -> Result<String, String> {
    println!("[ASR] Starting on-demand model download...");

    // 1. 确定目标路径 (使用统一 helper)
    let target_dir = get_asr_model_dir(&app_handle)?;
    if !target_dir.exists() {
        std::fs::create_dir_all(&target_dir)
            .map_err(|e| format!("Failed to create ASR directory: {}", e))?;
    }

    // 2. 定义下载任务列表 (文件名 -> URL -> 最小期望大小)
    // 使用 hf-mirror.com 镜像
    let files = vec![
        ("tokens.txt", "https://hf-mirror.com/csukuangfj/sherpa-onnx-paraformer-zh-2023-09-14/resolve/main/tokens.txt", 10),
        ("model.int8.onnx", "https://hf-mirror.com/csukuangfj/sherpa-onnx-paraformer-zh-2023-09-14/resolve/main/model.int8.onnx", 100_000_000),
    ];

    let client = reqwest::Client::new();

    for (filename, url, min_size) in files {
        let file_path = target_dir.join(filename);

        // 🚀 [严格校验] 如果文件已存在且大小符合预期(防止下载了半截或空的损坏文件)
        if file_path.exists() {
            let metadata = std::fs::metadata(&file_path).map_err(|e| e.to_string())?;
            if metadata.len() > min_size {
                println!(
                    "[ASR] File {} exists and size is valid ({}), skipping.",
                    filename,
                    metadata.len()
                );
                continue;
            } else {
                println!(
                    "[ASR] File {} exists but is too small ({} < {}), re-downloading...",
                    filename,
                    metadata.len(),
                    min_size
                );
            }
        }

        println!("[ASR] Downloading {} from {}...", filename, url);

        // 3. 执行下载请求 & 校验状态码
        let res = client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("Request failed for {}: {}", filename, e))?
            .error_for_status() // 👈 关键：检查 404/500 等错误
            .map_err(|e| format!("HTTP error for {}: {}", filename, e))?;

        // 4. 获取大小 (如果服务器不给 Content-Length，我们设为 0 并作为不确定进度处理)
        let total_size = res.content_length().unwrap_or(0);

        let mut file = File::create(&file_path)
            .map_err(|e| format!("Failed to create file {}: {}", filename, e))?;

        let mut downloaded: u64 = 0;
        let mut stream = res.bytes_stream();

        while let Some(item) = stream.next().await {
            let chunk = item.map_err(|e| format!("Error while downloading chunk: {}", e))?;
            file.write_all(&chunk)
                .map_err(|e| format!("Error while writing to file: {}", e))?;

            downloaded += chunk.len() as u64;

            // 计算进度 (防止除以 0)
            let percent = if total_size > 0 {
                (downloaded as f64 / total_size as f64) * 100.0
            } else {
                0.0 // 无法预估总大小时，保持 0 (或者前端可以设为 indeterminate)
            };

            let _ = app_handle.emit(
                "ASR_DOWNLOAD_PROGRESS",
                DownloadProgress {
                    file: filename.to_string(),
                    total: total_size,
                    downloaded,
                    percent,
                },
            );
        }

        println!("[ASR] Downloaded {}", filename);
    }

    Ok("ASR model download completed successfully!".to_string())
}

#[tauri::command]
pub async fn check_asr_model_status(app_handle: AppHandle) -> Result<String, String> {
    let target_dir = match get_asr_model_dir(&app_handle) {
        Ok(dir) => dir,
        Err(e) => {
            println!("[ASR Check] Failed to get dir: {}", e);
            return Ok(format!("ERROR_GET_DIR: {}", e));
        }
    };

    println!("[ASR Check] Checking directory: {:?}", target_dir);

    if !target_dir.exists() {
        return Ok(format!("DIR_MISSING: {:?}", target_dir));
    }

    let files = vec![("tokens.txt", 10), ("model.int8.onnx", 100_000_000)];

    for (filename, min_size) in files {
        let file_path = target_dir.join(filename);
        if !file_path.exists() {
            return Ok(format!("FILE_MISSING: {:?}", file_path));
        }
        let metadata = std::fs::metadata(&file_path).map_err(|e| e.to_string())?;
        if metadata.len() < min_size {
            return Ok(format!(
                "FILE_TOO_SMALL: {:?} ({} < {})",
                filename,
                metadata.len(),
                min_size
            ));
        }
    }

    println!("[ASR Check] All files valid -> READY");
    Ok("READY".to_string())
}
