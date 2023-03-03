use wildland_corex::dfs::interface::{Cause, Event, Operation};

use super::events_system::EventSystem;

#[derive(Clone)]
pub struct EventBuilder {
    system: Box<dyn EventSystem>,
    operation: Option<Operation>,
    operation_path: Option<String>,
    backend_type: Option<String>,
}

impl EventBuilder {
    pub fn new(system: Box<dyn EventSystem>) -> Self {
        Self {
            system,
            operation: None,
            operation_path: None,
            backend_type: None,
        }
    }

    pub fn operation(self, operation: impl Into<Operation>) -> Self {
        Self {
            operation: Some(operation.into()),
            ..self
        }
    }

    pub fn operation_path(self, operation_path: impl Into<String>) -> Self {
        Self {
            operation_path: Some(operation_path.into()),
            ..self
        }
    }

    pub fn backend_type(self, backend_type: impl Into<String>) -> Self {
        Self {
            backend_type: Some(backend_type.into()),
            ..self
        }
    }

    pub fn send(mut self, cause: Cause) {
        self.system.send_event(Event {
            operation: self.operation,
            operation_path: self.operation_path,
            backend_type: self.backend_type,
            cause,
        })
    }
}
