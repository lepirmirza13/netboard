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

    // Spawn event listener in blocking thread
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    std::thread::spawn(move || {
        if let Err(e) = rdev::listen(move |event| {
            if let Some(input_event) = InputEvent::from_rdev(&event) {
                // Send to channel (non-blocking)
                let _ = tx.send(input_event);
            }
        }) {
            eprintln!("Error listening to events: {:?}", e);
        }
    });

    println!("Client started successfully!");

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
