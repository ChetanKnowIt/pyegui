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

const STEADY_FRAMES: usize = 60;

/// Widgets per frame. Overhead scales with widget count, so the count is an
/// input rather than a constant: `WIDGETS` lets the workflow re-run the
/// comparison at a second count without a code change. `bench/bench.py` reads
/// the same variable, so both sides always measure the same thing.
fn widgets_per_frame() -> usize {
    std::env::var("WIDGETS")
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(500)
}

/// The scenario under test, read from the environment for the same reason
/// `WIDGETS` is: both halves read it, so a typo cannot make one side measure
/// something other than the other.
///
/// An unknown value is rejected rather than defaulted. A silent fallback to
/// `label` would produce a mismatched pair whose ratio looks like a real one,
/// which is the failure the combine step's parity guard exists to catch.
///
/// The names are duplicated from `bench/bench.py` rather than shared, because
/// a Rust binary cannot read the Python module at runtime. `tests/doc_claims.py`
/// asserts the two lists agree, since drift here is silent otherwise.
fn scenario() -> Result<&'static str, String> {
    let raw = std::env::var("SCENARIO").unwrap_or_else(|_| "label".to_string());
    match raw.as_str() {
        "label" => Ok("label"),
        "text_edit_plain" => Ok("text_edit_plain"),
        "text_edit_hint" => Ok("text_edit_hint"),
        _ => Err(format!(
            "unknown SCENARIO '{raw}'; valid scenarios are label, \
             text_edit_plain, text_edit_hint, python_side"
        )),
    }
}

#[derive(Default)]
struct Bench {
    started: Option<Instant>,
    first_frame_ms: Option<f64>,
    steady: Vec<f64>,
    frame_index: usize,
    // Read once in main, not per frame: `std::env` in the measured path would be
    // this benchmark's own overhead rather than the binding's.
    widgets: usize,
    scenario: &'static str,
    // One buffer, mutated in place across frames and reused by every widget
    // in the frame, matching the single module-level `Str` in bench/bench.py.
    // A frame-local would be equivalent -- `String::new()` does not allocate
    // and TextEdit writes back only into this buffer -- but a reader comparing
    // the two halves should not have to work that out.
    name: String,
}

impl eframe::App for Bench {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.started.is_none() {
            self.started = Some(Instant::now());
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let t0 = Instant::now();

            // Copied out first: matching on `self.scenario` holds an immutable
            // borrow of `self` for the whole match, which a `&mut self.name`
            // inside an arm could not coexist with.
            let scenario = self.scenario;
            let name = &mut self.name;

            match scenario {
                "label" => {
                    for i in 0..self.widgets {
                        ui.label(format!("row {i}"));
                    }
                }
                "text_edit_plain" => {
                    // `Ui::text_edit_singleline` is the shorthand that returns a
                    // `Response`, so there is no widget left to configure. The
                    // builder form is what the hint scenario needs, and using it
                    // for both keeps the only difference between the two arms
                    // the option itself.
                    for _ in 0..self.widgets {
                        ui.add(egui::TextEdit::singleline(name));
                    }
                }
                "text_edit_hint" => {
                    for _ in 0..self.widgets {
                        ui.add(egui::TextEdit::singleline(name).hint_text("name"));
                    }
                }
                _ => {}
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

        let per_widget_us = |ms: f64| ms / self.widgets as f64 * 1000.0;

        println!("{{");
        println!("  \"language\": \"rust\",");
        println!("  \"widgets_per_frame\": {},", self.widgets);
        println!("  \"scenario\": \"{}\",", self.scenario);
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

    // Validated once, before the window opens, and reused by the app: reading
    // `std::env` per frame would make this benchmark measure its own
    // environment lookup rather than the binding.
    //
    // Bound to a local rather than propagated with `?` inside the creator
    // closure: `scenario()`'s error is a String, and `AppCreator`'s is
    // eframe's, so `?` there would not compile. The exit happens before any
    // window is created, which is where a bad scenario should be caught.
    let scenario = match scenario() {
        Ok(scenario) => scenario,
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(2);
        }
    };

    eframe::run_native(
        "bench",
        options,
        Box::new(|_cc| {
            Ok(Box::new(Bench {
                widgets: widgets_per_frame(),
                scenario,
                name: String::new(),
                ..Default::default()
            }))
        }),
    )
}