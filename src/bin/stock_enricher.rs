use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

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

#[derive(Debug, Serialize)]
struct EnrichedStock {
    code: String,
    name: String,
    english_name: String,
    market_type: String,
    industry_type: String,
    listing_date: String,
    international_code: String,
    website: String,
    product_description: String,
    main_products: Vec<String>,
    last_updated: String,
}

fn main() {
    println!("📖 讀取股票資訊...");
    
    let json_content = fs::read_to_string("stock_infos_new.json")
        .expect("無法讀取 stock_infos_new.json");
    
    let json_value: serde_json::Value = serde_json::from_str(&json_content)
        .expect("無法解析 JSON");
    
    let mut stocks: HashMap<String, StockInfo> = HashMap::new();
    
    if let Some(obj) = json_value.as_object() {
        for (key, value) in obj {
            if key == "_metadata" {
                continue;
            }
            if let Ok(stock) = serde_json::from_value::<StockInfo>(value.clone()) {
                stocks.insert(key.clone(), stock);
            }
        }
    }
    
    println!("✅ 成功讀取 {} 筆股票資訊", stocks.len());
    
    // 轉換為上市上櫃股票資訊
    let mut enriched_stocks: Vec<EnrichedStock> = stocks
        .into_iter()
        .filter(|(_, stock)| stock.market_type == "上市" || stock.market_type == "上櫃")
        .map(|(_, stock)| EnrichedStock {
            code: stock.code,
            name: stock.name,
            english_name: stock.english_name,
            market_type: stock.market_type,
            industry_type: stock.industry_type,
            listing_date: stock.listing_date,
            international_code: stock.international_code,
            website: stock.website,
            product_description: stock.product_description,
            main_products: stock.main_products,
            last_updated: stock.last_updated,
        })
        .collect();
    
    // 按代碼排序
    enriched_stocks.sort_by(|a, b| a.code.cmp(&b.code));
    
    println!("📊 上市上櫃股票: {} 筆", enriched_stocks.len());
    
    // 統計市場類型
    let listed_count = enriched_stocks.iter().filter(|s| s.market_type == "上市").count();
    let otc_count = enriched_stocks.iter().filter(|s| s.market_type == "上櫃").count();
    
    println!("  - 上市: {} 筆", listed_count);
    println!("  - 上櫃: {} 筆", otc_count);
    
    // 統計產業類型
    let mut industries: HashMap<String, usize> = HashMap::new();
    for stock in &enriched_stocks {
        *industries.entry(stock.industry_type.clone()).or_insert(0) += 1;
    }
    
    println!("\n📈 產業分佈:");
    let mut industry_list: Vec<_> = industries.iter().collect();
    industry_list.sort_by(|a, b| b.1.cmp(a.1));
    for (industry, count) in industry_list.iter().take(10) {
        println!("  - {}: {} 家", industry, count);
    }
    
    // 統計網站資訊
    let with_website = enriched_stocks.iter().filter(|s| !s.website.is_empty()).count();
    let with_products = enriched_stocks.iter().filter(|s| !s.product_description.is_empty()).count();
    
    println!("\n🌐 資訊完整度:");
    println!("  - 有網站: {} 筆 ({:.1}%)", with_website, (with_website as f64 / enriched_stocks.len() as f64) * 100.0);
    println!("  - 有產品說明: {} 筆 ({:.1}%)", with_products, (with_products as f64 / enriched_stocks.len() as f64) * 100.0);
    
    // 儲存為 JSON
    let output_json = serde_json::to_string_pretty(&enriched_stocks)
        .expect("無法序列化 JSON");
    
    fs::write("stock_infos_enriched.json", output_json)
        .expect("無法寫入 stock_infos_enriched.json");
    
    println!("\n✅ 已儲存至 stock_infos_enriched.json");
    
    // 生成 HTML 報告
    generate_html(&enriched_stocks);
    
    // 生成 CSV
    generate_csv(&enriched_stocks);
}

