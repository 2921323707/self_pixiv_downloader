use std::env;
use std::net::SocketAddr;

use pixiv_platform_backend::api::{AppState, serve};

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let addr = env::var("PIXIV_PLATFORM_BIND")
        .unwrap_or_else(|_| "127.0.0.1:3000".to_owned())
        .parse::<SocketAddr>()?;
    let state = AppState::from_env();

    println!("pixiv_platform_backend listening on http://{addr}");
    serve(state, addr).await?;
    Ok(())
}
