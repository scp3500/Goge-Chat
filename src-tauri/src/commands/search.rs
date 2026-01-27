use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

#[derive(Deserialize, Debug)]
struct DuckDuckGoResult {
    #[serde(rename = "Abstract")]
    pub abstract_text: String,
    #[serde(rename = "AbstractURL")]
    pub abstract_url: String,
    #[serde(rename = "RelatedTopics")]
    pub related_topics: Vec<RelatedTopic>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum RelatedTopic {
    Topic {
        #[serde(rename = "Text")]
        text: String,
        #[serde(rename = "FirstURL")]
        url: String,
    },
    Group {
        #[serde(rename = "Topics")]
        topics: Vec<TopicItem>,
    },
}

#[derive(Deserialize, Debug)]
struct TopicItem {
    #[serde(rename = "Text")]
    pub text: String,
    #[serde(rename = "FirstURL")]
    pub url: String,
}

use regex::Regex;
use scraper::{Html, Selector};

pub async fn perform_search(
    _instance_url: &str,
    query: &str,
    provider: &str,
) -> Result<Vec<SearchResult>, String> {
    let augmented_query = augment_query(query, provider);

    // 依然使用原始 query 检测语言，防止被 site:github.com 等英文干扰判断
    if is_chinese(query) {
        println!(
            "🇨🇳 [SEARCH] 检测到中文，切换至 Bing 搜索: {}",
            augmented_query
        );
        perform_bing_search(&augmented_query).await
    } else {
        println!("🌐 [SEARCH] 使用默认 DuckDuckGo 搜索: {}", augmented_query);
        perform_duckduckgo_search(&augmented_query).await
    }
}

fn augment_query(query: &str, provider: &str) -> String {
    match provider {
        "developer" => {
            // 开发类：侧重 GitHub, StackOverflow, 及国内技术社区
            format!("{} (site:github.com OR site:stackoverflow.com OR site:v2ex.com OR site:juejin.cn OR site:csdn.net OR site:cnblogs.com OR site:zhihu.com)", query)
        }
        "academic" => {
            // 学术类：侧重论文、百科
            format!("{} (site:arxiv.org OR site:scholar.google.com OR site:researchgate.net OR site:wikipedia.org OR site:baike.baidu.com OR filetype:pdf)", query)
        }
        "wiki" => {
            // 只有百科
            format!("{} (site:wikipedia.org OR site:baike.baidu.com)", query)
        }
        _ => query.to_string(), // "all" 或其他情况不做处理
    }
}

fn is_chinese(query: &str) -> bool {
    // 匹配 CJK 统一汉字范围：U+4E00 - U+9FFF
    let re = Regex::new(r"[\u4e00-\u9fff]").unwrap();
    re.is_match(query)
}

async fn perform_bing_search(query: &str) -> Result<Vec<SearchResult>, String> {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36 Edg/121.0.0.0")
        .build()
        .map_err(|e| e.to_string())?;

    // 增加 count=20 参数以获取更多候选结果用于过滤
    let url = format!(
        "https://cn.bing.com/search?q={}&count=20",
        urlencoding::encode(query)
    );

    let response = client.get(&url).send().await.map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("Bing Search failed: {}", response.status()));
    }

    let html_content = response.text().await.map_err(|e| e.to_string())?;
    let document = Html::parse_document(&html_content);

    let result_selector = Selector::parse("li.b_algo").unwrap();
    let title_selector = Selector::parse("h2 > a").unwrap();
    let snippet_selector = Selector::parse("div.b_caption p").unwrap();

    let mut candidates = Vec::new();

    // 1. 收集所有候选结果
    for element in document.select(&result_selector).take(20) {
        let title_el = element.select(&title_selector).next();
        let snippet_el = element.select(&snippet_selector).next();

        if let (Some(title), Some(snippet)) = (title_el, snippet_el) {
            let title_text = title.text().collect::<Vec<_>>().join("");
            let url_str = title.value().attr("href").unwrap_or("").to_string();
            let snippet_text = snippet.text().collect::<Vec<_>>().join("");

            if !url_str.is_empty() {
                candidates.push(SearchResult {
                    title: title_text,
                    url: url_str,
                    snippet: snippet_text,
                });
            }
        }
    }

    println!("🔍 [SEARCH] 原始抓取到 {} 个候选结果", candidates.len());

    // 2. 多样性过滤 + 回填策略
    let mut final_results = Vec::new();
    let mut domain_counts = std::collections::HashMap::new();
    let mut skipped_indices = Vec::new();

    // Pass 1: 优先获取多样化结果
    for (index, item) in candidates.iter().enumerate() {
        if final_results.len() >= 8 {
            break;
        }

        let domain = item
            .url
            .split("://")
            .nth(1)
            .unwrap_or(&item.url)
            .split('/')
            .next()
            .unwrap_or("")
            .to_lowercase();

        let count = domain_counts.entry(domain.clone()).or_insert(0);

        if *count < 2 {
            *count += 1;
            final_results.push(item.clone());
        } else {
            skipped_indices.push(index);
        }
    }

    // Pass 2: 如果结果不足 8 个，从跳过的结果中回填
    if final_results.len() < 8 && !skipped_indices.is_empty() {
        println!(
            "⚠️ [SEARCH] 多样性过滤后只有 {} 个结果，正在回填...",
            final_results.len()
        );
        for index in skipped_indices {
            if final_results.len() >= 8 {
                break;
            }
            if let Some(item) = candidates.get(index) {
                final_results.push(item.clone());
            }
        }
    }

    if final_results.is_empty() {
        println!(
            "⚠️ [SEARCH] Bing 搜索未返回结果，HTML 预览: {:.200}",
            html_content
        );
    } else {
        println!(
            "✅ [SEARCH] Bing 搜索成功，最终返回 {} 条结果",
            final_results.len()
        );
    }

    Ok(final_results)
}

