use std::path::Path;

use ffmpeg_next as ffmpeg;

pub fn load_camera_stream() -> Result<CameraStream, Box<dyn std::error::Error>> {
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
        "pixel_format" => "bgr0", // 0rgb or bgr0 or others // TODO this value is wrong when OBS camera is used
    );

    // let input_context = ffmpeg::format::open(path0, &input_format)?;
    let input_context = ffmpeg::format::open_with(path0, &input_format, options)?;
    let mut input = input_context.input();
    let stream = input.stream_mut(0).unwrap();

    let context_decoder = ffmpeg::codec::context::Context::from_parameters(stream.parameters())?;
    let decoder = context_decoder.decoder().video()?;

    let scaler = ffmpeg::software::scaling::Context::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
        decoder.format(),
        decoder.width(),
        decoder.height(),
        ffmpeg::software::scaling::Flags::BILINEAR,
    )?;

    Ok(CameraStream {
        input,
        decoder,
        scaler,
    })
}

pub struct CameraStream {
    input: ffmpeg::format::context::Input,
    decoder: ffmpeg::decoder::Video,
    scaler: ffmpeg::software::scaling::Context,
}

pub type VideoFrame = ffmpeg::util::frame::Video;

impl CameraStream {
    pub fn pull_frame(&mut self) -> Result<VideoFrame, Box<dyn std::error::Error>> {
        let Self { input, decoder, .. } = self;

        if let Some((_stream, packet)) = input.packets().next() {
            decoder.send_packet(&packet)?;
        } else {
            decoder.send_eof()?;
        }

        let mut decoded_frame = VideoFrame::empty();
        decoder.receive_frame(&mut decoded_frame)?;
        let frame = self.scale_frame(&decoded_frame)?;
        Ok(frame)
    }

    fn scale_frame(&mut self, input: &VideoFrame) -> Result<VideoFrame, ffmpeg::Error> {
        let mut output = VideoFrame::empty();
        self.scaler.run(input, &mut output)?;
        Ok(output)
    }
}
