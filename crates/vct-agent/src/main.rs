//! vct-agent — headless service for the Linux dev box.
//!
//! Serves local build zips, accepts Windows→Linux uploads, streams `build`/`file`
//! events over SSE, and stages Linux→Windows files with `send`.

mod cli;
mod server;
mod state;
mod watch;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use vct_core::config;

#[derive(Parser)]
#[command(name = "vct-agent", version, about)]
struct Cli {
    /// Config file (default: `config.yml` next to this executable).
    #[arg(long, global = true)]
    config: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Write a commented default config.yml and exit.
    Init {
        /// Overwrite an existing config.
        #[arg(long)]
        force: bool,
    },
    /// Run the HTTP server.
    Run,
    /// Stage a file into the outbox for the Windows GUI to pick up.
    Send {
        file: PathBuf,
        /// Name the GUI sees (default: the file's own name).
        #[arg(long)]
        name: Option<String>,
    },
    /// Print agent status (queries a running instance if there is one).
    Status,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "vct_agent=info,vct_core=info".into()),
        )
        .init();

    let cli = Cli::parse();
    let config_path = match cli.config {
        Some(p) => p,
        None => config::default_path()?,
    };

    match cli.command {
        Command::Init { force } => cli::init(&config_path, force),
        Command::Run => cli::run(&config_path).await,
        Command::Send { file, name } => cli::send(&config_path, &file, name.as_deref()),
        Command::Status => cli::status(&config_path).await,
    }
}
