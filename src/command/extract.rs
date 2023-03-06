use crate::{
    command::{DurationOrPercent, HumanDuration},
    process::CommandExt,
};
use anyhow::{ensure, Context};
use rayon::prelude::*;
use std::{fmt, fs, path::PathBuf, process::Command};

/// Generate capture bmp images from a video using ffmpeg.
#[derive(clap::Parser, Debug, Clone)]
#[group(skip)]
pub struct Extract {
    /// Number of equidistant points in the video to capture.
    #[arg(long, short)]
    pub number: u32,

    /// Time or percentage at the start to ignore when calculating capture points.
    #[arg(long = "ignore-start", default_value = "0s")]
    pub ignore_start: DurationOrPercent,

    /// Time or percentage at the end to ignore when calculating capture points.
    #[arg(long = "ignore-end", default_value = "0s")]
    pub ignore_end: DurationOrPercent,

    /// Number of frames to output for each capture (greater than 1 for animated captures).
    #[arg(long, short = 'f', default_value_t = 1)]
    pub capture_frames: u32,

    /// Duration per capture for multi-frame captures.
    #[arg(long, short = 't', default_value = "1500ms")]
    pub capture_time: HumanDuration,

    /// Ffmpeg vfilter.
    #[arg(long)]
    pub vfilter: Option<String>,

    /// Number of threads / concurrent ffmpeg calls. 0=auto.
    #[arg(long, short = 'T', default_value_t = 3)]
    pub threads: usize,

    /// Directory to write capture images into. Defaults to the current directory.
    #[arg(long)]
    pub output_dir: Option<PathBuf>,

    /// Video file input.
    #[arg(required = true)]
    pub video: PathBuf,
}

impl Extract {
    pub fn run(&self) -> anyhow::Result<ExtractData> {
        let Self {
            number,
            ignore_start,
            ignore_end,
            threads,
            video,
            ..
        } = self;

        let video_duration_s = ffprobe::ffprobe(video)?
            .format
            .duration
            .context("invalid video duration")?
            .parse::<f32>()
            .context("invalid video duration")?;

        let duration_s = video_duration_s
            - ignore_start.to_secs(video_duration_s)
            - ignore_end.to_secs(video_duration_s);

        ensure!(
            duration_s > 0.0,
            "invalid negative video duration minus offsets"
        );

        rayon::ThreadPoolBuilder::new()
            .num_threads(*threads)
            .build()?
            .install(|| {
                let out_templates = (0..*number)
                    .into_par_iter()
                    .map(|n| {
                        let interval = duration_s / *number as f32;
                        let start_s = ignore_start.to_secs(video_duration_s)
                            + interval * 0.5
                            + interval * n as f32;
                        let out_template = self.out_template(start_s, duration_s);
                        self.capture(start_s, &out_template)?;
                        Ok(out_template)
                    })
                    .collect::<anyhow::Result<Vec<_>>>()?;

                Ok(ExtractData { out_templates })
            })
    }

    fn out_template(&self, start_s: f32, duration_s: f32) -> OutTemplate {
        let prefix = self.video.with_extension("");
        let prefix = prefix.file_name().unwrap_or_default().to_string_lossy();

        OutTemplate::new(prefix, start_s as _, duration_s as _, self.capture_frames)
    }

    fn capture(&self, start_s: f32, out_template: &OutTemplate) -> anyhow::Result<()> {
        let Self {
            capture_frames,
            capture_time,
            vfilter,
            output_dir,
            video,
            ..
        } = self;
        ensure!(
            *capture_frames > 0,
            "invalid capture-frames must be non-zero"
        );
        ensure!(
            capture_time.seconds > 0.0,
            "invalid capture-time must be non-zero"
        );

        let mut out = match output_dir {
            None => PathBuf::from("."),
            Some(dir) => {
                fs::create_dir_all(dir)?;
                dir.clone()
            }
        };
        out.push(out_template.to_string());

        let out = Command::new("ffmpeg")
            .arg2("-ss", start_s)
            .arg2("-t", capture_time.seconds)
            .arg2("-i", video)
            .arg2("-r", format!("{capture_frames}/{}", capture_time.seconds))
            .arg2("-fps_mode", "cfr")
            .arg2_opt("-vf", vfilter.as_ref())
            .arg2("-vframes", capture_frames)
            .arg("-y")
            .arg(&out)
            .output()?;

        ensure!(
            out.status.success(),
            "ffmpeg capture failed\n---stderr---\n{}\n------",
            String::from_utf8_lossy(&out.stderr).trim(),
        );

        Ok(())
    }
}

pub struct ExtractData {
    /// All ffmpeg capture output templates.
    pub out_templates: Vec<OutTemplate>,
}

/// "prefix-Ss-F.bmp" template.
///
/// S = seconds. Constant for a given template.
/// F = frames using a ffmpeg/printf `%0nd` style.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct OutTemplate {
    pub prefix: String,
    pub seconds: u32,
    second_w: usize,
    frame_w: usize,
}

impl OutTemplate {
    fn new(prefix: impl Into<String>, seconds: u32, max_seconds: u32, max_frames: u32) -> Self {
        let second_w = max_seconds.to_string().len();
        let frame_w = max_frames.to_string().len();
        Self {
            prefix: prefix.into(),
            seconds,
            second_w,
            frame_w,
        }
    }

    /// Return a string capture file name with the given frame number.
    pub fn with_frame(&self, f: u32) -> String {
        let Self {
            prefix,
            seconds,
            second_w,
            frame_w,
        } = self;
        format!("{prefix}-{seconds:0second_w$}s-{f:0frame_w$}.bmp")
    }
}

impl fmt::Display for OutTemplate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            prefix,
            seconds,
            second_w,
            frame_w,
        } = self;
        write!(f, "{prefix}-{seconds:0second_w$}s-%0{frame_w}d.bmp")
    }
}
