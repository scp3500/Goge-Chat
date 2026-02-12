use once_cell::sync::OnceCell;
use sherpa_rs::sense_voice::{SenseVoiceConfig, SenseVoiceRecognizer};
use std::sync::Mutex;
use std::time::Instant;
use tauri::{AppHandle, Manager};

// --- 1. 定义全局状态 ---
static RECOGNIZER: OnceCell<Mutex<SenseVoiceRecognizer>> = OnceCell::new();

// --- 2. 内部帮助函数：获取或初始化模型 ---
fn get_recognizer(app_handle: &AppHandle) -> Result<&'static Mutex<SenseVoiceRecognizer>, String> {
    RECOGNIZER.get_or_try_init(|| {
        println!("[ASR] Initializing SenseVoice Small model...");
        let start_time = Instant::now();

        // 1. 智能获取资源路径
        let resource_path = {
            let res_dir = app_handle
                .path()
                .resource_dir()
                .map_err(|e| format!("Failed to get resource dir: {}", e))?;

            let prod_path = res_dir.join("asr_model");
            if prod_path.exists() {
                prod_path
            } else {
                let nested_path = res_dir.join("resources").join("asr_model");
                if nested_path.exists() {
                    nested_path
                } else {
                    let exe_path = std::env::current_exe().unwrap_or_default();
                    let dev_path = exe_path
                        .parent()
                        .and_then(|p| p.parent())
                        .and_then(|p| p.parent())
                        .map(|p| p.join("resources").join("asr_model"))
                        .unwrap_or_else(|| prod_path.clone());
                    dev_path
                }
            }
        };

        // 2. 检查关键文件是否存在
        let encoder_path = resource_path.join("model.int8.onnx");
        let tokens_path = resource_path.join("tokens.txt");

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

        let config = SenseVoiceConfig {
            model: encoder_path_str,
            tokens: tokens_path_str,
            num_threads: Some(4),
            debug: true,
            use_itn: true,
            language: "zh".to_string(),
            provider: None,
        };

        let recognizer = SenseVoiceRecognizer::new(config)
            .map_err(|e| format!("SenseVoice Load Error: {}", e))?;

        let elapsed = start_time.elapsed();
        println!("[ASR] SenseVoice loaded in {:.2?}", elapsed);

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
