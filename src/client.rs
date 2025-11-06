use crate::protocol::InputEvent;
use anyhow::Result;
use evdev::Key;
use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::UdpSocket;

pub async fn run_client(server_addr: SocketAddr) -> Result<()> {
    println!("Starting client mode...");
    println!("Capturing input and sending to {}", server_addr);
    println!("Press Alt+Shift+Ctrl+E to stop and release devices");

    // Bind to any available port
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.connect(server_addr).await?;

    let socket = std::sync::Arc::new(socket);

    // Find all input devices and store them in Arc<Mutex> for shared access
    let mut shared_devices = Vec::new();
    for entry in std::fs::read_dir("/dev/input")? {
        let entry = entry?;
        let path = entry.path();

        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with("event") {
                match evdev::Device::open(&path) {
                    Ok(mut device) => {
                        let device_name = device.name().unwrap_or("unknown").to_string();

                        // Grab the device to prevent local input
                        if let Err(e) = device.grab() {
                            eprintln!("  Warning: Could not grab {}: {}", path.display(), e);
                        } else {
                            println!("  Grabbed device: {} ({})", device_name, path.display());
                        }

                        shared_devices.push(Arc::new(Mutex::new(device)));
                    }
                    Err(e) => {
                        eprintln!("  Could not open {}: {}", path.display(), e);
                    }
                }
            }
        }
    }

    if shared_devices.is_empty() {
        anyhow::bail!("No input devices found! Are you running with sudo?");
    }

    println!("Client started successfully!");
    println!("Monitoring {} input device(s)", shared_devices.len());
    println!("Input devices are GRABBED - they won't affect this PC");
    println!("Press Alt+Shift+Ctrl+E to exit");

    // Create channel for sending events
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    // Shared state for tracking pressed keys
    let pressed_keys = Arc::new(Mutex::new(HashSet::new()));

    // Clone device list for cleanup
    let devices_for_cleanup = shared_devices.clone();

    // Spawn a task for each device
    for device_arc in shared_devices {
        let tx = tx.clone();
        let pressed_keys = pressed_keys.clone();
        let devices_for_exit = devices_for_cleanup.clone();

        tokio::task::spawn_blocking(move || {
            loop {
                // Collect events while holding the lock
                let events = {
                    let mut device = match device_arc.lock() {
                        Ok(d) => d,
                        Err(_) => return,
                    };

                    let result = match device.fetch_events() {
                        Ok(events) => events.collect::<Vec<_>>(),
                        Err(e) => {
                            if e.kind() != std::io::ErrorKind::WouldBlock {
                                eprintln!("Error reading device: {}", e);
                                return;
                            }
                            // Device lock is dropped here automatically
                            std::thread::sleep(std::time::Duration::from_millis(1));
                            continue;
                        }
                    };
                    result
                    // Device lock is dropped here automatically
                };

                for event in events {
                    // Track key presses for exit combo detection
                    if event.event_type() == evdev::EventType::KEY {
                        let key_code = event.code();
                        let value = event.value();

                        if let Ok(mut keys) = pressed_keys.lock() {
                            if value == 1 {
                                // Key pressed
                                keys.insert(key_code);

                                // Check for exit combination: Alt+Shift+Ctrl+E
                                let has_ctrl = keys.contains(&Key::KEY_LEFTCTRL.code())
                                    || keys.contains(&Key::KEY_RIGHTCTRL.code());
                                let has_shift = keys.contains(&Key::KEY_LEFTSHIFT.code())
                                    || keys.contains(&Key::KEY_RIGHTSHIFT.code());
                                let has_alt = keys.contains(&Key::KEY_LEFTALT.code())
                                    || keys.contains(&Key::KEY_RIGHTALT.code());
                                let has_e = keys.contains(&Key::KEY_E.code());

                                if has_ctrl && has_shift && has_alt && has_e {
                                    println!("\nExit hotkey detected! Releasing devices...");

                                    // Ungrab all devices before exit
                                    for dev in devices_for_exit.iter() {
                                        if let Ok(mut d) = dev.lock() {
                                            let _ = d.ungrab();
                                        }
                                    }

                                    println!("All devices released. Exiting...");
                                    std::process::exit(0);
                                }
                            } else if value == 0 {
                                // Key released
                                keys.remove(&key_code);
                            }
                        }
                    }

                    let input_event = InputEvent::from_evdev(&event);
                    if tx.send(input_event).is_err() {
                        return; // Channel closed
                    }
                }
            }
        });
    }

    // Receive from channel and send over network
    while let Some(event) = rx.recv().await {
        match bincode::serialize(&event) {
            Ok(bytes) => {
                if let Err(e) = socket.send(&bytes).await {
                    eprintln!("Failed to send event: {}", e);
                }
            }
            Err(e) => {
                eprintln!("Failed to serialize event: {}", e);
            }
        }
    }

    Ok(())
}
