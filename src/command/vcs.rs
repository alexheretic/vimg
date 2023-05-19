use crate::{
    command::{self, label, sh_escape, sh_escape_filename},
    process::CommandExt,
    temporary,
};
use anyhow::ensure;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use std::{fs, path::PathBuf, process::Command, time::Duration};

/// Create a new contact sheet for a video.
///
/// Extracts capture frames and joins into sheet(s) then encodes into
/// an animated, or static, vcs avif.
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
    ///
    /// Example: The default 10fps will result in half-speed playback for
    /// the default args: -f30 -t1500ms (30 frames over 1.5 seconds).
    /// So using 20fps will result in realtime playback for: -f30 -t1500ms.
    #[arg(long, default_value_t = 10.0)]
    pub avif_fps: f32,

    /// Pixel width of each capture inside the grid. Will be scaled preserving aspect.
    ///
    /// Use this or -H (not both).
    #[arg(long, short = 'W', conflicts_with = "capture_height")]
    pub capture_width: Option<u32>,

    /// Pixel height of each capture inside the grid. Will be scaled preserving aspect.
    ///
    /// Use this or -W (not both).
    #[arg(long, short = 'H', conflicts_with = "capture_width", required = true)]
    pub capture_height: Option<u32>,

    #[clap(flatten)]
    pub args: command::Extract,

    /// Keep temporary files.
    #[arg(long, default_value_t = false)]
    pub keep: bool,
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
        let temp_dir = temporary::process_dir(self.args.output_dir.clone(), !self.keep);

        self.args.output_dir = Some(temp_dir.clone());
        self.args.capture_frames = self.args.capture_frames.or(Some(30));

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

        if self.keep {
            spinner.println(format!(
                "Keeping temporary files in {}",
                sh_escape(&temp_dir)
            ));
        }

        spinner.set_message("Extracting");
        let extract = self.args.run()?;

        for msg in &extract.warnings {
            spinner.println(format!("Warning: {msg}"));
        }

        spinner.set_message("Joining");
        let frame_w = self.args.capture_frames().to_string().len();
        let file_prefix = self.args.video.with_extension("");
        let file_prefix = file_prefix
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .replace('%', "");

        (0..self.args.capture_frames())
            .into_par_iter()
            .try_for_each(|f| {
                let capture_images: Vec<_> = extract
                    .out_templates
                    .iter()
                    .map(|tmpl| {
                        let mut o = temp_dir.to_path_buf();
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
                        let mut o = temp_dir.to_path_buf();
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

        // write to temp location until successful
        let temp_out_file = {
            let mut o = temp_dir.clone();
            o.push(format!("{file_prefix}.avif"));
            o
        };
        // output file if successful
        let out_file = self.output.unwrap_or_else(|| {
            let mut o = parent_dir;
            o.push(format!("{file_prefix}.avif"));
            o
        });

        spinner.set_message(format!("Encoding {}", sh_escape_filename(&out_file)));
        let out = Command::new("ffmpeg")
            .arg2("-r", self.avif_fps)
            .arg2("-i", {
                let mut o = temp_dir;
                o.push(format!("{file_prefix}-%0{frame_w}d.bmp"));
                o
            })
            .arg2("-c:v", "libaom-av1")
            .arg2(
                "-cpu-used",
                self.avif_preset
                    .unwrap_or(match self.args.capture_frames() {
                        1 => 1,
                        _ => 5,
                    }),
            )
            .arg2("-crf", self.avif_crf)
            .arg2("-pix_fmt", "yuv420p10le")
            .arg("-y")
            .arg(&temp_out_file)
            .output()?;
        ensure!(
            out.status.success(),
            "ffmpeg convert-to-avif failed\n---stderr---\n{}\n------",
            String::from_utf8_lossy(&out.stderr).trim(),
        );

        fs::rename(&temp_out_file, &out_file)
            .or_else(|_| fs::copy(&temp_out_file, &out_file).map(|_| ()))?;

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
