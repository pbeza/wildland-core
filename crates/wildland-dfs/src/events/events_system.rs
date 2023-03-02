use wildland_corex::dfs::interface::Event;

pub trait EventSystem {
    fn send_event(&self, event: Event);
}
