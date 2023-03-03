mod command;

use anyhow::anyhow;
use clap::Parser;
use image::{imageops::FilterType, DynamicImage, GenericImage};
use rayon::prelude::*;
use std::path::Path;

#[derive(Parser)]
#[command(version, about)]
enum Command {
    Join(command::join::Args),
}

fn main() -> anyhow::Result<()> {
    let action = Command::parse();

    match action {
        Command::Join(command::join::Args {
            columns,
            capture_width,
            capture_height,
            output,
            capture_images,
        }) => {
            let n_captures = capture_images.len() as u32;

            // load images concurrently
            let mut images: Vec<_> = capture_images
                .par_iter()
                .map(|i| {
                    load_image(i, capture_width, capture_height).map_err(|e| anyhow!("{i:?}: {e}"))
                })
                .collect::<Result<Vec<_>, _>>()?;

            let img0 = images.remove(0);
            let (cap_w, cap_h) = (img0.width(), img0.height());
            let (rows, cols) = if columns == 0 || n_captures <= columns {
                (1, n_captures as _)
            } else {
                let images = n_captures;
                let mut rows = images / columns;
                if images % columns != 0 {
                    rows += 1;
                }
                (rows, columns)
            };
            eprintln!(
                "arranging in {cols}x{rows} grid of {cap_w}x{cap_h} images -> {}x{}px",
                cap_w * cols,
                cap_h * rows
            );

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
        }
    }

    Ok(())
}

fn load_image(
    path: impl AsRef<Path>,
    resize_w: Option<u32>,
    resize_h: Option<u32>,
) -> anyhow::Result<DynamicImage> {
    let path = path.as_ref();
    let mut img = image::io::Reader::open(path)?.decode()?;

    if resize_h.is_some() || resize_w.is_some() {
        img = img.resize(
            resize_w.unwrap_or(u32::MAX),
            resize_h.unwrap_or(u32::MAX),
            FilterType::CatmullRom,
        );
    }

    Ok(img)
}
