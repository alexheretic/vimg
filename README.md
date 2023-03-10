# vimg
CLI for video images. Generate animated video contact sheets fast.
Uses _ffmpeg_.

![](https://raw.githubusercontent.com/alexheretic/vimg/main/bbb.1080p.avif)

_Note: Support for animated avif isn't everywhere yet, try viewing with chrom{e,ium}._

### Command: vcs
Create a new contact sheet for a video.

Extracts capture frames and joins into sheet(s) then encodes into an animated, or static, vcs avif.

```
vimg vcs [OPTIONS] -c <COLUMNS> -H <CAPTURE_HEIGHT> -n <NUMBER> <VIDEO>
```

See [examples](examples.md).

### Command: extract
Extract capture bmp images from a video using ffmpeg.

```
vimg extract [OPTIONS] -n <NUMBER> <VIDEO>
```

### Command: join
Join same-sized capture images into a single grid image.

```
vimg join [OPTIONS] --columns <COLUMNS> --output <OUTPUT> <CAPTURE_IMAGES>...
```

## Install
### Arch Linux
Available in the [AUR](https://aur.archlinux.org/packages/vimg).

### Windows
Pre-built **vimg.exe** included in the [latest release](https://github.com/alexheretic/vimg/releases/latest).

### Using cargo
Latest release
```sh
cargo install vimg
```

Latest code direct from git
```sh
cargo install --git https://github.com/alexheretic/vimg
``` 

### Requirements
**ffmpeg** that's not too old should be in `$PATH`.

## Minimum supported rust compiler
Maintained with [latest stable rust](https://gist.github.com/alexheretic/d1e98d8433b602e57f5d0a9637927e0c).
