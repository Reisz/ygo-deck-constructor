use std::{fs::File, io::Read, path::Path};

use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::Response;

const TEMPLATE: &str = "{bar} {bytes}/{total_bytes} ({eta} remaining)";

pub struct ProgressReader<R> {
    reader: R,
    progress: ProgressBar,
}

impl<R> ProgressReader<R> {
    fn init(self) -> Self {
        self.progress
            .set_style(ProgressStyle::with_template(TEMPLATE).unwrap());
        self
    }
}

impl ProgressReader<Response> {
    pub fn from_response(response: Response) -> Self {
        let progress = if let Some(length) = response.content_length() {
            ProgressBar::new(length)
        } else {
            ProgressBar::new_spinner()
        };

        Self {
            reader: response,
            progress,
        }
        .init()
    }
}

impl ProgressReader<File> {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let progress = ProgressBar::new(path.as_ref().metadata()?.len());
        let reader = File::open(path)?;
        Ok(Self { reader, progress }.init())
    }
}

impl<R: Read> Read for ProgressReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let count = self.reader.read(buf)?;
        self.progress.inc(count as u64);
        Ok(count)
    }
}
