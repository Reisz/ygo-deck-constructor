use std::{fmt::Display, pin::Pin, time::Instant};

use futures::{stream::FuturesUnordered, Future, Stream, TryStreamExt};
use indicatif::{
    HumanBytes, HumanDuration, MultiProgress, ProgressBar, ProgressBarIter, ProgressStyle,
};
use log::{info, Level, LevelFilter, Log};
use reqwest::{Client, IntoUrl};
use tokio::io::{AsyncRead, ReadBuf};
use tokio_util::compat::FuturesAsyncReadCompatExt;

pub struct UiManager {
    progress_bars: MultiProgress,
    client: Client,
    download_bar_style: ProgressStyle,
    download_spinner_style: ProgressStyle,
    iterator_style: ProgressStyle,
}

impl UiManager {
    fn prefix(level: Level) -> impl Display {
        match level {
            Level::Trace => console::style(".").bold().white(),
            Level::Debug => console::style("?").bold().cyan(),
            Level::Info => console::style(">").bold().blue(),
            Level::Warn => console::style("!").bold().yellow(),
            Level::Error => console::style("X").bold().red(),
        }
    }

    pub fn new() -> Self {
        let download_message = format!("{} Downloading {{msg:15}}", Self::prefix(Level::Info));
        let download_bar = "{bar:30} {remaining_bytes:>10} / {total_bytes:>10}";
        let download_spinner = "{spinner:43} {total_bytes:>10}";
        let download_meta = "({binary_bytes_per_sec:>12}, {eta:>3} remaining)";

        let download_bar_style = [&download_message, download_bar, download_meta].join(" ");
        let download_spinner_style = [&download_message, download_spinner, download_meta].join(" ");
        let iterator_style = "{bar} {human_pos}/{human_len} ({eta} remaining)";

        let progress_bars = MultiProgress::new();
        log::set_logger(Box::leak(Logger(progress_bars.clone()).into()))
            .expect("Error setting logger");
        log::set_max_level(LevelFilter::Info);

        Self {
            progress_bars,
            client: Client::new(),
            download_bar_style: ProgressStyle::with_template(&download_bar_style).unwrap(),
            download_spinner_style: ProgressStyle::with_template(&download_spinner_style).unwrap(),
            iterator_style: ProgressStyle::with_template(iterator_style).unwrap(),
        }
    }

    pub async fn get(
        &self,
        name: &'static str,
        url: impl IntoUrl,
    ) -> Result<impl AsyncRead, reqwest::Error> {
        let request = self.client.get(url).send().await?;
        let request = request.error_for_status()?;

        let progress = if let Some(len) = request.content_length() {
            ProgressBar::new(len).with_style(self.download_bar_style.clone())
        } else {
            ProgressBar::new_spinner().with_style(self.download_spinner_style.clone())
        };
        let progress = progress.with_message(name);
        let progress = self.progress_bars.add(progress);

        let reader = request
            .bytes_stream()
            .map_err(|e| futures::io::Error::new(futures::io::ErrorKind::Other, e))
            .into_async_read()
            .compat();
        Ok(DownloadFinishLogger::new(progress.wrap_async_read(reader)))
    }

    pub fn stream<T>(
        &self,
        stream: FuturesUnordered<impl Future<Output = T>>,
    ) -> impl Stream<Item = T> {
        let progress_bar = ProgressBar::new(stream.len().try_into().unwrap())
            .with_style(self.iterator_style.clone());
        self.progress_bars.add(progress_bar).wrap_stream(stream)
    }
}

impl Default for UiManager {
    fn default() -> Self {
        Self::new()
    }
}

struct DownloadFinishLogger<R> {
    start: Instant,
    inner: ProgressBarIter<R>,
}

impl<R> DownloadFinishLogger<R> {
    fn new(inner: ProgressBarIter<R>) -> Self {
        Self {
            start: Instant::now(),
            inner,
        }
    }
}

impl<R: AsyncRead + Unpin> AsyncRead for DownloadFinishLogger<R> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut ReadBuf,
    ) -> std::task::Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).as_mut().poll_read(cx, buf)
    }
}

impl<R> Drop for DownloadFinishLogger<R> {
    fn drop(&mut self) {
        let progress = &self.inner.progress;
        info!(
            "Finished downloading {} ({} in {})",
            progress.message(),
            HumanBytes(progress.position()),
            HumanDuration(self.start.elapsed())
        );
    }
}

struct Logger(MultiProgress);

impl Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let message = format!("{}", record.args());
        let mut lines = message.lines();

        self.0.suspend(|| {
            if let Some(line) = lines.next() {
                println!("{} {line}", UiManager::prefix(record.level()));
            }

            for line in lines {
                println!("  {line}");
            }
        });
    }

    fn flush(&self) {}
}