fn generate_html(stocks: &[EnrichedStock]) {
    let mut html = String::from(
        r#"<!DOCTYPE html>
<html lang="zh-TW">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>台灣上市上櫃股票資訊</title>
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
        .stock-name { font-size: 1.1em; font-weight: bold; margin: 5px 0; }
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
        .stock-link:hover { opacity: 0.8; }
        .collapsed .stocks-grid { display: none; }
        footer { background: #f8f9fa; padding: 20px; text-align: center; color: #666; }
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
            <h1>🏢 台灣上市上櫃股票資訊</h1>
            <p>Taiwan Listed & OTC Stocks - Complete Information</p>
            <div class="stats">
"#
    );
    
    let listed_count = stocks.iter().filter(|s| s.market_type == "上市").count();
    let otc_count = stocks.iter().filter(|s| s.market_type == "上櫃").count();
    let mut industries: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for stock in stocks {
        *industries.entry(stock.industry_type.clone()).or_insert(0) += 1;
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
            </div>
        </header>
        <div class="content">
"#,
        stocks.len(),
        industries.len()
    ));
    
    // 按產業分類
    let mut industry_stocks: std::collections::HashMap<String, Vec<&EnrichedStock>> = std::collections::HashMap::new();
    for stock in stocks {
        industry_stocks.entry(stock.industry_type.clone()).or_insert_with(Vec::new).push(stock);
    }
    
    // 按產業名稱排序
    let mut sorted_industries: Vec<_> = industry_stocks.iter().collect();
    sorted_industries.sort_by(|a, b| {
        let a_listed = a.1.iter().filter(|s| s.market_type == "上市").count();
        let b_listed = b.1.iter().filter(|s| s.market_type == "上市").count();
        b_listed.cmp(&a_listed)
    });
    
    for (industry, stocks_in_industry) in sorted_industries {
        let listed_in_industry = stocks_in_industry.iter().filter(|s| s.market_type == "上市").count();
        let otc_in_industry = stocks_in_industry.iter().filter(|s| s.market_type == "上櫃").count();
        
        // 上市股票
        let listed_stocks: Vec<_> = stocks_in_industry.iter().filter(|s| s.market_type == "上市").collect();
        if !listed_stocks.is_empty() {
            html.push_str(&format!(
                r#"            <div class="industry-section">
                <div class="industry-header" onclick="toggleIndustry(this)">
                    <span>{} (上市)</span>
                    <span>{} 家 ▼</span>
                </div>
                <div class="stocks-grid">
"#,
                industry,
                listed_in_industry
            ));
            
            for stock in listed_stocks {
                let website_link = if stock.website.is_empty() || stock.website == "no website" {
                    String::new()
                } else {
                    format!(r#"<a href="{}" target="_blank" class="stock-link">🌐 官網</a>"#, stock.website)
                };
                
                let products = if stock.main_products.is_empty() {
                    stock.product_description.clone()
                } else {
                    stock.main_products.join(", ")
                };
                
                html.push_str(&format!(
                    r#"                    <div class="stock-card">
                        <div class="stock-code">{}</div>
                        <div class="stock-name">{}</div>
                        <div class="stock-info">上市日期: {}</div>
                        <div class="stock-info">國際代碼: {}</div>
                        <div class="stock-info" style="color: #667eea; font-weight: bold; margin-top: 10px; font-size: 0.85em;">{}</div>
                        <div style="margin-top: 10px;">
                            {}
                            <a href="https://tw.stock.yahoo.com/quote/{}.TW" target="_blank" class="stock-link">📊 Yahoo</a>
                            <a href="https://goodinfo.tw/tw/StockDetail.asp?STOCK_ID={}" target="_blank" class="stock-link">📈 GoodInfo</a>
                        </div>
                    </div>
"#,
                    stock.code,
                    stock.name,
                    stock.listing_date,
                    stock.international_code,
                    products,
                    website_link,
                    stock.code,
                    stock.code
                ));
            }
            
            html.push_str(
                r#"                </div>
            </div>
"#
            );
        }
        
        // 上櫃股票
        let otc_stocks: Vec<_> = stocks_in_industry.iter().filter(|s| s.market_type == "上櫃").collect();
        if !otc_stocks.is_empty() {
            html.push_str(&format!(
                r#"            <div class="industry-section">
                <div class="industry-header" onclick="toggleIndustry(this)">
                    <span>{} (上櫃)</span>
                    <span>{} 家 ▼</span>
                </div>
                <div class="stocks-grid">
"#,
                industry,
                otc_in_industry
            ));
            
            for stock in otc_stocks {
                let website_link = if stock.website.is_empty() || stock.website == "no website" {
                    String::new()
                } else {
                    format!(r#"<a href="{}" target="_blank" class="stock-link">🌐 官網</a>"#, stock.website)
                };
                
                let products = if stock.main_products.is_empty() {
                    stock.product_description.clone()
                } else {
                    stock.main_products.join(", ")
                };
                
                html.push_str(&format!(
                    r#"                    <div class="stock-card">
                        <div class="stock-code">{}</div>
                        <div class="stock-name">{}</div>
                        <div class="stock-info">上市日期: {}</div>
                        <div class="stock-info">國際代碼: {}</div>
                        <div class="stock-info" style="color: #667eea; font-weight: bold; margin-top: 10px; font-size: 0.85em;">{}</div>
                        <div style="margin-top: 10px;">
                            {}
                            <a href="https://tw.stock.yahoo.com/quote/{}.TW" target="_blank" class="stock-link">📊 Yahoo</a>
                            <a href="https://goodinfo.tw/tw/StockDetail.asp?STOCK_ID={}" target="_blank" class="stock-link">📈 GoodInfo</a>
                        </div>
                    </div>
"#,
                    stock.code,
                    stock.name,
                    stock.listing_date,
                    stock.international_code,
                    products,
                    website_link,
                    stock.code,
                    stock.code
                ));
            }
            
            html.push_str(
                r#"                </div>
            </div>
"#
            );
        }
    }
    
    html.push_str(
        r#"        </div>
        <footer>
            <p>資料最後更新時間: 2026-01-29 | 台灣上市上櫃股票完整資訊</p>
        </footer>
    </div>
</body>
</html>
"#
    );
    
    fs::write("stock_infos_report.html", html.clone())
        .expect("無法寫入 HTML 檔案");
    
    // Minify HTML
    let cfg = minify_html::Cfg::default();
    let minified = minify_html::minify(html.as_bytes(), &cfg);
    fs::write("stock_infos_report.min.html", minified)
        .expect("無法寫入 minified HTML 檔案");
    
    println!("✅ 已生成 stock_infos_report.html");
    println!("✅ 已生成 stock_infos_report.min.html");
}

fn generate_csv(stocks: &[EnrichedStock]) {
    let mut csv = String::from("代碼,公司名稱,英文名稱,市場,產業,上市日期,網站,主要產品\n");
    
    for stock in stocks {
        let products = if stock.main_products.is_empty() {
            stock.product_description.clone()
        } else {
            stock.main_products.join("; ")
        };
        
        csv.push_str(&format!(
            "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\n",
            stock.code,
            stock.name,
            stock.english_name,
            stock.market_type,
            stock.industry_type,
            stock.listing_date,
            stock.website,
            products.replace("\"", "\"\"")
        ));
    }
    
    fs::write("stock_infos_report.csv", csv)
        .expect("無法寫入 CSV 檔案");
    
    println!("✅ 已生成 stock_infos_report.csv");
}
