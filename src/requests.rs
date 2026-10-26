pub enum RequestType {
    Connect,
}

impl RequestType {
    pub fn from_str(from: &str) -> Option<Self> {
        use RequestType::*;
        match from {
            "CONNECT" => { Some(Connect) },
            _ => { None },
        }
    }
}
