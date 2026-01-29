use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

#[derive(Debug, Serialize, Deserialize)]
struct BatchTask {
    code: String,
    name: String,
    industry_type: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "stock_infos_2026-01-27.json";
    
    println!("📖 讀取股票資料庫...");
    let content = fs::read_to_string(input_file)?;
    let database: HashMap<String, StockInfo> = serde_json::from_str(&content)?;
    
    let total = database.len();
    let needs_update: Vec<_> = database.values()
        .filter(|s| s.product_description.is_none())
        .collect();
    
    println!("✅ 載入 {} 家公司資料", total);
    println!("🔄 需要處理: {} 家", needs_update.len());
    println!();
    
    // 按產業分類
    let mut by_industry: HashMap<String, Vec<&StockInfo>> = HashMap::new();
    for stock in &needs_update {
        by_industry.entry(stock.industry_type.clone())
            .or_insert_with(Vec::new)
            .push(stock);
    }
    
    // 顯示產業統計
    println!("📊 產業分布:");
    let mut industries: Vec<_> = by_industry.iter().collect();
    industries.sort_by(|a, b| b.1.len().cmp(&a.1.len()));
    
    for (i, (industry, stocks)) in industries.iter().enumerate().take(20) {
        println!("  {}. {} - {} 家", i + 1, industry, stocks.len());
    }
    
    println!();
    println!("請選擇處理方式:");
    println!("  1. 按產業處理");
    println!("  2. 處理前 N 家公司");
    println!("  3. 隨機選取 N 家公司");
    println!("  4. 自訂代碼列表");
    print!("\n請輸入選項 (1-4): ");
    io::stdout().flush()?;
    
    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;
    let choice = choice.trim();
    
    let selected = match choice {
        "1" => select_by_industry(&by_industry)?,
        "2" => select_first_n(&needs_update)?,
        "3" => select_random_n(&needs_update)?,
        "4" => select_by_codes(&database)?,
        _ => {
            println!("❌ 無效的選項");
            return Ok(());
        }
    };
    
    if selected.is_empty() {
        println!("❌ 沒有選取任何公司");
        return Ok(());
    }
    
    println!();
    println!("✅ 已選取 {} 家公司", selected.len());
    println!();
    
    // 生成批次任務
    generate_batches(&selected)?;
    
    Ok(())
}

fn select_by_industry(by_industry: &HashMap<String, Vec<&StockInfo>>) -> Result<Vec<StockInfo>, Box<dyn Error>> {
    print!("請輸入產業名稱 (例如: 半導體業): ");
    io::stdout().flush()?;
    
    let mut industry = String::new();
    io::stdin().read_line(&mut industry)?;
    let industry = industry.trim();
    
    if let Some(stocks) = by_industry.get(industry) {
        print!("要處理幾家公司? (最多 {}): ", stocks.len());
        io::stdout().flush()?;
        
        let mut count = String::new();
        io::stdin().read_line(&mut count)?;
        let count: usize = count.trim().parse().unwrap_or(stocks.len());
        
        Ok(stocks.iter().take(count).map(|s| (*s).clone()).collect())
    } else {
        println!("❌ 找不到該產業");
        Ok(Vec::new())
    }
}

fn select_first_n(stocks: &[&StockInfo]) -> Result<Vec<StockInfo>, Box<dyn Error>> {
    print!("要處理前幾家公司? (最多 {}): ", stocks.len());
    io::stdout().flush()?;
    
    let mut count = String::new();
    io::stdin().read_line(&mut count)?;
    let count: usize = count.trim().parse().unwrap_or(10);
    
    Ok(stocks.iter().take(count).map(|s| (*s).clone()).collect())
}

fn select_random_n(stocks: &[&StockInfo]) -> Result<Vec<StockInfo>, Box<dyn Error>> {
    print!("要隨機選取幾家公司? (最多 {}): ", stocks.len());
    io::stdout().flush()?;
    
    let mut count = String::new();
    io::stdin().read_line(&mut count)?;
    let count: usize = count.trim().parse().unwrap_or(10);
    
    use rand::seq::SliceRandom;
    use rand::thread_rng;
    
    let mut rng = thread_rng();
    let mut selected: Vec<_> = stocks.iter().map(|s| (*s).clone()).collect();
    selected.shuffle(&mut rng);
    
    Ok(selected.into_iter().take(count).collect())
}

fn select_by_codes(database: &HashMap<String, StockInfo>) -> Result<Vec<StockInfo>, Box<dyn Error>> {
    println!("請輸入公司代碼,用逗號分隔 (例如: 2330,2317,2454):");
    
    let mut codes = String::new();
    io::stdin().read_line(&mut codes)?;
    
    let selected: Vec<_> = codes
        .trim()
        .split(',')
        .filter_map(|code| database.get(code.trim()))
        .cloned()
        .collect();
    
    Ok(selected)
}

fn generate_batches(stocks: &[StockInfo]) -> Result<(), Box<dyn Error>> {
    let batch_size = 10;
    let batches: Vec<_> = stocks.chunks(batch_size).collect();
    
    println!("📦 將分成 {} 個批次處理 (每批次 {} 家)", batches.len(), batch_size);
    println!();
    
    // 儲存批次資料
    let batch_file = "current_batch.json";
    let json = serde_json::to_string_pretty(&stocks)?;
    fs::write(batch_file, json)?;
    
    println!("💾 已儲存批次資料到: {}", batch_file);
    println!();
    
    // 顯示第一批次
    println!("{}", "=".repeat(80));
    println!("【批次 1/{}】", batches.len());
    println!("{}", "=".repeat(80));
    println!();
    
    for (i, stock) in batches[0].iter().enumerate() {
        println!("{}. {} ({}) - {}", 
            i + 1, 
            stock.name, 
            stock.code, 
            stock.industry_type
        );
    }
    
    println!();
    println!("{}", "=".repeat(80));
    println!();
    println!("💡 請將以上公司資料提供給 Kiro,格式如下:");
    println!();
    println!("「請幫我查詢以下公司的官網和主要產品資訊,並用100字以內歸納:");
    println!();
    
    for (i, stock) in batches[0].iter().enumerate() {
        println!("{}. {} ({})", i + 1, stock.name, stock.code);
    }
    
    println!();
    println!("請以 JSON 格式回覆:」");
    println!();
    println!("{{");
    for stock in batches[0].iter() {
        println!("  \"{}\": {{", stock.code);
        println!("    \"website\": \"官網網址\",");
        println!("    \"product_description\": \"產品描述\"");
        println!("  }},");
    }
    println!("}}");
    println!();
    println!("{}", "=".repeat(80));
    println!();
    println!("📝 處理完第一批次後,執行以下命令查看下一批次:");
    println!("   cargo run --release --bin show_next_batch");
    
    Ok(())
}
