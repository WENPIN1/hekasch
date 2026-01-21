use chrono::{DateTime, Duration, Local, NaiveDate};
use regex::Regex;
use scraper::{Html, Selector};
use std::error::Error;
use tokio::time::sleep;
use std::time::Duration as StdDuration;
use std::path::Path;
use std::fs;
use log::{debug, info};

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
    // 初始化 logger
    env_logger::init();
    
    // 配置：設定要抓取的時間範圍（小時）
    const HOURS_RANGE: i64 = 96; // 測試時使用 1 小時，正式使用時改為 96
    
    info!("正在抓取 IEK 產業情報網最近 {} 小時內的新聞...\n", HOURS_RANGE);

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()?;

    let now = Local::now();
    let cutoff_time = now - Duration::hours(HOURS_RANGE);
    let mut all_news_items = Vec::new();
    let mut page_index = 1;
    let mut should_continue = true;
    
    // 檢查輸出檔案是否已存在，並讀取第一筆 URL
    let output_filename = format!("iek_news_{}.md", now.format("%Y-%m-%d"));
    let existing_first_url = get_first_url_from_markdown(&output_filename);

    while should_continue {
        let url = if page_index == 1 {
            "https://ieknet.iek.org.tw/ieknews/Default.aspx".to_string()
        } else {
            format!("https://ieknet.iek.org.tw/ieknews/Default.aspx?currentPageIndex={}", page_index)
        };

        debug!("正在抓取第 {} 頁...", page_index);
        
        let response = client.get(&url).send().await?;
        let html_content = response.text().await?;
        
        let (news_items, has_old_news) = parse_news_with_check(&html_content, &cutoff_time)?;
        
        // 如果是第一頁且有新聞，檢查第一筆 URL 是否已存在
        if page_index == 1 && !news_items.is_empty() {
            if let Some(ref existing_url) = existing_first_url {
                if news_items[0].url == *existing_url {
                    info!("✓ 新聞資料已下載（第一筆 URL 相同），結束抓取");
                    return Ok(());
                }
            }
        }
        
        let valid_count = news_items.len();
        all_news_items.extend(news_items);
        
        debug!("  找到 {} 則 {} 小時內的新聞", valid_count, HOURS_RANGE);
        
        // 如果這一頁有超出指定時間的新聞，停止抓取
        if has_old_news {
            debug!("  發現超出 {} 小時的新聞，停止抓取\n", HOURS_RANGE);
            should_continue = false;
        } else if valid_count == 0 {
            debug!("  本頁無有效新聞，停止抓取\n");
            should_continue = false;
        } else {
            page_index += 1;
        }
    }

    // 輸出結果到終端
    if all_news_items.is_empty() {
        info!("未找到最近 {} 小時內的新聞", HOURS_RANGE);
    } else {
        let total_count = all_news_items.len();
        info!("總共找到 {} 則最近 {} 小時內的新聞\n", total_count, HOURS_RANGE);
        
        // 抓取每則新聞的詳細內容
        info!("正在抓取新聞詳細內容...\n");
        let mut i = 0;
        while i < total_count {
            let item = &mut all_news_items[i];
            debug!("  抓取第 {}/{} 則新聞詳細內容...", i + 1, total_count);
            match fetch_news_detail(&client, &item.url).await {
                Ok((detail_title, media, detail_date, views, detail_content, from_cache)) => {
                    item.detail_title = detail_title;
                    item.media = media;
                    item.detail_date = detail_date;
                    item.views = views;
                    item.detail_content = detail_content;
                    
                    if from_cache {
                        debug!(" ✓ (快取)");
                    } else {
                        debug!(" ✓");
                        // 只有從網路抓取時才暫停 100 毫秒
                        sleep(StdDuration::from_millis(100)).await;
                    }
                }
                Err(e) => {
                    debug!(" ✗ (錯誤: {})", e);
                }
            }
            
            // 每 10 則新聞存檔一次
            if (i + 1) % 10 == 0 || (i + 1) == total_count {
                debug!("  💾 儲存進度 ({}/{})...", i + 1, total_count);
                if let Err(e) = generate_markdown_file(&all_news_items, &now) {
                    debug!("  ⚠️  存檔失敗: {}", e);
                }
            }
            
            i += 1;
        }
        debug!("");
        
        for (i, item) in all_news_items.iter().enumerate() {
            debug!("【新聞 {}】", i + 1);
            debug!("標題: {}", item.title);
            debug!("連結: {}", item.url);
            debug!("日期: {}", item.date);
            debug!("類型: {}", if item.is_free { "免費" } else { "付費" });
            if !item.content.is_empty() {
                debug!("摘要: {}", item.content);
            }
            debug!("{}", "-".repeat(80));
        }
    }

    // 生成 Markdown 檔案
    generate_markdown_file(&all_news_items, &now)?;

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

