use std::{
    collections::HashSet,
    fs::{self, File, OpenOptions},
    io::{self, BufReader, BufWriter, Read, Write},
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{anyhow, Context, Result};
use common::{card::Id, IMAGE_DIRECTORY, IMAGE_FILE_ENDING};
use governor::{DefaultDirectRateLimiter, Jitter, Quota, RateLimiter};
use image::{codecs::avif::AvifEncoder, imageops::FilterType, DynamicImage};
use log::info;
use nonzero_ext::nonzero;
use tokio::{sync::Mutex, task::spawn_blocking};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipArchive, ZipWriter};

use crate::ygoprodeck::ARTWORK_URL;

/// URL of the deployed image cache.
pub const IMAGE_CACHE_URL: &str = "https://reisz.github.io/ygo-deck-constructor/images.zip";

/// Deployment location of the image cache.
///
/// This is part of the deployment, so it can be used in clean builds instead of re-processing.
pub const IMAGE_CACHE: &str = "dist/images.zip";

/// Name of the version file inside the image cache.
pub const VERSION_FILE: &str = "version.txt";

/// Current version of the image process.
pub const VERSION: u32 = 1;

const OUTPUT_SIZE: u32 = 96;

const DOWNLOAD_LIMIT: Quota = Quota::per_second(nonzero!(15_u32));
const DOWNLOAD_JITTER_MAX: Duration = Duration::from_millis(100);

macro_rules! output_file {
    ($id: expr) => {
        Path::new(&format!(
            "dist/{IMAGE_DIRECTORY}/{}.{IMAGE_FILE_ENDING}",
            $id
        ))
    };
}

fn zip_file(id: Id) -> String {
    format!("{id}.{IMAGE_FILE_ENDING}")
}

pub struct ImageLoader {
    cache_contents: HashSet<Id>,
    new_images: Mutex<Vec<Id>>,
    rate_limiter: DefaultDirectRateLimiter,
}

impl ImageLoader {
    pub fn new() -> Result<Self> {
        let output_path = PathBuf::from(format!("dist/{IMAGE_DIRECTORY}"));
        if !output_path.try_exists()? {
            fs::create_dir(output_path)?;
        }

        let open_for_reading = || {
            let cache = BufReader::new(File::open(IMAGE_CACHE)?);
            Ok::<_, anyhow::Error>(ZipArchive::new(cache)?)
        };

        let version = || -> Result<_> {
            let mut cache = open_for_reading()?;
            let mut output = String::new();
            cache.by_name(VERSION_FILE)?.read_to_string(&mut output)?;
            Ok(output.parse()?)
        }()
        .unwrap_or(0);

        let mut cache_contents = HashSet::new();
        if version != VERSION {
            info!("Image cache out of date. All images will be processed.");
            let cache = BufWriter::new(File::create(IMAGE_CACHE)?);
            let mut cache = ZipWriter::new(cache);
            cache.start_file(
                VERSION_FILE,
                SimpleFileOptions::default().compression_method(CompressionMethod::Stored),
            )?;
            write!(&mut cache, "{VERSION}")?;
            cache.finish()?;
        } else {
            let mut cache = open_for_reading()?;

            let suffix = format!(".{IMAGE_FILE_ENDING}");
            for file_name in cache.file_names() {
                if file_name == VERSION_FILE {
                    continue;
                }

                let id = file_name
                    .strip_suffix(&suffix)
                    .and_then(|id| id.parse().ok())
                    .ok_or_else(|| anyhow!("Unexpected file in image cache: {file_name}"))?;
                let id = Id::new(id);

                cache_contents.insert(id);
            }

            for id in &cache_contents {
                if !output_file!(id).try_exists()? {
                    io::copy(
                        &mut cache.by_name(&zip_file(*id))?,
                        &mut BufWriter::new(File::create_new(output_file!(id))?),
                    )?;
                }
            }
        }

        Ok(Self {
            cache_contents,
            new_images: Mutex::default(),
            rate_limiter: RateLimiter::direct(DOWNLOAD_LIMIT),
        })
    }

    pub async fn ensure_image(&self, id: Id) -> Result<()> {
        if self.cache_contents.contains(&id) {
            return Ok(());
        }

        // Download
        self.rate_limiter
            .until_ready_with_jitter(Jitter::up_to(DOWNLOAD_JITTER_MAX))
            .await;
        let image = download(id).await?;

        // Process and save
        spawn_blocking(move || {
            let image = process_image(&image);
            let writer = BufWriter::new(File::create(output_file!(id))?);
            let encoder = AvifEncoder::new_with_speed_quality(writer, 1, 30);
            image.write_with_encoder(encoder)?;
            Ok::<_, anyhow::Error>(())
        })
        .await??;

        // Register for caching
        self.new_images.lock().await.push(id);
        Ok(())
    }

    pub async fn finish(&self) -> Result<()> {
        let cache = OpenOptions::new()
            .read(true)
            .write(true)
            .open(IMAGE_CACHE)
            .context(IMAGE_CACHE)?;
        let mut cache = ZipWriter::new_append(cache)?;

        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        for id in self.new_images.lock().await.iter().copied() {
            let mut input = BufReader::new(File::open(output_file!(id))?);
            cache.start_file(zip_file(id), options)?;
            io::copy(&mut input, &mut cache)?;
        }

        cache.finish()?;
        Ok(())
    }
}

async fn download(id: Id) -> Result<DynamicImage> {
    let url = format!("{ARTWORK_URL}{id}.jpg");
    let image = reqwest::get(&url).await?.error_for_status()?;
    let image = image::load_from_memory(&image.bytes().await?)
        .with_context(|| format!("Failed to load image at {url}"))?;

    Ok(image)
}

fn process_image(image: &DynamicImage) -> DynamicImage {
    let size = image.width().min(image.height());
    // Center horizontally for wide artworks
    let x = (image.width() - size) / 2;
    // Align at top for full card artworks
    let y = 0;

    image
        .crop_imm(x, y, size, size)
        .resize(OUTPUT_SIZE, OUTPUT_SIZE, FilterType::Lanczos3)
}
