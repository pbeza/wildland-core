use wildland_corex::dfs::interface::Event;

pub trait EventSystem {
    fn send_event(&mut self, event: Event);
    fn clone_box(&self) -> Box<dyn EventSystem>;
}

impl Clone for Box<dyn EventSystem> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
