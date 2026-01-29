use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::io::{self, Write};

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

fn main() -> Result<(), Box<dyn Error>> {
    let batch_file = "current_batch.json";
    
    if !std::path::Path::new(batch_file).exists() {
        println!("❌ 找不到批次檔案: {}", batch_file);
        println!("請先執行: cargo run --release --bin batch_processor");
        return Ok(());
    }
    
    let content = fs::read_to_string(batch_file)?;
    let stocks: Vec<StockInfo> = serde_json::from_str(&content)?;
    
    let batch_size = 10;
    let total_batches = (stocks.len() + batch_size - 1) / batch_size;
    
    println!("📦 總共 {} 家公司,分成 {} 個批次", stocks.len(), total_batches);
    println!();
    
    // 找出下一個未處理的批次
    let mut current_batch = 0;
    for (i, chunk) in stocks.chunks(batch_size).enumerate() {
        let all_processed = chunk.iter().all(|s| s.product_description.is_some());
        if !all_processed {
            current_batch = i;
            break;
        }
    }
    
    if current_batch >= total_batches {
        println!("✅ 所有批次都已處理完成!");
        return Ok(());
    }
    
    let batches: Vec<_> = stocks.chunks(batch_size).collect();
    let current = batches[current_batch];
    
    println!("{}", "=".repeat(80));
    println!("【批次 {}/{}】", current_batch + 1, total_batches);
    println!("{}", "=".repeat(80));
    println!();
    
    for (i, stock) in current.iter().enumerate() {
        let status = if stock.product_description.is_some() { "✓" } else { " " };
        println!("{} {}. {} ({}) - {}", 
            status,
            i + 1, 
            stock.name, 
            stock.code, 
            stock.industry_type
        );
    }
    
    println!();
    println!("{}", "=".repeat(80));
    println!();
    println!("💡 請將以下公司資料提供給 Kiro:");
    println!();
    
    for (i, stock) in current.iter().enumerate() {
        if stock.product_description.is_none() {
            println!("{}. {} ({}) - {}", 
                i + 1, 
                stock.name, 
                stock.code,
                stock.industry_type
            );
        }
    }
    
    println!();
    println!("請 Kiro 幫忙查詢這些公司的官網和主要產品資訊,並以 JSON 格式回覆。");
    println!();
    println!("{}", "=".repeat(80));
    
    Ok(())
}
