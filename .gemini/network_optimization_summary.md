# 🚀 大模型网络请求延时优化总结

## 实施时间
2026-02-18

## 优化目标
减少 TTFT (Time To First Token)，提升用户体验

---

## 已实施的优化

### 1. **HTTP 客户端连接池优化** ✅
**文件**: `src-tauri/src/lib.rs`

**改进点**:
- ✅ 增加每个 Host 的最大空闲连接数：`2 → 10`
- ✅ 延长连接存活时间：`默认 → 90秒`
- ✅ 启用 `TCP_NODELAY`，禁用 Nagle 算法
- ✅ **启用 HTTP/3 (QUIC) 协议**
- ✅ 启用 Brotli 和 Gzip 压缩
- ✅ 使用 `hickory-dns` 加速 DNS 解析

**预期收益**: 
- 节省 **50-200ms** 的 TCP 握手时间
- 节省 **30-100ms** 的 DNS 解析时间
- 节省 **50-150ms** 的 HTTP/3 协议优势（消除队头阻塞）

---

### 2. **连接预热 (Connection Pre-warming)** ✅
**文件**: 
- `src-tauri/src/commands/ai.rs` (后端)
- `src/components/chat/ChatInput.vue` (前端)

**工作原理**:
1. 用户在输入框输入 **≥3 个字符** 时触发
2. 防抖 500ms 后，后台向当前选中的 Provider 发起预热请求
3. 提前完成 DNS 解析、TCP 握手、TLS 握手
4. 用户点击"发送"时，直接复用已建立的连接

**预期收益**: 
- 节省 **100-500ms** 的建连时间（取决于网络环境）
- 对于国内访问 DeepSeek 等服务，效果尤为明显

---

### 3. **已有的并行优化** ✅
**文件**: `src-tauri/src/commands/ai.rs`

- ✅ 搜索 (Search) 和记忆检索 (Memory) 并行执行
- ✅ 配置加载使用内存缓存 (ConfigState)

---

## 预期总体效果

| 场景 | 优化前 TTFT | 优化后 TTFT | 节省时间 |
|------|------------|------------|---------|
| 首次请求 (冷启动) | ~800ms | ~500ms | **~300ms** |
| 后续请求 (热连接) | ~400ms | ~200ms | **~200ms** |
| 预热后请求 | ~400ms | ~100ms | **~300ms** |

---

## 使用方式

### 用户侧
**无需任何操作**，优化已自动生效：
1. 开始输入消息（≥3 个字符）
2. 等待 500ms，后台自动预热连接
3. 点击发送，享受极速响应 ⚡

### 开发者侧
查看预热日志：
```
🔥 [PREWARM] Connection to https://api.deepseek.com established in 245ms
```

---

## 📝 HTTP/3 配置说明

HTTP/3 已通过 `.cargo/config.toml` 自动启用，无需手动设置环境变量。

**工作原理**:
- 使用 QUIC 协议替代 TCP
- 消除队头阻塞 (Head-of-Line Blocking)
- 支持连接迁移（网络切换时无需重新握手）

**兼容性**:
- ✅ Cloudflare CDN（大部分 AI 服务商使用）
- ✅ Google 服务（Gemini API）
- ⚠️ 部分服务商可能尚未支持，会自动降级到 HTTP/2

---

## 监控建议

在 `src-tauri/src/commands/ai.rs` 中，已有性能监控日志：
```rust
println!("⏱️ [性能] 首字总响应: {:?}", start_total.elapsed());
```

建议关注以下指标：
- TTFT (Time To First Token)
- 配置加载耗时
- 搜索/记忆检索耗时
- 连接预热耗时

---

## 注意事项

1. **连接预热不会影响正常使用**
   - 预热失败会静默处理，不会阻塞用户操作

2. **避免重复预热**
   - 同一个 Provider 只预热一次
   - 切换 Provider 时会自动预热新的连接

3. **防抖机制**
   - 用户快速输入时，只在停顿 500ms 后才触发预热
   - 避免频繁的网络请求

---

## 测试建议

1. **冷启动测试**
   - 关闭应用，清除缓存
   - 重新启动，发送第一条消息
   - 观察 TTFT

2. **预热效果测试**
   - 输入 ≥3 个字符
   - 等待 1 秒（确保预热完成）
   - 点击发送，观察响应速度

3. **不同网络环境测试**
   - 良好网络：预热收益约 100ms
   - 较差网络：预热收益可达 500ms+

---

## 技术参考

- [reqwest 连接池文档](https://docs.rs/reqwest/latest/reqwest/)
- [HTTP/3 性能优势](https://blog.cloudflare.com/http3-the-past-present-and-future/)
- [TCP_NODELAY 原理](https://en.wikipedia.org/wiki/Nagle%27s_algorithm)
