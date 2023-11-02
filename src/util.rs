use nostr::Event;

use log::LevelFilter;
use simplelog::{CombinedLogger, ConfigBuilder, TermLogger, WriteLogger, TerminalMode};
use std::error::Error;
use std::fs::File;
use std::path::PathBuf;
use std::convert::Infallible;
use tokio::sync::mpsc;
use std::io::Write; 
use std::fs;

use nostr::TagKind;

pub fn get_default_data_dir() -> PathBuf {
    let home_dir = dirs::home_dir().expect("Home directory not found");

    // Determine the platform-specific path
    let platform_path = if cfg!(target_os = "linux") {
        // Path for Linux (Debian/Ubuntu)
        home_dir.join("./civkit-node")
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
pub fn init_logger(data_dir: &PathBuf, log_level: &str ) -> Result<(), Box<dyn Error + Send + Sync>> {
    
    if !data_dir.exists() {
        fs::create_dir_all(data_dir)?;
    }
    
    let log_file = data_dir.join("debug.log");
    let config = ConfigBuilder::new().build();
    let level_filter = match log_level {
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => panic!("Invalid log level in config"),
    };

    let log_writer = File::create(&log_file)?;
    let file_logger = WriteLogger::new(level_filter, config.clone(), log_writer);
    let term_logger = TermLogger::new(level_filter, config, TerminalMode::Mixed).unwrap();

    CombinedLogger::init(vec![file_logger, term_logger])
        .map_err(|err| Box::new(err) as Box<dyn Error + Send + Sync>)
}

// Function to assert if an event is a NIP-16 ephemeral event
pub fn is_ephemeral(ev: &Event) -> bool {
	if 20000 <= ev.kind.as_u32() && ev.kind.as_u32() < 30000 {
		return true;
	}
	return false;
}

// Function to assert if an event is a NIP-16 repleceable event
pub fn is_replaceable(ev: &Event) -> bool {
	if 10000 <= ev.kind.as_u32() && ev.kind.as_u32() < 20000 {
		return true;
	}
	return false;
}

// Function to assert if an event is a credential msg
pub fn is_credential(ev: &Event) -> bool {
	for tag in &ev.tags {
		if tag.kind() == TagKind::Credential {
			return true;
		}
	}
	return false;
}
