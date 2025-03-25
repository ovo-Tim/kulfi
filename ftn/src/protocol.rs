#[derive(Debug)]
pub enum Protocol {
    Ping,
    Quit,
    Http,
    Tcp,
}

impl Protocol {
    pub fn parse(msg: Vec<u8>) -> eyre::Result<(Protocol, Vec<u8>)> {
        Ok((Protocol::Ping, msg))
    }
}
