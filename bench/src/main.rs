//! The Rust side of the pyegui benchmark, for comparison.
//!
//! Deliberately the same work as `bench/bench.py`: the same number of labels per
//! frame, the same number of frames, and the same measurement boundary -- the
//! time spent building the frame's widgets, excluding eframe's compositing,
//! which is identical in either language and would only add noise.
//!
//! The point is not that Rust is faster. It is to measure what the Python
//! binding *adds*, by measuring the same thing twice.
//!
//! Written against `eframe::App` rather than a bare `run_native` closure:
//! `AppCreator` is `FnOnce(&CreationContext) -> Result<Box<dyn App>>`, so a
//! closure cannot be both the creator and the app. The trait is also eframe's
//! documented entry point.
//!
//! Run with:
//!
//!     cargo run --release --manifest-path bench/Cargo.toml
//!
//! A window opens, so on a headless machine run it under `xvfb-run`.

use std::time::Instant;

use eframe::egui;

const WIDGETS_PER_FRAME: usize = 500;
const STEADY_FRAMES: usize = 60;

#[derive(Default)]
struct Bench {
    started: Option<Instant>,
    first_frame_ms: Option<f64>,
    steady: Vec<f64>,
    frame_index: usize,
}

impl eframe::App for Bench {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.started.is_none() {
            self.started = Some(Instant::now());
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let t0 = Instant::now();

            for i in 0..WIDGETS_PER_FRAME {
                ui.label(format!("row {i}"));
            }

            let elapsed = t0.elapsed().as_secs_f64() * 1000.0;

            // The first frame is identified by the measurement list being empty
            // rather than by a separate counter, so the two cannot drift.
            if self.steady.is_empty() {
                self.first_frame_ms = Some(elapsed);
            }
            self.steady.push(elapsed);
        });

        self.frame_index += 1;

        if self.frame_index >= STEADY_FRAMES {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        } else {
            // egui idles when it sees no change, and an idle frame never
            // advances the loop -- 500 identical labels is not a visible change
            // after the first frame. bench/bench.py does the same for the same
            // reason.
            ctx.request_repaint();
        }
    }

    // Report while the state still exists: after run_native returns, the app has
    // been dropped and its measurements with it.
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.report();
    }
}

impl Bench {
    fn report(&self) {
        let total_ms = self.started.map_or(0.0, |s| s.elapsed().as_secs_f64() * 1000.0);

        let min = self.steady.iter().cloned().fold(f64::INFINITY, f64::min);
        let mean = if self.steady.is_empty() {
            f64::NAN
        } else {
            self.steady.iter().sum::<f64>() / self.steady.len() as f64
        };
        let var = if self.steady.len() > 1 {
            self.steady.iter().map(|v| (v - mean).powi(2)).sum::<f64>()
                / (self.steady.len() - 1) as f64
        } else {
            0.0
        };

        let per_widget_us = |ms: f64| ms / WIDGETS_PER_FRAME as f64 * 1000.0;

        println!("{{");
        println!("  \"language\": \"rust\",");
        println!("  \"widgets_per_frame\": {WIDGETS_PER_FRAME},");
        println!("  \"frames_measured\": {},", self.steady.len());
        println!(
            "  \"first_frame_ms\": {:.4},",
            self.first_frame_ms.unwrap_or(f64::NAN)
        );
        println!("  \"steady_frame_ms\": {{");
        println!("    \"min\": {min:.4},");
        println!("    \"mean\": {mean:.4},");
        println!("    \"stdev\": {:.4}", var.sqrt());
        println!("  }},");
        println!("  \"per_widget_us\": {{");
        println!("    \"min\": {:.3},", per_widget_us(min));
        println!("    \"mean\": {:.3}", per_widget_us(mean));
        println!("  }},");
        println!("  \"run_native_total_ms\": {total_ms:.2}");
        println!("}}");
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([640.0, 480.0])
            .with_title("bench"),
        ..Default::default()
    };

    eframe::run_native(
        "bench",
        options,
        Box::new(|_cc| Ok(Box::new(Bench::default()))),
    )
}