use chrono::Local;
use scraper::Html;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct StockInfo {
    code: String,
    name: String,
    market_type: String,
    industry_type: String,
    listing_date: String,
    international_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    website: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    product_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_updated: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    english_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    main_products: Option<Vec<String>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    
    let input_file = "stock_infos_test.json";
    let today = Local::now().format("%Y-%m-%d").to_string();
    let output_file = format!("stock_infos_{}.json", today);
    
    println!("📖 讀取股票資料庫: {}", input_file);
    let content = fs::read_to_string(input_file)?;
    
    // 先讀取為 Value，過濾掉 metadata
    let raw_data: serde_json::Value = serde_json::from_str(&content)?;
    let mut database: HashMap<String, StockInfo> = HashMap::new();
    
    if let Some(obj) = raw_data.as_object() {
        for (key, value) in obj {
            // 跳過 metadata 和其他非股票資料
            if key.starts_with('_') {
                continue;
            }
            
            // 嘗試解析為 StockInfo
            match serde_json::from_value::<StockInfo>(value.clone()) {
                Ok(stock_info) => {
                    database.insert(key.clone(), stock_info);
                }
                Err(e) => {
                    println!("⚠️  跳過無效資料 {}: {}", key, e);
                }
            }
        }
    }
    
    let total = database.len();
    println!("✅ 載入 {} 家公司資料", total);
    
    let mut processed = 0;
    let mut success_count = 0;
    let mut failed_count = 0;
    let codes: Vec<String> = database.keys().cloned().collect();
    
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .timeout(Duration::from_secs(30))
        .build()?;
    
    for code in codes {
        let stock = database.get(&code).unwrap().clone();
        processed += 1;
        
        println!("\n[{}/{}] 處理: {} - {}", processed, total, stock.code, stock.name);
        
        // 組成 ticker 字串 - 先試 .TW，抓不到再試 .TWO
        let ticker = if stock.market_type == "上市" {
            format!("{}.TW", stock.code)
        } else {
            format!("{}.TWO", stock.code)
        };
        
        println!("  🔍 查詢 Yahoo Finance: {}", ticker);
        
        // 從 Yahoo Finance 抓取主要經營業務
        match fetch_company_business(&client, &ticker).await {
            Ok(business) => {
                if !business.is_empty() {
                    // 安全地截取字串用於顯示
                    let display_text = business.chars().take(50).collect::<String>();
                    println!("  ✓ 主要經營業務: {}...", display_text);
                    
                    // 將主要經營業務轉換為產品列表
                    let products = parse_business_to_products(&business);
                    
                    // 更新資料庫
                    if let Some(stock_mut) = database.get_mut(&code) {
                        stock_mut.main_products = Some(products);
                        stock_mut.last_updated = Some(Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
                    }
                    success_count += 1;
                } else {
                    println!("  ⚠️  未找到主要經營業務");
                    failed_count += 1;
                }
            }
            Err(e) => {
                println!("  ✗ 查詢失敗: {}", e);
                
                // 如果是 .TW 失敗，嘗試 .TWO
                if ticker.ends_with(".TW") {
                    let alt_ticker = format!("{}.TWO", stock.code);
                    println!("  🔄 嘗試替代 ticker: {}", alt_ticker);
                    
                    match fetch_company_business(&client, &alt_ticker).await {
                        Ok(business) => {
                            if !business.is_empty() {
                                let display_text = business.chars().take(50).collect::<String>();
                                println!("  ✓ 主要經營業務: {}...", display_text);
                                
                                let products = parse_business_to_products(&business);
                                
                                if let Some(stock_mut) = database.get_mut(&code) {
                                    stock_mut.main_products = Some(products);
                                    stock_mut.last_updated = Some(Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
                                }
                                success_count += 1;
                            } else {
                                failed_count += 1;
                            }
                        }
                        Err(_) => {
                            failed_count += 1;
                        }
                    }
                } else {
                    failed_count += 1;
                }
            }
        }
        
        // 每處理 10 筆就儲存一次並顯示進度
        if processed % 10 == 0 {
            println!("\n💾 儲存進度 ({}/{})...", processed, total);
            println!("   成功: {}, 失敗: {}", success_count, failed_count);
            save_database(&output_file, &database)?;
        }
        
        // 避免請求過快
        sleep(Duration::from_millis(1000)).await;
    }
    
    // 最終儲存
    println!("\n💾 儲存最終結果...");
    save_database(&output_file, &database)?;
    
    println!("\n✅ 完成! 結果已儲存至: {}", output_file);
    println!("📊 總共處理: {} 家公司", processed);
    println!("✓ 成功更新: {} 家公司", success_count);
    println!("✗ 失敗: {} 家公司", failed_count);
    
    Ok(())
}

async fn fetch_company_business(client: &reqwest::Client, ticker: &str) -> Result<String, Box<dyn Error>> {
    let url = format!("https://tw.stock.yahoo.com/quote/{}/profile", ticker);
    
    let response = client.get(&url).send().await?;
    let html = response.text().await?;
    
    // 解析 HTML
    let document = Html::parse_document(&html);
    
    // 嘗試找到「主要經營業務」的內容
    let text = document.root_element().text().collect::<String>();
    
    // 尋找「主要經營業務」後面的內容
    if let Some(pos) = text.find("主要經營業務") {
        // 取得「主要經營業務」後面的文字
        let after_text = &text[pos + "主要經營業務".len()..];
        
        // 找到下一個可能的分隔符號
        let end_markers = vec![
            "配股資訊",
            "股利所屬期間",
            "公司地址",
            "市值",
            "簽證會計師",
            "已發行普通股數",
            "董監持股比例",
            "所屬集團",
            "產業類別",
            "財務資訊",
            "獲利能力",
        ];
        
        let mut end_pos = after_text.len().min(1000); // 限制最大長度
        for marker in end_markers {
            if let Some(pos) = after_text.find(marker) {
                if pos < end_pos && pos > 5 {
                    end_pos = pos;
                }
            }
        }
        
        // 提取主要經營業務內容
        let business = after_text[..end_pos]
            .trim()
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            // 過濾掉不相關的內容
            .filter(|line| {
                !line.contains("COMPANY_CPA") &&
                !line.contains("Yahoo Finance") &&
                !line.contains("服務條款") &&
                !line.contains("隱私權") &&
                !line.contains(".TW") &&
                !line.contains(".TWO") &&
                !line.contains("台股資料來源") &&
                !line.contains("臺灣證券交易所") &&
                !line.contains("財團法人") &&
                line.len() > 3 &&
                line.len() < 500
            })
            .collect::<Vec<_>>()
            .join(" ");
        
        // 清理內容
        let business = business
            .replace("：", "")
            .replace(":", "")
            .replace("\"", "")
            .replace(",", "")
            .trim()
            .to_string();
        
        if !business.is_empty() && business.len() > 5 {
            return Ok(business);
        }
    }
    
    Err("找不到主要經營業務".into())
}

fn parse_business_to_products(business: &str) -> Vec<String> {
    // 將主要經營業務文字轉換為產品列表
    // 使用常見的分隔符號分割
    let delimiters = vec!['、', '，', ',', '；', ';', '及', '和', '與'];
    
    let mut products = Vec::new();
    let mut current = String::new();
    
    for ch in business.chars() {
        if delimiters.contains(&ch) {
            let trimmed = current.trim().to_string();
            if !trimmed.is_empty() && trimmed.len() > 2 && trimmed.len() < 100 {
                products.push(trimmed);
            }
            current.clear();
        } else {
            current.push(ch);
        }
    }
    
    // 加入最後一個
    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() && trimmed.len() > 2 && trimmed.len() < 100 {
        products.push(trimmed);
    }
    
    // 如果沒有成功分割，就把整個業務描述作為一個產品
    if products.is_empty() && !business.is_empty() {
        // 限制長度
        let business_trimmed = if business.len() > 200 {
            format!("{}...", &business[..200])
        } else {
            business.to_string()
        };
        products.push(business_trimmed);
    }
    
    // 限制產品數量
    products.truncate(10);
    
    // 如果還是空的，返回預設值
    if products.is_empty() {
        products.push("N/A".to_string());
    }
    
    products
}

fn save_database(filename: &str, database: &HashMap<String, StockInfo>) -> Result<(), Box<dyn Error>> {
    let json = serde_json::to_string_pretty(database)?;
    fs::write(filename, json)?;
    Ok(())
}
