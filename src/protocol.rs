use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputEvent {
    KeyPress { code: u64 },
    KeyRelease { code: u64 },
    MouseMove { x: f64, y: f64 },
    MousePress { button: MouseButton },
    MouseRelease { button: MouseButton },
    MouseWheel { delta_x: i64, delta_y: i64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Unknown(u8),
}

impl InputEvent {
    pub fn from_rdev(event: &rdev::Event) -> Option<Self> {
        match event.event_type {
            rdev::EventType::KeyPress(key) => Some(InputEvent::KeyPress {
                code: key_to_code(key),
            }),
            rdev::EventType::KeyRelease(key) => Some(InputEvent::KeyRelease {
                code: key_to_code(key),
            }),
            rdev::EventType::MouseMove { x, y } => Some(InputEvent::MouseMove { x, y }),
            rdev::EventType::ButtonPress(button) => Some(InputEvent::MousePress {
                button: map_mouse_button(button),
            }),
            rdev::EventType::ButtonRelease(button) => Some(InputEvent::MouseRelease {
                button: map_mouse_button(button),
            }),
            rdev::EventType::Wheel { delta_x, delta_y } => Some(InputEvent::MouseWheel {
                delta_x,
                delta_y,
            }),
        }
    }

    pub fn to_rdev(&self) -> Option<rdev::EventType> {
        match self {
            InputEvent::KeyPress { code } => {
                Some(rdev::EventType::KeyPress(code_to_key(*code)?))
            }
            InputEvent::KeyRelease { code } => {
                Some(rdev::EventType::KeyRelease(code_to_key(*code)?))
            }
            InputEvent::MouseMove { x, y } => Some(rdev::EventType::MouseMove { x: *x, y: *y }),
            InputEvent::MousePress { button } => {
                Some(rdev::EventType::ButtonPress(unmap_mouse_button(button)))
            }
            InputEvent::MouseRelease { button } => {
                Some(rdev::EventType::ButtonRelease(unmap_mouse_button(button)))
            }
            InputEvent::MouseWheel { delta_x, delta_y } => Some(rdev::EventType::Wheel {
                delta_x: *delta_x,
                delta_y: *delta_y,
            }),
        }
    }
}

fn map_mouse_button(button: rdev::Button) -> MouseButton {
    match button {
        rdev::Button::Left => MouseButton::Left,
        rdev::Button::Right => MouseButton::Right,
        rdev::Button::Middle => MouseButton::Middle,
        rdev::Button::Unknown(n) => MouseButton::Unknown(n),
    }
}

fn unmap_mouse_button(button: &MouseButton) -> rdev::Button {
    match button {
        MouseButton::Left => rdev::Button::Left,
        MouseButton::Right => rdev::Button::Right,
        MouseButton::Middle => rdev::Button::Middle,
        MouseButton::Unknown(n) => rdev::Button::Unknown(*n),
    }
}

fn key_to_code(key: rdev::Key) -> u64 {
    // Map rdev::Key to u64 for serialization
    unsafe { std::mem::transmute::<rdev::Key, u64>(key) }
}

fn code_to_key(code: u64) -> Option<rdev::Key> {
    // Map u64 back to rdev::Key
    Some(unsafe { std::mem::transmute::<u64, rdev::Key>(code) })
}
