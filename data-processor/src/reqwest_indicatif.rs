use std::{
    fs::File,
    io::Read,
    path::Path,
    pin::Pin,
    task::{Context, Poll},
};

use anyhow::Result;
use futures::TryStreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use pin_project::pin_project;
use reqwest::Response;
use tokio::io::{AsyncRead, ReadBuf};
use tokio_util::compat::FuturesAsyncReadCompatExt;

const TEMPLATE: &str = "{bar} {bytes}/{total_bytes} ({eta} remaining)";

#[pin_project]
pub struct ProgressReader<R> {
    #[pin]
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

impl ProgressReader<Pin<Box<dyn AsyncRead>>> {
    pub fn from_response(response: Response) -> Self {
        let progress = if let Some(length) = response.content_length() {
            ProgressBar::new(length)
        } else {
            ProgressBar::new_spinner()
        };

        let reader = response
            .bytes_stream()
            .map_err(|e| futures::io::Error::new(futures::io::ErrorKind::Other, e))
            .into_async_read()
            .compat();

        Self {
            reader: Box::pin(reader),
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

impl<R: AsyncRead> AsyncRead for ProgressReader<R> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut ReadBuf,
    ) -> Poll<std::io::Result<()>> {
        let this = self.project();

        let start = buf.filled().len();
        let result = this.reader.poll_read(cx, buf);
        if let Poll::Ready(Ok(())) = result {
            let count = buf.filled().len() - start;
            this.progress.inc(count as u64);
        }
        result
    }
}
