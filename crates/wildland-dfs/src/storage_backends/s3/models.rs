use wildland_corex::dfs::interface::UnixTimestamp;

#[derive(Debug, Clone)]
pub struct WriteResp {
    pub bytes_count: usize,
    pub new_object_name: String,
    pub new_modification_time: UnixTimestamp,
    pub new_e_tag: String,
}

#[derive(Debug, Clone)]
pub struct CreateNewEmptyResp {
    pub object_name: String,
    pub e_tag: String,
}
