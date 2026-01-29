use chrono::Local;
use regex::Regex;
use scraper::{Html, Selector};
use serde_json::json;
use std::error::Error;
use std::time::Duration;
use tokio::time::sleep;

// 引入主程式的模組
use stock_crawler::company_info::{load_stock_database, save_stock_database, needs_update, StockInfo};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    
    let input_file = "stock_infos_2026-01-27.json";
    let output_file = "stock_infos_with_products_2026-01-27.json";
    
    println!("📖 讀取股票資料庫...");
    let mut database = load_stock_database(input_file)?;
    
    let total = database.len();
    let needs_update_count = database.values().filter(|s| needs_update(s)).count();
    
    println!("✅ 載入 {} 家公司資料", total);
    println!("🔄 需要更新: {} 家", needs_update_count);
    
    if needs_update_count == 0 {
        println!("✨ 所有公司資料都已是最新!");
        return Ok(());
    }
    
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .timeout(Duration::from_secs(30))
        .build()?;
    
    let mut processed = 0;
    let codes: Vec<String> = database.keys().cloned().collect();
    
    for code in codes {
        let stock = database.get(&code).unwrap().clone();
        
        if !needs_update(&stock) {
            continue;
        }
        
        processed += 1;
        println!("\n[{}/{}] 處理: {} - {}", processed, needs_update_count, stock.code, stock.name);
        
        // 1. 搜尋公司官網
        let website = match search_company_website(&client, &stock.code, &stock.name).await {
            Ok(url) => {
                println!("  ✓ 找到官網: {}", url);
                Some(url)
            }
            Err(e) => {
                println!("  ✗ 搜尋官網失敗: {}", e);
                None
            }
        };
        
        // 2. 如果找到官網,抓取內容並用 AI 歸納
        let product_description = if let Some(ref url) = website {
            sleep(Duration::from_millis(500)).await;
            
            match fetch_website_content(&client, url).await {
                Ok(content) => {
                    println!("  ✓ 抓取官網內容成功 ({} 字元)", content.len());
                    
                    // 使用 AI 歸納產品資訊
                    match summarize_products_with_ai(&client, &stock.name, &content).await {
                        Ok(summary) => {
                            println!("  ✓ AI 歸納完成: {}", summary);
                            Some(summary)
                        }
                        Err(e) => {
                            println!("  ✗ AI 歸納失敗: {}", e);
                            None
                        }
                    }
                }
                Err(e) => {
                    println!("  ✗ 抓取官網內容失敗: {}", e);
                    None
                }
            }
        } else {
            None
        };
        
        // 3. 更新資料庫
        if let Some(stock_mut) = database.get_mut(&code) {
            stock_mut.website = website;
            stock_mut.product_description = product_description;
            stock_mut.last_updated = Some(Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
        }
        
        // 每處理 10 家公司就儲存一次
        if processed % 10 == 0 {
            println!("\n💾 儲存進度...");
            save_stock_database(output_file, &database)?;
        }
        
        // 避免請求過快
        sleep(Duration::from_secs(2)).await;
    }
    
    // 最終儲存
    println!("\n💾 儲存最終結果...");
    save_stock_database(output_file, &database)?;
    
    println!("\n✅ 完成! 結果已儲存至: {}", output_file);
    
    Ok(())
}

async fn search_company_website(
    client: &reqwest::Client,
    code: &str,
    name: &str,
) -> Result<String, Box<dyn Error>> {
    // 使用 Google 搜尋找公司官網
    let query = format!("{} {} 公司 官網", code, name);
    let search_url = format!(
        "https://www.google.com/search?q={}",
        urlencoding::encode(&query)
    );
    
    let response = client.get(&search_url).send().await?;
    let html = response.text().await?;
    let document = Html::parse_document(&html);
    
    // 嘗試從搜尋結果中提取第一個有效的公司網站
    let link_selector = Selector::parse("a").unwrap();
    let url_regex = Regex::new(r"https?://[^\s&]+").unwrap();
    
    for element in document.select(&link_selector) {
        if let Some(href) = element.value().attr("href") {
            if let Some(url_match) = url_regex.find(href) {
                let url = url_match.as_str();
                
                // 過濾掉 Google 自己的網址和一些常見的非官網網址
                if !url.contains("google.com") 
                    && !url.contains("youtube.com")
                    && !url.contains("facebook.com")
                    && !url.contains("wikipedia.org")
                    && !url.contains("yahoo.com")
                    && !url.contains("goodinfo.tw")
                    && (url.contains(".com.tw") || url.contains(".tw") || url.contains(".com"))
                {
                    return Ok(url.to_string());
                }
            }
        }
    }
    
    Err("找不到有效的公司官網".into())
}

