use chrono::Local;
use log::{info, warn};
use reqwest::blocking::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StockInfo {
    code: String,
    name: String,
    market_type: String,
    industry_type: String,
    listing_date: String,
    international_code: String,
}

fn main() {
    env_logger::init();
    
    info!("🚀 開始從台灣證券交易所抓取完整股票資訊...");
    
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
        .build()
        .expect("無法建立 HTTP 客戶端");
    
    // 目標網址
    let urls = vec![
        ("https://isin.twse.com.tw/isin/C_public.jsp?strMode=2", "上市"),
        ("https://isin.twse.com.tw/isin/C_public.jsp?strMode=4", "上櫃"),
        ("https://isin.twse.com.tw/isin/C_public.jsp?strMode=5", "興櫃"),
    ];
    
    let mut all_stocks: HashMap<String, StockInfo> = HashMap::new();
    
    for (url, market_name) in urls {
        info!("📥 正在抓取 {} 股票資料...", market_name);
        
        match fetch_stocks(&client, url, market_name) {
            Ok(stocks) => {
                info!("  ✅ 成功抓取 {} 筆 {} 股票", stocks.len(), market_name);
                all_stocks.extend(stocks);
            }
            Err(e) => {
                warn!("  ⚠️  抓取 {} 股票失敗: {}", market_name, e);
            }
        }
        
        // 避免請求太快
        thread::sleep(Duration::from_millis(500));
    }
    
    info!("📊 總共抓取 {} 筆股票資料", all_stocks.len());
    
    // 過濾出普通股票（4位數字代碼）
    let ordinary_stocks: HashMap<String, StockInfo> = all_stocks
        .into_iter()
        .filter(|(code, _)| code.len() == 4 && code.chars().all(|c| c.is_numeric()))
        .collect();
    
    info!("🔍 過濾後剩餘 {} 筆普通股票", ordinary_stocks.len());
    
    // 儲存為 JSON
    save_to_json(&ordinary_stocks);
    
    // 生成 HTML
    generate_html(&ordinary_stocks);
    
    // 生成 Markdown
    generate_markdown(&ordinary_stocks);
    
    info!("✅ 所有檔案已生成完成！");
}

fn fetch_stocks(
    client: &Client,
    url: &str,
    _market_name: &str,
) -> Result<HashMap<String, StockInfo>, Box<dyn std::error::Error>> {
    let response = client.get(url).send()?;
    
    // 設定正確的編碼（Big5）
    let bytes = response.bytes()?;
    let (text, _, _) = encoding_rs::BIG5.decode(&bytes);
    
    let document = Html::parse_document(&text);
    let table_selector = Selector::parse("table.h4").unwrap();
    let row_selector = Selector::parse("tr").unwrap();
    let cell_selector = Selector::parse("td").unwrap();
    
    let mut stocks = HashMap::new();
    
    if let Some(table) = document.select(&table_selector).next() {
        for (idx, row) in table.select(&row_selector).enumerate() {
            // 跳過標題行
            if idx == 0 {
                continue;
            }
            
            let cells: Vec<_> = row.select(&cell_selector).collect();
            
            if cells.len() != 7 {
                continue;
            }
            
            // 解析第一欄（代碼和名稱）
            let first_cell = cells[0].text().collect::<String>();
            let parts: Vec<&str> = first_cell.split('\u{3000}').collect(); // 全形空格
            
            if parts.len() != 2 {
                continue;
            }
            
            let code = parts[0].trim().to_string();
            let name = parts[1].trim().to_string();
            
            let international_code = cells[1].text().collect::<String>().trim().to_string();
            let listing_date = cells[2].text().collect::<String>().trim().to_string();
            let market_type = cells[3].text().collect::<String>().trim().to_string();
            let industry_type = cells[4].text().collect::<String>().trim().to_string();
            
            let stock = StockInfo {
                code: code.clone(),
                name,
                market_type,
                industry_type,
                listing_date,
                international_code,
            };
            
            stocks.insert(code, stock);
        }
    }
    
    Ok(stocks)
}

