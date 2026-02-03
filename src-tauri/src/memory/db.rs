use arrow_array::{
    FixedSizeListArray, Float32Array, RecordBatch, RecordBatchIterator, StringArray,
};
use arrow_schema::{DataType, Field, Schema};
use lancedb::connection::Connection;
use lancedb::query::QueryBase;
// Resolve naming collision with tauri::path::BaseDirectory::Executable
use futures_util::TryStreamExt;
use lancedb::query::ExecutableQuery as _;
use lancedb::table::OptimizeAction;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tauri::AppHandle;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactRecord {
    pub id: String,
    pub content: String,
    pub mode: String,
    pub role_id: String,
    pub metadata: String, // JSON string: { is_instruction, timestamp }
}

#[derive(Debug, serde::Serialize)]
pub struct DatabaseDiagnostic {
    pub total_records: usize,
    pub records_by_role: HashMap<String, usize>,
    pub records_by_mode: HashMap<String, usize>,
    pub duplicate_count: usize,
    pub oldest_timestamp: Option<i64>,
    pub newest_timestamp: Option<i64>,
}

pub struct LanceDbManager {
    uri: String,
}

impl LanceDbManager {
    pub fn new(_app_handle: &AppHandle) -> Result<Self, String> {
        // --- 1. 定位“便携式”数据目录 (当前可执行文件同级目录下的 data) ---
        let exe_path = std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let exe_dir = exe_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));
        let data_dir = exe_dir.join("data").join("alice_memory");

        if !data_dir.exists() {
            std::fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
        }

        let uri = data_dir.to_str().ok_or("路径转换失败")?.to_string();
        Ok(Self { uri })
    }

    async fn connect(&self) -> Result<Connection, lancedb::Error> {
        lancedb::connect(&self.uri).execute().await
    }

    pub async fn ensure_table(&self, dim: usize) -> Result<(), String> {
        let conn = self.connect().await.map_err(|e| e.to_string())?;
        let table_names = conn
            .table_names()
            .execute()
            .await
            .map_err(|e| e.to_string())?;

        let mut should_create = !table_names.contains(&"memories".to_string());

        if !should_create {
            // 检查 Schema 是否过期 (是否包含 mode 字段)
            let table = conn
                .open_table("memories")
                .execute()
                .await
                .map_err(|e| e.to_string())?;
            let schema = table.schema().await.map_err(|e| e.to_string())?;
            if schema.field_with_name("mode").is_err() {
                println!("⚠️ [Memory] Detected old schema, recreating table...");
                drop(table);
                conn.drop_table("memories")
                    .await
                    .map_err(|e| e.to_string())?;
                should_create = true;
            }
        }

        if should_create {
            let schema = Arc::new(Schema::new(vec![
                Field::new(
                    "vector",
                    DataType::FixedSizeList(
                        Arc::new(Field::new("item", DataType::Float32, true)),
                        dim as i32,
                    ),
                    false,
                ),
                Field::new("id", DataType::Utf8, false),
                Field::new("content", DataType::Utf8, false),
                Field::new("mode", DataType::Utf8, false),
                Field::new("role_id", DataType::Utf8, false),
                Field::new("metadata", DataType::Utf8, false),
            ]));

            conn.create_empty_table("memories", schema)
                .execute()
                .await
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub async fn insert_fact(&self, vector: Vec<f32>, fact: FactRecord) -> Result<(), String> {
        let conn = self.connect().await.map_err(|e| e.to_string())?;
        let table = conn
            .open_table("memories")
            .execute()
            .await
            .map_err(|e| e.to_string())?;

        let schema_res = table.schema().await;
        let schema = schema_res.map_err(|e| e.to_string())?;
        let dim = match schema.field_with_name("vector").unwrap().data_type() {
            DataType::FixedSizeList(_, d) => *d as usize,
            _ => return Err("Schema 错误".to_string()),
        };

        let vec_array = Arc::new(Float32Array::from(vector)) as Arc<dyn arrow_array::Array>;
        let list_array = FixedSizeListArray::try_new(
            Arc::new(Field::new("item", DataType::Float32, true)),
            dim as i32,
            vec_array,
            None,
        )
        .map_err(|e| e.to_string())?;
        let id_array = StringArray::from(vec![fact.id.as_str()]);
        let content_array = StringArray::from(vec![fact.content.as_str()]);
        let mode_array = StringArray::from(vec![fact.mode.as_str()]);
        let role_id_array = StringArray::from(vec![fact.role_id.as_str()]);
        let metadata_array = StringArray::from(vec![fact.metadata.as_str()]);

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(list_array),
                Arc::new(id_array),
                Arc::new(content_array),
                Arc::new(mode_array),
                Arc::new(role_id_array),
                Arc::new(metadata_array),
            ],
        )
        .map_err(|e| e.to_string())?;

        table
            .add(RecordBatchIterator::new(vec![Ok(batch)], schema))
            .execute()
            .await
            .map_err(|e| e.to_string())?;

        // --- 自动优化：确保磁盘文件数量和大小与逻辑数据保持一致 ---
        let _ = self.optimize_table().await;

        Ok(())
    }

    pub async fn delete_fact(&self, id: &str) -> Result<(), String> {
        println!("🗑️ [数据库] 执行准确 ID 删除: {}", id);
        let conn = self.connect().await.map_err(|e| e.to_string())?;
        let table = conn
            .open_table("memories")
            .execute()
            .await
            .map_err(|e| e.to_string())?;

        table
            .delete(&format!("id = '{}'", id))
            .await
            .map_err(|e| e.to_string())?;

        // --- 自动优化：物理擦除已删除的数据 ---
        let _ = self.optimize_table().await;

        Ok(())
    }

    pub async fn delete_fact_by_content(&self, content: &str) -> Result<(), String> {
        println!("🧹 [数据库] 执行内容模糊匹配清理: '{}'", content);
        let conn = self.connect().await.map_err(|e| e.to_string())?;
        let table = conn
            .open_table("memories")
            .execute()
            .await
            .map_err(|e| e.to_string())?;

        let safe_content = content.replace("'", "''");
        table
            .delete(&format!("content = '{}'", safe_content))
            .await
            .map_err(|e| e.to_string())?;

        // --- 自动优化：物理擦除已删除的数据 ---
        let _ = self.optimize_table().await;

        Ok(())
    }

    pub async fn clear_memories(&self) -> Result<(), String> {
        let conn = self.connect().await.map_err(|e| e.to_string())?;
        // 尝试删除表 (可能失败，如果表已被删除)
        let _ = conn.drop_table("memories").await;
        // 关键：重新调用 ensure_table 确保目录和结构清空后重启
        self.ensure_table(512).await?;
        println!("🗑️ [数据库] 记忆库已执行核弹级重置。");
        Ok(())
    }

    pub async fn search_similar_facts(
        &self,
        vector: Vec<f32>,
        limit: usize,
        filter: Option<String>,
    ) -> Result<Vec<(FactRecord, f32)>, String> {
        let conn = self.connect().await.map_err(|e| e.to_string())?;
        let table = conn
            .open_table("memories")
            .execute()
            .await
            .map_err(|e| e.to_string())?;

        let mut query = table
            .vector_search(vector.clone())
            .map_err(|e| e.to_string())?
            .limit(limit);

        if let Some(f) = filter {
            query = query.only_if(f);
        }

        let results: Vec<RecordBatch> = query
            .execute()
            .await
            .map_err(|e: lancedb::Error| e.to_string())?
            .try_collect::<Vec<RecordBatch>>()
            .await
            .map_err(|e: lancedb::Error| e.to_string())?;

        let mut facts = Vec::new();
        for batch in results {
            let ids = batch
                .column_by_name("id")
                .unwrap()
                .as_any()
                .downcast_ref::<StringArray>()
                .unwrap();
            let contents = batch
                .column_by_name("content")
                .unwrap()
                .as_any()
                .downcast_ref::<StringArray>()
                .unwrap();
            let metadatas = batch
                .column_by_name("metadata")
                .unwrap()
                .as_any()
                .downcast_ref::<StringArray>()
                .unwrap();
            let modes = batch
                .column_by_name("mode")
                .unwrap()
                .as_any()
                .downcast_ref::<StringArray>()
                .unwrap();
            let role_ids = batch
                .column_by_name("role_id")
                .unwrap()
                .as_any()
                .downcast_ref::<StringArray>()
                .unwrap();
            let distances =
                batch
                    .column_by_name("_distance")
                    .map(|c: &Arc<dyn arrow_array::Array>| {
                        c.as_any()
                            .downcast_ref::<arrow_array::Float32Array>()
                            .unwrap()
                    });

            for i in 0..batch.num_rows() {
                facts.push((
                    FactRecord {
                        id: ids.value(i).to_string(),
                        content: contents.value(i).to_string(),
                        mode: modes.value(i).to_string(),
                        role_id: role_ids.value(i).to_string(),
                        metadata: metadatas.value(i).to_string(),
                    },
                    distances.map(|d: &Float32Array| d.value(i)).unwrap_or(0.0),
                ));
            }
        }
        Ok(facts)
    }

    pub async fn get_all_memories(&self) -> Result<Vec<FactRecord>, String> {
        let conn = self.connect().await.map_err(|e| e.to_string())?;
        let table = conn
            .open_table("memories")
            .execute()
            .await
            .map_err(|e| e.to_string())?;

        let stream = table.query().execute().await.map_err(|e| e.to_string())?;
        let results = stream
            .try_collect::<Vec<RecordBatch>>()
            .await
            .map_err(|e| e.to_string())?;

        let mut facts = Vec::new();
        for batch in results {
            let ids = batch
                .column_by_name("id")
                .unwrap()
                .as_any()
                .downcast_ref::<StringArray>()
                .unwrap();
            let contents = batch
                .column_by_name("content")
                .unwrap()
                .as_any()
                .downcast_ref::<StringArray>()
                .unwrap();
            let modes = batch
                .column_by_name("mode")
                .unwrap()
                .as_any()
                .downcast_ref::<StringArray>()
                .unwrap();
            let role_ids = batch
                .column_by_name("role_id")
                .unwrap()
                .as_any()
                .downcast_ref::<StringArray>()
                .unwrap();
            let metadatas = batch
                .column_by_name("metadata")
                .unwrap()
                .as_any()
                .downcast_ref::<StringArray>()
                .unwrap();

            for i in 0..batch.num_rows() {
                facts.push(FactRecord {
                    id: ids.value(i).to_string(),
                    content: contents.value(i).to_string(),
                    mode: modes.value(i).to_string(),
                    role_id: role_ids.value(i).to_string(),
                    metadata: metadatas.value(i).to_string(),
                });
            }
        }
        Ok(facts)
    }

    pub async fn optimize_table(&self) -> Result<(), String> {
        let start = std::time::Instant::now();
        let conn = self.connect().await.map_err(|e| e.to_string())?;
        let table = conn
            .open_table("memories")
            .execute()
            .await
            .map_err(|e| e.to_string())?;

        // 使用 All 进行全面优化（包含压缩）
        table
            .optimize(OptimizeAction::All)
            .await
            .map_err(|e| e.to_string())?;

        let duration = start.elapsed();
        if duration.as_millis() > 200 {
            println!("⏱️ [数据库] 优化完成，耗时: {:?}", duration);
        }
        Ok(())
    }

    pub async fn get_diagnostic(&self) -> Result<DatabaseDiagnostic, String> {
        let memories = self.get_all_memories().await?;

        let mut role_count = HashMap::new();
        let mut mode_count = HashMap::new();
        let mut id_set = HashSet::new();
        let mut duplicates = 0;
        let mut timestamps = Vec::new();

        for mem in &memories {
            *role_count.entry(mem.role_id.clone()).or_insert(0) += 1;
            *mode_count.entry(mem.mode.clone()).or_insert(0) += 1;

            if !id_set.insert(mem.id.clone()) {
                duplicates += 1;
            }

            // 解析 metadata 中的 timestamp
            if let Ok(meta) = serde_json::from_str::<serde_json::Value>(&mem.metadata) {
                if let Some(ts) = meta.get("timestamp").and_then(|v| v.as_i64()) {
                    timestamps.push(ts);
                }
            }
        }

        timestamps.sort();

        Ok(DatabaseDiagnostic {
            total_records: memories.len(),
            records_by_role: role_count,
            records_by_mode: mode_count,
            duplicate_count: duplicates,
            oldest_timestamp: timestamps.first().cloned(),
            newest_timestamp: timestamps.last().cloned(),
        })
    }
}
