use crate::protocol::InputEvent;
use anyhow::Result;
use std::net::SocketAddr;
use tokio::net::UdpSocket;

pub async fn run_server(bind_addr: SocketAddr) -> Result<()> {
    println!("Starting server mode...");
    println!("Listening on {}", bind_addr);
    println!("Press Ctrl+C to stop");

    let socket = UdpSocket::bind(bind_addr).await?;
    let mut buf = vec![0u8; 65535];

    println!("Server started successfully!");
    println!("Waiting for input from client...");

    loop {
        match socket.recv_from(&mut buf).await {
            Ok((len, _addr)) => {
                let data = &buf[..len];

                match bincode::deserialize::<InputEvent>(data) {
                    Ok(event) => {
                        // Simulate the event
                        if let Some(event_type) = event.to_rdev() {
                            if let Err(e) = rdev::simulate(&event_type) {
                                eprintln!("Failed to simulate event: {:?}", e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to deserialize event: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to receive data: {}", e);
            }
        }
    }
}
