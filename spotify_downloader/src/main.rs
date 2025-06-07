use colored::*;
use dotenvy::dotenv;
use indicatif::{ProgressBar, ProgressStyle};
use rspotify::{ClientCredsSpotify, Credentials};
use rspotify::clients::BaseClient;
use rspotify::model::TrackId;
use std::env;
use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use std::thread;
use std::time::Duration;

#[tokio::main]
async fn main() {
    dotenv().ok();

    println!("{}", "Enter a Spotify track URL: ".bright_blue());
    io::stdout().flush().unwrap();

    let mut spotify_url = String::new();
    io::stdin().read_line(&mut spotify_url).expect("Failed to read input");
    
    let spotify_url = spotify_url.split('?').next().unwrap().trim();

    let client_id = env::var("RSPOTIFY_CLIENT_ID").expect("Missing RSPOTIFY_CLIENT_ID");
    let client_secret = env::var("RSPOTIFY_CLIENT_SECRET").expect("Missing RSPOTIFY_CLIENT_SECRET");

    let creds = Credentials::new(&client_id, &client_secret);
    let spotify = ClientCredsSpotify::new(creds);

    let spinner = ProgressBar::new_spinner();
    spinner.set_message("Authenticating with Spotify...".bright_yellow().to_string());
    spinner.enable_steady_tick(Duration::from_millis(100));
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ")
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );

    spotify.request_token().await.expect("Spotify authentication failed.");
    
    spinner.set_message(format!("Fetching track info for {}", spotify_url.bright_green()));

    let track_id_str = spotify_url.split('/').last().unwrap().split('?').next().unwrap();

    let spotify_uri = format!("spotify:track:{}", track_id_str);

    let track_id = TrackId::from_uri(&spotify_uri).expect("Invalid Spotify track");
    
    let track = spotify.track(track_id, None).await.expect("Track not found.");

    spinner.finish_and_clear();

    let query = format!("{} - {}", track.artists[0].name, track.name);
    println!(
        "{} {}",
        "🎵 Searching and downloading:".bright_green(),
        query.bright_white().bold()
    );

    let pb = ProgressBar::new(10);
    pb.set_style(
        ProgressStyle::with_template("[{bar:10.cyan/blue}] {pos:>2}/{len:2} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb.set_message("📥 Downloading...");

    let loading = Arc::new(AtomicBool::new(true));
    let loading_clone = loading.clone();

    let handle = thread::spawn(move || {
        let mut i = 0;
        while loading_clone.load(Ordering::SeqCst) {
            pb.set_position(i % 11);
            thread::sleep(Duration::from_millis(300));
            i += 1;
        }
        pb.finish_with_message("✅ Download complete");
    });

    let status = Command::new("yt-dlp")
        .arg("-x")
        .arg("--audio-format")
        .arg("mp3")
        .arg(&format!("ytsearch1:{}", query))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("Failed to run yt-dlp");

    loading.store(false, Ordering::SeqCst);
    handle.join().unwrap();

    if status.success() {
        println!("{}", "✅ MP3 downloaded successfully.".bright_green().bold());
    } else {
        eprintln!("{}", "❌ Download failed.".bright_red().bold());
    }
}
