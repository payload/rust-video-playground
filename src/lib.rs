#![doc = include_str!("../README.md")]

pub use ffmpeg_next as ffmpeg;

mod camera;
pub use camera::{load_camera_stream, CameraStream, VideoFrame};

mod ppm;
pub use ppm::write_video_frame_as_ppm;
