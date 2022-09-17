use std::{io::Write, path::Path};

use ffmpeg::codec::traits::Decoder;
use ffmpeg_next as ffmpeg;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    ffmpeg::init()?;
    ffmpeg::device::register_all();

    let input_format = ffmpeg::device::input::video()
        .find(|format| format.name() == "avfoundation")
        .unwrap();

    let path0 = &Path::new("default");
    let options = ffmpeg::dict!(
        "list_devices" => "false",
        // "video_device_index" => "0",
        "framerate" => "30",
        "pixel_format" => "0rgb",
    );

    // let input_context = ffmpeg::format::open(path0, &input_format)?;
    let input_context = ffmpeg::format::open_with(path0, &input_format, options)?;
    let mut input = input_context.input();
    let mut stream = input.stream_mut(0).unwrap();

    let context_decoder = ffmpeg::codec::context::Context::from_parameters(stream.parameters())?;
    let mut decoder = context_decoder.decoder().video()?;

    let mut scaler = ffmpeg::software::scaling::Context::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
        ffmpeg::util::format::Pixel::RGB24,
        decoder.width(),
        decoder.height(),
        ffmpeg::software::scaling::Flags::BILINEAR,
    )?;

    let mut frame_index = 0;

    let mut receive_and_process_decoded_frames =
        |decoder: &mut ffmpeg::decoder::Video| -> Result<(), ffmpeg::Error> {
            let mut decoded = ffmpeg::util::frame::Video::empty();
            while decoder.receive_frame(&mut decoded).is_ok() {
                let mut rgb_frame = ffmpeg::util::frame::Video::empty();
                scaler.run(&decoded, &mut rgb_frame)?;
                save_file(&rgb_frame, frame_index).unwrap();
                frame_index += 1;
            }
            Ok(())
        };

    for (stream, packet) in input.packets() {
        if stream.index() == stream.index() {
            decoder.send_packet(&packet)?;
            receive_and_process_decoded_frames(&mut decoder)?;
        }
    }
    decoder.send_eof()?;
    receive_and_process_decoded_frames(&mut decoder)?;

    Ok(())
}

fn save_file(
    frame: &ffmpeg::util::frame::Video,
    index: usize,
) -> std::result::Result<(), std::io::Error> {
    let mut file = std::fs::File::create(format!("frame{}.ppm", index))?;
    file.write_all(format!("P6\n{} {}\n255\n", frame.width(), frame.height()).as_bytes())?;
    file.write_all(frame.data(0))?;
    Ok(())
}
