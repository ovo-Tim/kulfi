impl ftnet::Identity {
    pub async fn read(
        _path: &std::path::Path,
        id: String,
        client_pools: ftnet_utils::ConnectionPools,
    ) -> eyre::Result<Self> {
        Self::from_id52(id.as_str(), client_pools)
    }
}
