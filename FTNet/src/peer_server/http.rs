pub async fn http(
    _fastn_port: u16,
    _send: &mut iroh::endpoint::SendStream,
    _recv: tokio_util::codec::FramedRead<iroh::endpoint::RecvStream, tokio_util::codec::LinesCodec>,
) {
}
