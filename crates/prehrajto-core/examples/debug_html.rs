//! Debug script to inspect HTML structure from prehraj.to

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()?;
    
    println!("Fetching search results for 'Doctor Who S07E01'...\n");
    
    let html = client
        .get("https://prehraj.to/hledej/Doctor%20Who%20S07E01")
        .header("Accept-Language", "cs-CZ")
        .send()
        .await?
        .text()
        .await?;
    
    // Save HTML to file for inspection
    std::fs::write("debug_search.html", &html)?;
    println!("HTML saved to debug_search.html");
    
    // Print a snippet around video cards
    if let Some(start) = html.find("<main") {
        let snippet = &html[start..std::cmp::min(start + 5000, html.len())];
        println!("\n=== HTML snippet (first 5000 chars from <main>) ===\n");
        println!("{}", snippet);
    }
    
    Ok(())
}
