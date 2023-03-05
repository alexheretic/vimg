use crate::{command, process::CommandExt};
use anyhow::ensure;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use std::{path::PathBuf, process::Command, time::Duration};

/// Create a new video contact sheet.
#[derive(clap::Parser, Debug, Clone)]
#[group(skip)]
pub struct Create {
    /// Number of capture columns in output.
    #[arg(long, short)]
    pub columns: u32,

    /// Output file name. Defaults to input with .avif extension.
    #[arg(long, short)]
    pub output: Option<PathBuf>,

    /// crf quality level for encoding the output avif.
    #[arg(long, default_value_t = 35)]
    pub output_crf: u8,

    /// preset/cpu-used for encoding the output avif.
    #[arg(long, default_value_t = 6)]
    pub output_preset: u8,

    /// Output avif framerate for multi-frame outputs.
    #[arg(long, default_value_t = 20.0)]
    pub output_fps: f32,

    /// Pixel width of each capture inside the grid. Will be scaled preserving aspect.
    #[arg(long, short = 'W', conflicts_with = "capture_height")]
    pub capture_width: Option<u32>,

    /// Pixel height of each capture inside the grid. Will be scaled preserving aspect.
    #[arg(long, short = 'H', conflicts_with = "capture_width")]
    pub capture_height: Option<u32>,

    #[clap(flatten)]
    pub args: command::Extract,
}

impl Create {
    pub fn run(mut self) -> anyhow::Result<()> {
        ensure!(
            self.output
                .as_ref()
                .map_or(true, |p| p.extension().and_then(|e| e.to_str())
                    == Some("avif")),
            "output must be avif"
        );

        let parent_dir = self
            .args
            .output_dir
            .clone()
            .unwrap_or_else(|| PathBuf::from("."));
        let temp_dir = tempfile::tempdir_in(&parent_dir)?;
        self.args.output_dir = Some(temp_dir.path().to_path_buf());

        let ex_scale = self.extract_scale();
        self.args.vfilter = match (self.args.vfilter, ex_scale) {
            (Some(vf), Some(scale)) => Some(format!("{vf},{scale}")),
            (vf, scale) => vf.or(scale),
        };

        let spinner = indicatif::ProgressBar::new_spinner().with_style(
            indicatif::ProgressStyle::default_spinner()
                .template("{spinner:.cyan.bold} {elapsed_precise:.bold} {msg}")?,
        );
        spinner.enable_steady_tick(Duration::from_millis(100));
        spinner.set_message("Extracting");
        let extract = self.args.run()?;

        spinner.set_message("Joining");
        let frame_w = self.args.capture_frames.to_string().len();
        let file_prefix = self.args.video.with_extension("");
        let file_prefix = file_prefix
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();

        (0..self.args.capture_frames)
            .into_par_iter()
            .try_for_each(|f| {
                let capture_images: Vec<_> = extract
                    .out_templates
                    .iter()
                    .map(|tmpl| {
                        let mut o = temp_dir.path().to_path_buf();
                        o.push(tmpl.with_frame(f + 1));
                        o
                    })
                    .collect();

                command::Join {
                    columns: self.columns,
                    capture_width: None,
                    capture_height: None,
                    output: {
                        let mut o = temp_dir.path().to_path_buf();
                        o.push(format!("{file_prefix}-{f:0frame_w$}.bmp"));
                        o
                    },
                    capture_images,
                }
                .run()
            })?;

        spinner.set_message("Encoding avif");
        let out = Command::new("ffmpeg")
            .arg2("-r", self.output_fps)
            .arg2("-i", {
                let mut o = temp_dir.path().to_path_buf();
                o.push(format!("{file_prefix}-%0{frame_w}d.bmp"));
                o
            })
            .arg2("-c:v", "libaom-av1")
            .arg2("-cpu-used", self.output_preset)
            .arg2("-crf", self.output_crf)
            .arg2("-pix_fmt", "yuv420p10le")
            .arg("-y")
            .arg(self.output.unwrap_or_else(|| {
                let mut o = parent_dir;
                o.push(format!("{file_prefix}.avif"));
                o
            }))
            .output()?;
        ensure!(
            out.status.success(),
            "ffmpeg convert-to-avif failed\n---stderr---\n{}\n------",
            String::from_utf8_lossy(&out.stderr).trim(),
        );

        spinner.finish();
        Ok(())
    }

    fn extract_scale(&self) -> Option<String> {
        if let Some(h) = self.capture_height {
            return Some(format!("scale=-1:{h}:flags=bicubic"));
        }
        let w = self.capture_width?;
        Some(format!("scale{w}:-1:flags=bicubic"))
    }
}
