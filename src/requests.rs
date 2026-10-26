pub enum RequestType<'a> {
    Connect,
    Unsupported(&'a str)
}

impl<'a> RequestType<'a> {
    pub fn from_str(from: &'a str) -> Self {
        use RequestType::*;
        match from {
            "CONNECT" => { Connect },
            something_else => { Unsupported(something_else) },
        }
    }
}
