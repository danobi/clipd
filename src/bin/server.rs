use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use log::{error, info, LevelFilter};
use simple_logger::SimpleLogger;
use structopt::StructOpt;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};

use clipd::frame::{RequestFrame, ResponseFrame};

#[derive(StructOpt, Debug)]
struct Opt {
    /// Port to run server on
    #[structopt(short, long, default_value = "3399")]
    port: u16,
}

/// Handles a single client/connection
///
/// Note that we do not return any errors here. If there are errors, we log
/// it and return. Can't really do anything else.
async fn handle_client(mut client: TcpStream, addr: SocketAddr, clipboard: Arc<Mutex<Vec<u8>>>) {
    // Grab request from the wire
    let request = match RequestFrame::from_socket(&mut client).await {
        Ok(r) => r,
        Err(e) => {
            error!("Invalid request from {}: {}", addr, e);
            return;
        }
    };

    // Dispatch the request
    let response = match request {
        RequestFrame::Push(data) => {
            info!("PUSH from {}: {}B", addr, data.len());
            let mut clipboard = clipboard.lock().unwrap();
            *clipboard = data;
            ResponseFrame::PushOk
        }
        RequestFrame::Pull => {
            info!("PULL from {}", addr);
            let clipboard = clipboard.lock().unwrap();
            ResponseFrame::PullOk(clipboard.clone())
        }
    };

    // Respond to client
    let response_bytes = response.to_bytes();
    match client.write_all(&response_bytes).await {
        Ok(_) => (),
        Err(e) => {
            error!("Failed to write reponse to to {}: {}", addr, e);
            return;
        }
    };
}

async fn serve(port: u16) -> Result<()> {
    let clipboard: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::with_capacity(4096)));
    let listener = TcpListener::bind(("0.0.0.0", port)).await?;
    info!("Listening on: 0.0.0.0:{}", port);

    loop {
        // Handle clients concurrently
        let (socket, client_addr) = listener.accept().await?;
        let clipboard = clipboard.clone();
        tokio::spawn(async move { handle_client(socket, client_addr, clipboard).await });
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts = Opt::from_args();
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap();

    serve(opts.port).await
}