async fn fetch_website_content(
    client: &reqwest::Client,
    url: &str,
) -> Result<String, Box<dyn Error>> {
    let response = client.get(url).send().await?;
    let html = response.text().await?;
    let document = Html::parse_document(&html);
    
    // 提取主要文字內容
    let body_selector = Selector::parse("body").unwrap();
    let mut content = String::new();
    
    if let Some(body) = document.select(&body_selector).next() {
        content = body.text().collect::<Vec<_>>().join(" ");
    }
    
    // 清理和限制長度
    content = content
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    
    // 限制在 3000 字元以內,避免 AI 請求過大
    if content.len() > 3000 {
        content.truncate(3000);
    }
    
    Ok(content)
}

async fn summarize_products_with_ai(
    client: &reqwest::Client,
    company_name: &str,
    content: &str,
) -> Result<String, Box<dyn Error>> {
    // 這裡需要使用 AI API,例如 OpenAI 或其他服務
    // 由於需要 API key,這裡提供一個簡化版本
    
    // 檢查環境變數中是否有 OpenAI API key
    let api_key = std::env::var("OPENAI_API_KEY")
        .or_else(|_| std::env::var("ANTHROPIC_API_KEY"))
        .map_err(|_| "未設定 AI API key (OPENAI_API_KEY 或 ANTHROPIC_API_KEY)")?;
    
    // 使用 OpenAI API
    if std::env::var("OPENAI_API_KEY").is_ok() {
        return summarize_with_openai(client, &api_key, company_name, content).await;
    }
    
    // 使用 Anthropic API
    if std::env::var("ANTHROPIC_API_KEY").is_ok() {
        return summarize_with_anthropic(client, &api_key, company_name, content).await;
    }
    
    Err("無可用的 AI API".into())
}

async fn summarize_with_openai(
    client: &reqwest::Client,
    api_key: &str,
    company_name: &str,
    content: &str,
) -> Result<String, Box<dyn Error>> {
    let prompt = format!(
        "請根據以下網站內容,用100字以內歸納「{}」公司的主要產品或服務:\n\n{}",
        company_name, content
    );
    
    let request_body = json!({
        "model": "gpt-3.5-turbo",
        "messages": [
            {
                "role": "system",
                "content": "你是一個專業的商業分析師,擅長歸納公司的核心產品和服務。請用繁體中文回答,並控制在100字以內。"
            },
            {
                "role": "user",
                "content": prompt
            }
        ],
        "max_tokens": 200,
        "temperature": 0.3
    });
    
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;
    
    let response_json: serde_json::Value = response.json().await?;
    
    let summary = response_json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("無法解析 AI 回應")?
        .trim()
        .to_string();
    
    Ok(summary)
}

async fn summarize_with_anthropic(
    client: &reqwest::Client,
    api_key: &str,
    company_name: &str,
    content: &str,
) -> Result<String, Box<dyn Error>> {
    let prompt = format!(
        "請根據以下網站內容,用100字以內歸納「{}」公司的主要產品或服務:\n\n{}",
        company_name, content
    );
    
    let request_body = json!({
        "model": "claude-3-haiku-20240307",
        "max_tokens": 200,
        "messages": [
            {
                "role": "user",
                "content": prompt
            }
        ],
        "system": "你是一個專業的商業分析師,擅長歸納公司的核心產品和服務。請用繁體中文回答,並控制在100字以內。"
    });
    
    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;
    
    let response_json: serde_json::Value = response.json().await?;
    
    let summary = response_json["content"][0]["text"]
        .as_str()
        .ok_or("無法解析 AI 回應")?
        .trim()
        .to_string();
    
    Ok(summary)
}
