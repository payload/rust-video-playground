use ffmpeg_next as ffmpeg;
use std::io::Write;

pub fn write_video_frame_as_ppm(
    path: impl AsRef<std::path::Path>,
    frame: &ffmpeg::util::frame::Video,
) -> std::result::Result<(), std::io::Error> {
    let mut file = std::fs::File::create(path)?;
    file.write_all(format!("P6\n{} {}\n255\n", frame.width(), frame.height()).as_bytes())?;
    file.write_all(frame.data(0))?;
    Ok(())
}
