use chrono::Local;
use regex::Regex;
use scraper::{Html, Selector};
use serde_json::json;
use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;
use tokio::time::sleep;

use stock_crawler::company_info::{StockInfo, StockDatabase};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    
    println!("🧪 測試公司資訊抓取工具");
    println!("只處理 3 家公司作為測試\n");
    
    // 創建測試資料
    let mut database: StockDatabase = HashMap::new();
    
    // 測試 3 家知名公司
    database.insert("2330".to_string(), StockInfo {
        code: "2330".to_string(),
        name: "台積電".to_string(),
        market_type: "上市".to_string(),
        industry_type: "半導體業".to_string(),
        listing_date: "1994/09/05".to_string(),
        international_code: "TW0002330008".to_string(),
        website: None,
        product_description: None,
        last_updated: None,
    });
    
    database.insert("2317".to_string(), StockInfo {
        code: "2317".to_string(),
        name: "鴻海".to_string(),
        market_type: "上市".to_string(),
        industry_type: "電腦及週邊設備業".to_string(),
        listing_date: "1991/06/20".to_string(),
        international_code: "TW0002317005".to_string(),
        website: None,
        product_description: None,
        last_updated: None,
    });
    
    database.insert("2454".to_string(), StockInfo {
        code: "2454".to_string(),
        name: "聯發科".to_string(),
        market_type: "上市".to_string(),
        industry_type: "半導體業".to_string(),
        listing_date: "2001/07/23".to_string(),
        international_code: "TW0002454006".to_string(),
        website: None,
        product_description: None,
        last_updated: None,
    });
    
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .timeout(Duration::from_secs(30))
        .build()?;
    
    let codes: Vec<String> = database.keys().cloned().collect();
    
    for (i, code) in codes.iter().enumerate() {
        let stock = database.get(code).unwrap().clone();
        
        println!("[{}/3] 處理: {} - {}", i + 1, stock.code, stock.name);
        
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
                            println!("  ✓ AI 歸納完成");
                            println!("    {}", summary);
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
        if let Some(stock_mut) = database.get_mut(code) {
            stock_mut.website = website;
            stock_mut.product_description = product_description;
            stock_mut.last_updated = Some(Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
        }
        
        println!();
        sleep(Duration::from_secs(2)).await;
    }
    
    // 顯示結果
    println!("📊 測試結果:\n");
    for (code, stock) in &database {
        println!("代號: {}", code);
        println!("名稱: {}", stock.name);
        println!("官網: {}", stock.website.as_ref().unwrap_or(&"未找到".to_string()));
        println!("產品: {}", stock.product_description.as_ref().unwrap_or(&"未取得".to_string()));
        println!("{}", "-".repeat(80));
    }
    
    println!("\n✅ 測試完成!");
    
    Ok(())
}

async fn search_company_website(
    client: &reqwest::Client,
    code: &str,
    name: &str,
) -> Result<String, Box<dyn Error>> {
    let query = format!("{} {} 公司 官網", code, name);
    let search_url = format!(
        "https://www.google.com/search?q={}",
        urlencoding::encode(&query)
    );
    
    let response = client.get(&search_url).send().await?;
    let html = response.text().await?;
    let document = Html::parse_document(&html);
    
    let link_selector = Selector::parse("a").unwrap();
    let url_regex = Regex::new(r#"https?://[^\s&"'<>]+"#).unwrap();
    
    for element in document.select(&link_selector) {
        if let Some(href) = element.value().attr("href") {
            if let Some(url_match) = url_regex.find(href) {
                let url = url_match.as_str();
                
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
    
    let body_selector = Selector::parse("body").unwrap();
    let mut content = String::new();
    
    if let Some(body) = document.select(&body_selector).next() {
        content = body.text().collect::<Vec<_>>().join(" ");
    }
    
    content = content
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    
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
    let api_key = std::env::var("OPENAI_API_KEY")
        .or_else(|_| std::env::var("ANTHROPIC_API_KEY"))
        .map_err(|_| "未設定 AI API key (OPENAI_API_KEY 或 ANTHROPIC_API_KEY)")?;
    
    if std::env::var("OPENAI_API_KEY").is_ok() {
        return summarize_with_openai(client, &api_key, company_name, content).await;
    }
    
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
