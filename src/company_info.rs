use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StockInfo {
    pub code: String,
    pub name: String,
    pub market_type: String,
    pub industry_type: String,
    pub listing_date: String,
    pub international_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated: Option<String>,
}

pub type StockDatabase = HashMap<String, StockInfo>;

pub fn load_stock_database(filename: &str) -> Result<StockDatabase, Box<dyn Error>> {
    let content = fs::read_to_string(filename)?;
    let database: StockDatabase = serde_json::from_str(&content)?;
    Ok(database)
}

pub fn save_stock_database(filename: &str, database: &StockDatabase) -> Result<(), Box<dyn Error>> {
    let json = serde_json::to_string_pretty(database)?;
    fs::write(filename, json)?;
    Ok(())
}

pub fn needs_update(stock: &StockInfo) -> bool {
    // 如果沒有產品描述或官網,需要更新
    stock.product_description.is_none() || stock.website.is_none()
}
