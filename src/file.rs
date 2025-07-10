use std::path::Path;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncReadExt, BufReader, Result};

pub async fn read_file<T: AsRef<str>>(path: T) -> Result<String> {
  let file = OpenOptions::new().read(true).open(path.as_ref()).await?;
  let mut content = String::new();
  let mut reader = BufReader::new(file);
  reader.read_to_string(&mut content).await?;

  Ok(content)
}

pub fn file_exists<T: AsRef<str>>(path: T) -> bool {
  Path::new(path.as_ref()).exists()
}
