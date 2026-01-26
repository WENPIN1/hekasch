# 台灣股票資訊爬蟲 (Stock Crawler)

這是一個從台灣證券交易所 ISIN 網站自動抓取完整股票資訊的工具，使用 Rust 編寫。

## 🎯 特色

- ✅ **完整資料**: 自動抓取所有上市、上櫃、興櫃股票
- 📊 **大量資料**: 成功抓取 2315+ 家普通股票
- 🏭 **產業分類**: 自動分類為 95+ 個產業
- 💾 **多種格式**: 同時生成 JSON、HTML、Markdown 三種格式
- 🔍 **詳細資訊**: 包含股票代號、名稱、產業、上市日期、國際代碼
- 🌐 **外部連結**: 提供 Google 搜尋、Yahoo 財經、GoodInfo 連結
- 🚀 **自動化**: 一鍵執行，無需手動維護

## 📦 執行方式

```bash
# 使用 info level logging（推薦）
RUST_LOG=info cargo run --bin stock_crawler

# 或使用 debug level（顯示更多細節）
RUST_LOG=debug cargo run --bin stock_crawler
```

## 📊 執行結果

### 終端輸出

```
🚀 開始從台灣證券交易所抓取完整股票資訊...
📥 正在抓取 上市 股票資料...
  ✅ 成功抓取 36226 筆 上市 股票
📥 正在抓取 上櫃 股票資料...
  ✅ 成功抓取 12265 筆 上櫃 股票
📥 正在抓取 興櫃 股票資料...
  ✅ 成功抓取 362 筆 興櫃 股票
📊 總共抓取 48853 筆股票資料
🔍 過濾後剩餘 2315 筆普通股票
💾 JSON 已儲存至: stock_infos_2026-01-26.json
📄 HTML 已儲存至: taiwan_stocks_complete_2026-01-26.html
📝 Markdown 已儲存至: taiwan_stocks_complete_2026-01-26.md
✅ 所有檔案已生成完成！
```

### 生成的檔案

1. **stock_infos_YYYY-MM-DD.json** (460KB)
   - 完整的 JSON 格式資料
   - 適合程式讀取和處理
   - 包含所有股票的詳細資訊

2. **taiwan_stocks_complete_YYYY-MM-DD.html** (1.9MB)
   - 美觀的網頁格式
   - 可展開/收合的產業分類
   - 包含外部連結

3. **taiwan_stocks_complete_YYYY-MM-DD.md** (535KB)
   - Markdown 格式文件
   - 適合 GitHub 和文件系統
   - 包含完整的表格和連結

## 📋 資料內容

### 股票資訊欄位

每支股票包含以下資訊：

```json
{
  "2330": {
    "code": "2330",
    "name": "台積電",
    "market_type": "上市",
    "industry_type": "半導體業",
    "listing_date": "1994/09/05",
    "international_code": "TW0002330008"
  }
}
```

### 資料統計

- **總股票數**: 2315 家（僅包含4位數字代碼的普通股票）
- **產業分類**: 95 個
- **市場類型**: 上市、上櫃、興櫃

### 產業分類範例

- 半導體業
- 電腦及週邊設備業
- 光電業
- 通信網路業
- 電子零組件業
- 電子通路業
- 資訊服務業
- 軟體業
- 數位雲端
- 金融保險業
- 塑膠工業
- 食品工業
- 水泥工業
- 鋼鐵工業
- 汽車工業
- 航運業
- 生技醫療業
- 貿易百貨業
- 油電燃氣業
- 化學工業
- 紡織纖維
- 建材營造
- 觀光餐旅
- ... 等 95 個產業

## 🔗 提供的連結

每支股票都包含以下連結：

1. **🔍 Google 搜尋**: 快速搜尋公司官方網站
   - 格式: `https://www.google.com/search?q={代號}+{公司名稱}+公司`

2. **📊 Yahoo 財經**: 查看即時股價、技術分析、財務報表
   - 格式: `https://tw.stock.yahoo.com/quote/{代號}.TW`

3. **📈 GoodInfo**: 詳細的公司財務資訊和分析
   - 格式: `https://goodinfo.tw/tw/StockDetail.asp?STOCK_ID={代號}`

## 🛠️ 技術細節

### 資料來源

- **上市股票**: https://isin.twse.com.tw/isin/C_public.jsp?strMode=2
- **上櫃股票**: https://isin.twse.com.tw/isin/C_public.jsp?strMode=4
- **興櫃股票**: https://isin.twse.com.tw/isin/C_public.jsp?strMode=5

### 處理流程

