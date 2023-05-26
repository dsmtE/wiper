use std::sync::Arc;

use eyre::Result;
use log::{error, info};

use super::IoEvent;
use crate::{app::{App, Arguments}, utils::walker::delete_entries};

/// In the IO thread, we handle IO event without blocking the UI thread
pub struct IoAsyncHandler {
    app: Arc<tokio::sync::Mutex<App>>,
}

impl IoAsyncHandler {
    pub fn new(app: Arc<tokio::sync::Mutex<App>>) -> Self {
        Self { app }
    }

    /// We could be async here
    pub async fn handle_io_event(&mut self, io_event: IoEvent) {
        let result: std::result::Result<(), eyre::ErrReport> = match io_event {
            IoEvent::InitializeFromArgs(args) => self.do_initialize(&args).await,
            IoEvent::DeleteEntries(entries) => self.do_selected_entries_deletion(entries.as_slice()).await,
        };

        if let Err(err) = result {
            error!("Oops, something wrong happen: {:?}", err);
        }

        let mut app = self.app.lock().await;
        app.loaded();
    }

    /// We use dummy implementation here, just wait 1s
    async fn do_initialize(&mut self, args: &Arguments) -> Result<()> {
        info!("🚀 Initialize the application");
        let mut app = self.app.lock().await;
        app.initialize_from_args(args);
        info!("👍 Application initialized");
        Ok(())
    }

    async fn do_selected_entries_deletion(&mut self, entries: &[walkdir::DirEntry]) -> Result<()> {
        info!("🚀 Delete selected entries");
        let mut app = self.app.lock().await;
        delete_entries(entries);

        app.scan_dir_update();

        info!("👍 Selected entries deleted");
        Ok(())
    }
}