async fn perform_duckduckgo_search(query: &str) -> Result<Vec<SearchResult>, String> {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36")
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;

    // 第一步：尝试 DuckDuckGo Instant Answer API
    let api_url = format!(
        "https://api.duckduckgo.com/?q={}&format=json&no_html=1&skip_disambig=1",
        urlencoding::encode(query)
    );

    println!("🔍 [SEARCH] 正在通过 DuckDuckGo API 搜索: {}", query);

    let mut all_results = Vec::new();

    match client.get(&api_url).send().await {
        Ok(response) => {
            println!("📡 [SEARCH] API 响应状态: {}", response.status());
            if response.status().is_success() {
                match response.json::<DuckDuckGoResult>().await {
                    Ok(result) => {
                        println!(
                            "📦 [SEARCH] 成功解析 JSON, Abstract 长度: {}, RelatedTopics 数量: {}",
                            result.abstract_text.len(),
                            result.related_topics.len()
                        );

                        if !result.abstract_text.is_empty() {
                            all_results.push(SearchResult {
                                title: "主要摘要".to_string(),
                                url: result.abstract_url,
                                snippet: result.abstract_text,
                            });
                        }

                        for topic in result.related_topics {
                            match topic {
                                RelatedTopic::Topic { text, url } if !text.is_empty() => {
                                    all_results.push(SearchResult {
                                        title: text.chars().take(50).collect(),
                                        url,
                                        snippet: text,
                                    });
                                }
                                RelatedTopic::Group { topics } => {
                                    for item in topics.iter().take(3) {
                                        if !item.text.is_empty() {
                                            all_results.push(SearchResult {
                                                title: item.text.chars().take(50).collect(),
                                                url: item.url.clone(),
                                                snippet: item.text.clone(),
                                            });
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }

                        if !all_results.is_empty() {
                            println!(
                                "✅ [SEARCH] 从 Instant Answer API 获取到 {} 条结果",
                                all_results.len()
                            );
                            return Ok(all_results);
                        }
                    }
                    Err(e) => {
                        println!("❌ [SEARCH] JSON 解析失败: {}", e);
                    }
                }
            } else {
                println!("⚠️ [SEARCH] API 返回非成功状态: {}", response.status());
            }
        }
        Err(e) => {
            println!("❌ [SEARCH] API 请求失败: {}", e);
        }
    }

    // 第二步：尝试 HTML 搜索
    println!("⚠️ [SEARCH] Instant Answer 无结果，尝试 HTML 搜索...");

    let html_url = format!(
        "https://html.duckduckgo.com/html/?q={}",
        urlencoding::encode(query)
    );

    match client.get(&html_url).send().await {
        Ok(response) => {
            println!("📡 [SEARCH] HTML 响应状态: {}", response.status());
            if !response.status().is_success() {
                return Err(format!("HTML 搜索失败: {}", response.status()));
            }

            let html = response.text().await.map_err(|e| e.to_string())?;
            println!("📄 [SEARCH] HTML 内容长度: {} 字符", html.len());

            // 简单的 HTML 解析
            let mut results = Vec::new();

            // 寻找结果块
            let blocks: Vec<&str> = html.split("<div class=\"result__body\">").collect();
            println!(
                "🔍 [SEARCH] 找到 {} 个结果块",
                blocks.len().saturating_sub(1)
            );

            for (idx, block) in blocks.iter().skip(1).take(5).enumerate() {
                let title = block
                    .split("class=\"result__a\">")
                    .nth(1)
                    .and_then(|s| s.split("</a>").next())
                    .unwrap_or("无标题")
                    .replace("<b>", "")
                    .replace("</b>", "");

                let url = block
                    .split("class=\"result__url\">")
                    .nth(1)
                    .and_then(|s| s.split("</a>").next())
                    .unwrap_or("")
                    .trim()
                    .to_string();

                let snippet = block
                    .split("class=\"result__snippet\">")
                    .nth(1)
                    .and_then(|s| s.split("</a>").next())
                    .unwrap_or("")
                    .replace("<b>", "")
                    .replace("</b>", "");

                println!(
                    "  结果 {}: title={}, url={}, snippet_len={}",
                    idx + 1,
                    title.chars().take(30).collect::<String>(),
                    if url.is_empty() {
                        "(空)"
                    } else {
                        &url[..url.len().min(40)]
                    },
                    snippet.len()
                );

                if !url.is_empty() {
                    results.push(SearchResult {
                        title: title.trim().to_string(),
                        url: format!("https://{}", url),
                        snippet: snippet.trim().to_string(),
                    });
                }
            }

            if results.is_empty() {
                println!("❌ [SEARCH] HTML 解析未提取到任何有效结果");
                // 打印前500个字符用于调试
                println!(
                    "📝 [SEARCH] HTML 预览: {}...",
                    html.chars().take(500).collect::<String>()
                );
                return Err("未能从 HTML 搜索中提取到有效结果".to_string());
            }

            println!("✅ [SEARCH] 从 HTML 搜索获取到 {} 条结果", results.len());
            Ok(results)
        }
        Err(e) => {
            println!("❌ [SEARCH] HTML 请求失败: {}", e);
            Err(format!("HTML 搜索请求失败: {}", e))
        }
    }
}
