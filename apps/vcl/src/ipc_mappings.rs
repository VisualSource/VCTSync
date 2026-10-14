use nanoserde::{DeJson, SerJson};

#[derive(Debug, Default, DeJson)]
pub struct MessageDialogConfig {
    pub title: String,
    pub description: String,
    pub buttons: String,
    pub level: Option<String>,
}

#[derive(SerJson)]
pub struct MessageDialogResult {
    pub result: String,
}

impl MessageDialogResult {
    pub fn new(value: String) -> Self {
        Self { result: value }
    }
}

#[derive(Debug, SerJson)]
pub struct OpenDialogResult {
    pub path: Option<String>,
}

impl OpenDialogResult {
    pub fn new(path: Option<String>) -> Self {
        Self { path }
    }
}

#[derive(Debug, DeJson)]
pub struct InstallBuildConfig {
    id: String,
    version: String,
    source_url: String,
    content_type: String,
    timestamp: String,
    branch: String,
    digest: String,
}
