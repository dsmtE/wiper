use clap::Parser;
use std::sync::Arc;

use eyre::Result;
use wiper::app::{App, Arguments};
use wiper::io::handler::IoAsyncHandler;
use wiper::io::IoEvent;
use wiper::start_ui;

use wiper::utils::walker::{
    count_and_size, get_dir_list_from_path, is_node_modules,
};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Arguments::parse();
    log::debug!("{:?}", args);

    // Test
    let entries = get_dir_list_from_path(&args.root_path, &is_node_modules)
        .collect::<Vec<_>>();

    for entry in entries {
        let path = entry.path();
        let (count, size) = count_and_size(path);
        println!(
            "path : {}, files count: {} size: {:.2}MB",
            path.display(),
            count,
            size as f32 / 1000000.0
        );
    }

    let (sync_io_tx, mut sync_io_rx) = tokio::sync::mpsc::channel::<IoEvent>(100);

    // We need to share the App between thread
    let app = Arc::new(tokio::sync::Mutex::new(App::new(sync_io_tx.clone())));
    let app_ui = Arc::clone(&app);

    // Handle IO in a specifc thread
    tokio::spawn(async move {
        let mut handler = IoAsyncHandler::new(app);
        while let Some(io_event) = sync_io_rx.recv().await {
            handler.handle_io_event(io_event).await;
        }
    });

    start_ui(&app_ui, args).await?;

    Ok(())
}
