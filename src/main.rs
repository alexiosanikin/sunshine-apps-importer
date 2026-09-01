use anyhow::Result;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    if let Err(e) = sunshine_stream_apps_manager::cli::run().await {
        eprintln!("Error: {e:?}");
        std::process::exit(1);
    }

    Ok(())
}
