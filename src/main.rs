use std::sync::Arc;
use wifi_gui::tui::TuiManager;
use zbus::{self, Connection};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Arc::new(Connection::system().await?);

    if let Err(e) = TuiManager::run(conn).await {
        eprintln!("Application error: {}", e);
        std::process::exit(1);
    }
    Ok(())
}
