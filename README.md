# rust_video_playground

## Examples

* [ffmpeg_mac_webcam](../ffmpeg_mac_webcam/index.html):
  FFMPEG is used on Mac OS to access a webcam video input and write frames to PPM files.

## How To

* build documentation: `cargo doc --no-deps --examples`
* watch docs and tests: `cargo watch -x 'doc --no-deps --examples' -x 'test --no-fail-fast --all-targets'`