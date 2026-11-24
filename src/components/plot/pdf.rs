//! Vector PDF export from the same scene the screen draws.

use std::fmt::Write as _;

use egui::{Color32, Rect};

use super::ExportOptions;
use super::scene::{Dash, Prim, Scene};

/// Write a one-page vector PDF. The page size is the export size in points.
pub(super) fn write(scene: &Scene, options: &ExportOptions) -> Vec<u8> {
    let width = options.width.max(120.0);
    let height = options.height.max(120.0);
    let mut content = String::new();
    content.push_str("1 J 1 j\n");
    for prim in scene.under.iter().chain(&scene.data).chain(&scene.over) {
        write_prim(&mut content, prim, height);
    }
    let objects = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_owned(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_owned(),
        format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {width:.2} {height:.2}] /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>"
        ),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>".to_owned(),
        format!("<< /Length {} >>\nstream\n{content}\nendstream", content.len()),
    ];
    let mut pdf = String::from("%PDF-1.4\n%\u{00e2}\u{00e3}\u{00cf}\u{00d3}\n");
    let mut offsets = Vec::new();
    for (index, object) in objects.iter().enumerate() {
        offsets.push(pdf.len());
        let _ = write!(pdf, "{} 0 obj\n{object}\nendobj\n", index + 1);
    }
    let xref = pdf.len();
    let _ = write!(pdf, "xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1);
    for offset in offsets {
        let _ = writeln!(pdf, "{offset:010} 00000 n ");
    }
    let _ = write!(
        pdf,
        "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
        objects.len() + 1
    );
    pdf.into_bytes()
}

fn write_prim(out: &mut String, prim: &Prim, page_height: f32) {
    match prim {
        Prim::Line {
            points,
            width,
            color,
            dash,
        } => {
            if points.len() < 2 || color.a() == 0 {
                return;
            }
            stroke_color(out, *color);
            let _ = writeln!(out, "{width:.2} w");
            if let Dash::Dashed(dash, gap) = dash {
                let _ = writeln!(out, "[{dash:.2} {gap:.2}] 0 d");
            }
            path(out, points, page_height, false);
            out.push_str("S\n");
            out.push_str("[] 0 d\n");
        }
        Prim::Area { points, base_y, color } => {
            fill_shape(out, points, *base_y, *color, page_height);
        }
        Prim::Polygon { points, color } => {
            if points.len() >= 3 && color.a() > 0 {
                fill_color(out, *color);
                path(out, points, page_height, true);
                out.push_str("f\n");
            }
        }
        Prim::Quad { points, color } => {
            if color.a() > 0 {
                fill_color(out, *color);
                path(out, points, page_height, true);
                out.push_str("f\n");
            }
        }
        Prim::Fan { center, points, color } => {
            if color.a() > 0 && points.len() >= 2 {
                fill_color(out, *color);
                let mut all = points.clone();
                all.push(*center);
                path(out, &all, page_height, true);
                out.push_str("f\n");
            }
        }
        Prim::Rect { rect, fill, stroke, .. } => {
            if fill.a() > 0 {
                fill_color(out, *fill);
                rect_path(out, *rect, page_height);
                out.push_str("f\n");
            }
            if let Some((width, color)) = stroke
                && color.a() > 0
            {
                stroke_color(out, *color);
                let _ = writeln!(out, "{width:.2} w");
                rect_path(out, *rect, page_height);
                out.push_str("S\n");
            }
        }
        Prim::Circle {
            center,
            radius,
            fill,
            stroke,
        } => {
            if *radius <= 0.0 {
                return;
            }
            if fill.a() > 0 {
                fill_color(out, *fill);
                circle_path(out, *center, *radius, page_height);
                out.push_str("f\n");
            }
            if let Some((width, color)) = stroke
                && color.a() > 0
            {
                stroke_color(out, *color);
                let _ = writeln!(out, "{width:.2} w");
                circle_path(out, *center, *radius, page_height);
                out.push_str("S\n");
            }
        }
        Prim::Text {
            pos,
            text,
            size,
            color,
            baseline,
            angle: _,
        } => {
            if text.is_empty() || color.a() == 0 {
                return;
            }
            text_color(out, *color);
            let y = page_height - pos.y - baseline;
            let escaped = text
                .chars()
                .map(|character| {
                    if character.is_ascii_graphic() || character == ' ' {
                        character
                    } else {
                        '?'
                    }
                })
                .flat_map(|character| match character {
                    '(' | ')' | '\\' => vec!['\\', character],
                    _ => vec![character],
                })
                .collect::<String>();
            let _ = writeln!(out, "BT /F1 {size:.2} Tf {:.2} {y:.2} Td ({escaped}) Tj ET", pos.x);
        }
    }
}

