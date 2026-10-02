//! The Rust side of the pyegui benchmark, for comparison.
//!
//! This is deliberately the same work as `bench/bench.py`: the same number of
//! widgets per frame, the same number of frames, and the same measurement
//! boundary -- the time spent building the frame's widgets, excluding eframe's
//! own compositing, which is identical in both languages and would only add
//! noise.
//!
//! The point is not to show that Rust is faster. The point is to show what the
//! Python binding *adds*, by measuring the same thing twice. If the two numbers
//! land close together, the binding is not the bottleneck in a normal app, and
//! that is the useful conclusion.
//!
//! Run with:
//!
//!     cargo run --release --manifest-path bench/Cargo.toml
//!
//! A window opens, so on a headless machine run it under `xvfb-run`.

use std::time::Instant;

const WIDGETS_PER_FRAME: usize = 500;
const STEADY_FRAMES: usize = 60;

fn main() -> eframe::Result {
    let start = Instant::now();
    let mut first_frame_ms: Option<f64> = None;
    let mut steady: Vec<f64> = Vec::with_capacity(STEADY_FRAMES);
    let mut frame_index = 0usize;

    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([640.0, 480.0])
            .with_title("bench"),
        ..Default::default()
    };

    eframe::run_native(
        "bench",
        native_options,
        Box::new(move |_cc, _| {
            Ok(Box::new(move |ctx, _frame| {
                let t0 = Instant::now();

                for i in 0..WIDGETS_PER_FRAME {
                    egui::Label::new(format!("row {i}")).ui(ctx);
                }

                let elapsed = t0.elapsed().as_secs_f64() * 1000.0;

                if frame_index == 0 {
                    first_frame_ms = Some(elapsed);
                }
                steady.push(elapsed);
                frame_index += 1;

                if frame_index >= STEADY_FRAMES {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                } else {
                    // The Python bench uses request_repaint for the same reason:
                    // egui idles when it sees no change, and an idle frame would
                    // never advance the loop.
                    ctx.request_repaint();
                }
            }))
        }),
    )?;

    let total_ms = start.elapsed().as_secs_f64() * 1000.0;
    let first = first_frame_ms.unwrap_or(f64::NAN);

    let min = steady.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = steady.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let mean = steady.iter().sum::<f64>() / steady.len() as f64;
    let var = steady.iter().map(|v| (v - mean).powi(2)).sum::<f64>()
        / (steady.len().saturating_sub(1)).max(1) as f64;

    let per_widget_us = |ms: f64| ms / WIDGETS_PER_FRAME as f64 * 1000.0;

    println!("{{");
    println!("  \"language\": \"rust\",");
    println!("  \"widgets_per_frame\": {WIDGETS_PER_FRAME},");
    println!("  \"frames_measured\": {},", steady.len());
    println!("  \"first_frame_ms\": {:.4},", first);
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

    Ok(())
}