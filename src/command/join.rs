use anyhow::anyhow;
use image::GenericImage;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

/// Join same-sized capture images into a single grid image.
// TODO
// * --label
// * --padding
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
        let mut images: Vec<_> = capture_images
            .par_iter()
            .map(|i| self.load_image(i).map_err(|e| anyhow!("{i:?}: {e}")))
            .collect::<Result<Vec<_>, _>>()?;

        let img0 = images.remove(0);
        let (cap_w, cap_h) = (img0.width(), img0.height());
        let (rows, cols) = if *columns == 0 || n_captures <= *columns {
            (1, n_captures)
        } else {
            let images = n_captures;
            let mut rows = images / columns;
            if images % columns != 0 {
                rows += 1;
            }
            (rows, *columns)
        };
        // eprintln!(
        //     "arranging in {cols}x{rows} grid of {cap_w}x{cap_h} images -> {}x{}px",
        //     cap_w * cols,
        //     cap_h * rows
        // );

        let mut all = image::RgbaImage::new(cap_w * cols, cap_h * rows);
        all.copy_from(&img0, 0, 0)?;
        drop(img0);

        for (idx, img) in images.into_iter().enumerate() {
            let idx = idx as u32 + 1;
            let x = (idx % cols) * cap_w;
            let y = (idx / cols) * cap_h;
            all.copy_from(&img, x as _, y as _)?;
        }

        image::DynamicImage::from(all).into_rgb8().save(output)?;

        Ok(())
    }

    fn load_image(&self, path: impl AsRef<Path>) -> anyhow::Result<image::DynamicImage> {
        let path = path.as_ref();
        let mut img = image::io::Reader::open(path)?.decode()?;

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
