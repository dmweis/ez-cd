use anyhow::Result;
use clap::Parser;
use ez_cd::{setup_tracing, ErrorWrapper};
use std::path::PathBuf;
use tar::Builder;
use tracing::*;
use zenoh::config::Config as ZenohConfig;
use zenoh::prelude::r#async::*;

/// Hopper body controller
#[derive(Parser)]
#[command(author, version)]
struct Args {
    /// File to send for upload
    /// should be debian file for simplest use-case
    #[arg(short, long)]
    file: PathBuf,

    /// Target device
    #[arg(short, long)]
    device: String,

    /// Zenoh endpoints to connect to
    #[arg(long)]
    connect: Vec<zenoh_config::EndPoint>,

    /// Zenoh endpoints to listen on
    #[arg(long)]
    listen: Vec<zenoh_config::EndPoint>,

    /// Sets the level of verbosity
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args: Args = Args::parse();
    setup_tracing(args.verbose);

    let target_topic = format!("ezcd/{}/dpkg", args.device);

    let mut zenoh_config = ZenohConfig::default();
    zenoh_config
        .connect
        .endpoints
        .extend(args.connect.into_iter());
    zenoh_config
        .listen
        .endpoints
        .extend(args.listen.into_iter());

    let zenoh_session = zenoh::open(zenoh_config)
        .res()
        .await
        .map_err(ErrorWrapper::ZenohError)?
        .into_arc();

    info!("Archiving package");
    let mut archive = Builder::new(Vec::new());
    // ignore permissions for files
    archive.mode(tar::HeaderMode::Deterministic);

    archive.append_path_with_name(&args.file, "package.deb")?;

    let archive = archive.into_inner()?;

    info!("Sending archive on: {:?}", target_topic);
    zenoh_session
        .put(target_topic, archive)
        .res()
        .await
        .map_err(ErrorWrapper::ZenohError)?;

    Ok(())
}
