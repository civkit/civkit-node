use log::LevelFilter;
use simplelog::{CombinedLogger, ConfigBuilder, TermLogger, WriteLogger, TerminalMode};
use std::error::Error;
use std::fs::File;
use std::path::PathBuf;
use std::convert::Infallible;
use tokio::sync::mpsc;
use std::io::Write; 

pub fn get_default_data_dir() -> PathBuf {
    let home_dir = dirs::home_dir().expect("Home directory not found");

    // Determine the platform-specific path
    let platform_path = if cfg!(target_os = "linux") {
        // Path for Linux (Debian/Ubuntu)
        home_dir.join(".local/share/civkit-node")
    } else if cfg!(target_os = "macos") {
        // Path for MacOS
        home_dir.join("Library/Application Support/civkit-node")
    } else if cfg!(target_os = "windows") {
        // Path for Windows
        dirs::data_dir().expect("Data directory not found").join("civkit-node")
    } else {
        // Default path for other platforms
        home_dir.join("civkit-node")
    };

    platform_path
}

// Function to initialize the logger with the given data directory
pub fn init_logger(data_dir: &PathBuf) -> Result<(), Box<dyn Error + Send + Sync>> {
    let log_file = data_dir.join("debug.log");

    let config = ConfigBuilder::new().build();
    let log_writer = File::create(&log_file)?;
    let log_writer = WriteLogger::new(LevelFilter::Trace, config.clone(), log_writer);

    CombinedLogger::init(vec![
        TermLogger::new(LevelFilter::Info, config.clone(), TerminalMode::Mixed).unwrap(),
        log_writer,
    ])
    .map_err(|err| Box::new(err) as Box<dyn Error + Send + Sync>)
}
