use std::fs::File;
use std::io::{self, Write, copy};
use std::path::{PathBuf};
use dirs::download_dir;
use reqwest::Client;
use mime_guess::get_mime_extensions_str;
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    print!("Enter the file URL (image/zip/rar): ");
    io::stdout().flush()?;

    let mut file_url = String::new();
    io::stdin().read_line(&mut file_url)?;
    let file_url = file_url.trim();

    let parsed_url = Url::parse(file_url)?;
    let fallback_name = parsed_url.path_segments().and_then(|segments| segments.last()).unwrap_or("download");

    let client = Client::new();
    let response = client.get(file_url).send().await?;

    if !response.status().is_success() {
        eprintln!("Failed to download file: {}", response.status());
        return Ok(());
    }
    
    let content_type = response.headers().get("content-type").and_then(|val| val.to_str().ok()).unwrap_or("application/octet-stream");
    
    let ext  = get_mime_extensions_str(content_type).and_then(|exts| exts.first().map(|e| e.to_string())).unwrap_or_else(|| { PathBuf::from(fallback_name).extension().and_then(|e| e.to_str()).unwrap_or("bin").to_string()});
    
    let file_name = format!("download.{}", ext);
    let mut output_path = download_dir().unwrap_or_else(|| PathBuf::from("."));
    output_path.push(file_name);
    
    let bytes = response.bytes().await?;
    let mut out = File::create(&output_path)?;
    copy(&mut bytes.as_ref(), &mut out)?;
    
    println!("File downloaded succesfully to {:?}", output_path);
    Ok(())
}
