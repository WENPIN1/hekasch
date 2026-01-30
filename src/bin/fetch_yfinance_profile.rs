use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;
use chrono::Local;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StockInfo {
    code: String,
    name: String,
    market_type: String,
    industry_type: String,
    listing_date: String,
    international_code: String,
    #[serde(default)]
    website: String,
    #[serde(default)]
    product_description: String,
    #[serde(default)]
    english_name: String,
    #[serde(default)]
    main_products: Vec<String>,
    #[serde(default)]
    last_updated: String,
}

fn append_status(message: &str) {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    let log_line = format!("[{}] {}\n", timestamp, message);
    
    match fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("INFO-1.md")
    {
        Ok(mut file) => {
            use std::io::Write;
            let _ = writeln!(file, "{}", log_line.trim());
        }
        Err(e) => eprintln!("無法寫入 INFO-1.md: {}", e),
    }
}

fn fetch_yfinance_profile(code: &str) -> Option<String> {
    let ticker = format!("{}.TW", code);
    let python_code = format!(
        r#"
import yfinance as yf
import json

try:
    ticker = yf.Ticker("{}")
    info = ticker.info
    
    # 優先取得 longBusinessSummary，其次 sector
    business_summary = info.get('longBusinessSummary', '')
    if not business_summary:
        business_summary = info.get('sector', '')
    
    print(business_summary if business_summary else '')
except Exception as e:
    print('')
"#,
        ticker
    );
    
    match Command::new("python3")
        .arg("-c")
        .arg(&python_code)
        .output()
    {
        Ok(result) => {
            let output = String::from_utf8_lossy(&result.stdout).trim().to_string();
            if !output.is_empty() {
                Some(output)
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

fn main() {
    println!("📖 讀取股票資訊...");
    append_status("開始從 yfinance 取得產品描述");
    
    let json_content = fs::read_to_string("stock_infos_enriched.json")
        .expect("無法讀取 stock_infos_enriched.json");
    
    let mut stocks: Vec<StockInfo> = serde_json::from_str(&json_content)
        .expect("無法解析 JSON");
    
    let total = stocks.len();
    println!("✅ 成功讀取 {} 筆股票資訊", total);
    append_status(&format!("✅ 成功讀取 {} 筆股票資訊", total));
    
    let mut updated_count = 0;
    let mut failed_count = 0;
    
    // 處理全部股票
    for (idx, stock) in stocks.iter_mut().enumerate() {
        // 只處理 4 位數字代碼
        if stock.code.len() != 4 || !stock.code.chars().all(|c| c.is_numeric()) {
            continue;
        }
        
        if let Some(profile) = fetch_yfinance_profile(&stock.code) {
            stock.product_description = profile;
            updated_count += 1;
        } else {
            failed_count += 1;
        }
        
        // 每 10 筆輸出狀態
        if (idx + 1) % 10 == 0 {
            let progress = format!(
                "進度: {}/{} | 成功: {} | 失敗: {}",
                idx + 1, total, updated_count, failed_count
            );
            println!("📊 {}", progress);
            append_status(&format!("📊 {}", progress));
        }
    }
    
    println!("\n📈 最終結果:");
    println!("  - 總股票: {} 筆", total);
    println!("  - 成功更新: {} 筆", updated_count);
    println!("  - 失敗: {} 筆", failed_count);
    
    let final_msg = format!(
        "✅ 完成 | 總股票: {} | 成功: {} | 失敗: {}",
        total, updated_count, failed_count
    );
    append_status(&final_msg);
    
    // 儲存結果
    let output_json = serde_json::to_string_pretty(&stocks)
        .expect("無法序列化 JSON");
    
    fs::write("stock_infos_enriched.json", output_json)
        .expect("無法寫入結果");
    
    println!("\n✅ 已更新 stock_infos_enriched.json");
    append_status("✅ 已更新 stock_infos_enriched.json");
}
