Sharend - rust shader renderer

```sh
cargo run -- shader.frag 2048 1024 30 10 | ffmpeg -y -f rawvideo -pixel_format rgb24 -video_size 2048x1024 -framerate 30 -i pipe:0 -c:v libx265 -tag:v hvc1 output.mp4
```