fn save_to_json(stocks: &HashMap<String, StockInfo>) {
    let json = serde_json::to_string_pretty(stocks).expect("無法序列化為 JSON");
    let filename = format!("stock_infos_{}.json", Local::now().format("%Y-%m-%d"));
    fs::write(&filename, json).expect("無法寫入 JSON 檔案");
    info!("💾 JSON 已儲存至: {}", filename);
}

fn generate_html(stocks: &HashMap<String, StockInfo>) {
    let mut html = String::from(
        r#"<!DOCTYPE html>
<html lang="zh-TW">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>台灣股票完整資訊</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: "Microsoft JhengHei", Arial, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            padding: 20px;
        }
        .container {
            max-width: 1400px;
            margin: 0 auto;
            background: white;
            border-radius: 20px;
            box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
            overflow: hidden;
        }
        header {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 40px;
            text-align: center;
        }
        h1 { font-size: 2.5em; margin-bottom: 10px; }
        .stats {
            display: flex;
            justify-content: center;
            gap: 40px;
            margin-top: 20px;
        }
        .stat-number { font-size: 2em; font-weight: bold; }
        .content { padding: 40px; }
        .industry-section {
            margin-bottom: 40px;
            border: 2px solid #e0e0e0;
            border-radius: 15px;
            overflow: hidden;
        }
        .industry-header {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 20px 30px;
            cursor: pointer;
            display: flex;
            justify-content: space-between;
        }
        .stocks-grid {
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
            gap: 20px;
            padding: 30px;
            background: #f8f9fa;
        }
        .stock-card {
            background: white;
            border-radius: 10px;
            padding: 20px;
            box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
            border-left: 4px solid #667eea;
        }
        .stock-code { font-size: 1.3em; font-weight: bold; color: #667eea; }
        .stock-info { margin: 5px 0; font-size: 0.9em; color: #666; }
        .stock-link {
            display: inline-block;
            padding: 6px 12px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            text-decoration: none;
            border-radius: 5px;
            font-size: 0.85em;
            margin: 5px 5px 0 0;
        }
        .collapsed .stocks-grid { display: none; }
        footer { background: #f8f9fa; padding: 20px; text-align: center; }
    </style>
    <script>
        function toggleIndustry(element) {
            element.closest('.industry-section').classList.toggle('collapsed');
        }
    </script>
</head>
<body>
    <div class="container">
        <header>
            <h1>🏢 台灣股票完整資訊</h1>
            <p>Taiwan Stock Market - Complete Information</p>
            <div class="stats">
"#,
    );
    
    // 按產業分類
    let mut by_industry: HashMap<String, Vec<&StockInfo>> = HashMap::new();
    for stock in stocks.values() {
        by_industry
            .entry(format!("{} ({})", stock.industry_type, stock.market_type))
            .or_insert_with(Vec::new)
            .push(stock);
    }
    
    html.push_str(&format!(
        r#"                <div class="stat-item">
                    <span class="stat-number">{}</span>
                    <span class="stat-label">家公司</span>
                </div>
                <div class="stat-item">
                    <span class="stat-number">{}</span>
                    <span class="stat-label">個產業</span>
                </div>
"#,
        stocks.len(),
        by_industry.len()
    ));
    
    html.push_str(r#"            </div>
        </header>
        <div class="content">
"#);
    
    let mut industries: Vec<_> = by_industry.iter_mut().collect();
    industries.sort_by(|a, b| a.0.cmp(b.0));
    
    for (industry, stocks_list) in industries {
        stocks_list.sort_by(|a, b| a.code.cmp(&b.code));
        
        html.push_str(&format!(
            r#"            <div class="industry-section">
                <div class="industry-header" onclick="toggleIndustry(this)">
                    <span>{}</span>
                    <span>{} 家 ▼</span>
                </div>
                <div class="stocks-grid">
"#,
            industry,
            stocks_list.len()
        ));
        
        for stock in stocks_list {
            let google_link = format!("https://www.google.com/search?q={}+{}+公司", stock.code, stock.name);
            let yahoo_link = format!("https://tw.stock.yahoo.com/quote/{}.TW", stock.code);
            let goodinfo_link = format!("https://goodinfo.tw/tw/StockDetail.asp?STOCK_ID={}", stock.code);
            
            html.push_str(&format!(
                r#"                    <div class="stock-card">
                        <div class="stock-code">{}</div>
                        <div class="stock-name">{}</div>
                        <div class="stock-info">上市日期: {}</div>
                        <div class="stock-info">國際代碼: {}</div>
                        <div>
                            <a href="{}" target="_blank" class="stock-link">🔍 官網</a>
                            <a href="{}" target="_blank" class="stock-link">📊 Yahoo</a>
                            <a href="{}" target="_blank" class="stock-link">📈 GoodInfo</a>
                        </div>
                    </div>
"#,
                stock.code,
                stock.name,
                stock.listing_date,
                stock.international_code,
                google_link,
                yahoo_link,
                goodinfo_link
            ));
        }
        
        html.push_str(r#"                </div>
            </div>
"#);
    }
    
    html.push_str(&format!(
        r#"        </div>
        <footer>
            <p>資料來源：台灣證券交易所 ISIN 網站</p>
            <p>更新時間：{}</p>
        </footer>
    </div>
</body>
</html>
"#,
        Local::now().format("%Y-%m-%d %H:%M:%S")
    ));
    
    let filename = format!("taiwan_stocks_complete_{}.html", Local::now().format("%Y-%m-%d"));
    fs::write(&filename, html).expect("無法寫入 HTML 檔案");
    info!("📄 HTML 已儲存至: {}", filename);
}

fn generate_markdown(stocks: &HashMap<String, StockInfo>) {
    let mut md = String::new();
    
    md.push_str("# 台灣股票完整資訊\n\n");
    md.push_str("> Taiwan Stock Market - Complete Information\n\n");
    md.push_str(&format!("**資料來源**: 台灣證券交易所 ISIN 網站\n\n"));
    md.push_str(&format!("**更新時間**: {}\n\n", Local::now().format("%Y-%m-%d %H:%M:%S")));
    
    // 按產業分類
    let mut by_industry: HashMap<String, Vec<&StockInfo>> = HashMap::new();
    for stock in stocks.values() {
        by_industry
            .entry(format!("{} ({})", stock.industry_type, stock.market_type))
            .or_insert_with(Vec::new)
            .push(stock);
    }
    
    md.push_str("## 📊 統計資訊\n\n");
    md.push_str(&format!("- **總公司數**: {} 家\n", stocks.len()));
    md.push_str(&format!("- **產業分類**: {} 個\n\n", by_industry.len()));
    md.push_str("---\n\n");
    
    md.push_str("## 📈 產業分類\n\n");
    
    let mut industries: Vec<_> = by_industry.iter_mut().collect();
    industries.sort_by(|a, b| a.0.cmp(b.0));
    
    for (industry, stocks_list) in industries {
        stocks_list.sort_by(|a, b| a.code.cmp(&b.code));
        
        md.push_str(&format!("### {}\n\n", industry));
        md.push_str(&format!("> 共 {} 家公司\n\n", stocks_list.len()));
        
        md.push_str("| 代號 | 公司名稱 | 上市日期 | 國際代碼 | 官網 | Yahoo | GoodInfo |\n");
        md.push_str("|------|---------|---------|---------|------|-------|----------|\n");
        
        for stock in stocks_list {
            let google_link = format!("https://www.google.com/search?q={}+{}+公司", stock.code, stock.name);
            let yahoo_link = format!("https://tw.stock.yahoo.com/quote/{}.TW", stock.code);
            let goodinfo_link = format!("https://goodinfo.tw/tw/StockDetail.asp?STOCK_ID={}", stock.code);
            
            md.push_str(&format!(
                "| **{}** | {} | {} | {} | [🔍]({}) | [📊]({}) | [📈]({}) |\n",
                stock.code,
                stock.name,
                stock.listing_date,
                stock.international_code,
                google_link,
                yahoo_link,
                goodinfo_link
            ));
        }
        
        md.push_str("\n");
    }
    
    md.push_str("---\n\n");
    md.push_str(&format!("*最後更新: {}*\n", Local::now().format("%Y-%m-%d %H:%M:%S")));
    
    let filename = format!("taiwan_stocks_complete_{}.md", Local::now().format("%Y-%m-%d"));
    fs::write(&filename, md).expect("無法寫入 Markdown 檔案");
    info!("📝 Markdown 已儲存至: {}", filename);
}
