use clap::Parser;
use eyre::Result;
use wiper::{app::{App, Arguments}, start_ui};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Arguments::parse();
    log::debug!("{:?}", args);

    // We need to share the App between thread
    let mut app = App::new_from_args(&args);

    start_ui(&mut app).await?;

    Ok(())
}
