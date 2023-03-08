# vimg examples

## Example: Animated 1080p vcs
Defaut args produce a 30 frame 1.5s capture played back at 10fps.

```
vimg vcs -c5 -n25 -H216 bbb.mkv
```
-> [bbb.1080p.avif](bbb.1080p.avif)

## Example: Animated 1440p realtime vcs
Playback at 20fps with the default args produces a realtime animated avif.

```
vimg vcs -c5 -n25 -H288 --avif-fps=20 bbb.mkv
```

-> [bbb.1440p.avif](bbb.1440p.avif)

## Example: Still ultrawide vcs
Using a single capture frame `-f1` results in a still/static vcs. 
7 columns & 35 captures results in close to 21:9 ultrawide resolution grid.

```
vimg vcs -c7 -n35 -H288 -f1 --avif-fps=20 bbb.mkv
```

-> [bbb.wide.avif](bbb.wide.avif)
