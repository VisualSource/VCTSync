//! Talking to the Linux agent, and tailing the local BepInEx log.

use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result};
use futures::{SinkExt, Stream};
use vct_core::build::LocalBuild;
use vct_core::http::client;
use vct_core::protocol::{BuildsResponse, Event, OutboxItem, OutboxResponse};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Agent {
    /// Normalised base URL, e.g. `http://192.168.1.94:9787`.
    pub base: String,
    pub token: String,
}

impl Agent {
    pub fn new(addr: &str, token: &str) -> Option<Self> {
        let addr = addr.trim();
        if addr.is_empty() || token.trim().is_empty() {
            return None;
        }
        let base = if addr.starts_with("http://") || addr.starts_with("https://") {
            addr.trim_end_matches('/').to_string()
        } else {
            format!("http://{}", addr.trim_end_matches('/'))
        };
        Some(Self {
            base,
            token: token.trim().to_string(),
        })
    }

    pub async fn list_builds(self) -> Result<Vec<LocalBuild>> {
        let r: BuildsResponse = client()
            .get(format!("{}/builds", self.base))
            .bearer_auth(&self.token)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(r.builds)
    }

    pub async fn download_build(self, id: String) -> Result<Vec<u8>> {
        let b = client()
            .get(format!("{}/builds/{id}/download", self.base))
            .bearer_auth(&self.token)
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;
        Ok(b.to_vec())
    }

    pub async fn upload_file(self, name: String, bytes: Vec<u8>) -> Result<()> {
        client()
            .post(format!("{}/files", self.base))
            .query(&[("name", name.as_str())])
            .bearer_auth(&self.token)
            .body(bytes)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn outbox_list(self) -> Result<Vec<OutboxItem>> {
        let r: OutboxResponse = client()
            .get(format!("{}/outbox", self.base))
            .bearer_auth(&self.token)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(r.items)
    }

    pub async fn outbox_get(self, name: String) -> Result<Vec<u8>> {
        let b = client()
            .get(format!("{}/outbox/{name}", self.base))
            .bearer_auth(&self.token)
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;
        Ok(b.to_vec())
    }

    pub async fn outbox_delete(self, name: String) -> Result<()> {
        client()
            .delete(format!("{}/outbox/{name}", self.base))
            .bearer_auth(&self.token)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}

// ---- SSE subscription -----------------------------------------------------

#[derive(Debug, Clone)]
pub enum SseMsg {
    Connected,
    Disconnected,
    Event(Event),
}

/// Long-lived `/events` stream with reconnect. `fn(&Agent) -> impl Stream` so it
/// fits `Subscription::run_with`.
pub fn sse_sub(agent: &Agent) -> impl Stream<Item = SseMsg> + Send + use<> {
    let agent = agent.clone();
    iced::stream::channel(64, async move |mut out| {
        loop {
            if let Err(e) = pump_sse(&agent, &mut out).await {
                tracing::debug!("sse: {e:#}");
            }
            let _ = out.send(SseMsg::Disconnected).await;
            tokio::time::sleep(Duration::from_secs(3)).await;
        }
    })
}

async fn pump_sse(
    agent: &Agent,
    out: &mut futures::channel::mpsc::Sender<SseMsg>,
) -> Result<()> {
    use futures::StreamExt;

    let resp = client()
        .get(format!("{}/events", agent.base))
        .bearer_auth(&agent.token)
        .send()
        .await?
        .error_for_status()?;
    out.send(SseMsg::Connected).await.ok();

    let mut body = resp.bytes_stream();
    let mut buf = String::new();
    while let Some(chunk) = body.next().await {
        buf.push_str(&String::from_utf8_lossy(&chunk?));
        while let Some(pos) = buf.find("\n\n") {
            let block: String = buf.drain(..pos + 2).collect();
            for line in block.lines() {
                if let Some(data) = line.strip_prefix("data:") {
                    if let Ok(ev) = serde_json::from_str::<Event>(data.trim()) {
                        out.send(SseMsg::Event(ev)).await.ok();
                    }
                }
            }
        }
    }
    Ok(())
}

// ---- local log tail -----------------------------------------------------------

#[derive(Debug, Clone)]
pub enum LogMsg {
    /// The file was (re)opened — clear the view.
    Reset,
    Lines(Vec<String>),
}

/// Follow `path`, coping with the game truncating it on relaunch.
pub fn log_sub(path: &PathBuf) -> impl Stream<Item = LogMsg> + Send + use<> {
    let path = path.clone();
    iced::stream::channel(64, async move |mut out| {
        let mut last_len: u64 = 0;
        let mut carry = String::new();
        let mut announced_missing = false;

        loop {
            match tokio::fs::metadata(&path).await {
                Ok(meta) => {
                    announced_missing = false;
                    let len = meta.len();
                    if len < last_len {
                        // truncated → relaunch
                        last_len = 0;
                        carry.clear();
                        out.send(LogMsg::Reset).await.ok();
                    }
                    if len > last_len {
                        if let Ok(bytes) = read_range(&path, last_len, len).await {
                            last_len = len;
                            carry.push_str(&String::from_utf8_lossy(&bytes));
                            let mut lines: Vec<String> = Vec::new();
                            while let Some(nl) = carry.find('\n') {
                                let line: String = carry.drain(..nl + 1).collect();
                                lines.push(line.trim_end().to_string());
                            }
                            if !lines.is_empty() {
                                out.send(LogMsg::Lines(lines)).await.ok();
                            }
                        }
                    }
                }
                Err(_) if !announced_missing => {
                    announced_missing = true;
                    last_len = 0;
                    out.send(LogMsg::Reset).await.ok();
                }
                Err(_) => {}
            }
            tokio::time::sleep(Duration::from_millis(700)).await;
        }
    })
}

async fn read_range(path: &PathBuf, from: u64, to: u64) -> Result<Vec<u8>> {
    use tokio::io::{AsyncReadExt, AsyncSeekExt};
    let mut f = tokio::fs::File::open(path).await.context("open log")?;
    f.seek(std::io::SeekFrom::Start(from)).await?;
    let mut buf = vec![0u8; (to - from) as usize];
    f.read_exact(&mut buf).await?;
    Ok(buf)
}
