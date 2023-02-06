use wildland_corex::dfs::interface::Stat;

#[derive(Debug)]
pub struct ObjectAttributes {
    pub stat: Stat,
    pub etag: Option<String>,
}

#[derive(Debug)]
pub struct WriteResp {
    pub bytes_count: usize,
    pub etag: Option<String>,
}
