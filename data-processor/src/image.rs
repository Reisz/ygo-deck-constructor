use std::{
    collections::HashSet,
    fs::{self, File, OpenOptions},
    io::{BufReader, BufWriter, Cursor, Seek, Write},
    num::NonZeroU32,
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering::Relaxed},
        Arc, Mutex,
    },
    thread,
};

use anyhow::{anyhow, Result};
use common::card::Id;
use futures::executor::block_on;
use governor::{clock::Clock, Quota, RateLimiter};
use image::{imageops::FilterType, DynamicImage, ImageFormat};
use rayon::prelude::ParallelIterator;
use zip::{write::FileOptions, CompressionMethod, ZipArchive, ZipWriter};

use crate::{
    iter_utils::{CollectParallelWithoutErrors, IntoParProgressIterator},
    ygoprodeck::ARTWORK_URL,
    MISSING_IMAGES,
};

const IMAGE_ARCHIVE: &str = "data/images.zip";

const FILE_ENDING: &str = ".webp";
const IMAGE_FORMAT: ImageFormat = ImageFormat::WebP;

const OUTPUT_SIZE: u32 = 96;

pub fn available_ids() -> Result<HashSet<Id>> {
    if !Path::new(IMAGE_ARCHIVE).try_exists()? {
        return Ok(HashSet::new());
    };

    let archive = ZipArchive::new(File::open(IMAGE_ARCHIVE)?)?;
    let result = archive
        .file_names()
        .filter_map(|name| Some(Id::new(name.strip_suffix(FILE_ENDING)?.parse().ok()?)))
        .collect();
    Ok(result)
}

pub fn save_missing_ids(ids: &[Id]) -> Result<()> {
    let mut file = BufWriter::new(File::create(MISSING_IMAGES)?);
    bincode::serialize_into(&mut file, ids)?;
    Ok(())
}

fn load_missing_ids() -> Result<Vec<Id>> {
    if !Path::new(MISSING_IMAGES).try_exists()? {
        return Ok(Vec::new());
    }

    let result = bincode::deserialize_from(BufReader::new(File::open(MISSING_IMAGES)?))?;
    Ok(result)
}

async fn download(id: Id) -> Result<DynamicImage> {
    let url = format!("{}{}.jpg", ARTWORK_URL, id.get());
    let image = reqwest::get(&url).await?;
    if !image.status().is_success() {
        return Err(anyhow!("Server returned '{}' for {}", image.status(), &url));
    }
    let image = image::load(Cursor::new(image.bytes().await?), image::ImageFormat::Jpeg)?;

    Ok(image)
}

fn make_square(image: &DynamicImage) -> DynamicImage {
    let size = image.width().min(image.height());
    // Center horizontally for wide artworks
    let x = (image.width() - size) / 2;
    // Align at top for full card artworks
    let y = 0;
    image.crop_imm(x, y, size, size)
}

async fn load_image(id: Id) -> Result<Vec<u8>> {
    let image = download(id).await?;
    let image = make_square(&image);
    let image = image.resize(OUTPUT_SIZE, OUTPUT_SIZE, FilterType::Lanczos3);

    let mut data = Vec::new();
    image.write_to(&mut Cursor::new(&mut data), IMAGE_FORMAT)?;
    Ok(data)
}

fn write_image<W: Write + Seek>(id: Id, data: &[u8], writer: &mut ZipWriter<W>) -> Result<()> {
    writer.start_file(
        format!("{}{FILE_ENDING}", id.get()),
        FileOptions::default().compression_method(CompressionMethod::Stored),
    )?;
    writer.write_all(data)?;
    Ok(())
}

pub fn load_missing_images() -> Result<()> {
    let exit_requested = Arc::new(AtomicBool::new(false));
    ctrlc::set_handler({
        let exit_requested = exit_requested.clone();
        move || exit_requested.store(true, Relaxed)
    })
    .expect("error setting Ctrl-C handler");

    let writer = if Path::new(IMAGE_ARCHIVE).try_exists()? {
        ZipWriter::new_append(
            OpenOptions::new()
                .read(true)
                .write(true)
                .open(IMAGE_ARCHIVE)?,
        )?
    } else {
        ZipWriter::new(File::create(IMAGE_ARCHIVE)?)
    };
    let writer = Mutex::new(writer);

    let clock = governor::clock::DefaultClock::default();
    let rate_limit =
        RateLimiter::direct_with_clock(Quota::per_second(NonZeroU32::new(15).unwrap()), &clock);

    let ids = load_missing_ids()?;
    let remaining_ids: Vec<_> = ids
        .into_par_progress_iter()
        .flat_map(|id| {
            if exit_requested.load(Relaxed) {
                return vec![Ok(id)];
            }

            while let Err(time) = rate_limit.check() {
                thread::sleep(time.wait_time_from(clock.now()));
            }

            let result = || -> Result<()> {
                let data = block_on(load_image(id))?;
                write_image(id, &data, &mut writer.lock().unwrap())
            }();

            match result {
                Ok(()) => vec![],
                Err(err) => vec![Ok(id), Err(err)],
            }
        })
        .collect_without_errors();

    writer.lock().unwrap().finish()?;

    if remaining_ids.is_empty() {
        fs::remove_file(MISSING_IMAGES)?;
        println!("All images loaded");
    } else {
        save_missing_ids(&remaining_ids)?;
        println!("{} images unprocessed", remaining_ids.len());
    }

    Ok(())
}