fn path(out: &mut String, points: &[egui::Pos2], page_height: f32, close: bool) {
    let Some(first) = points.first() else {
        return;
    };
    let _ = writeln!(out, "{:.2} {:.2} m", first.x, page_height - first.y);
    for point in points.iter().skip(1) {
        let _ = writeln!(out, "{:.2} {:.2} l", point.x, page_height - point.y);
    }
    if close {
        out.push_str("h\n");
    }
}

fn rect_path(out: &mut String, rect: Rect, page_height: f32) {
    let _ = writeln!(
        out,
        "{:.2} {:.2} {:.2} {:.2} re",
        rect.left(),
        page_height - rect.bottom(),
        rect.width(),
        rect.height()
    );
}

fn circle_path(out: &mut String, center: egui::Pos2, radius: f32, page_height: f32) {
    const K: f32 = 0.552_284_8;
    let (x, y) = (center.x, page_height - center.y);
    let (r, k) = (radius, radius * K);
    let _ = writeln!(out, "{:.2} {:.2} m", x + r, y);
    let _ = writeln!(
        out,
        "{:.2} {:.2} {:.2} {:.2} {:.2} {:.2} c",
        x + r,
        y + k,
        x + k,
        y + r,
        x,
        y + r
    );
    let _ = writeln!(
        out,
        "{:.2} {:.2} {:.2} {:.2} {:.2} {:.2} c",
        x - k,
        y + r,
        x - r,
        y + k,
        x - r,
        y
    );
    let _ = writeln!(
        out,
        "{:.2} {:.2} {:.2} {:.2} {:.2} {:.2} c",
        x - r,
        y - k,
        x - k,
        y - r,
        x,
        y - r
    );
    let _ = writeln!(
        out,
        "{:.2} {:.2} {:.2} {:.2} {:.2} {:.2} c",
        x + k,
        y - r,
        x + r,
        y - k,
        x + r,
        y
    );
}

fn fill_shape(out: &mut String, points: &[egui::Pos2], base_y: f32, color: Color32, page_height: f32) {
    if points.len() < 2 || color.a() == 0 {
        return;
    }
    fill_color(out, color);
    let mut all = points.to_vec();
    all.push(egui::pos2(points[points.len() - 1].x, base_y));
    all.push(egui::pos2(points[0].x, base_y));
    path(out, &all, page_height, true);
    out.push_str("f\n");
}

fn rgb(color: Color32) -> (f32, f32, f32) {
    let [red, green, blue, _] = color.to_srgba_unmultiplied();
    (
        f32::from(red) / 255.0,
        f32::from(green) / 255.0,
        f32::from(blue) / 255.0,
    )
}

fn fill_color(out: &mut String, color: Color32) {
    let (red, green, blue) = rgb(color);
    let _ = writeln!(out, "{red:.3} {green:.3} {blue:.3} rg");
}

fn stroke_color(out: &mut String, color: Color32) {
    let (red, green, blue) = rgb(color);
    let _ = writeln!(out, "{red:.3} {green:.3} {blue:.3} RG");
}

fn text_color(out: &mut String, color: Color32) {
    let (red, green, blue) = rgb(color);
    let _ = writeln!(out, "{red:.3} {green:.3} {blue:.3} rg");
}

#[cfg(test)]
mod tests {
    use super::super::scene::Scene;
    use super::*;
    use egui::{Color32, Pos2, vec2};

    #[test]
    fn a_pdf_has_a_catalog_page_and_eof() {
        let scene = Scene {
            size: vec2(320.0, 200.0),
            frame: Rect::from_min_size(Pos2::ZERO, vec2(320.0, 200.0)),
            under: vec![Prim::Rect {
                rect: Rect::from_min_size(Pos2::ZERO, vec2(320.0, 200.0)),
                radius: 0.0,
                fill: Color32::WHITE,
                stroke: None,
            }],
            data: vec![Prim::Line {
                points: vec![egui::pos2(10.0, 10.0), egui::pos2(100.0, 80.0)],
                width: 1.5,
                color: Color32::RED,
                dash: Dash::Solid,
            }],
            over: vec![Prim::Text {
                pos: egui::pos2(10.0, 20.0),
                text: "Signal".to_owned(),
                size: 12.0,
                color: Color32::BLACK,
                baseline: 9.0,
                angle: 0.0,
            }],
            drawn: Vec::new(),
        };
        let pdf = write(&scene, &ExportOptions::default());
        let text = String::from_utf8_lossy(&pdf);
        assert!(text.starts_with("%PDF-1.4"));
        assert!(text.contains("/Type /Catalog"));
        assert!(text.contains("/Type /Page"));
        assert!(text.contains("%%EOF"));
    }
}
