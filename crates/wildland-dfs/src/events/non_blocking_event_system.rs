use std::time::Duration;

use crossbeam_channel::{unbounded, Receiver, Sender};
use wildland_corex::dfs::interface::{Event, EventSubscriber};

use super::events_system::EventSystem;

pub struct NonBlockingEventSystem {
    tx: Sender<Event>,
    rx: Receiver<Event>,
}

impl Default for NonBlockingEventSystem {
    fn default() -> Self {
        let (tx, rx) = unbounded();
        Self { tx, rx }
    }
}

impl NonBlockingEventSystem {
    pub fn new() -> Self {
        Default::default()
    }
}

impl EventSystem for NonBlockingEventSystem {
    fn send_event(&self, event: Event) {
        let _ = self.tx.try_send(event);
    }

    fn get_subscriber(&self) -> Box<dyn EventSubscriber> {
        Box::new(DFSEventSubscriber {
            rx: self.rx.clone(),
        })
    }
}

struct DFSEventSubscriber {
    rx: Receiver<Event>,
}

impl EventSubscriber for DFSEventSubscriber {
    fn pool_event(&self, millis: u64) -> Option<Event> {
        let timeout = Duration::from_millis(millis);
        self.rx.recv_timeout(timeout).ok()
    }
}
