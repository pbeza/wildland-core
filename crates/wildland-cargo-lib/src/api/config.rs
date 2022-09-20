pub trait CargoCfg {
    fn get_log_level(&self) -> String;
    fn get_log_file(&self) -> Option<String>;
}
