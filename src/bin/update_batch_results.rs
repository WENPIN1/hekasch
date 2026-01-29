use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs;

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
}

#[derive(Debug, Deserialize)]
struct UpdateData {
    website: String,
    product_description: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    // 讀取主資料庫
    let db_file = "stock_infos_2026-01-27.json";
    let content = fs::read_to_string(db_file)?;
    let mut database: HashMap<String, StockInfo> = serde_json::from_str(&content)?;
    
    // 讀取更新資料
    let update_file = "batch_updates.json";
    if !std::path::Path::new(update_file).exists() {
        eprintln!("❌ 找不到更新檔案: {}", update_file);
        eprintln!("請建立 batch_updates.json 檔案");
        return Ok(());
    }
    
    let update_content = fs::read_to_string(update_file)?;
    let updates: HashMap<String, UpdateData> = serde_json::from_str(&update_content)?;
    
    let mut updated_count = 0;
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    
    for (code, update_data) in updates {
        if let Some(stock) = database.get_mut(&code) {
            stock.website = Some(update_data.website);
            stock.product_description = Some(update_data.product_description);
            stock.last_updated = Some(now.clone());
            updated_count += 1;
            println!("✓ 更新: {} ({})", stock.name, code);
        } else {
            eprintln!("⚠ 找不到公司代碼: {}", code);
        }
    }
    
    // 儲存更新後的資料庫
    let json = serde_json::to_string_pretty(&database)?;
    fs::write(db_file, json)?;
    
    println!();
    println!("✅ 成功更新 {} 家公司資料", updated_count);
    println!("💾 已儲存到: {}", db_file);
    
    // 備份更新檔案
    let backup_file = format!("batch_updates_backup_{}.json", 
        chrono::Local::now().format("%Y%m%d_%H%M%S"));
    fs::copy(update_file, &backup_file)?;
    println!("📦 備份到: {}", backup_file);
    
    Ok(())
}
