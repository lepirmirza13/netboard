use crate::protocol::InputEvent;
use anyhow::Result;
use evdev::{uinput::VirtualDeviceBuilder, AttributeSet, Key, RelativeAxisType};
use std::net::SocketAddr;
use tokio::net::UdpSocket;

pub async fn run_server(bind_addr: SocketAddr) -> Result<()> {
    println!("Starting server mode...");
    println!("Listening on {}", bind_addr);
    println!("Press Ctrl+C to stop");

    // Create virtual device with all input capabilities
    let mut keys = AttributeSet::<Key>::new();

    // Add all keyboard keys (includes mouse buttons BTN_LEFT, BTN_RIGHT, etc.)
    for key_code in 0..=0x2FF {
        keys.insert(Key(key_code));
    }

    // Relative axes for mouse movement
    let mut rel_axes = AttributeSet::<RelativeAxisType>::new();
    rel_axes.insert(RelativeAxisType::REL_X);
    rel_axes.insert(RelativeAxisType::REL_Y);
    rel_axes.insert(RelativeAxisType::REL_WHEEL);
    rel_axes.insert(RelativeAxisType::REL_HWHEEL);
    rel_axes.insert(RelativeAxisType::REL_WHEEL_HI_RES);
    rel_axes.insert(RelativeAxisType::REL_HWHEEL_HI_RES);

    let virtual_device = VirtualDeviceBuilder::new()?
        .name("NetBoard Virtual Device")
        .with_keys(&keys)?
        .with_relative_axes(&rel_axes)?
        .build()?;

    println!("Server started successfully!");
    println!("Virtual input device created");
    println!("Waiting for input from client...");

    let socket = UdpSocket::bind(bind_addr).await?;
    let mut buf = vec![0u8; 65535];

    // Use std::sync::Mutex for thread-safe access to virtual_device
    let virtual_device = std::sync::Arc::new(std::sync::Mutex::new(virtual_device));

    loop {
        match socket.recv_from(&mut buf).await {
            Ok((len, addr)) => {
                let data = &buf[..len];

                match bincode::deserialize::<InputEvent>(data) {
                    Ok(event) => {
                        let evdev_event = event.to_evdev();
                        println!("Received event: type={}, code={}, value={}",
                            event.event_type, event.code, event.value);
                        let device = virtual_device.clone();

                        // Emit in blocking task to avoid blocking async runtime
                        tokio::task::spawn_blocking(move || {
                            if let Ok(mut dev) = device.lock() {
                                if let Err(e) = dev.emit(&[evdev_event]) {
                                    eprintln!("Failed to emit event: {}", e);
                                }
                            }
                        });
                    }
                    Err(e) => {
                        eprintln!("Failed to deserialize event from {}: {}", addr, e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to receive data: {}", e);
            }
        }
    }
}
