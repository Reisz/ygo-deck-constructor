use anyhow::Result;
use data_processor::image::load_missing_images;

fn main() -> Result<()> {
    load_missing_images()
}
