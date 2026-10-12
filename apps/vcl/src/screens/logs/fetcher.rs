use std::{path::PathBuf, sync::Arc};

use iced::task::{Straw, sipper};

pub fn fetch_log_file() -> impl Straw<Vec<String>, (), Arc<anyhow::Error>> {
    sipper(async move |_| {
        let log_path = PathBuf::new();

        let file = tokio::fs::read_to_string(log_path)
            .await
            .map_err(|err| Arc::new(anyhow::Error::from(err)))?;

        let lines = file.lines().map(|e| e.to_owned()).collect::<Vec<String>>();

        Ok(lines)
    })
}
