use std::error::Error;
use std::fs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct AITask {
    code: String,
    name: String,
    website: String,
    content: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let tasks_file = "ai_tasks.json";
    
    if !std::path::Path::new(tasks_file).exists() {
        eprintln!("❌ 找不到 ai_tasks.json 檔案");
        eprintln!("請先執行: cargo run --release --bin fetch_websites");
        return Ok(());
    }
    
    let content = fs::read_to_string(tasks_file)?;
    let tasks: Vec<AITask> = serde_json::from_str(&content)?;
    
    println!("📋 待處理的公司數量: {}", tasks.len());
    println!("\n請將以下內容複製給 Kiro,讓 AI 幫忙歸納:\n");
    println!("{}", "=".repeat(80));
    
    // 每次處理 5 家公司
    for (i, chunk) in tasks.chunks(5).enumerate() {
        println!("\n【批次 {}】\n", i + 1);
        
        for task in chunk {
            println!("公司代號: {}", task.code);
            println!("公司名稱: {}", task.name);
            println!("官網: {}", task.website);
            println!("網站內容摘要:");
            println!("{}", &task.content[..task.content.len().min(500)]);
            println!("\n請用100字以內歸納「{}」的主要產品或服務。", task.name);
            println!("{}", "-".repeat(80));
        }
        
        println!("\n{}", "=".repeat(80));
        
        if i < tasks.chunks(5).len() - 1 {
            println!("\n按 Enter 繼續下一批次...");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).ok();
        }
    }
    
    println!("\n💡 提示: 你可以將 Kiro 的回應整理後,使用 merge_results.rs 合併回資料庫");
    
    Ok(())
}
