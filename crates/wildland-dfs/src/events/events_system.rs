use wildland_corex::dfs::interface::{Event, EventSubscriber};

pub trait EventSystem {
    fn send_event(&self, event: Event);
    fn get_subscriber(&self) -> Box<dyn EventSubscriber>;
}
