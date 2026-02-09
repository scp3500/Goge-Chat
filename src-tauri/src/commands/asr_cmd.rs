use once_cell::sync::OnceCell;
use sherpa_rs::sense_voice::{SenseVoiceConfig, SenseVoiceRecognizer};
use std::sync::Mutex;
use std::time::Instant;
use tauri::{AppHandle, Manager};

// --- 1. 定义全局状态 ---

// 使用 OnceCell 确保模型只加载一次
static RECOGNIZER: OnceCell<Mutex<SenseVoiceRecognizer>> = OnceCell::new();

// --- 2. 内部帮助函数：获取或初始化模型 ---
fn get_recognizer(app_handle: &AppHandle) -> Result<&'static Mutex<SenseVoiceRecognizer>, String> {
    RECOGNIZER.get_or_try_init(|| {
        println!("[ASR] Initializing Sherpa-Onnx Paraformer model...");
        let start_time = Instant::now();

        // 1. 获取资源路径
        let resource_path = app_handle
            .path()
            .resource_dir()
            .map_err(|e| format!("Failed to get resource dir: {}", e))?
            .join("resources")
            .join("asr_model");

        println!("[ASR] Looking for model resources at: {:?}", resource_path);

        // 2. 检查关键文件是否存在
        let encoder_path = resource_path.join("model.int8.onnx");
        let tokens_path = resource_path.join("tokens.txt");

        if !encoder_path.exists() {
            return Err(format!("Model file not found at: {:?}", encoder_path));
        }
        if !tokens_path.exists() {
            return Err(format!("Tokens file not found at: {:?}", tokens_path));
        }

        println!("[ASR] Found model files at: {:?}", resource_path);

        // 3. 配置模型 (适配 SenseVoice)
        // fix: Windows paths starting with \\?\ can crash C++ libs
        // Also normalize keys to forward slashes for C++ compatibility
        let encoder_path_str = encoder_path
            .to_string_lossy()
            .to_string()
            .replace("\\\\?\\", "")
            .replace("\\", "/");
        let tokens_path_str = tokens_path
            .to_string_lossy()
            .to_string()
            .replace("\\\\?\\", "")
            .replace("\\", "/");

        // Verify file integrity (basic check)
        let encoder_meta = std::fs::metadata(&encoder_path_str)
            .map_err(|e| format!("Failed to read model metadata: {}", e))?;
        if encoder_meta.len() < 10 * 1024 * 1024 {
            // < 10MB
            return Err(format!(
                "Model file seems too small ({:?} bytes). Please check if it downloaded correctly.",
                encoder_meta.len()
            ));
        }

        println!("[ASR] Initializing SenseVoice with normalized paths:");
        println!(
            "[ASR]   Model:  '{}' (Size: {} bytes)",
            encoder_path_str,
            encoder_meta.len()
        );
        println!("[ASR]   Tokens: '{}'", tokens_path_str);

        let config = SenseVoiceConfig {
            model: encoder_path_str,
            tokens: tokens_path_str,
            num_threads: Some(4),
            debug: true,
            use_itn: true,
            language: "zh".to_string(), // Explicitly set language to avoid empty string issues
            provider: None,
        };

        println!("[ASR] Calling SenseVoiceRecognizer::new()...");
        // 4. 加载模型
        let recognizer = SenseVoiceRecognizer::new(config)
            .map_err(|e| format!("Sherpa-Onnx Crash/Error: {}", e))?;

        let elapsed = start_time.elapsed();
        println!("[ASR] Model loaded successfully in {:.2?}", elapsed);

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

        // 0.6.8 版本直接使用 transcribe 方法
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
