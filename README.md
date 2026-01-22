# IEK 產業情報網新聞爬蟲

這是一個用 Rust 編寫的網頁爬蟲，用於抓取 [IEK 產業情報網](https://ieknet.iek.org.tw/ieknews/Default.aspx) 最近 96 小時內的新聞。

## 功能特點

- 自動抓取最近 96 小時內發布的新聞
- 智能檢測：如果當天輸出檔案已存在且第一筆新聞 URL 相同，自動跳過抓取
- 支援多頁面遞增抓取（自動遞增 `currentPageIndex` 參數）
- 智能停止：當發現超出 96 小時的新聞時自動停止
- 解析新聞標題、URL、日期及詳細內容
- 識別免費/付費新聞類型
- 自動生成帶日期的 HTML 或 Markdown 報告檔案
- 使用非同步請求提升效能
- 從 `<div class="listItem row no-gutters">` 下的 `<article class="col-md listText">` 元素提取資訊
- 抓取詳細內容：標題、媒體/記者、日期、瀏覽數、內容
- 智能快取系統：將已抓取的新聞內容存入 `news_cache/` 目錄，避免重複抓取
- 自動儲存進度：每 10 則新聞自動存檔一次
- 網路請求間隔：每次請求間隔 100ms，避免伺服器負載過重
- 使用 log 系統記錄執行過程，可透過環境變數控制輸出等級

## 安裝需求

- Rust 1.70 或更高版本
- Cargo（Rust 套件管理器）

## 使用方法

### HTML 版本（預設）

1. 編譯並執行程式：

```bash
# 使用 info level logging（推薦）
RUST_LOG=info cargo run --bin ieknet_scraper

# 或使用 debug level（會顯示更多細節）
RUST_LOG=debug cargo run --bin ieknet_scraper
```

2. 或者先編譯再執行：

```bash
cargo build --release --bin ieknet_scraper
RUST_LOG=info ./target/release/ieknet_scraper
```

### Markdown 版本

1. 編譯並執行程式：

```bash
# 使用 info level logging（推薦）
RUST_LOG=info cargo run --bin ieknet_markdown

# 或使用 debug level（會顯示更多細節）
RUST_LOG=debug cargo run --bin ieknet_markdown
```

2. 或者先編譯再執行：

```bash
cargo build --release --bin ieknet_markdown
RUST_LOG=info ./target/release/ieknet_markdown
```

### Logging 等級說明

- `RUST_LOG=info` - 顯示主要進度訊息（推薦）
- `RUST_LOG=debug` - 顯示詳細的除錯訊息
- `RUST_LOG=ieknet_scraper=info` - 只顯示程式本身的 info 訊息，過濾依賴庫的訊息
- `RUST_LOG=ieknet_markdown=info` - 只顯示 markdown 版本的 info 訊息

## 輸出範例

### 終端輸出

```
正在抓取 IEK 產業情報網最近 96 小時內的新聞...

正在抓取第 1 頁...
  找到 10 則 96 小時內的新聞
正在抓取第 2 頁...
  找到 10 則 96 小時內的新聞
...
正在抓取第 10 頁...
  找到 8 則 96 小時內的新聞
  發現超出 96 小時的新聞，停止抓取

總共找到 98 則最近 96 小時內的新聞

正在抓取新聞詳細內容...
  抓取第 1/98 則新聞詳細內容...
  抓取第 2/98 則新聞詳細內容... (快取)
  💾 儲存進度 (10/98)...
  ...

✅ 已將結果儲存至: iek_news_2026-01-21.html

🧹 開始清理一週前的快取檔案...
✅ 清理完成：刪除了 15 個檔案，釋放 45230 bytes 空間
```

### HTML 檔案

程式會自動生成一個帶有日期的 HTML 檔案（例如：`iek_news_2026-01-21.html`），包含：
- 美觀的網頁排版
- 所有新聞的標題、連結、日期
- 媒體/記者、瀏覽數等詳細資訊
- 完整的新聞內容（保留原始 HTML 格式）
- 免費標籤（付費新聞不顯示標籤）
- 可點擊的連結直接跳轉到原文

### Markdown 檔案

Markdown 版本會生成 `.md` 檔案（例如：`iek_news_2026-01-21.md`），包含：
- Markdown 格式的標題和內容
- 所有新聞的標題、連結、日期
- 媒體/記者、瀏覽數等詳細資訊
- 轉換為 Markdown 格式的新聞內容
- 免費標籤（付費新聞不顯示標籤）
- 可點擊的連結

## 快取系統

程式會將每則新聞的詳細內容快取到 `news_cache/` 目錄下，檔名為新聞的 `nsl_id`。
- 如果快取檔案已存在且有內容，會直接讀取快取，不會重新抓取
- 快取檔案包含完整的 HTML 片段（標題、媒體、日期、瀏覽數、內容）
- 使用快取時會顯示 "(快取)" 標記，且不會有 100ms 延遲

### 自動清理舊快取

程式支援自動清理一週前的快取檔案，透過環境變數 `REMOVE_OLD_NEWS` 控制：

```bash
# 啟用自動清理（刪除一週前的快取檔案）
RUST_LOG=info REMOVE_OLD_NEWS=true cargo run --bin ieknet_scraper

# 或 Markdown 版本
RUST_LOG=info REMOVE_OLD_NEWS=true cargo run --bin ieknet_markdown
```

清理功能說明：
- 只有當 `REMOVE_OLD_NEWS=true` 時才會執行清理
- 自動刪除修改時間早於一週（7天）前的 `.html` 快取檔案
- 在程式正常結束前執行清理
- 顯示刪除的檔案數量和釋放的空間大小
- 使用 `RUST_LOG=debug` 可查看每個被刪除檔案的詳細資訊

## 配置選項

在 `src/main.rs` 或 `src/markdown.rs` 中可以調整以下參數：

```rust
const HOURS_RANGE: i64 = 96; // 抓取時間範圍（小時）
```

- 測試時可設為 `1` 小時
- 正式使用時設為 `96` 小時（4 天）

## 依賴套件

- `reqwest` - HTTP 客戶端
- `scraper` - HTML 解析
- `chrono` - 日期時間處理
- `tokio` - 非同步運行時
- `regex` - 正則表達式

## 工作原理

1. 從第 1 頁開始抓取 `https://ieknet.iek.org.tw/ieknews/Default.aspx`
2. 解析頁面中的新聞項目，提取標題、URL、日期
3. 過濾出 96 小時內的新聞
4. 如果發現超出 96 小時的新聞，停止抓取
5. 否則遞增 `currentPageIndex` 參數，繼續抓取下一頁
6. 對每則新聞，檢查快取是否存在：
   - 若快取存在且有內容，直接讀取
   - 若快取不存在，抓取詳細內容並存入快取
7. 每 10 則新聞自動存檔一次
8. 將所有結果輸出到終端並生成 HTML 或 Markdown 報告

## 授權

MIT License
