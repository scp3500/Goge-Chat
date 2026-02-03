use candle_core::{Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config};
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};
use tokenizers::Tokenizer;

pub struct EmbeddingEngine {
    model: BertModel,
    tokenizer: Tokenizer,
    device: Device,
}

impl EmbeddingEngine {
    pub fn new(app_handle: &AppHandle) -> Result<Self, String> {
        // 🔥 强制在 CPU 运行，避免抢占 4070 显存
        let device = Device::Cpu;

        // 获取动态资源路径 (Tauri 2.0 标准解析器)
        let actual_dir = app_handle
            .path()
            .resolve("resources/bge-small-zh-v1.5", BaseDirectory::Resource)
            .map_err(|e| format!("无法解析资源路径: {}", e))?;

        if !actual_dir.exists() {
            return Err(format!(
                "找不到模型目录: {:?}。请运行下载脚本或手动放置模型文件。",
                actual_dir
            ));
        }

        let config_path = actual_dir.join("config.json");
        let weights_path = actual_dir.join("model.safetensors");
        let tokenizer_path = actual_dir.join("tokenizer.json");

        let config =
            std::fs::read_to_string(config_path).map_err(|e| format!("读取 config 失败: {}", e))?;
        let config: Config =
            serde_json::from_str(&config).map_err(|e| format!("解析 config 失败: {}", e))?;

        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| format!("加载 tokenizer 失败: {}", e))?;

        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&[weights_path], candle_core::DType::F32, &device)
                .map_err(|e| format!("加载权值失败: {}", e))?
        };

        let model = BertModel::load(vb, &config).map_err(|e| format!("初始化 BERT 失败: {}", e))?;

        Ok(Self {
            model,
            tokenizer,
            device,
        })
    }

    pub fn get_vector(&self, text: &str) -> Result<Vec<f32>, String> {
        let tokens = self
            .tokenizer
            .encode(text, true)
            .map_err(|e| format!("Tokenize 失败: {}", e))?;
        let token_ids = tokens.get_ids().to_vec();
        let token_ids_tensor = Tensor::new(token_ids.as_slice(), &self.device)
            .map_err(|e| format!("创建 Tensor 失败: {}", e))?
            .unsqueeze(0)
            .map_err(|e| format!("Unsqueeze 失败: {}", e))?;

        // 简单的 BERT 推理（取 [CLS] 向量作为 Embedding）
        let output = self
            .model
            .forward(
                &token_ids_tensor,
                &token_ids_tensor.zeros_like().unwrap(),
                None,
            )
            .map_err(|e| format!("模型推理失败: {}", e))?;

        // 取 [CLS] (索引 0) 的向量
        let cls_vector = output.get(0).unwrap().get(0).unwrap();

        // L2 归一化 (L2 Normalization)
        // v_normalized = v / sqrt(sum(v_i^2))
        let norm = cls_vector
            .sqr()
            .map_err(|e| format!("Sqr 失败: {}", e))?
            .sum_all()
            .map_err(|e| format!("Sum 失败: {}", e))?
            .sqrt()
            .map_err(|e| format!("Sqrt 失败: {}", e))?;

        let normalized_vector = cls_vector
            .broadcast_div(&norm)
            .map_err(|e| format!("归一化失败: {}", e))?;

        let vector: Vec<f32> = normalized_vector
            .to_vec1()
            .map_err(|e| format!("转换向量失败: {}", e))?;

        Ok(vector)
    }
}
