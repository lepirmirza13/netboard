use crate::protocol::InputEvent;
use anyhow::Result;
use std::net::SocketAddr;
use tokio::net::UdpSocket;

pub async fn run_client(server_addr: SocketAddr) -> Result<()> {
    println!("Starting client mode...");
    println!("Capturing input and sending to {}", server_addr);
    println!("Press Ctrl+C to stop");

    // Bind to any available port
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.connect(server_addr).await?;

    let socket = std::sync::Arc::new(socket);

    // Find all input devices
    let mut devices = Vec::new();
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

                        devices.push(device);
                    }
                    Err(e) => {
                        eprintln!("  Could not open {}: {}", path.display(), e);
                    }
                }
            }
        }
    }

    if devices.is_empty() {
        anyhow::bail!("No input devices found! Are you running with sudo?");
    }

    println!("Client started successfully!");
    println!("Monitoring {} input device(s)", devices.len());
    println!("Input devices are GRABBED - they won't affect this PC");
    println!("Press Ctrl+C to release devices and stop");

    // Create channel for sending events
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    // Spawn a task for each device
    for mut device in devices {
        let tx = tx.clone();
        tokio::task::spawn_blocking(move || {
            loop {
                match device.fetch_events() {
                    Ok(events) => {
                        for event in events {
                            let input_event = InputEvent::from_evdev(&event);
                            if tx.send(input_event).is_err() {
                                return; // Channel closed
                            }
                        }
                    }
                    Err(e) => {
                        if e.kind() != std::io::ErrorKind::WouldBlock {
                            eprintln!("Error reading device: {}", e);
                            return;
                        }
                        std::thread::sleep(std::time::Duration::from_millis(1));
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
