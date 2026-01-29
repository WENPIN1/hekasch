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
    
    let stocks: HashMap<String, StockInfo> = serde_json::from_str(&json_content)
        .expect("無法解析 JSON");
    
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
        body { font-family: Arial, sans-serif; margin: 20px; background: #f5f5f5; }
        h1 { color: #333; text-align: center; }
        .summary { background: white; padding: 15px; border-radius: 5px; margin-bottom: 20px; }
        table { width: 100%; border-collapse: collapse; background: white; }
        th { background: #2c3e50; color: white; padding: 10px; text-align: left; }
        td { padding: 10px; border-bottom: 1px solid #ddd; }
        tr:hover { background: #f9f9f9; }
        .listed { color: #27ae60; font-weight: bold; }
        .otc { color: #e74c3c; font-weight: bold; }
        a { color: #3498db; text-decoration: none; }
        a:hover { text-decoration: underline; }
    </style>
</head>
<body>
    <h1>台灣上市上櫃股票資訊</h1>
"#
    );
    
    let listed_count = stocks.iter().filter(|s| s.market_type == "上市").count();
    let otc_count = stocks.iter().filter(|s| s.market_type == "上櫃").count();
    
    html.push_str(&format!(
        r#"    <div class="summary">
        <p><strong>總計:</strong> {} 筆股票</p>
        <p><strong>上市:</strong> {} 筆 | <strong>上櫃:</strong> {} 筆</p>
    </div>
    <table>
        <thead>
            <tr>
                <th>代碼</th>
                <th>公司名稱</th>
                <th>英文名稱</th>
                <th>市場</th>
                <th>產業</th>
                <th>上市日期</th>
                <th>網站</th>
                <th>主要產品</th>
            </tr>
        </thead>
        <tbody>
"#,
        stocks.len(),
        listed_count,
        otc_count
    ));
    
    for stock in stocks {
        let market_class = if stock.market_type == "上市" { "listed" } else { "otc" };
        let website_link = if stock.website.is_empty() {
            "N/A".to_string()
        } else {
            format!(r#"<a href="{}" target="_blank">官網</a>"#, stock.website)
        };
        
        let products = if stock.main_products.is_empty() {
            stock.product_description.clone()
        } else {
            stock.main_products.join(", ")
        };
        
        html.push_str(&format!(
            r#"            <tr>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td class="{}">{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
            </tr>
"#,
            stock.code,
            stock.name,
            stock.english_name,
            market_class,
            stock.market_type,
            stock.industry_type,
            stock.listing_date,
            website_link,
            products
        ));
    }
    
    html.push_str(
        r#"        </tbody>
    </table>
</body>
</html>
"#
    );
    
    fs::write("stock_infos_report.html", html)
        .expect("無法寫入 HTML 檔案");
    
    println!("✅ 已生成 stock_infos_report.html");
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
