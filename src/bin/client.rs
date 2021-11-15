use std::fs::read_to_string;
use std::io::{self, Read};
use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use dirs::home_dir;
use serde::Deserialize;
use structopt::StructOpt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use clipd::frame::{RequestFrame, ResponseFrame};

#[derive(StructOpt, Debug)]
struct Opt {
    /// Pull remote clipboard
    #[structopt(long)]
    pull: bool,
    /// Path to client config file
    ///
    /// Default is ~/.config/clipd/client.toml
    #[structopt(short, long, parse(from_os_str))]
    config: Option<PathBuf>,
}

#[derive(Deserialize, Debug)]
struct Config {
    server: String,
    port: u16,
}

async fn pull(socket: &mut TcpStream) -> Result<()> {
    // Send request
    let req = RequestFrame::Pull;
    socket.write_all(&req.to_bytes()).await?;

    // Receive response
    let resp = ResponseFrame::from_socket(socket).await?;
    match resp {
        ResponseFrame::PushOk => bail!("Unexpected response: PushOk"),
        ResponseFrame::PushErr(_) => bail!("Unexpected response: PushErr"),
        ResponseFrame::PullOk(bytes) => {
            let payload = String::from_utf8(bytes)?;
            print!("{}", payload);
            Ok(())
        }
        ResponseFrame::PullErr(bytes) => {
            let err = String::from_utf8(bytes)?;
            bail!("Error pulling from server: {}", err);
        }
    }
}

async fn push(socket: &mut TcpStream) -> Result<()> {
    // Grab stdin input
    let mut input = String::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    handle.read_to_string(&mut input)?;

    // Send request
    let req = RequestFrame::Push(input.into_bytes());
    socket.write_all(&req.to_bytes()).await?;

    // Receive response
    let resp = ResponseFrame::from_socket(socket).await?;
    match resp {
        ResponseFrame::PushOk => Ok(()),
        ResponseFrame::PushErr(bytes) => {
            let err = String::from_utf8(bytes)?;
            bail!("Error pushing to server: {}", err);
        }
        ResponseFrame::PullOk(_) => bail!("Unexpected response: PullOk"),
        ResponseFrame::PullErr(_) => bail!("Unexpected response: PullErr"),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts = Opt::from_args();

    // Parse config
    let config_path = match opts.config {
        Some(p) => p,
        None => match home_dir() {
            Some(mut home) => {
                home.push(".config/clipd/client.toml");
                home
            }
            None => bail!("Could not retrieve home directory"),
        },
    };
    let config_str = read_to_string(&config_path).with_context(|| {
        format!(
            "Failed to read config file at {}",
            config_path.to_string_lossy()
        )
    })?;
    let config: Config =
        toml::from_str(&config_str).with_context(|| "Failed to parse config file".to_string())?;

    let mut socket = TcpStream::connect(format!("{}:{}", config.server, config.port)).await?;
    if opts.pull {
        pull(&mut socket).await
    } else {
        push(&mut socket).await
    }
}
