//! SVG and PNG export of the plot.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use egui_workbench::components::plot::{ExportOptions, FitKind, Marker, Plot, PlotState, Series};
use egui_workbench::{ThemeId, egui, theme};

fn plot<'a>(line: &'a [[f64; 2]], dots: &'a [[f64; 2]]) -> Plot<'a> {
    Plot::new("export")
        .title("Signal & samples <test>")
        .x_label("Time (s)")
        .y_label("Voltage (V)")
        .series(Series::line("signal", line))
        .series(Series::line("marked", &line[..40]).marker(Marker::Circle))
        .series(Series::scatter("samples", dots))
        .fit(FitKind::Linear, dots)
}

fn data() -> (Vec<[f64; 2]>, Vec<[f64; 2]>) {
    let line = (0..200)
        .map(|i| {
            let x = f64::from(i) * 0.05;
            [x, (x * 1.3).sin() * 2.0 + x * 0.4]
        })
        .collect();
    let dots = (0..20)
        .map(|i| {
            let x = f64::from(i) * 0.5;
            [x, 0.4 * x + 0.5 + f64::from(i % 3) * 0.3]
        })
        .collect();
    (line, dots)
}

fn context(id: ThemeId) -> egui::Context {
    let ctx = egui::Context::default();
    theme::apply_id(&ctx, id);
    // Fonts load on the first frame.
    let mut output = ctx.run_ui(egui::RawInput::default(), |_| {});
    output.textures_delta.clear();
    ctx
}

#[test]
fn svg_is_a_complete_escaped_document() {
    let (line, dots) = data();
    let ctx = context(ThemeId::GithubLight);
    let svg = plot(&line, &dots).to_svg(&ctx, &PlotState::default(), &ExportOptions::default());
    assert!(svg.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"960\" height=\"540\""));
    assert!(svg.trim_end().ends_with("</svg>"));
    assert!(svg.contains("Signal &amp; samples &lt;test&gt;"), "text is escaped");
    assert!(svg.contains("clip-path=\"url(#plot-frame)\""));
    assert!(svg.contains("<polyline"));
    assert!(svg.contains("rotate(-90"), "the y label is rotated");
    assert!(svg.contains("stroke-dasharray"), "the fit is dashed");
}

#[test]
fn png_renders_at_the_requested_scale() {
    let (line, dots) = data();
    for id in [ThemeId::GithubLight, ThemeId::GithubDark] {
        let ctx = context(id);
        let options = ExportOptions {
            width: 800.0,
            height: 450.0,
            scale: 2.0,
            transparent: false,
        };
        let png = plot(&line, &dots)
            .to_png(&ctx, &PlotState::default(), &options)
            .unwrap();
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
        // IHDR width and height.
        let width = u32::from_be_bytes([png[16], png[17], png[18], png[19]]);
        let height = u32::from_be_bytes([png[20], png[21], png[22], png[23]]);
        assert_eq!((width, height), (1600, 900));
    }
}
