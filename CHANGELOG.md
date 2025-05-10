# v0.2.0
* Use svt-av1 to encode avifs instead of aom-av1, speeds up encoding.
* By default use svt-av1 preset 6 for multi-frame avifs.
* Add `vcs` option `--avif-codec VCODEC` for specifying the ffmpeg vcodec for encoding the output avif,
  e.g. use vcodec "libaom-av1" for more like the old behaviour.
* Change default `--avif-fps=20` (was 10) meaning default args will yield real time avifs.

# v0.1.4
* Update dependencies.

# v0.1.3
* Fix `vcs` `-W` ffmpeg vfilter bug.
* Fix label background pixel oob panic.

# v0.1.2
* Cleanup temp dir on error / ctrl-c.

# v0.1.1
* Add `print-completions` command.

# v0.1.0
* Add `vcs`, `extract`, `join` commands.
