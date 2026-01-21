use chrono::{DateTime, Duration, Local, NaiveDate};
use regex::Regex;
use scraper::{Html, Selector};
use std::error::Error;
use tokio::time::sleep;
use std::time::Duration as StdDuration;

#[derive(Debug)]
struct NewsItem {
    title: String,
    url: String,
    date: String,
    content: String,
    is_free: bool,
    detail_title: String,
    media: String,
    detail_date: String,
    views: String,
    detail_content: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 配置：設定要抓取的時間範圍（小時）
    const HOURS_RANGE: i64 = 96; // 測試時使用 1 小時，正式使用時改為 96
    
    println!("正在抓取 IEK 產業情報網最近 {} 小時內的新聞...\n", HOURS_RANGE);

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()?;

    let now = Local::now();
    let cutoff_time = now - Duration::hours(HOURS_RANGE);
    let mut all_news_items = Vec::new();
    let mut page_index = 1;
    let mut should_continue = true;

    while should_continue {
        let url = if page_index == 1 {
            "https://ieknet.iek.org.tw/ieknews/Default.aspx".to_string()
        } else {
            format!("https://ieknet.iek.org.tw/ieknews/Default.aspx?currentPageIndex={}", page_index)
        };

        println!("正在抓取第 {} 頁...", page_index);
        
        let response = client.get(&url).send().await?;
        let html_content = response.text().await?;
        
        let (news_items, has_old_news) = parse_news_with_check(&html_content, &cutoff_time)?;
        
        let valid_count = news_items.len();
        all_news_items.extend(news_items);
        
        println!("  找到 {} 則 {} 小時內的新聞", valid_count, HOURS_RANGE);
        
        // 如果這一頁有超出指定時間的新聞，停止抓取
        if has_old_news {
            println!("  發現超出 {} 小時的新聞，停止抓取\n", HOURS_RANGE);
            should_continue = false;
        } else if valid_count == 0 {
            println!("  本頁無有效新聞，停止抓取\n");
            should_continue = false;
        } else {
            page_index += 1;
        }
    }

    // 輸出結果到終端
    if all_news_items.is_empty() {
        println!("未找到最近 {} 小時內的新聞", HOURS_RANGE);
    } else {
        let total_count = all_news_items.len();
        println!("總共找到 {} 則最近 {} 小時內的新聞\n", total_count, HOURS_RANGE);
        
        // 抓取每則新聞的詳細內容
        println!("正在抓取新聞詳細內容...\n");
        let mut i = 0;
        while i < total_count {
            let item = &mut all_news_items[i];
            print!("  抓取第 {}/{} 則新聞詳細內容...", i + 1, total_count);
            match fetch_news_detail(&client, &item.url).await {
                Ok((detail_title, media, detail_date, views, detail_content)) => {
                    item.detail_title = detail_title;
                    item.media = media;
                    item.detail_date = detail_date;
                    item.views = views;
                    item.detail_content = detail_content;
                    println!(" ✓");
                }
                Err(e) => {
                    println!(" ✗ (錯誤: {})", e);
                }
            }
            
            // 每次抓取後暫停 100 毫秒
            sleep(StdDuration::from_millis(100)).await;
            
            // 每 10 則新聞存檔一次
            if (i + 1) % 10 == 0 || (i + 1) == total_count {
                println!("  💾 儲存進度 ({}/{})...", i + 1, total_count);
                if let Err(e) = generate_html_file(&all_news_items, &now) {
                    println!("  ⚠️  存檔失敗: {}", e);
                }
            }
            
            i += 1;
        }
        println!();
        
        for (i, item) in all_news_items.iter().enumerate() {
            println!("【新聞 {}】", i + 1);
            println!("標題: {}", item.title);
            println!("連結: {}", item.url);
            println!("日期: {}", item.date);
            println!("類型: {}", if item.is_free { "免費" } else { "付費" });
            if !item.content.is_empty() {
                println!("摘要: {}", item.content);
            }
            println!("{}", "-".repeat(80));
        }
    }

    // 生成 HTML 檔案
    generate_html_file(&all_news_items, &now)?;

    Ok(())
}

