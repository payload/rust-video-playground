use std::sync::mpsc::TryRecvError;

use rust_video_playground::*;

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "eframe template",
        native_options,
        Box::new(|cc| Box::new(CameraApp::new(cc))),
    );
}

#[derive(Default)]
struct CameraApp {
    egui_ctx: Option<egui::Context>,
    camera_frame_tex: Option<egui::TextureHandle>,
    camera_frame_rx: Option<std::sync::mpsc::Receiver<egui::TextureHandle>>,
}

impl CameraApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // note about egui: for different appearance, consider calling set_visuals and set_fonts

        // note about eframe persistance: cc.storage can be persistet and loaded again, see eframe::get_value
        // idea: save camera settings and reload next time

        Self {
            egui_ctx: Some(cc.egui_ctx.clone()),
            ..Self::default()
        }
    }

    fn load_camera_stream(&mut self) {
        let egui_ctx = self.egui_ctx.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        self.camera_frame_rx = Some(rx);

        std::thread::spawn(move || {
            let mut stream = match load_camera_stream() {
                Ok(stream) => stream,
                Err(err) => return eprintln!("{err}"),
            };

            loop {
                match stream.pull_frame() {
                    Ok(frame) => Self::send_frame_texture(&egui_ctx, &frame, &tx),
                    Err(err) => eprintln!("{err}"),
                }
            }
        });
    }

    fn get_camera_frame_tex(&mut self) -> Option<&egui::TextureHandle> {
        if let Some(rx) = &self.camera_frame_rx {
            match rx.try_recv() {
                Ok(tex) => self.camera_frame_tex = Some(tex),
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    self.camera_frame_tex = None;
                    self.camera_frame_rx = None;
                }
            }
        } else {
            self.camera_frame_tex = None;
        }
        self.camera_frame_tex.as_ref()
    }

    fn send_frame_texture(
        egui_ctx: &Option<egui::Context>,
        frame: &VideoFrame,
        tx: &std::sync::mpsc::Sender<egui::TextureHandle>,
    ) {
        if let Some(ctx) = egui_ctx {
            let tex = Self::frame_to_texture(ctx, &frame);
            if let Err(err) = tx.send(tex) {
                eprintln!("{err}");
            }
            ctx.request_repaint();
        }
    }

    fn frame_to_texture(ctx: &egui::Context, frame: &VideoFrame) -> egui::TextureHandle {
        let size = [frame.width() as usize, frame.height() as usize];
        let image = egui::ColorImage::from_rgba_unmultiplied(size, frame.data(0));
        ctx.load_texture("camera_frame", image, egui::TextureFilter::Nearest)
    }
}

impl eframe::App for CameraApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("get camera stream").clicked() {
                self.load_camera_stream();
            }
            for tex in self.get_camera_frame_tex() {
                ui.image(tex, tex.size_vec2() / 4.0);
            }
        });
    }
}
