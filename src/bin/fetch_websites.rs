use chrono::Local;
use regex::Regex;
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;
use tokio::time::sleep;

use stock_crawler::company_info::{load_stock_database, save_stock_database, StockInfo};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    
    let input_file = "stock_infos_2026-01-27.json";
    let output_file = "stock_infos_with_websites_2026-01-27.json";
    
    println!("📖 讀取股票資料庫...");
    let mut database = load_stock_database(input_file)?;
    
    let total = database.len();
    let needs_update_count = database.values().filter(|s| s.website.is_none()).count();
    
    println!("✅ 載入 {} 家公司資料", total);
    println!("🔄 需要更新: {} 家", needs_update_count);
    
    if needs_update_count == 0 {
        println!("✨ 所有公司都已有官網資料!");
        return Ok(());
    }
    
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .timeout(Duration::from_secs(30))
        .build()?;
    
    let mut processed = 0;
    let codes: Vec<String> = database.keys().cloned().collect();
    
    // 創建一個檔案來儲存需要 AI 歸納的內容
    let mut ai_tasks = Vec::new();
    
    for code in codes {
        let stock = database.get(&code).unwrap().clone();
        
        if stock.website.is_some() {
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
        
        // 2. 如果找到官網,抓取內容
        if let Some(ref url) = website {
            sleep(Duration::from_millis(500)).await;
            
            match fetch_website_content(&client, url).await {
                Ok(content) => {
                    println!("  ✓ 抓取官網內容成功 ({} 字元)", content.len());
                    
                    // 儲存到待處理列表
                    ai_tasks.push(AITask {
                        code: stock.code.clone(),
                        name: stock.name.clone(),
                        website: url.clone(),
                        content: content.clone(),
                    });
                }
                Err(e) => {
                    println!("  ✗ 抓取官網內容失敗: {}", e);
                }
            }
        }
        
        // 3. 更新資料庫
        if let Some(stock_mut) = database.get_mut(&code) {
            stock_mut.website = website;
            stock_mut.last_updated = Some(Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
        }
        
        // 每處理 10 家公司就儲存一次
        if processed % 10 == 0 {
            println!("\n💾 儲存進度...");
            save_stock_database(output_file, &database)?;
            save_ai_tasks("ai_tasks.json", &ai_tasks)?;
        }
        
        // 避免請求過快
        sleep(Duration::from_secs(2)).await;
    }
    
    // 最終儲存
    println!("\n💾 儲存最終結果...");
    save_stock_database(output_file, &database)?;
    save_ai_tasks("ai_tasks.json", &ai_tasks)?;
    
    println!("\n✅ 完成!");
    println!("📄 官網資料已儲存至: {}", output_file);
    println!("📋 待 AI 歸納的內容已儲存至: ai_tasks.json");
    println!("\n💡 下一步: 使用 Kiro 批次處理 ai_tasks.json 中的內容");
    
    Ok(())
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct AITask {
    code: String,
    name: String,
    website: String,
    content: String,
}

fn save_ai_tasks(filename: &str, tasks: &[AITask]) -> Result<(), Box<dyn Error>> {
    let json = serde_json::to_string_pretty(tasks)?;
    std::fs::write(filename, json)?;
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
    
    // 限制在 2000 字元以內
    if content.len() > 2000 {
        content.truncate(2000);
    }
    
    Ok(content)
}