fn parse_news_with_check(html: &str, cutoff_time: &DateTime<Local>) -> Result<(Vec<NewsItem>, bool), Box<dyn Error>> {
    let document = Html::parse_document(html);
    let mut news_items = Vec::new();
    let mut has_old_news = false;
    let date_re = Regex::new(r"(\d{4}/\d{1,2}/\d{1,2})")?;

    // 選擇所有 <div class="listItem row no-gutters"> 元素
    let list_item_selector = Selector::parse("div.listItem.row.no-gutters").unwrap();
    let article_selector = Selector::parse("article.col-md.listText").unwrap();
    let link_selector = Selector::parse("h2 a").unwrap();
    let date_selector = Selector::parse("li.date").unwrap();
    
    for list_item in document.select(&list_item_selector) {
        // 在 listItem 內尋找 article
        if let Some(article) = list_item.select(&article_selector).next() {
            // 提取連結和標題
            if let Some(link) = article.select(&link_selector).next() {
                let url = link
                    .value()
                    .attr("href")
                    .unwrap_or("")
                    .replace("&amp;", "&");
                
                let title = link
                    .value()
                    .attr("title")
                    .unwrap_or("")
                    .to_string();
                
                if title.is_empty() {
                    continue;
                }

                // 提取日期
                let mut date_str = String::new();
                for date_elem in article.select(&date_selector) {
                    let date_text: String = date_elem.text().collect();
                    if let Some(date_match) = date_re.find(&date_text) {
                        date_str = date_match.as_str().to_string();
                        break;
                    }
                }

                if date_str.is_empty() {
                    continue;
                }

                // 檢查日期是否在指定時間內
                if !is_within_hours(&date_str, cutoff_time) {
                    has_old_news = true;
                    continue;
                }

                // 提取其他資訊
                let article_text: String = article.text().collect();
                let is_free = article_text.contains("Free") || article_text.contains("免費");
                
                // 提取摘要
                let content = article_text
                    .lines()
                    .find(|line| {
                        let trimmed = line.trim();
                        trimmed.len() > 20 
                            && !trimmed.contains(&title)
                            && !date_re.is_match(trimmed)
                    })
                    .unwrap_or("")
                    .trim()
                    .to_string();

                // 確保 URL 是完整的
                let full_url = if url.starts_with("http") {
                    url
                } else if url.starts_with("/") {
                    format!("https://ieknet.iek.org.tw{}", url)
                } else {
                    format!("https://ieknet.iek.org.tw/{}", url)
                };

                news_items.push(NewsItem {
                    title,
                    url: full_url,
                    date: date_str,
                    content,
                    is_free,
                    detail_title: String::new(),
                    media: String::new(),
                    detail_date: String::new(),
                    views: String::new(),
                    detail_content: String::new(),
                });
            }
        }
    }

    Ok((news_items, has_old_news))
}

fn is_within_hours(date_str: &str, cutoff_time: &DateTime<Local>) -> bool {
    let date_str = date_str.replace("/", "-");
    
    if let Ok(naive_date) = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
        if let Some(date_time) = naive_date
            .and_hms_opt(0, 0, 0)
            .and_then(|dt| dt.and_local_timezone(Local).single())
        {
            return date_time >= *cutoff_time;
        }
    }

    false
}

async fn fetch_news_detail(client: &reqwest::Client, url: &str) -> Result<(String, String, String, String, String), Box<dyn std::error::Error>> {
    let response = client.get(url).send().await?;
    let html_content = response.text().await?;
    let document = Html::parse_document(&html_content);

    // 提取標題
    let title_selector = Selector::parse("div.headingCh.mt-2#title").unwrap();
    let detail_title = document
        .select(&title_selector)
        .next()
        .map(|elem| elem.inner_html().trim().to_string())
        .unwrap_or_default();

    // 提取媒體/記者
    let media_selector = Selector::parse("li.list-inline-item.mr-4[title='媒體、記者']").unwrap();
    let media = document
        .select(&media_selector)
        .next()
        .map(|elem| elem.text().collect::<String>().trim().to_string())
        .unwrap_or_default();

    // 提取日期
    let date_selector = Selector::parse("li.list-inline-item.mr-3[title='日期']").unwrap();
    let detail_date = document
        .select(&date_selector)
        .next()
        .map(|elem| elem.text().collect::<String>().trim().to_string())
        .unwrap_or_default();

    // 提取瀏覽數
    let views_selector = Selector::parse("li.list-inline-item[title='瀏覽數']").unwrap();
    let views = document
        .select(&views_selector)
        .next()
        .map(|elem| elem.text().collect::<String>().trim().to_string())
        .unwrap_or_default();

    // 提取詳細內容
    let content_selector = Selector::parse("div.detailContent").unwrap();
    let detail_content = document
        .select(&content_selector)
        .next()
        .map(|elem| elem.inner_html().trim().to_string())
        .unwrap_or_default();

    Ok((detail_title, media, detail_date, views, detail_content))
}