1. 從證交所 ISIN 網站抓取 HTML 資料
2. 使用 Big5 編碼解析網頁內容
3. 解析 HTML 表格，提取股票資訊
4. 過濾出4位數字代碼的普通股票
5. 按產業分類整理資料
6. 生成 JSON、HTML、Markdown 三種格式

### 依賴套件

```toml
reqwest = { version = "0.12", features = ["blocking", "json"] }
scraper = "0.20"
chrono = "0.4"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
encoding_rs = "0.8"
log = "0.4"
env_logger = "0.11"
```

## 📝 使用範例

### 1. 抓取最新資料

```bash
RUST_LOG=info cargo run --bin stock_crawler
```

### 2. 讀取 JSON 資料

```rust
use std::fs;
use serde_json::Value;

let json_data = fs::read_to_string("stock_infos_2026-01-26.json")?;
let stocks: Value = serde_json::from_str(&json_data)?;

// 取得台積電資訊
if let Some(tsmc) = stocks.get("2330") {
    println!("公司名稱: {}", tsmc["name"]);
    println!("產業: {}", tsmc["industry_type"]);
}
```

### 3. 過濾特定產業

```rust
// 過濾出半導體業的股票
let semiconductor_stocks: Vec<_> = stocks
    .values()
    .filter(|stock| stock["industry_type"] == "半導體業")
    .collect();
```

## 🎨 HTML 版本特色

### 視覺設計
- 漸層色彩背景（紫色系）
- 白色卡片設計
- 陰影效果增加立體感
- 平滑的動畫過渡效果

### 互動功能
1. **展開/收合產業**: 點擊產業標題可展開或收合
2. **查詢公司官網**: 點擊「🔍 官網」開啟 Google 搜尋
3. **查看股價資訊**: 點擊「📊 Yahoo」查看即時股價
4. **查看財務分析**: 點擊「📈 GoodInfo」查看詳細財務資訊

### 顯示資訊
- 股票代號
- 公司名稱
- 上市日期
- 國際代碼
- 外部連結

## 📖 Markdown 版本特色

### 文件結構
- 統計資訊（總公司數、產業數、更新時間）
- 按產業分類的表格
- 可點擊的外部連結
- 完整的股票資訊

### 適用場景
- GitHub README 文件
- 技術文件
- 版本控制
- 純文字環境
- API 文件

## ⚠️ 注意事項

1. **網路連線**: 需要穩定的網路連線才能抓取資料
2. **執行時間**: 完整抓取約需 10-15 秒
3. **資料時效**: 資料來自證交所公開網站，即時更新
4. **編碼處理**: 使用 Big5 編碼解析網頁內容
5. **過濾規則**: 只保留4位數字代碼的普通股票（排除 ETF、權證等）

## 🚀 進階使用

### 定期更新

可以使用 cron 或其他排程工具定期執行：

```bash
# 每天早上 9:00 執行
0 9 * * * cd /path/to/project && RUST_LOG=info cargo run --bin stock_crawler
```

### 整合到其他系統

```rust
// 讀取 JSON 資料並整合到資料庫
use serde_json::Value;
use std::fs;

let json_data = fs::read_to_string("stock_infos_2026-01-26.json")?;
let stocks: Value = serde_json::from_str(&json_data)?;

// 插入到資料庫
for (code, info) in stocks.as_object().unwrap() {
    // 插入資料庫邏輯
}
```

### 自訂過濾條件

修改 `src/stock_crawler.rs` 中的過濾邏輯：

```rust
// 只保留上市股票
let listed_only: HashMap<String, StockInfo> = all_stocks
    .into_iter()
    .filter(|(code, stock)| {
        code.len() == 4 
        && code.chars().all(|c| c.is_numeric())
        && stock.market_type == "上市"
    })
    .collect();

// 只保留特定產業
let semiconductor_only: HashMap<String, StockInfo> = all_stocks
    .into_iter()
    .filter(|(code, stock)| {
        code.len() == 4 
        && code.chars().all(|c| c.is_numeric())
        && stock.industry_type == "半導體業"
    })
    .collect();
```

## 📚 相關資源

### 官方網站
- [台灣證券交易所](https://www.twse.com.tw/)
- [櫃買中心](https://www.tpex.org.tw/)
- [公開資訊觀測站](https://mops.twse.com.tw/)

### 資訊網站
- [Yahoo 財經](https://tw.stock.yahoo.com/)
- [GoodInfo 台灣股市資訊網](https://goodinfo.tw/)

### 參考文章
- [Get All Stock Code Information from TWSE](https://docsaid.org/en/blog/get-taiwan-all-stocks-info)

## 📄 授權

MIT License

---

*最後更新: 2026-01-26*
