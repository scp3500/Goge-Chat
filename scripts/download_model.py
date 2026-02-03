import os
import shutil
from modelscope.hub.snapshot_download import snapshot_download

def download_model():
    model_id = 'AI-ModelScope/bge-small-zh-v1.5'
    
    # 确定目标路径
    current_dir = os.path.dirname(os.path.abspath(__file__))
    project_root = os.path.dirname(current_dir)
    target_dir = os.path.join(project_root, 'src-tauri', 'resources', 'bge-small-zh-v1.5')
    
    print(f"🚀 正在从 ModelScope 下载模型 {model_id}...")
    
    # 下载模型到临时缓存
    download_path = snapshot_download(model_id)
    
    print(f"✅ 下载完成，准备移动到: {target_dir}")
    
    # 创建目标目录
    if not os.path.exists(target_dir):
        os.makedirs(target_dir)
    
    # 需要复制的文件列表
    files_to_copy = [
        'config.json',
        'model.safetensors',
        'tokenizer.json',
        'tokenizer_config.json',
        'special_tokens_map.json',
        'vocab.txt'
    ]
    
    for file_name in files_to_copy:
        src = os.path.join(download_path, file_name)
        dst = os.path.join(target_dir, file_name)
        if os.path.exists(src):
            print(f"📦 正在复制 {file_name}...")
            shutil.copy2(src, dst)
        else:
            print(f"⚠️ 跳过 {file_name} (中未找到)")

    print(f"\n✨ 模型整合成功！路径: {target_dir}")

if __name__ == "__main__":
    download_model()
