use eyre::WrapErr;

pub fn id52_to_public_key(id: &str) -> eyre::Result<iroh::PublicKey> {
    let bytes = data_encoding::BASE32_DNSSEC.decode(id.as_bytes())?;
    if bytes.len() != 32 {
        return Err(eyre::anyhow!(
            "read: id has invalid length: {}",
            bytes.len()
        ));
    }

    let bytes: [u8; 32] = bytes.try_into().unwrap(); // unwrap ok as already asserted

    iroh::PublicKey::from_bytes(&bytes).wrap_err_with(|| "failed to parse id to public key")
}

pub fn public_key_to_id52(key: &iroh::PublicKey) -> String {
    data_encoding::BASE32_DNSSEC.encode(key.as_bytes())
}

pub type FrameReader =
    tokio_util::codec::FramedRead<iroh::endpoint::RecvStream, tokio_util::codec::LinesCodec>;

pub fn frame_reader(recv: iroh::endpoint::RecvStream) -> FrameReader {
    FrameReader::new(recv, tokio_util::codec::LinesCodec::new())
}
