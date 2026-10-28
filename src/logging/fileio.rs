
use std::path::Path;
use time::Timestamp;
use tokio::{fs::{File, OpenOptions}, io::AsyncWriteExt};

use super::datatypes::NpLog;

const MIRROR_TO_STDOUT: bool = true;
const LOGS_PREFIX: &'static str = "logs/"; 

pub struct LogFile {
    file: File,
}

impl LogFile {
    pub async fn new(path: &Path) -> anyhow::Result<Self> {
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
        let binding = format!("{}log_{}.txt", LOGS_PREFIX, formatted);
        Self::new(Path::new(&binding)).await
    }

    pub async fn write_log(&mut self, new_log: NpLog) -> anyhow::Result<()> {
        let string_log = new_log.to_string()?;
        if MIRROR_TO_STDOUT {
            println!("{}", &string_log);
        }
        self.file.write_all((string_log + "\n").as_bytes()).await?;
        Ok(())
    }
}