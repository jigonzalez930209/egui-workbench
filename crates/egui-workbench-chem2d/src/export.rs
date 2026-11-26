//! Publication SVG and PNG export for a 2D document.

use egui_workbench_chem_core::BondOrder;

use crate::document::ChemDocument;
use crate::interact::atom_label;

/// Publication style.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ExportStyle {
    /// ACS 1996: black bonds, white background.
    Acs,
    /// RSC: dark gray bonds, white background.
    Rsc,
    /// Wiley: black bonds, generous margins.
    Wiley,
    /// High-contrast dark theme.
    Dark,
}

impl ExportStyle {
    fn colors(self) -> (String, String, String) {
        match self {
            Self::Acs => ("#ffffff".to_owned(), "#111111".to_owned(), "#333333".to_owned()),
            Self::Rsc => ("#ffffff".to_owned(), "#222222".to_owned(), "#444444".to_owned()),
            Self::Wiley => ("#ffffff".to_owned(), "#000000".to_owned(), "#222222".to_owned()),
            Self::Dark => ("#111827".to_owned(), "#f9fafb".to_owned(), "#d1d5db".to_owned()),
        }
    }
}

/// SVG/PNG export options.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExportOptions {
    /// Output width in pixels at 96 DPI.
    pub width: u32,
    /// Output height in pixels at 96 DPI.
    pub height: u32,
    /// Publication style.
    pub style: ExportStyle,
    /// Padding in pixels.
    pub padding: f32,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            width: 1200,
            height: 800,
            style: ExportStyle::Acs,
            padding: 40.0,
        }
    }
}

/// Render the document as SVG.
pub fn to_svg(document: &ChemDocument, options: ExportOptions) -> String {
    let (background, bond_color, text) = options.style.colors();
    let bounds = document.bounds().unwrap_or([-1.0, -1.0, 1.0, 1.0]);
    let width = (bounds[2] - bounds[0]).max(0.5);
    let height = (bounds[3] - bounds[1]).max(0.5);
    let padding = options.padding.max(4.0);
    let scale = ((options.width as f32 - 2.0 * padding) / width as f32)
        .min((options.height as f32 - 2.0 * padding) / height as f32)
        .max(1.0);
    let offset_x = (options.width as f32 - width as f32 * scale) * 0.5;
    let offset_y = (options.height as f32 - height as f32 * scale) * 0.5;
    let map = |x: f64, y: f64| {
        (
            offset_x + (x as f32 - bounds[0] as f32) * scale,
            offset_y + (bounds[3] as f32 - y as f32) * scale,
        )
    };
    let mut output = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\">\n<rect width=\"100%\" height=\"100%\" fill=\"{background}\"/>\n",
        options.width, options.height, options.width, options.height
    );
    for bond in &document.molecule.bonds {
        let (Some(a), Some(b)) = (document.molecule.atom(bond.a), document.molecule.atom(bond.b)) else {
            continue;
        };
        let (x1, y1) = map(a.position[0], a.position[1]);
        let (x2, y2) = map(b.position[0], b.position[1]);
        let width = if bond.order == BondOrder::Double || bond.order == BondOrder::Aromatic {
            3.0
        } else {
            1.6
        };
        output.push_str(&format!(
            "<line x1=\"{x1:.2}\" y1=\"{y1:.2}\" x2=\"{x2:.2}\" y2=\"{y2:.2}\" stroke=\"{bond_color}\" stroke-width=\"{width}\" stroke-linecap=\"round\"/>\n"
        ));
    }
    for atom in &document.molecule.atoms {
        let (x, y) = map(atom.position[0], atom.position[1]);
        let radius = 4.0 + atom.element.covalent_radius() as f32 * 3.0;
        let [r, g, b] = atom.element.cpk_color();
        output.push_str(&format!(
            "<circle cx=\"{x:.2}\" cy=\"{y:.2}\" r=\"{radius:.2}\" fill=\"#{r:02x}{g:02x}{b:02x}\" stroke=\"{bond_color}\" stroke-width=\"0.8\"/>\n"
        ));
        let label = atom_label(&document.molecule, atom.id);
        if !label.is_empty() {
            output.push_str(&format!(
                "<text x=\"{x:.2}\" y=\"{y:.2}\" font-family=\"sans-serif\" font-size=\"11\" fill=\"{text}\" text-anchor=\"middle\" dominant-baseline=\"middle\">{}</text>\n",
                escape(&label)
            ));
        }
    }
    for annotation in &document.annotations {
        if let crate::annotations::Annotation::Conditions(conditions) = annotation {
            let (x, y) = map(conditions.position[0], conditions.position[1]);
            let lines = conditions
                .lines
                .iter()
                .map(|line| crate::annotations::format_chemical_text(line))
                .collect::<Vec<_>>()
                .join(" ");
            output.push_str(&format!(
                "<text x=\"{x:.2}\" y=\"{y:.2}\" font-family=\"sans-serif\" font-size=\"12\" fill=\"{text}\">{}</text>\n",
                escape(&lines)
            ));
        }
    }
    output.push_str("</svg>\n");
    output
}

/// Most pixels a PNG export may allocate (about 400 MB of RGBA).
const MAX_PIXELS: u64 = 100_000_000;

/// Render the document as a PNG at `dpi` (96 DPI is 1x).
///
/// # Errors
/// When the SVG cannot be rasterized or encoded.
pub fn to_png(document: &ChemDocument, options: ExportOptions, dpi: u32) -> Result<Vec<u8>, String> {
    let svg = to_svg(document, options);
    let mut usvg_options = resvg::usvg::Options::default();
    // The SVG is generated here, but its labels come from the document: never let one pull
    // in a file through an `href`.
    usvg_options.image_href_resolver.resolve_string = Box::new(|_, _| None);
    let tree = resvg::usvg::Tree::from_str(&svg, &usvg_options).map_err(|error| error.to_string())?;
    // `dpi` and the size are caller-controlled: u32::MAX would ask for terapixels.
    let scale = dpi.clamp(72, 1_200) as f32 / 96.0;
    let width = (options.width as f32 * scale).round().max(1.0) as u32;
    let height = (options.height as f32 * scale).round().max(1.0) as u32;
    if u64::from(width) * u64::from(height) > MAX_PIXELS {
        return Err("the image is too large".to_owned());
    }
    let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height).ok_or_else(|| "pixmap".to_owned())?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::from_scale(
            width as f32 / options.width as f32,
            height as f32 / options.height as f32,
        ),
        &mut pixmap.as_mut(),
    );
    pixmap.encode_png().map_err(|error| error.to_string())
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_workbench_chem_core::{DocumentId, Element};

    fn document() -> ChemDocument {
        let mut document = ChemDocument::new(DocumentId::new(1), "test");
        let c = document
            .molecule
            .add_atom(Element::from_symbol("C").expect("C"), [0.0, 0.0, 0.0]);
        let o = document
            .molecule
            .add_atom(Element::from_symbol("O").expect("O"), [1.2, 0.0, 0.0]);
        document.molecule.add_bond(c, o, BondOrder::Double).expect("bond");
        document
    }

    #[test]
    fn svg_contains_the_atoms_and_style() {
        let svg = to_svg(&document(), ExportOptions::default());
        assert!(svg.starts_with("<?xml"));
        assert!(svg.contains("<svg"));
        assert!(svg.contains("fill=\"#ffffff\""));
        assert!(svg.contains("stroke-width"));
    }

    #[test]
    fn png_has_the_png_signature() {
        let png = to_png(&document(), ExportOptions::default(), 96).expect("png");
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
    }
}
