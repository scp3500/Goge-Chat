use futures_util::StreamExt;
use once_cell::sync::Lazy;
use reqwest::Client;
// use serde::Serialize;
use std::fs;
use std::sync::Mutex;
use tauri::command;
use tokio::io::AsyncWriteExt;

// --- 配置 ---
const GENIE_TTS_BASE_URL: &str = "http://127.0.0.1:9880";

static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| Client::new());
static REQUEST_COUNTER: Lazy<Mutex<u64>> = Lazy::new(|| Mutex::new(0));

// --- 数据结构 ---

#[derive(serde::Serialize, Clone)]
#[serde(tag = "type", content = "data")]
pub enum TtsEvent {
    #[serde(rename = "chunk")]
    Chunk(Vec<u8>),
    #[serde(rename = "done")]
    Done,
    #[serde(rename = "metadata")]
    Metadata { backend_prep_ms: u64 },
}

/// 获取下一个请求 ID
#[command]
pub fn next_tts_request_id() -> u64 {
    let mut counter = REQUEST_COUNTER.lock().unwrap();
    *counter += 1;
    println!("[TTS] [信息] 新请求 ID: {}", *counter);
    *counter
}

#[command]
pub async fn generate_tts(
    text: String,
    request_id: u64,
    sequence_id: u32,
    on_event: tauri::ipc::Channel<TtsEvent>, // 🚀 [优化] 使用 Channel 进行二进制直连
) -> Result<String, String> {
    // 检查请求是否过期
    {
        let current_id = REQUEST_COUNTER.lock().unwrap();
        if request_id < *current_id {
            return Err("REQUEST_OBSOLETE".to_string());
        }
    }

    if sequence_id == 0 {
        println!(
            "[TTS] [开始] 生成 (ID: {}, 序号: {}): [{}]",
            request_id, sequence_id, text
        );
    }

    // 🚀 [关键修复] 适配 api.py，且不再发送参考音频参数，让服务器使用命令行指定的默认值 (-dr, -dt, -dl)
    // 显式请求 media_type=raw 以获得最纯粹的 PCM 流，方便前端直接播放
    let params = [
        ("text", text.as_str()),
        ("text_language", "zh"),
        ("device", "cuda"),
        ("media_type", "raw"),
        ("streaming_mode", "true"), // 🚀 [优化] 开启流式模式，降低首包延迟
        ("min_chunk_length", "1"),  // 🚀 [极限] 强制最小包发送
    ];

    // 发起 GET 请求
    let response = HTTP_CLIENT
        .get(GENIE_TTS_BASE_URL)
        .query(&params)
        .send()
        .await
        .map_err(|e| format!("连接 External GPT-SoVITS 失败: {}", e))?;

    if !response.status().is_success() {
        let err_msg = response.text().await.unwrap_or_else(|_| "未知错误".into());
        return Err(format!("External API 响应错误: {}", err_msg));
    }

    // 🚀 [真正流式] 处理字节流并实时推送
    let mut stream = response.bytes_stream();
    let mut all_audio_bytes = Vec::new();
    let start_time = std::time::Instant::now(); // 🚀 [性能监测] 记录开始接收流的时间
    let mut is_first_chunk = true;

    while let Some(chunk_result) = stream.next().await {
        // ... (cancellation check remains same)
        {
            let current_id = REQUEST_COUNTER.lock().unwrap();
            if request_id < *current_id {
                println!("[TTS] [停止] 过期请求 ID {}", request_id);
                break;
            }
        }

        let chunk = chunk_result.map_err(|e| format!("读取流数据失败: {}", e))?;
        if chunk.is_empty() {
            continue;
        }

        // 🚀 [性能监测] 记录首包耗时 (Time to First Byte)
        if is_first_chunk && sequence_id == 0 {
            let backend_prep_ms = start_time.elapsed().as_millis() as u64;
            println!("[性能] TTS 后端准备: {}ms", backend_prep_ms);
            let _ = on_event.send(TtsEvent::Metadata { backend_prep_ms });
            is_first_chunk = false;
        }

        // 累积完整音频用于异步存盘
        all_audio_bytes.extend_from_slice(&chunk);

        // 🚀 [关键一步] 二进制直连传输 (不再使用 Base64)
        let _ = on_event.send(TtsEvent::Chunk(chunk.to_vec()));
    }

    // 发送结束标记
    let _ = on_event.send(TtsEvent::Done);

    // 🏆 [异步静默存盘] 保持不变，用于缓存
    let audio_bytes_clone = all_audio_bytes.clone();
    tokio::spawn(async move {
        // 🚀 [路径优化] 将缓存放在 exe 同级目录的 data 文件夹下
        let exe_path = std::env::current_exe().unwrap_or_default();
        let exe_dir = exe_path.parent().unwrap_or(&exe_path);
        let tts_cache_dir = exe_dir.join("data").join("tts_cache");

        if !tts_cache_dir.exists() {
            let _ = fs::create_dir_all(&tts_cache_dir);
        }

        let filename = format!("genie_tts_{}_{}.wav", request_id, sequence_id);
        let file_path = tts_cache_dir.join(filename);

        // 如果想存成真正的 wav，这里需要补 header，但对缓存来说 raw 其实也行
        // 为简单起见，仍存为 wav 命名但内容是 raw，或者暂时直接存
        if let Ok(mut file) = tokio::fs::File::create(&file_path).await {
            let _ = file.write_all(&audio_bytes_clone).await;
            let _ = file.flush().await;
        }
    });

    if sequence_id == 0 {
        println!(
            "[TTS] [完成] 流传输结束 (大小: {} 字节)",
            all_audio_bytes.len()
        );
    }

    // 返回 "STREAMING" 标记，前端知道不用等待返回结果
    Ok("STREAMING".to_string())
}