async fn fetch_news_detail(client: &reqwest::Client, url: &str) -> Result<(String, String, String, String, String, bool), Box<dyn std::error::Error>> {
    // 從 URL 中提取 nsl_id
    let nsl_id = extract_nsl_id(url);
    
    // 檢查快取
    if let Some(ref id) = nsl_id {
        let cache_path = format!("news_cache/{}.html", id);
        if Path::new(&cache_path).exists() {
            let metadata = fs::metadata(&cache_path)?;
            if metadata.len() > 0 {
                // 從快取讀取
                let cached_html = fs::read_to_string(&cache_path)?;
                let (detail_title, media, detail_date, views, detail_content) = parse_cached_html(&cached_html)?;
                return Ok((detail_title, media, detail_date, views, detail_content, true)); // true 表示使用快取
            }
        }
    }
    
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

    // 儲存到快取
    if let Some(ref id) = nsl_id {
        fs::create_dir_all("news_cache")?;
        let cache_path = format!("news_cache/{}.html", id);
        
        // 組合完整的 HTML fragment
        let full_fragment = format!(
            r#"<div class="cached-news">
<div class="headingCh mt-2" id="title">{}</div>
<ul class="list-inline">
<li class="list-inline-item mr-4" title="媒體、記者">{}</li>
<li class="list-inline-item mr-3" title="日期">{}</li>
<li class="list-inline-item" title="瀏覽數">{}</li>
</ul>
<div class="detailContent">{}</div>
</div>"#,
            detail_title, media, detail_date, views, detail_content
        );
        
        fs::write(&cache_path, &full_fragment)?;
    }

    Ok((detail_title, media, detail_date, views, detail_content, false)) // false 表示從網路抓取
}

fn extract_nsl_id(url: &str) -> Option<String> {
    // 從 URL 中提取 nsl_id 參數
    // 例如: https://ieknet.iek.org.tw/ieknews/news_more.aspx?actiontype=ieknews&indu_idno=0&nsl_id=2d6e228903aa4876b147cb71eb3ff878
    if let Some(query_start) = url.find("nsl_id=") {
        let id_start = query_start + 7; // "nsl_id=" 的長度
        let id_part = &url[id_start..];
        // 找到下一個 & 或字串結尾
        let id_end = id_part.find('&').unwrap_or(id_part.len());
        return Some(id_part[..id_end].to_string());
    }
    None
}

fn parse_cached_html(html: &str) -> Result<(String, String, String, String, String), Box<dyn std::error::Error>> {
    let document = Html::parse_document(html);
    
    let title_selector = Selector::parse("div.headingCh.mt-2#title").unwrap();
    let detail_title = document
        .select(&title_selector)
        .next()
        .map(|elem| elem.inner_html().trim().to_string())
        .unwrap_or_default();

    let media_selector = Selector::parse("li.list-inline-item.mr-4[title='媒體、記者']").unwrap();
    let media = document
        .select(&media_selector)
        .next()
        .map(|elem| elem.text().collect::<String>().trim().to_string())
        .unwrap_or_default();

    let date_selector = Selector::parse("li.list-inline-item.mr-3[title='日期']").unwrap();
    let detail_date = document
        .select(&date_selector)
        .next()
        .map(|elem| elem.text().collect::<String>().trim().to_string())
        .unwrap_or_default();

    let views_selector = Selector::parse("li.list-inline-item[title='瀏覽數']").unwrap();
    let views = document
        .select(&views_selector)
        .next()
        .map(|elem| elem.text().collect::<String>().trim().to_string())
        .unwrap_or_default();

    let content_selector = Selector::parse("div.detailContent").unwrap();
    let detail_content = document
        .select(&content_selector)
        .next()
        .map(|elem| elem.inner_html().trim().to_string())
        .unwrap_or_default();

    Ok((detail_title, media, detail_date, views, detail_content))
}

fn get_first_url_from_markdown(filename: &str) -> Option<String> {
    // 如果檔案不存在，返回 None
    if !Path::new(filename).exists() {
        return None;
    }
    
    // 讀取檔案內容
    let content = match fs::read_to_string(filename) {
        Ok(c) => c,
        Err(_) => return None,
    };
    
    // 使用正則表達式尋找第一個新聞項目的 URL
    // 格式: ## 1. [標題](URL)
    let re = Regex::new(r"##\s+1\.\s+\[.+?\]\((.+?)\)").unwrap();
    
    if let Some(captures) = re.captures(&content) {
        if let Some(url_match) = captures.get(1) {
            return Some(url_match.as_str().to_string());
        }
    }
    
    None
}

