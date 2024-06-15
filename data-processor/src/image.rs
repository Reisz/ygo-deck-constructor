use std::{
    collections::HashSet,
    fs::{self, File, OpenOptions},
    io::{self, BufReader, BufWriter},
    path::Path,
    time::Duration,
};

use anyhow::{anyhow, Context, Result};
use common::card::Id;
use governor::{DefaultDirectRateLimiter, Jitter, Quota, RateLimiter};
use image::{imageops::FilterType, DynamicImage};
use nonzero_ext::nonzero;
use tokio::{sync::Mutex, task::spawn_blocking};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipArchive, ZipWriter};

use crate::{ygoprodeck::ARTWORK_URL, IMAGE_CACHE, IMAGE_DIRECTORY};

const FILE_ENDING: &str = ".webp";

const OUTPUT_SIZE: u32 = 96;

const DOWNLOAD_LIMIT: Quota = Quota::per_second(nonzero!(15_u32));
const DOWNLOAD_JITTER_MAX: Duration = Duration::from_millis(100);

macro_rules! output_file {
    ($id: expr) => {
        Path::new(&format!("{IMAGE_DIRECTORY}/{}{FILE_ENDING}", $id.get()))
    };
}

fn zip_file(id: Id) -> String {
    format!("{}{FILE_ENDING}", id.get())
}

pub struct ImageLoader {
    cache_contents: HashSet<Id>,
    new_images: Mutex<Vec<Id>>,
    rate_limiter: DefaultDirectRateLimiter,
}

impl ImageLoader {
    pub fn new() -> Result<Self> {
        if !Path::new(IMAGE_DIRECTORY).try_exists()? {
            fs::create_dir(Path::new(IMAGE_DIRECTORY))?;
        }

        let mut cache_contents = HashSet::new();

        let cache = BufReader::new(File::open(IMAGE_CACHE)?);
        let mut cache = ZipArchive::new(cache)?;

        for file_name in cache.file_names() {
            let id = file_name
                .strip_suffix(FILE_ENDING)
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
        spawn_blocking(move || process_image(&image).save(output_file!(id))).await??;

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
    let url = format!("{}{}.jpg", ARTWORK_URL, id.get());
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
