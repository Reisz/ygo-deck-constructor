use std::{
    collections::HashSet,
    fs::{self, File, OpenOptions},
    io::{BufReader, BufWriter, Cursor, Write},
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
use governor::{clock::Clock, Quota, RateLimiter};
use image::{imageops::FilterType, ImageOutputFormat};
use indicatif::ParallelProgressIterator;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use zip::{write::FileOptions, CompressionMethod, ZipArchive, ZipWriter};

use crate::{default_progress_style, ygoprodeck::ARTWORK_URL, MISSING_IMAGES};

const IMAGE_ARCHIVE: &str = "data/images.zip";

const FILE_ENDING: &str = ".webp";
const IMAGE_FORMAT: ImageOutputFormat = ImageOutputFormat::WebP;

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
    let options = FileOptions::default().compression_method(CompressionMethod::Stored);

    let clock = governor::clock::DefaultClock::default();
    let rate_limit =
        RateLimiter::direct_with_clock(Quota::per_second(NonZeroU32::new(15).unwrap()), &clock);

    let ids = load_missing_ids()?;
    let errors = Mutex::new(Vec::new());
    let remaining_ids: Vec<_> = ids
        .into_par_iter()
        .progress_with_style(default_progress_style())
        .filter_map(|id| {
            if exit_requested.load(Relaxed) {
                return Some(id);
            }

            while let Err(time) = rate_limit.check() {
                thread::sleep(time.wait_time_from(clock.now()));
            }

            let result = (|| -> Result<()> {
                let url = format!("{}{}.jpg", ARTWORK_URL, id.get());
                let image = reqwest::blocking::get(&url)?;
                if !image.status().is_success() {
                    return Err(anyhow!("Server returned '{}' for {}", image.status(), &url));
                }
                let image = image::load(Cursor::new(image.bytes()?), image::ImageFormat::Jpeg)?;

                let square_size = image.width().min(image.height());
                let image = image.crop_imm(0, 0, square_size, square_size);

                let image = image.resize(OUTPUT_SIZE, OUTPUT_SIZE, FilterType::Lanczos3);

                let mut data = Vec::new();
                image.write_to(&mut Cursor::new(&mut data), IMAGE_FORMAT)?;

                let mut writer = writer.lock().unwrap();
                writer.start_file(format!("{}{FILE_ENDING}", id.get()), options)?;
                writer.write_all(&data)?;

                Ok(())
            })();

            if let Err(e) = result {
                errors.lock().unwrap().push((id.get(), e));
                Some(id)
            } else {
                None
            }
        })
        .collect();

    for (id, e) in errors.lock().unwrap().iter() {
        eprintln!("Error processing artwork with id {id}: {e}");
    }

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
