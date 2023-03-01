use std::rc::Rc;

use wildland_corex::dfs::interface::{Cause, Event, Operation};

use super::events_system::EventSystem;

#[derive(Clone)]
pub struct EventBuilder {
    system: Rc<dyn EventSystem>,
    operation: Option<Operation>,
    operation_path: Option<String>,
    backend_type: Option<String>,
}

impl EventBuilder {
    pub fn new(system: Rc<dyn EventSystem>) -> Self {
        Self {
            system,
            operation: None,
            operation_path: None,
            backend_type: None,
        }
    }

    pub fn operation(self, operation: impl Into<Option<Operation>>) -> Self {
        Self {
            operation: operation.into(),
            ..self
        }
    }

    pub fn operation_path(self, operation_path: impl Into<Option<String>>) -> Self {
        Self {
            operation_path: operation_path.into(),
            ..self
        }
    }

    pub fn backend_type(self, backend_type: impl Into<Option<String>>) -> Self {
        Self {
            backend_type: backend_type.into(),
            ..self
        }
    }

    pub fn send(&self, cause: Cause) {
        self.system.send_event(Event {
            operation: self.operation.clone(),
            operation_path: self.operation_path.clone(),
            backend_type: self.backend_type.clone(),
            cause,
        })
    }
}