fn generate_html_file(news_items: &[NewsItem], now: &DateTime<Local>) -> Result<(), Box<dyn Error>> {
    let filename = format!("iek_news_{}.html", now.format("%Y-%m-%d"));
    
    let mut html = String::from(r#"<!DOCTYPE html>
<html lang="zh-TW">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>IEK 產業情報網 - 最近 96 小時新聞</title>
    <style>
        body {
            font-family: "Microsoft JhengHei", "微軟正黑體", Arial, sans-serif;
            max-width: 1200px;
            margin: 0 auto;
            padding: 20px;
            background-color: #f5f5f5;
        }
        h1 {
            color: #333;
            border-bottom: 3px solid #0066cc;
            padding-bottom: 10px;
        }
        .meta {
            color: #666;
            font-size: 14px;
            margin-bottom: 30px;
        }
        .news-item {
            background: white;
            margin-bottom: 20px;
            padding: 20px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
        .news-item h2 {
            margin-top: 0;
            color: #0066cc;
            font-size: 18px;
        }
        .news-item h2 a {
            color: #0066cc;
            text-decoration: none;
        }
        .news-item h2 a:hover {
            text-decoration: underline;
        }
        .news-info {
            color: #666;
            font-size: 14px;
            margin: 10px 0;
        }
        .news-info span {
            margin-right: 15px;
        }
        .badge {
            display: inline-block;
            padding: 3px 8px;
            border-radius: 3px;
            font-size: 12px;
            font-weight: bold;
        }
        .badge-free {
            background-color: #28a745;
            color: white;
        }
        .content {
            color: #333;
            line-height: 1.6;
            margin-top: 10px;
        }
        .footer {
            text-align: center;
            margin-top: 40px;
            padding-top: 20px;
            border-top: 1px solid #ddd;
            color: #666;
            font-size: 14px;
        }
    </style>
</head>
<body>
    <h1>IEK 產業情報網 - 最近 96 小時新聞</h1>
    <div class="meta">
        <p>抓取時間: "#);
    
    html.push_str(&now.format("%Y年%m月%d日 %H:%M:%S").to_string());
    html.push_str(&format!("</p>\n        <p>新聞數量: {} 則</p>\n    </div>\n", news_items.len()));
    
    for (i, item) in news_items.iter().enumerate() {
        html.push_str(&format!(r#"
    <div class="news-item">
        <h2>{}. <a href="{}" target="_blank">{}</a></h2>
        <div class="news-info">
            <span>📅 {}</span>"#,
            i + 1,
            item.url,
            if !item.detail_title.is_empty() { &item.detail_title } else { &item.title },
            item.date
        ));
        
        // 只在免費新聞時顯示 badge
        if item.is_free {
            html.push_str(r#"
            <span class="badge badge-free">免費</span>"#);
        }
        
        // 顯示媒體、日期、瀏覽數
        if !item.media.is_empty() {
            html.push_str(&format!(r#"
            <span>📰 {}</span>"#, item.media));
        }
        if !item.detail_date.is_empty() {
            html.push_str(&format!(r#"
            <span>🕒 {}</span>"#, item.detail_date));
        }
        if !item.views.is_empty() {
            html.push_str(&format!(r#"
            <span>👁 {}</span>"#, item.views));
        }
        
        html.push_str("\n        </div>");
        
        // 顯示詳細內容
        if !item.detail_content.is_empty() {
            html.push_str(&format!(r#"
        <div class="content">{}</div>"#, item.detail_content));
        } else if !item.content.is_empty() {
            html.push_str(&format!(r#"
        <div class="content">{}</div>"#, item.content));
        }
        
        html.push_str("\n    </div>");
    }
    
    html.push_str(r#"
    <div class="footer">
        <p>資料來源: <a href="https://ieknet.iek.org.tw/ieknews/Default.aspx" target="_blank">IEK 產業情報網</a></p>
    </div>
</body>
</html>"#);
    
    std::fs::write(&filename, html)?;
    println!("\n✅ 已將結果儲存至: {}", filename);
    
    Ok(())
}
