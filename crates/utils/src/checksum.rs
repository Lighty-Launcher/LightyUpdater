use anyhow::Result;
use sha1::{Digest, Sha1};
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

const BUFFER_SIZE: usize = 8192; // 8KB buffer for streaming

pub async fn compute_sha1<P: AsRef<Path>>(path: P) -> Result<String> {
    let mut file = File::open(path).await?;
    let mut hasher = Sha1::new();
    let mut buffer = vec![0u8; BUFFER_SIZE];

    loop {
        let bytes_read = file.read(&mut buffer).await?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    Ok(hex::encode(result))
}

pub fn compute_sha1_sync<P: AsRef<Path>>(path: P) -> Result<String> {
    let (sha1, _) = compute_sha1_with_size_sync(path)?;
    Ok(sha1)
}

pub fn compute_sha1_with_size_sync<P: AsRef<Path>>(path: P) -> Result<(String, u64)> {
    use std::fs::File;
    use std::io::Read;

    let mut file = File::open(path)?;
    let mut hasher = Sha1::new();
    let mut buffer = vec![0u8; BUFFER_SIZE];
    let mut total_bytes = 0u64;

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
        total_bytes += bytes_read as u64;
    }

    let result = hasher.finalize();
    Ok((hex::encode(result), total_bytes))
}
