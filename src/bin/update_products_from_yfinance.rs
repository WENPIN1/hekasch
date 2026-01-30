use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::process::Command;

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

fn main() {
    println!("📖 讀取股票資訊...");
    
    let json_content = fs::read_to_string("stock_infos_enriched.json")
        .expect("無法讀取 stock_infos_enriched.json");
    
    let mut stocks: Vec<StockInfo> = serde_json::from_str(&json_content)
        .expect("無法解析 JSON");
    
    println!("✅ 成功讀取 {} 筆股票資訊", stocks.len());
    
    // 試做前 5 個股票
    let test_count = 5;
    let mut updated_count = 0;
    
    for stock in stocks.iter_mut().take(test_count) {
        if stock.code.len() != 4 || !stock.code.chars().all(|c| c.is_numeric()) {
            continue;
        }
        
        println!("\n📊 處理: {} ({})", stock.code, stock.name);
        
        // 使用 yfinance 取得資訊
        let ticker = format!("{}.TW", stock.code);
        let python_code = format!(
            r#"
import yfinance as yf
import json

try:
    ticker = yf.Ticker("{}")
    info = ticker.info
    
    # 提取主要業務資訊
    sector = info.get('sector', '')
    industry = info.get('industry', '')
    business_summary = info.get('longBusinessSummary', '')
    
    result = {{
        'sector': sector,
        'industry': industry,
        'business_summary': business_summary[:200] if business_summary else ''
    }}
    print(json.dumps(result, ensure_ascii=False))
except Exception as e:
    print(json.dumps({{'error': str(e)}}, ensure_ascii=False))
"#,
            ticker
        );
        
        let output = Command::new("python3")
            .arg("-c")
            .arg(&python_code)
            .output();
        
        match output {
            Ok(result) => {
                let stdout = String::from_utf8_lossy(&result.stdout);
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&stdout) {
                    if let Some(business_summary) = data.get("business_summary").and_then(|v| v.as_str()) {
                        if !business_summary.is_empty() {
                            stock.product_description = business_summary.to_string();
                            stock.main_products = vec![business_summary.to_string()];
                            updated_count += 1;
                            println!("  ✅ 已更新: {}", business_summary.chars().take(50).collect::<String>());
                        }
                    }
                }
            }
            Err(e) => {
                println!("  ⚠️  yfinance 查詢失敗: {}", e);
            }
        }
    }
    
    println!("\n📈 試做結果:");
    println!("  - 處理股票: {} 筆", test_count);
    println!("  - 成功更新: {} 筆", updated_count);
    
    // 儲存試做結果
    let output_json = serde_json::to_string_pretty(&stocks)
        .expect("無法序列化 JSON");
    
    fs::write("stock_infos_test_update.json", output_json)
        .expect("無法寫入試做結果");
    
    println!("\n✅ 試做結果已儲存至 stock_infos_test_update.json");
    
    // 顯示前 3 個更新的股票
    println!("\n📋 前 3 個更新的股票:");
    for stock in stocks.iter().take(3) {
        println!("\n代碼: {}", stock.code);
        println!("名稱: {}", stock.name);
        println!("產品: {}", stock.product_description.chars().take(80).collect::<String>());
    }
}
