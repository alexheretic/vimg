use crate::{
    command::{self, label, ExtractData},
    process::CommandExt,
};
use anyhow::ensure;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

/// Create a new video contact sheet.
#[derive(clap::Parser, Debug, Clone)]
#[group(skip)]
pub struct Vcs {
    /// Number of capture columns in output.
    #[arg(long, short)]
    pub columns: u32,

    /// Output file name. Defaults to input with .avif extension.
    #[arg(long, short)]
    pub output: Option<PathBuf>,

    /// Crf quality level for encoding the output avif.
    #[arg(long, default_value_t = 30)]
    pub avif_crf: u8,

    /// Preset/cpu-used for encoding the output avif.
    ///
    /// Default 1 for single-frame, 5 for multi-frame.
    #[arg(long)]
    pub avif_preset: Option<u8>,

    /// Output avif framerate for multi-frame outputs.
    #[arg(long, default_value_t = 20.0)]
    pub avif_fps: f32,

    /// Pixel width of each capture inside the grid. Will be scaled preserving aspect.
    #[arg(long, short = 'W', conflicts_with = "capture_height")]
    pub capture_width: Option<u32>,

    /// Pixel height of each capture inside the grid. Will be scaled preserving aspect.
    #[arg(long, short = 'H', conflicts_with = "capture_width")]
    pub capture_height: Option<u32>,

    #[clap(flatten)]
    pub args: command::Extract,
}

impl Vcs {
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

        let fixes = self.check_extract(&extract, temp_dir.path())?;
        if fixes > 0 {
            spinner.println(format!(
                "Warning: Duplicated {fixes} extract(s) to cover for missing frames"
            ));
        }

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

                let label = extract
                    .out_templates
                    .iter()
                    .map(|tmpl| label::seconds_text(tmpl.seconds))
                    .collect();

                command::Join {
                    columns: self.columns,
                    output: {
                        let mut o = temp_dir.path().to_path_buf();
                        o.push(format!("{file_prefix}-{f:0frame_w$}.bmp"));
                        o
                    },
                    capture_images,
                    capture_width: None,
                    capture_height: None,
                    label,
                }
                .run()
            })?;

        let out_file = self.output.unwrap_or_else(|| {
            let mut o = parent_dir;
            o.push(format!("{file_prefix}.avif"));
            o
        });

        spinner.set_message(format!(
            "Encoding {}",
            shell_escape::escape(out_file.display().to_string().into())
        ));
        let out = Command::new("ffmpeg")
            .arg2("-r", self.avif_fps)
            .arg2("-i", {
                let mut o = temp_dir.path().to_path_buf();
                o.push(format!("{file_prefix}-%0{frame_w}d.bmp"));
                o
            })
            .arg2("-c:v", "libaom-av1")
            .arg2(
                "-cpu-used",
                self.avif_preset.unwrap_or(match self.args.capture_frames {
                    1 => 1,
                    _ => 5,
                }),
            )
            .arg2("-crf", self.avif_crf)
            .arg2("-pix_fmt", "yuv420p10le")
            .arg("-y")
            .arg(out_file)
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

    /// In the cases we failed to extract every expect frame it may be possible to cover
    /// a small amount of these by duplicating the previous frame. This case should be rare.
    fn check_extract(&self, extract: &ExtractData, temp_dir: &Path) -> anyhow::Result<usize> {
        /// Max number of fixes per template
        const MAX_FIXES: usize = 6;

        let mut total_fixes = 0;

        // ensure all captures exist
        for tmpl in &extract.out_templates {
            let mut first = temp_dir.to_path_buf();
            first.push(tmpl.with_frame(1));
            ensure!(
                first.is_file(),
                "Failed to extract: {}",
                shell_escape::escape(first.display().to_string().into())
            );

            let mut prev = first;
            let mut fixes = 0;
            for f in 2..=self.args.capture_frames {
                let mut next = temp_dir.to_path_buf();
                next.push(tmpl.with_frame(f));
                if !next.is_file() {
                    ensure!(
                        fixes < MAX_FIXES,
                        "Failed to extract (too many to fix): {}",
                        shell_escape::escape(next.display().to_string().into())
                    );
                    fs::copy(&prev, &next)?;
                    fixes += 1;
                }
                prev = next;
            }
            total_fixes += fixes;
        }

        Ok(total_fixes)
    }
}
