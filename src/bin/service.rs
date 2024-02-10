use anyhow::{Context, Result};
use clap::Parser;
use ez_cd::{get_simple_install_topic, setup_tracing, AppConfig, ErrorWrapper};
use std::path::PathBuf;
use tar::Archive;
use tempdir::TempDir;
use tokio::process::Command;
use tracing::*;
use zenoh::prelude::r#async::*;

/// EZ-CD service
#[derive(Parser)]
#[command(author, version)]
struct Args {
    /// application configuration
    #[arg(long)]
    config: Option<PathBuf>,

    /// Sets the level of verbosity
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Device name
    /// By default hostname is used
    #[arg(long)]
    device_name: Option<String>,

    /// Zenoh endpoints to connect to
    #[arg(long)]
    connect: Vec<zenoh_config::EndPoint>,

    /// Zenoh endpoints to listen on
    #[arg(long)]
    listen: Vec<zenoh_config::EndPoint>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Args = Args::parse();
    setup_tracing(args.verbose);
    info!("Starting EZ-CD service");

    let hostname = hostname::get()?;
    let hostname = hostname
        .to_str()
        .to_owned()
        .context("Failed to read hostname string")?;
    info!(?hostname, "Found hostname");
    let device_name = args.device_name.unwrap_or_else(|| hostname.to_owned());
    info!(?device_name, "Using device name");

    let subscriber_topic = get_simple_install_topic(&device_name);
    info!(?subscriber_topic, "Subscribing on topic");

    let app_config = AppConfig::read_config(&args.config)?;

    info!("Starting zenoh");
    let mut zenoh_config = app_config.zenoh_config()?;
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
        .map_err(ErrorWrapper::ZenohError)?;

    let subscriber = zenoh_session
        .declare_subscriber(&subscriber_topic)
        .reliable()
        .res()
        .await
        .map_err(ErrorWrapper::ZenohError)?;

    loop {
        match subscriber.recv_async().await {
            Ok(sample) => {
                info!("Received new install command");
                let encoding = sample.encoding.clone();
                if let Ok(payload) = Vec::<u8>::try_from(sample.value) {
                    let archive = Archive::new(payload.as_slice());
                    info!("Loaded archive");
                    if let Err(err) = install_debian_package(archive).await {
                        error!(error =? err, "Failed running install command");
                    }
                } else {
                    error!(
                        "Failed to extract binary payload from message. Unexpected encoding {:?}",
                        encoding
                    );
                }
            }
            Err(err) => {
                error!(error =? err, "Error receiving zenoh message")
            }
        }
    }
}

async fn install_debian_package(mut archive: Archive<&[u8]>) -> anyhow::Result<()> {
    let tmp_dir = TempDir::new("install_directory")?;
    info!(temp_dir =? tmp_dir.path(), "Unpacking archive");

    archive.unpack(tmp_dir.path())?;

    let package_path = tmp_dir.path().join("package.deb");
    let package_exists = package_path.exists();
    if !package_exists {
        error!("Package not found in archive");
        anyhow::bail!("Package not found in archive");
    }

    info!("Installing new package");
    let output = Command::new("dpkg")
        .kill_on_drop(true)
        .current_dir(tmp_dir.path())
        .arg("--force-confold")
        .arg("-i")
        .arg(package_path)
        .output()
        .await
        .context("Failed to spawn dpkg")?;

    let stderr_output =
        std::str::from_utf8(&output.stderr).context("Failed to parse stderr to utf-8")?;
    let stdout_output =
        std::str::from_utf8(&output.stdout).context("Failed to parse stdout to utf-8")?;
    let exit_code = output.status;

    if !exit_code.success() {
        error!(
            stdout =? stdout_output,
            stderr =? stderr_output,
            exit_code =? exit_code.code(),
            "dpkg install failed",
        );
    } else {
        info!(
            stdout =? stdout_output,
            stderr =? stderr_output,
            exit_code =? exit_code.code(),
            "Package successfully installed");
    }

    Ok(())
}