fn generate_markdown_file(news_items: &[NewsItem], now: &DateTime<Local>) -> Result<(), Box<dyn Error>> {
    let filename = format!("iek_news_{}.md", now.format("%Y-%m-%d"));
    
    let mut markdown = String::new();
    
    // 標題
    markdown.push_str("# IEK 產業情報網 - 最近 96 小時新聞\n\n");
    
    // 元資訊
    markdown.push_str(&format!("**抓取時間**: {}\n\n", now.format("%Y年%m月%d日 %H:%M:%S")));
    markdown.push_str(&format!("**新聞數量**: {} 則\n\n", news_items.len()));
    markdown.push_str("---\n\n");
    
    // 新聞項目
    for (i, item) in news_items.iter().enumerate() {
        // 標題與連結
        let title = if !item.detail_title.is_empty() { &item.detail_title } else { &item.title };
        markdown.push_str(&format!("## {}. [{}]({})\n\n", i + 1, title, item.url));
        
        // 資訊列
        let mut info_parts = vec![format!("📅 {}", item.date)];
        
        // 只在免費新聞時顯示 badge
        if item.is_free {
            info_parts.push("🆓 **免費**".to_string());
        }
        
        // 顯示媒體、日期、瀏覽數
        if !item.media.is_empty() {
            info_parts.push(format!("📰 {}", item.media));
        }
        if !item.detail_date.is_empty() {
            info_parts.push(format!("🕒 {}", item.detail_date));
        }
        if !item.views.is_empty() {
            info_parts.push(format!("👁 {}", item.views));
        }
        
        markdown.push_str(&info_parts.join(" | "));
        markdown.push_str("\n\n");
        
        // 顯示詳細內容
        if !item.detail_content.is_empty() {
            markdown.push_str(&convert_html_to_markdown(&item.detail_content));
        } else if !item.content.is_empty() {
            markdown.push_str(&convert_html_to_markdown(&item.content));
        }
        
        markdown.push_str("\n\n---\n\n");
    }
    
    // 頁尾
    markdown.push_str("**資料來源**: [IEK 產業情報網](https://ieknet.iek.org.tw/ieknews/Default.aspx)\n");
    
    std::fs::write(&filename, markdown)?;
    info!("\n✅ 已將結果儲存至: {}", filename);
    
    Ok(())
}

// 簡單的 HTML 轉 Markdown 函數
fn convert_html_to_markdown(html: &str) -> String {
    let mut text = html.to_string();
    
    // 移除 HTML 標籤但保留內容
    // 處理段落
    text = text.replace("<p>", "\n").replace("</p>", "\n");
    text = text.replace("<br>", "\n").replace("<br/>", "\n").replace("<br />", "\n");
    
    // 處理標題
    text = text.replace("<h1>", "\n### ").replace("</h1>", "\n");
    text = text.replace("<h2>", "\n### ").replace("</h2>", "\n");
    text = text.replace("<h3>", "\n#### ").replace("</h3>", "\n");
    text = text.replace("<h4>", "\n#### ").replace("</h4>", "\n");
    
    // 處理粗體和斜體
    text = text.replace("<strong>", "**").replace("</strong>", "**");
    text = text.replace("<b>", "**").replace("</b>", "**");
    text = text.replace("<em>", "*").replace("</em>", "*");
    text = text.replace("<i>", "*").replace("</i>", "*");
    
    // 處理列表
    text = text.replace("<ul>", "\n").replace("</ul>", "\n");
    text = text.replace("<ol>", "\n").replace("</ol>", "\n");
    text = text.replace("<li>", "- ").replace("</li>", "\n");
    
    // 處理連結 - 簡單處理，保留 URL
    // 更複雜的處理需要正則表達式
    text = text.replace("<a ", "\n[").replace("</a>", "]");
    
    // 處理 div 和 span
    text = text.replace("<div>", "\n").replace("</div>", "\n");
    text = text.replace("<span>", "").replace("</span>", "");
    
    // 移除其他常見標籤
    let tags_to_remove = vec![
        "<table>", "</table>", "<tr>", "</tr>", "<td>", "</td>", "<th>", "</th>",
        "<thead>", "</thead>", "<tbody>", "</tbody>",
        "<img>", "</img>", "<figure>", "</figure>",
    ];
    for tag in tags_to_remove {
        text = text.replace(tag, " ");
    }
    
    // 清理多餘的空白行
    while text.contains("\n\n\n") {
        text = text.replace("\n\n\n", "\n\n");
    }
    
    text.trim().to_string()
}
