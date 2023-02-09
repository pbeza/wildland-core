use wildland_corex::dfs::interface::Stat;

#[derive(Debug, Clone)]
pub struct ObjectAttributes {
    pub stat: Stat,
    pub etag: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WriteResp {
    pub bytes_count: usize,
    pub etag: Option<String>,
}
