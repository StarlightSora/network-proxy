
#[derive(Debug, Clone, Copy)]
pub enum RequestType {
    Connect,
}

impl<'a> TryFrom<&'a str> for RequestType {
    type Error = &'a str;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            "CONNECT" => Ok(Self::Connect),
            something_else => Err(something_else),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NetRequest<'a> {
    kind: RequestType,
    target: &'a str,
    protocol: &'a str,
}

pub enum NetRequestError<'a> {
    InvalidRequestType(&'a str),
}

impl<'a> NetRequest<'a> {
    pub fn new(kind: &'a str, target: &'a str, protocol: &'a str) -> Result<Self, NetRequestError<'a>> {
        match RequestType::try_from(kind) {
            Ok(kind) => { Ok(Self {
            kind,
            target,
            protocol
        }) },
            Err(something_else) => { Err(NetRequestError::InvalidRequestType(something_else)) },
        }
    }
    pub fn deconstruct(self) -> (RequestType, &'a str, &'a str) {
        (self.kind, self.target, self.protocol)
    }
    pub fn request_type(&self) -> RequestType {
        self.kind
    }
    pub fn target(&self) -> &str {
        self.target
    }
    pub fn protocol(&self) -> &str {
        self.protocol
    }
}
