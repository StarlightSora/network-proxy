
use std::path::Path;
use time::Timestamp;
use tokio::{fs::{File, OpenOptions, create_dir_all}, io::AsyncWriteExt};

use super::datatypes::NpLog;

const MIRROR_TO_STDOUT: bool = false;
const LOGS_PREFIX: &'static str = "output/logs/"; 

#[derive(Debug)]
pub struct LogFile {
    file: File,
}

impl LogFile {
    pub async fn new(path: &Path) -> anyhow::Result<Self> {
        create_dir_all(LOGS_PREFIX).await?;
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .await?;
        Ok(Self {
            file,
        })
    }
    pub async fn from_date_time(unix_time: Timestamp) -> anyhow::Result<Self> {
        let formatted = unix_time.format(&super::datatypes::ISO8601_CFG_SEC)?;
        let formatted = Self::to_file_safe(&formatted);
        let binding = format!("{}log_{}.txt", LOGS_PREFIX, formatted);
        Self::new(Path::new(&binding)).await
    }

    pub async fn try_clone(&self) -> anyhow::Result<Self> {
        Ok(Self {
            file: self.file.try_clone().await?,
        })
    }

    pub async fn write_log(&mut self, new_log: NpLog) -> anyhow::Result<()> {
        let string_log = new_log.to_string()?;
        if MIRROR_TO_STDOUT {
            println!("{}", &string_log);
        }
        self.file.write_all((string_log + "\n").as_bytes()).await?;
        Ok(())
    }
    
    pub fn to_file_safe(incoming: &str) -> String {
        incoming.replace(":", "_") // FIXME: this does a new allocation, which is probably unnecessary
    }
}