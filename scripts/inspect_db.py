import lancedb
import pandas as pd
import os

# 1. 自动定位路径
db_path = r"D:\my-code\my-gemini-shell\src-tauri\target\debug\data\alice_memory"
if not os.path.exists(db_path):
    print(f"❌ 路径不存在: {db_path}")
    exit()

db = lancedb.connect(db_path)
table = db.open_table("memories")

print("="*60)
print("🔍 Antigravity 记忆库深度扫描报告")
print("="*60)

# --- 1. 逻辑视图 (当前生效的) ---
df_active = table.to_pandas()
print(f"\n✅ [逻辑层] UI 显示的活动条目: {len(df_active)} 条")
if not df_active.empty:
    print(df_active[['id', 'content', 'mode', 'role_id']].to_string())

# --- 2. 物理分片扫描 ---
print("\n" + "-"*60)
fragments = table.fragments()
print(f"📂 [磁盘层] 发现 .lance 分片(文件)数量: {len(fragments)} 个")

total_raw_rows = 0
for i, frag in enumerate(fragments):
    row_count = frag.count_rows()
    total_raw_rows += row_count
    # 获取该分片的元数据信息 (LanceDB 允许查看该分片是否包含删除标记)
    print(f"📄 分片 #{i+1}: 记录总数: {row_count}")

print(f"\n📊 统计总结: 磁盘总计持有 {total_raw_rows} 行数据记录")
print(f"⚠️  冗余差值: {total_raw_rows - len(df_active)} 条 (这些是已删除或已过期的历史版本)")
print("="*60)
print("💡 提示: 想要物理删除残留文件，请在程序中使用'优化数据库'功能。")