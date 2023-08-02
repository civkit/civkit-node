use std::path::PathBuf;
use dirs;

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
