//! Viewer registry tests.

use super::*;

use super::*;

#[test]
fn detection_uses_extensions_and_sniffing() {
    assert_eq!(ViewerRegistry::detect("a.csv", b""), ViewerKind::Table);
    assert_eq!(ViewerRegistry::detect("a.json", b""), ViewerKind::Json);
    assert_eq!(ViewerRegistry::detect("a.yaml", b""), ViewerKind::Yaml);
    assert_eq!(ViewerRegistry::detect("a.toml", b""), ViewerKind::Toml);
    assert_eq!(ViewerRegistry::detect("a.md", b""), ViewerKind::Markdown);
    assert_eq!(ViewerRegistry::detect("a.log", b""), ViewerKind::Logs);
    assert_eq!(ViewerRegistry::detect("a.png", b""), ViewerKind::Image);
    assert_eq!(ViewerRegistry::detect("a.svg", b"<svg></svg>"), ViewerKind::Svg);
    assert_eq!(ViewerRegistry::detect("unknown", b"{\"a\":1}"), ViewerKind::Json);
    assert_eq!(ViewerRegistry::detect("unknown", b"hello"), ViewerKind::Text);
    assert_eq!(ViewerRegistry::detect("unknown", b"\0\x01"), ViewerKind::Hex);
}

#[test]
fn every_builtin_opens_from_one_show_call() {
    let ctx = egui::Context::default();
    let mut registry = ViewerRegistry::new();
    let files = [
        ViewerFile::new("a.csv", "x,y\n1,2\n"),
        ViewerFile::new("a.json", "{\"a\":[1,true]}"),
        ViewerFile::new("a.yaml", "a: [1, true]"),
        ViewerFile::new("a.toml", "a = 1\n"),
        ViewerFile::new("a.md", "# Title\n\nText."),
        ViewerFile::new("a.log", "INFO ok\nERROR bad\n"),
        ViewerFile::new("a.rs", "fn main() {}"),
        ViewerFile::new("a.bin", [0u8, 1, 2, 255]),
        ViewerFile::new(
            "a.svg",
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"8\" height=\"8\"><rect width=\"8\" height=\"8\"/></svg>",
        ),
    ];
    for file in files {
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let _ = registry.show(ui, &file);
        });
        output.textures_delta.clear();
    }
}
