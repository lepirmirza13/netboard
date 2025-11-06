use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputEvent {
    pub event_type: u16,
    pub code: u16,
    pub value: i32,
}

impl InputEvent {
    pub fn from_evdev(event: &evdev::InputEvent) -> Self {
        InputEvent {
            event_type: event.event_type().0,
            code: event.code(),
            value: event.value(),
        }
    }

    pub fn to_evdev(&self) -> evdev::InputEvent {
        evdev::InputEvent::new(
            evdev::EventType(self.event_type),
            self.code,
            self.value,
        )
    }
}
