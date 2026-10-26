
pub enum LogFrom {
    App,
    Net,
}

pub enum LogType {
    Verbose,
    Info,
    Warning,
    Error,
}

pub struct NpLog {
    from: LogFrom,
    variant: LogType,
    body: String,
}

impl NpLog {
    pub fn new(from: LogFrom, variant: LogType, body: String) -> Self {
        Self { from, variant, body }
    }
    pub fn parse(&self) -> String {
        format!("[{}][{}] {}", self.from.as_str(), self.variant.as_str(), self.body)
    }
}

pub trait LogMetadata {
    fn as_str(&self) -> &str;
}

impl LogMetadata for LogType {
    fn as_str(&self) -> &str {
        use LogType::*;
        match self {
            Verbose => { "VERB" },
            Info => { "INFO" },
            Warning => { "WARN" },
            Error => { "ERR " },
        }
    }
}

impl LogMetadata for LogFrom {
    fn as_str(&self) -> &str {
        use LogFrom::*;
        match self {
            App => { "APP" },
            Net => { "NET" },
        }
    }
}