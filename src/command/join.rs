use clap::Parser;
use std::path::PathBuf;

/// Join same-sized capture images into a single grid image.
#[derive(Parser, Debug, Clone)]
#[group(skip)]
pub struct Args {
    /// Number of capture columns in output.
    #[arg(long, short)]
    pub columns: u32,

    /// Width of each capture inside output. Will be scaled preserving aspect.
    #[arg(long, short = 'w')]
    pub capture_width: Option<u32>,

    /// Output file name.
    #[arg(long, short)]
    pub output: PathBuf,

    /// Images to join.
    #[arg(required = true)]
    pub capture_images: Vec<PathBuf>,
}
