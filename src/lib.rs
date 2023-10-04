use serde::{Serialize, Deserialize};

/// Struct representing a Torrent
#[derive(Serialize, Deserialize, Debug)]
pub struct Torrent {
    method: String,
    id: String,
    name: String,
    path: String,
    status: String,
    progress: f64,
    size: i64,
    total_size: i64,
    speed: i64,
}

/// Enum representing download methods
pub enum DownloadMethod {
    QBitTorrent,
    // Add other methods as needed
}

/// Adds a torrent based on the given method, name, and path
pub fn add(method: DownloadMethod, name: &str, path: &str) -> Result<String, Box<dyn std::error::Error>> {
    match method {
        DownloadMethod::QBitTorrent => {
            return wei_run::run("wei-qbittorrent",
                vec!["add", name, path]
            );
        }
        // Handle other methods as needed
    }
}

/// Lists all torrents or a specific torrent based on the given method and name
/// Returns information such as the download method, hash, name, download speed, progress, save path, status, current size, and total size
pub fn list(method: DownloadMethod, name: &str) -> Result<String, Box<dyn std::error::Error>> {
    match method {
        DownloadMethod::QBitTorrent => {
            return wei_run::run(
                "wei-qbittorrent", 
                vec![
                    "list",
                    name
                ]
            );
        }
        // Handle other methods as needed
    }
}

/// Deletes a torrent based on the given method and name
/// Options include deleting the file, or just the torrent
pub fn del(method: DownloadMethod, name: &str) -> Result<String, Box<dyn std::error::Error>> {
    match method {
        DownloadMethod::QBitTorrent => {
            return wei_run::run(
                "wei-qbittorrent", 
                vec![
                    "del",
                    name
                ]
            );
        }
        // Handle other methods as needed
    }
}