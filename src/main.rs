use anyhow::{Context as _, Result};
use clap::Parser;
use futures::{SinkExt, StreamExt};
use tokio::{
    io::{BufReader, BufWriter},
    net::{TcpListener, TcpStream},
};
use tokio_util::codec::{FramedRead, FramedWrite};
use tracing::{debug, error, info, info_span, instrument, Instrument as _};
use verbosity::Verbosity;

use crate::codec::{Request, Response};

mod codec;
mod converter;
mod verbosity;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port to listen on.
    #[arg(short, long, default_value_t = 1178)]
    port: u16,

    #[clap(flatten)]
    verbosity: Verbosity<verbosity::ErrorLevel>,
}

#[instrument]
async fn handle_requests(mut stream: TcpStream) -> Result<()> {
    let (reader, writer) = stream.split();
    let mut source = FramedRead::new(BufReader::new(reader), codec::RequestCodec);
    let mut sink = FramedWrite::new(BufWriter::new(writer), codec::ResponseCodec);

    while let Some(request) = source.next().await {
        debug!(?request);
        let response = match request? {
            Request::CloseConnection => break,
            Request::Convert(input) => Response::Candidates(converter::convert(&input)),
            Request::GetVersion => Response::Version,
            Request::GetHostInfo => Response::HostInfo,
            Request::Complete(input) => Response::Candidates(converter::convert(&input)),
        };
        debug!(?response);
        sink.send(response).await?;
    }
    Ok(())
}

// NB: Use single thread so that we can obtain the current offset safely.
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = Args::parse();
    tracing_subscriber::fmt()
        .with_max_level(args.verbosity.level_filter())
        .init();

    debug!(?args);

    let listener = TcpListener::bind(("0.0.0.0", args.port))
        .await
        .context("Failed to create TCP socket")?;
    info!(address=%listener.local_addr()?, "Started listening");

    loop {
        match listener.accept().await {
            Ok((socket, address)) => {
                let span = info_span!("session", address=%address);
                tokio::spawn(
                    async {
                        info!("accepted a connection");
                        if let Err(error) = handle_requests(socket).await {
                            error!(?error, "Failed while handling request");
                        }
                        info!("finished handling a connection");
                    }
                    .instrument(span),
                );
            }
            Err(e) => {
                error!(error=?e, "Failed to accept request from client");
            }
        }
    }
}
