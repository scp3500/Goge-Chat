use tauri::{AppHandle, ipc::Channel, State}; // ✨ 增加 State 导入
use futures_util::StreamExt;
use crate::commands::config_cmd;
use crate::models::{Message, ChatRequest};
// ✨ 引入原子操作所需的枚举
use std::sync::atomic::Ordering; 

#[tauri::command]
pub async fn ask_ai(
    app: AppHandle, 
    // ✨ 【关键改动 1】：注入你在 lib.rs 中注册的全局状态
    state: State<'_, crate::GoleState>, 
    msg: Vec<Message>, 
    on_event: Channel<String>
) -> Result<(), String> {
    // 1. 【动态读取】加载配置
    let config = config_cmd::load_config(app).await?;
    
    // 2. 【安全校验】
    if config.api_key.trim().is_empty() {
        return Err("API Key 未配置，请前往设置页面填写".to_string());
    }

    let client = reqwest::Client::new();
    let payload = ChatRequest { 
        model: "deepseek-chat".to_string(), 
        messages: msg, 
        stream: true 
    };

    // 3. 【请求执行】
    let response = client.post("https://api.deepseek.com/chat/completions")
        .header("Authorization", format!("Bearer {}", config.api_key))
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let mut stream = response.bytes_stream();
    let mut buffer = String::new();

    // 4. 【流式循环处理】
    while let Some(chunk) = stream.next().await {
        
        // ✨ 【关键改动 2】：物理刹车！检查标志位
        // 只要前端调用了 stop_ai_generation，这里就会检测到 true
        if state.stop_flag.load(Ordering::Relaxed) {
            println!("⚡ [后端信号] 用户打断了生成，正在关闭流...");
            break; // 🔴 直接跳出循环，后续数据不再处理，请求自然结束
        }

        let chunk = chunk.map_err(|e| e.to_string())?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(newline_idx) = buffer.find('\n') {
            let line = buffer.drain(..=newline_idx).collect::<String>();
            let line = line.trim();

            if line == "data: [DONE]" { 
                return Ok(()); 
            }

            if let Some(data) = line.strip_prefix("data: ") {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                    if let Some(content) = json["choices"][0]["delta"]["content"].as_str() {
                        // 5. 【流式推送】给前端
                        on_event.send(content.to_string()).map_err(|e| e.to_string())?;
                    }
                }
            }
        }
    }

    println!("✅ AI 生成任务已彻底释放");
    Ok(())
}