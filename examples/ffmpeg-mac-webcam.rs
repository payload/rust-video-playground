//! This example uses FFMPEG on MacOs to access a camera input device for RGB frames
//! and outputs those frames as frame123.ppm files in the current working directory.
//! 
//! It shows how to use the FFMPEG API to access frames from an input device in a loop.
//! FFMPEG is used because it implements camera device access on Mac with the
//! [AVFoundation API](https://developer.apple.com/av-foundation/).
//! Especially requesting
//! [permission for accessing the camera](https://developer.apple.com/documentation/avfoundation/capture_setup/requesting_authorization_for_media_capture_on_macos)
//! was otherwise not easy for me to achieve with rust yet.
//! 
//! Running the example is not failsafe. Run it twice if necessary.
//! 
//! There is some logging coming from the underlying [C code from FFMPEG](https://github.com/FFmpeg/FFmpeg).
//! There is probably a way to control this.
//! 
//! [FFMPEG documentation](https://www.ffmpeg.org/documentation.html) is huge but I only searched selectively
//! for relevant information.
//! To control AVFoundation as a video input device "format" I needed to understand that parameter documentation
//! is available like command line input parameters, but these map to key value options on the function calls.
//! See [AVFoundation documentation](https://www.ffmpeg.org/ffmpeg-devices.html#avfoundation) and the `options` argument
//! to the `ffmpeg::format::open_with` call.
//! 
//! It is also helpful to look into the underlying ffmpeg-next and -sys source code to find out about the exact
//! function calls these dependencies do on the C API from FFMPEG, like a call to `avformat_open_input`.
//! Note that libav* are libraries from the FFMPEG project.
//! Compare how FFMPEG command line tools are using those function calls itself.
//! 
//! It was also tremendously helpful to read the "simplest_*" projects from [Lei Xiaohua](https://github.com/leixiaohua1020),
//! like [simplest_ffmpeg_readcamera](https://github.com/leixiaohua1020/simplest_ffmpeg_device/blob/master/simplest_ffmpeg_readcamera/simplest_ffmpeg_readcamera.cpp).
//! Thanks [@leixiaohua1020](https://leixiaohua1020.github.io/)

use std::{io::Write, path::Path};

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
    let stream = input.stream_mut(0).unwrap();

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
