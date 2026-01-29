use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
struct CompanyDemo {
    code: String,
    name: String,
    website: String,
    content_sample: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("🎯 Kiro AI 歸納示範");
    println!("{}", "=".repeat(80));
    println!();
    
    // 準備示範資料 (手動提供知名公司的官網和內容摘要)
    let demos = vec![
        CompanyDemo {
            code: "2330".to_string(),
            name: "台積電".to_string(),
            website: "https://www.tsmc.com".to_string(),
            content_sample: "Taiwan Semiconductor Manufacturing Company Limited (TSMC) is the world's largest dedicated independent semiconductor foundry. We provide advanced wafer semiconductor manufacturing services including 5nm, 3nm and more advanced process technologies. Our customers include Apple, AMD, NVIDIA, Qualcomm and other leading technology companies worldwide.".to_string(),
        },
        CompanyDemo {
            code: "2317".to_string(),
            name: "鴻海".to_string(),
            website: "https://www.foxconn.com".to_string(),
            content_sample: "Foxconn Technology Group is the world's largest electronics manufacturer. We provide manufacturing services for smartphones, servers, networking equipment, consumer electronics and more. Our major clients include Apple, Dell, HP, Sony and other global brands.".to_string(),
        },
        CompanyDemo {
            code: "2454".to_string(),
            name: "聯發科".to_string(),
            website: "https://www.mediatek.com".to_string(),
            content_sample: "MediaTek Inc. is a leading fabless semiconductor company that powers more than 2 billion connected devices a year. We design and develop innovative systems-on-chip (SoC) for mobile devices, home entertainment, connectivity and IoT products.".to_string(),
        },
        CompanyDemo {
            code: "2412".to_string(),
            name: "中華電".to_string(),
            website: "https://www.cht.com.tw".to_string(),
            content_sample: "Chunghwa Telecom is Taiwan's largest telecommunications company. We provide mobile services, broadband internet, fixed-line telephony, data communications and digital services to consumers and enterprises across Taiwan.".to_string(),
        },
        CompanyDemo {
            code: "2308".to_string(),
            name: "台達電".to_string(),
            website: "https://www.deltaww.com".to_string(),
            content_sample: "Delta Electronics is a global leader in power and thermal management solutions. We provide power supplies, industrial automation, building automation, renewable energy solutions and electric vehicle charging infrastructure.".to_string(),
        },
    ];
    
    // 儲存為 JSON
    let json = serde_json::to_string_pretty(&demos)?;
    fs::write("demo_companies.json", &json)?;
    
    println!("✅ 已創建示範資料: demo_companies.json");
    println!();
    println!("📋 以下是 5 家知名公司的資料,請幫我用100字以內歸納各公司的主要產品:");
    println!("{}", "=".repeat(80));
    println!();
    
    for (i, demo) in demos.iter().enumerate() {
        println!("【公司 {}】", i + 1);
        println!("代號: {}", demo.code);
        println!("名稱: {}", demo.name);
        println!("官網: {}", demo.website);
        println!("內容摘要:");
        println!("{}", demo.content_sample);
        println!();
        println!("{}", "-".repeat(80));
        println!();
    }
    
    println!("💡 請將上述內容複製,然後在 Kiro 中詢問:");
    println!();
    println!("「請根據以上 5 家公司的網站內容,用100字以內歸納各公司的主要產品或服務,");
    println!("  並以 JSON 格式回覆,格式如下:");
    println!("  {{");
    println!("    \"2330\": \"產品描述...\",");
    println!("    \"2317\": \"產品描述...\"");
    println!("  }}」");
    println!();
    println!("{}", "=".repeat(80));
    
    Ok(())
}
