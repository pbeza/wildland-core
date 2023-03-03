use std::num::NonZeroUsize;

use ring_channel::{ring_channel, RingReceiver, RingSender};
use wildland_corex::dfs::interface::{Event, EventReceiver};

use super::events_system::EventSystem;

#[derive(Clone)]
pub struct NonBlockingEventSystem {
    tx: RingSender<Event>,
    rx: RingReceiver<Event>,
}

impl Default for NonBlockingEventSystem {
    fn default() -> Self {
        let (tx, rx) = ring_channel(NonZeroUsize::new(100).unwrap());
        Self { tx, rx }
    }
}

impl NonBlockingEventSystem {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn get_receiver(&self) -> NonBlockingEventSubscriber {
        NonBlockingEventSubscriber {
            rx: self.rx.clone(),
        }
    }
}

impl EventSystem for NonBlockingEventSystem {
    fn send_event(&mut self, event: Event) {
        let _ = self.tx.send(event);
    }

    fn clone_box(&self) -> Box<dyn EventSystem> {
        Box::new(self.clone())
    }
}

pub struct NonBlockingEventSubscriber {
    rx: RingReceiver<Event>,
}

impl EventReceiver for NonBlockingEventSubscriber {
    fn recv(&mut self) -> Option<Event> {
        self.rx.recv().ok()
    }
}
