use std::{
    collections::HashSet,
    fs::{self, File, OpenOptions},
    io::{self, BufReader, BufWriter, Read, Write},
    path::PathBuf,
    time::Duration,
};

use anyhow::{anyhow, Context, Result};
use common::{
    card::CardPassword,
    transfer::{self, IMAGE_DIRECTORY, IMAGE_FILE_ENDING},
};
use governor::{DefaultDirectRateLimiter, Jitter, Quota, RateLimiter};
use image::{codecs::avif::AvifEncoder, imageops::FilterType, DynamicImage};
use log::info;
use nonzero_ext::nonzero;
use tokio::{sync::Mutex, task::spawn_blocking};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipArchive, ZipWriter};

use crate::{ygoprodeck::ARTWORK_URL, OUTPUT_DIRECTORY};

/// Name of the image cache file.
///
/// This is part of the deployment, so it can be used in clean builds to avoid re-processing all
/// images.
pub const CACHE_FILENAME: &str = "images.zip";

/// Name of the version file inside the image cache.
pub const VERSION_FILE: &str = "version.txt";

/// Current version of the image process.
pub const VERSION: u32 = 1;

const OUTPUT_SIZE: u32 = 96;

const DOWNLOAD_LIMIT: Quota = Quota::per_second(nonzero!(15_u32));
const DOWNLOAD_JITTER_MAX: Duration = Duration::from_millis(100);

fn output_file(password: CardPassword) -> PathBuf {
    let mut path = PathBuf::from(OUTPUT_DIRECTORY);
    path.push(transfer::IMAGE_DIRECTORY);
    path.push(password.to_string());
    path.set_extension(transfer::IMAGE_FILE_ENDING);
    path
}

fn zip_file(password: CardPassword) -> String {
    format!("{password}.{IMAGE_FILE_ENDING}")
}

pub struct ImageLoader {
    cache_contents: HashSet<CardPassword>,
    new_images: Mutex<Vec<CardPassword>>,
    rate_limiter: DefaultDirectRateLimiter,
}

impl ImageLoader {
    pub fn new() -> Result<Self> {
        let output_path = &PathBuf::from(OUTPUT_DIRECTORY).join(IMAGE_DIRECTORY);
        if !output_path.try_exists()? {
            fs::create_dir(output_path)?;
        }

        let cache_path = &PathBuf::from(OUTPUT_DIRECTORY).join(CACHE_FILENAME);
        let open_for_reading = || {
            let cache = BufReader::new(File::open(cache_path)?);
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
            let cache = BufWriter::new(File::create(cache_path)?);
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

                let password = file_name
                    .strip_suffix(&suffix)
                    .and_then(|password| password.parse().ok())
                    .ok_or_else(|| anyhow!("Unexpected file in image cache: {file_name}"))?;

                cache_contents.insert(password);
            }

            for &password in &cache_contents {
                if !output_file(password).try_exists()? {
                    io::copy(
                        &mut cache.by_name(&zip_file(password))?,
                        &mut BufWriter::new(File::create_new(output_file(password))?),
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

    pub async fn ensure_image(&self, password: CardPassword) -> Result<()> {
        if self.cache_contents.contains(&password) {
            return Ok(());
        }

        // Download
        self.rate_limiter
            .until_ready_with_jitter(Jitter::up_to(DOWNLOAD_JITTER_MAX))
            .await;
        let image = download(password).await?;

        // Process and save
        spawn_blocking(move || {
            let image = process_image(&image);
            let writer = BufWriter::new(File::create(output_file(password))?);
            let encoder = AvifEncoder::new_with_speed_quality(writer, 1, 30);
            image.write_with_encoder(encoder)?;
            Ok::<_, anyhow::Error>(())
        })
        .await??;

        // Register for caching
        self.new_images.lock().await.push(password);
        Ok(())
    }

    pub async fn finish(&self) -> Result<()> {
        let path = &PathBuf::from(OUTPUT_DIRECTORY).join(CACHE_FILENAME);
        let cache = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .context(path.display().to_string())?;
        let mut cache = ZipWriter::new_append(cache)?;

        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        for password in self.new_images.lock().await.iter().copied() {
            let mut input = BufReader::new(File::open(output_file(password))?);
            cache.start_file(zip_file(password), options)?;
            io::copy(&mut input, &mut cache)?;
        }

        cache.finish()?;
        Ok(())
    }
}

async fn download(password: CardPassword) -> Result<DynamicImage> {
    let url = format!("{ARTWORK_URL}{password}.jpg");
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
