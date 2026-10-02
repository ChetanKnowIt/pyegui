// The same app as examples/hello.py, in egui 0.31.1.
//
// This is the ergonomics comparison, not the timing one. Same widgets, same
// interaction state, no second framework to learn: `Response` here is
// `egui::Response`, not a binding's redefinition of it.
//
// Kept honest deliberately -- this is the real egui API, not a flattering
// rewrite. Rust's type system carries things Python has to spell out, and where
// pyegui accepts a plain callable instead of a generic `impl FnOnce`, that is
// the difference being demonstrated rather than papered over.

use eframe::egui;

struct App {
    name: String,
    clicks: usize,
    checked: bool,
    slider: f32,
    scroll: egui::ScrollArea,
}

impl Default for App {
    fn default() -> Self {
        Self {
            name: "world".to_owned(),
            clicks: 0,
            checked: false,
            slider: 0.5,
            scroll: egui::ScrollArea::vertical(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("hello, egui");

            // egui's Response is the same object pyegui returns.
            let clicked: egui::Response = ui.button("click me");
            if clicked.clicked() {
                self.clicks += 1;
            }
            ui.label(format!("clicked {} times", self.clicks));

            let response = ui.checkbox(&mut self.checked, "enabled");
            ui.label(format!("checkbox changed: {}", response.changed()));

            let drag = ui.add(
                egui::Slider::new(&mut self.slider, 0.0..=1.0).text("amount"),
            );
            ui.label(format!("dragged: {}", drag.dragged()));

            let text = ui.text_edit_singleline(&mut self.name);
            ui.label(format!("text changed: {}", text.changed()));

            self.scroll.show(ui, |ui| {
                for i in 0..50 {
                    ui.label(format!("row {i}"));
                }
            });

            if ui.button("quit").clicked() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });
    }
}

fn main() -> eframe::Result {
    eframe::run_native(
        "hello",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(App::default()))),
    )
}
