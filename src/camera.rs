use std::path::Path;

use ffmpeg_next as ffmpeg;

use crate::mac_bindings::*;

fn get_ffmpeg_input_device_backend() -> Result<ffmpeg::Format, ffmpeg::Error> {
    ffmpeg::device::input::video()
        .find(|format| format.name() == "avfoundation")
        .ok_or(ffmpeg::Error::Unknown)
}

fn print_ffmpeg_input_device_backends(selected: &str) {
    for d in ffmpeg::device::input::video() {
        let name = d.name();
        if name == selected {
            println!("[*] ffmpeg {name}");
        } else {
            println!("[ ] ffmpeg {name}");
        }
    }
}

fn print_camera_devices(backend: &ffmpeg::Format) {
    if backend.name() != "avfoundation" {
        unimplemented!("only implemented for Mac OS AVFoundation");
    }

    let devices = AVCaptureDevice::devicesWithMediaType(AVMediaTypeVideo);
    for (index, device) in devices.iter().enumerate() {
        println!("[{index}] {:?}", device.localizedName());
    }
}

fn choose_camera_input(
    backend: &ffmpeg::Format,
) -> Result<ffmpeg::format::context::Input, ffmpeg::Error> {
    if backend.name() != "avfoundation" {
        unimplemented!("only implemented for Mac OS AVFoundation");
    }

    let path0 = &Path::new("default");
    let options = ffmpeg::dict!(
        "video_device_index" => "0",
        "framerate" => "30",
        "video_size" => "640,360",
        "pixel_format" => "bgr0", // 0rgb or bgr0 or others
    );
    let context = ffmpeg::format::open_with(path0, backend, options)?;
    Ok(context.input())
}

pub fn load_camera_stream() -> Result<CameraStream, Box<dyn std::error::Error>> {
    ffmpeg::init()?;
    ffmpeg::device::register_all();

    let backend = get_ffmpeg_input_device_backend()?;
    print_ffmpeg_input_device_backends(backend.name());
    print_camera_devices(&backend);
    let mut input = choose_camera_input(&backend)?;

    let stream = input.stream_mut(0).unwrap();
    let context_decoder = ffmpeg::codec::context::Context::from_parameters(stream.parameters())?;
    let decoder = context_decoder.decoder().video()?;

    Ok(CameraStream { input, decoder })
}

pub struct CameraStream {
    input: ffmpeg::format::context::Input,
    decoder: ffmpeg::decoder::Video,
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

        let output = ffmpeg::format::Pixel::RGBA;
        let mut converter = decoder.converter(output)?;
        let mut frame = VideoFrame::new(output, decoded_frame.width(), decoded_frame.height());

        // When I used RGB32 I got unexpected results. This safes me from doing this error again.
        assert_eq!(converter.output().format, output);
        assert_eq!(frame.format(), output);

        converter.run(&decoded_frame, &mut frame)?;

        Ok(frame)
    }
}
