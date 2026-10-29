use time::{Timestamp, format_description::well_known::{Iso8601, iso8601}};

pub const ISO8601_CFG_DECI: Iso8601<{
    iso8601::Config::DEFAULT
        .set_formatted_components(iso8601::FormattedComponents::DateTime)
        .encode()
}> = Iso8601::<_>;
pub const ISO8601_CFG_SEC: Iso8601<{
    iso8601::Config::DEFAULT
        .set_formatted_components(iso8601::FormattedComponents::DateTime)
        .set_time_precision(iso8601::TimePrecision::Second { decimal_digits: None })
        .encode()
}> = Iso8601::<_>;

#[derive(Debug, Clone, Copy)]
pub enum LogFrom {
    App,
    Net,
}

#[derive(Debug, Clone, Copy)]
pub enum LogType {
    Verbose,
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct NpLog {
    from: LogFrom,
    variant: LogType,
    time: Timestamp,
    body: String,
}

impl NpLog {
    pub fn new(from: LogFrom, variant: LogType, time: Timestamp, body: String) -> Self {
        Self { from, variant, time, body }
    }
    pub fn now(from: LogFrom, variant: LogType, body: String) -> Self {
        Self::new(
            from,
            variant,
            Timestamp::now(),
            body,
        )
    }
    pub fn to_string(&self) -> anyhow::Result<String> {
        let fmt_time = self.time.format(&ISO8601_CFG_DECI)?;
        Ok(format!("[{}][{}][{}] {}", fmt_time, self.from.as_str(), self.variant.as_str(), self.body))
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
            Error => { "EROR" },
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