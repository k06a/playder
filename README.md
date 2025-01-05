# Playder - player for shaders

```sh
cargo run -- shader.frag 2048 1024 30 10 | ffmpeg -y -f rawvideo -pixel_format rgb24 -video_size 2048x1024 -framerate 30 -i pipe:0 -pix_fmt yuv420p -c:v libx265 -tag:v hvc1 output.mp4
```

```sh
cargo run -- shader.frag 7680 3840 60 10 | ffmpeg -y -f rawvideo -pixel_format rgb24 -video_size 7680x3840 -framerate 60 -i pipe:0 -pix_fmt yuv420p -c:v libx265 -tag:v hvc1 output.mp4
```

