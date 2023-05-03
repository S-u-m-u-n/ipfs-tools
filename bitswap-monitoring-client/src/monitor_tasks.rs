use std::fs::File;
use std::io::prelude::*;
use std::io::LineWriter;
use std::path::Path;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde_json::Value;
use tokio::sync::mpsc;
use tokio::time::interval;
use tokio::time::Duration;
use itertools::Itertools;

pub struct MonitorTasks {
    pub json_encoder_task: mpsc::UnboundedSender<(String, Value)>,
}

impl MonitorTasks {
    pub async fn create() -> Self {
        let (json_tx, mut json_rx) = mpsc::unbounded_channel::<(String, Value)>();
        let (file_tx, mut file_rx) = mpsc::unbounded_channel::<(String, String)>();

        // JSON encoding task
        let file_tx_clone = file_tx.clone();
        tokio::spawn(async move {
            let mut events_buffer: Vec<(String, Value)> = Vec::new();
            let buffer_duration = Duration::from_secs(60 * 2); // 1 hour

            loop {
                let timeout = tokio::time::sleep(buffer_duration);
                tokio::pin!(timeout);

                tokio::select! {
                    _ = &mut timeout => {
                        let events_by_monitor = events_buffer
                            .drain(..)
                            .into_group_map();

                        for (monitor_name, events) in events_by_monitor {
                            let json_lines = events.into_iter()
                                .map(|event| serde_json::to_string_pretty(&event).unwrap().replace("\n", "").replace(" ", "") + "\n")
                                .collect::<Vec<_>>()
                                .join("");

                            file_tx_clone.send((monitor_name, json_lines)).unwrap();
                        }
                    }
                    Some((monitor_name, event)) = json_rx.recv() => {
                        events_buffer.push((monitor_name, event));
                    }
                }
            }
        });


        // File writing task
        tokio::spawn(async move {
            let mut buffer: Vec<(String, String)> = Vec::new();
            let mut interval = interval(Duration::from_secs(120));

            loop {
                tokio::select! {
                    Some((monitor_name, line)) = file_rx.recv() => {
                        buffer.push((monitor_name, line));
                    },
                    _ = interval.tick() => {
                        if !buffer.is_empty() {
                            let mut lines_by_monitor: std::collections::HashMap<String, String> = std::collections::HashMap::new();
                            for (monitor_name, line) in buffer.drain(..) {
                                let entry = lines_by_monitor.entry(monitor_name).or_insert(String::new());
                                entry.push_str(&line);
                            }

                            for (monitor_name, lines) in lines_by_monitor {
                                if let Err(e) = MonitorTasks::batch_write_to_file(&monitor_name, &lines).await {
                                    eprintln!("Error writing to file: {}", e);
                                }
                            }
                        }
                    }
                }
            }
        });

        Self {
            json_encoder_task: json_tx,
        }
    }

    async fn batch_write_to_file(monitor_name: &str, json_lines: &str) -> std::io::Result<()> {
        let date = chrono::Utc::now().format("%Y-%m-%d-%H-%M");
        let file_name = format!("/ipfs-tools/archive/{}/{}.json.gz", monitor_name, date);
        let file_path = Path::new(&file_name);
        let file = File::create(file_path)?;
        let encoder = GzEncoder::new(file, Compression::default());
        let mut writer = LineWriter::new(encoder);
    
        writer.write_all(json_lines.as_bytes())?;
        writer.flush()?;
    
        Ok(())
    }
}