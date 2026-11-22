//! The native file viewer: workbooks with tabs, CSV/TSV, and plain text.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use egui_kittest::Harness;
use egui_workbench::ThemeId;
use egui_workbench::components::viewer::{Document, FileFormat, FileViewer, TextFile};
use egui_workbench::egui;

struct Board {
    viewer: FileViewer,
}

fn harness(viewer: FileViewer) -> Harness<'static, Board> {
    Harness::builder()
        .with_size(egui::vec2(900.0, 560.0))
        .with_context_setup(|ctx| egui_workbench::theme::apply_id(ctx, ThemeId::GithubLight))
        .build_ui_state(
            |ui, board: &mut Board| {
                egui::CentralPanel::default().show(ui, |ui| board.viewer.show(ui));
            },
            Board { viewer },
        )
}

#[test]
fn an_xlsx_workbook_lists_its_sheets_and_cells() {
    let viewer = FileViewer::open("tests/fixtures/sales.xlsx").expect("workbook");
    assert_eq!(viewer.format(), FileFormat::Spreadsheet);
    assert_eq!(viewer.sheet_names(), ["Sales", "Notes"]);
    let Document::Workbook(workbook) = viewer.document() else {
        panic!("expected a workbook");
    };
    assert_eq!(workbook.sheets.len(), 2);
    assert_eq!(workbook.sheets[0].table.height(), 5);
    assert_eq!(workbook.sheets[0].table.width(), 3);
    assert_eq!(workbook.sheets[0].table.cell(1, 0).text, "North");
    assert_eq!(workbook.sheets[1].name, "Notes");
}

#[test]
fn csv_and_tsv_become_one_sheet_workbooks() {
    let csv = FileViewer::from_bytes("scores.csv", "name,score\nAda,10\nGrace,12\n").expect("csv");
    assert_eq!(csv.sheet_names(), ["scores"]);
    let Document::Workbook(workbook) = csv.document() else {
        panic!("expected a workbook");
    };
    assert_eq!(workbook.sheets[0].table.width(), 2);
    let tsv = FileViewer::from_bytes("scores.tsv", "name\tscore\nAda\t10\n").expect("tsv");
    assert_eq!(tsv.format(), FileFormat::Tsv);
}

#[test]
fn text_files_keep_their_lines_and_reject_binary() {
    let text = TextFile::from_bytes(b"first\r\nsecond\nthird").expect("text");
    assert_eq!(text.len(), 3);
    assert_eq!(text.line(0), "first");
    assert_eq!(text.line(2), "third");
    assert!(TextFile::from_bytes(b"a\0b").is_err());
}

#[test]
fn the_viewer_renders_every_document_kind() {
    for viewer in [
        FileViewer::open("tests/fixtures/sales.xlsx").expect("workbook"),
        FileViewer::from_bytes("scores.csv", "name,score\nAda,10\nGrace,12\n").expect("csv"),
        FileViewer::from_bytes("notes.txt", "first line\nsecond line\n").expect("text"),
    ] {
        let mut harness = harness(viewer);
        harness.run_steps(4);
    }
}
