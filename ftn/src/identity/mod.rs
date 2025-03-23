mod create;
mod read;

#[derive(Debug)]
pub struct Identity {
    pub id: String,
    pub fastn_port: u16,
}
