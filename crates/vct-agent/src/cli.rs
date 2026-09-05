use std::net::SocketAddr;
use std::path::Path;

use anyhow::{Context, Result};
use vct_core::config::{self, AgentConfig};
use vct_core::http;
use vct_core::protocol::StatusResponse;

use crate::server;
use crate::state::AppState;
use crate::watch;

const CONFIG_HEADER: &str = "\
# vct-agent config. Lives next to the executable; override with --config.
#
#   bind             host:port for the HTTP server (0.0.0.0 = all interfaces)
#   token            shared secret; the Windows GUI config must carry the same string
#   scan_dirs        directories watched for VoidCrewTerminus-*.zip build outputs
#   incoming_dir     where Windows->Linux uploads (POST /files) are written
#   outbox_dir       where `vct-agent send <file>` stages Linux->Windows files
#   max_upload_bytes reject uploads larger than this
";

pub fn init(config_path: &Path, force: bool) -> Result<()> {
    if config_path.exists() && !force {
        anyhow::bail!(
            "{} already exists (use --force to overwrite)",
            config_path.display()
        );
    }
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    let base = std::env::current_dir().unwrap_or_else(|_| ".".into());
    let cfg = AgentConfig::scaffold(&base);
    let body = serde_yaml_ng::to_string(&cfg)?;
    std::fs::write(config_path, format!("{CONFIG_HEADER}\n{body}"))
        .with_context(|| format!("writing {}", config_path.display()))?;

    println!("wrote {}", config_path.display());
    println!("token: {}", cfg.token);
    println!("edit scan_dirs / incoming_dir / outbox_dir, then `vct-agent run`.");
    Ok(())
}

pub async fn run(config_path: &Path) -> Result<()> {
    let cfg: AgentConfig = config::load(config_path).with_context(|| {
        format!(
            "no usable config at {} — run `vct-agent init` first",
            config_path.display()
        )
    })?;
    cfg.validate()?;

    std::fs::create_dir_all(&cfg.incoming_dir).ok();
    std::fs::create_dir_all(&cfg.outbox_dir).ok();

    let addr: SocketAddr = cfg.bind.parse()?;
    let state = AppState::new(cfg);
    watch::spawn(state.clone());

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("binding {addr}"))?;
    tracing::info!("vct-agent listening on http://{addr}");

    axum::serve(listener, server::app(state))
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
            tracing::info!("shutting down");
        })
        .await?;
    Ok(())
}

pub fn send(config_path: &Path, file: &Path, name: Option<&str>) -> Result<()> {
    let cfg: AgentConfig = config::load(config_path)?;
    anyhow::ensure!(file.is_file(), "{} is not a file", file.display());

    let raw = name
        .map(str::to_string)
        .or_else(|| file.file_name().map(|n| n.to_string_lossy().into_owned()))
        .context("could not determine a name for the file")?;
    let name = server::safe_name(&raw).with_context(|| format!("unsafe name: {raw:?}"))?;

    std::fs::create_dir_all(&cfg.outbox_dir)?;
    let dest = cfg.outbox_dir.join(&name);
    std::fs::copy(file, &dest).with_context(|| format!("copying to {}", dest.display()))?;

    println!("staged {} -> {}", file.display(), dest.display());
    println!("a running agent will notify the GUI; otherwise it shows on the GUI's next connect.");
    Ok(())
}

pub async fn status(config_path: &Path) -> Result<()> {
    let cfg: AgentConfig = config::load(config_path)?;

    println!("config:       {}", config_path.display());
    println!("bind:         {}", cfg.bind);
    println!("scan_dirs:");
    for d in &cfg.scan_dirs {
        let mark = if d.is_dir() { "" } else { "  (missing)" };
        println!("  {}{}", d.display(), mark);
    }
    println!("incoming_dir: {}", cfg.incoming_dir.display());
    let pending = server::outbox::list(&cfg.outbox_dir);
    println!(
        "outbox_dir:   {} ({} pending)",
        cfg.outbox_dir.display(),
        pending.len()
    );

    let url = format!("http://{}/status", loopback(&cfg.bind));
    match http::client()
        .get(&url)
        .bearer_auth(&cfg.token)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => match resp.json::<StatusResponse>().await {
            Ok(s) => {
                println!("\nrunning:        yes");
                println!("connected GUIs: {}", s.connected_guis);
                println!("builds indexed: {}", s.builds);
            }
            Err(e) => println!("\nrunning:        yes, but /status did not parse: {e}"),
        },
        Ok(resp) => println!("\nrunning:        responded {} on {url}", resp.status()),
        Err(_) => println!("\nrunning:        no (nothing answered on {url})"),
    }
    Ok(())
}

/// Turn a bind address into something connectable from this host.
fn loopback(bind: &str) -> String {
    match bind.parse::<SocketAddr>() {
        Ok(a) if a.ip().is_unspecified() && a.is_ipv4() => format!("127.0.0.1:{}", a.port()),
        Ok(a) if a.ip().is_unspecified() => format!("[::1]:{}", a.port()),
        _ => bind.to_string(),
    }
}
