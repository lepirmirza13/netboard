mod client;
mod protocol;
mod server;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::net::SocketAddr;

#[derive(Parser)]
#[command(name = "netboard")]
#[command(about = "Ultra-fast network keyboard and mouse sharing", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run as client (send input to server)
    Client {
        /// Server address to connect to (e.g., 192.168.1.100:9999)
        #[arg(short, long)]
        server: String,
    },
    /// Run as server (receive and inject input)
    Server {
        /// Address to bind to (e.g., 0.0.0.0:9999)
        #[arg(short, long, default_value = "0.0.0.0:9999")]
        bind: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Client { server } => {
            let addr: SocketAddr = server.parse()?;
            client::run_client(addr).await?;
        }
        Commands::Server { bind } => {
            let addr: SocketAddr = bind.parse()?;
            server::run_server(addr).await?;
        }
    }

    Ok(())
}
