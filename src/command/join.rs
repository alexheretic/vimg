pub mod label;

use anyhow::anyhow;
use image::GenericImage;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

/// Join same-sized capture images into a single grid image.
#[derive(clap::Parser, Debug, Clone)]
#[group(skip)]
pub struct Join {
    /// Number of capture columns in output.
    #[arg(long, short)]
    pub columns: u32,

    /// Pixel width of each capture inside the grid. Will be scaled preserving aspect.
    #[arg(long, short = 'W')]
    pub capture_width: Option<u32>,

    /// Pixel height of each capture inside the grid. Will be scaled preserving aspect.
    #[arg(long, short = 'H')]
    pub capture_height: Option<u32>,

    /// Output file name.
    #[arg(long, short)]
    pub output: PathBuf,

    #[arg(long)]
    pub label: Vec<String>,

    /// Images to join.
    #[arg(required = true)]
    pub capture_images: Vec<PathBuf>,
}

impl Join {
    pub fn run(&self) -> anyhow::Result<()> {
        let Self {
            columns,
            output,
            capture_images,
            ..
        } = self;

        let n_captures = capture_images.len() as u32;

        // load images concurrently
        let images: Vec<_> = capture_images
            .par_iter()
            .map(|i| self.load_image(i).map_err(|e| anyhow!("{i:?}: {e}")))
            .collect::<Result<Vec<_>, _>>()?;

        let (cap_w, cap_h) = (images[0].width(), images[0].height());
        let (rows, cols) = if *columns == 0 || n_captures <= *columns {
            (1, n_captures)
        } else {
            let images = n_captures;
            let mut rows = images / columns;
            if !images.is_multiple_of(*columns) {
                rows += 1;
            }
            (rows, *columns)
        };

        let mut labels = self.label.clone();
        labels.resize_with(images.len(), String::new);

        let mut all = image::RgbaImage::new(cap_w * cols, cap_h * rows);
        for (idx, (img, label)) in images.into_iter().zip(labels).enumerate() {
            let idx = idx as u32;
            let x = (idx % cols) * cap_w;
            let y = (idx / cols) * cap_h;
            let img = label::draw(img, &label, &label::Config::default())?;
            all.copy_from(&img, x as _, y as _)?;
        }

        image::DynamicImage::from(all).into_rgb8().save(output)?;

        Ok(())
    }

    fn load_image(&self, path: impl AsRef<Path>) -> anyhow::Result<image::DynamicImage> {
        let path = path.as_ref();
        let mut img = image::ImageReader::open(path)?.decode()?;

        if self.capture_width.is_some() || self.capture_height.is_some() {
            img = img.resize(
                self.capture_width.unwrap_or(u32::MAX),
                self.capture_height.unwrap_or(u32::MAX),
                image::imageops::FilterType::CatmullRom,
            );
        }

        Ok(img)
    }
}
