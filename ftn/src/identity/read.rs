impl ftn::Identity {
    pub async fn read(path: &std::path::Path, id: String) -> eyre::Result<Self> {
        println!("ftn::Identity::run: {path:?}, {id}");

        Ok(Self { id })
    }
}
