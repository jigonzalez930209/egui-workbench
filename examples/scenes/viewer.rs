//! The native file viewer: an Excel workbook with sheet tabs, CSV, TSV, and plain text.

use egui_workbench::components::prelude::*;
use egui_workbench::components::viewer::FileViewer;
use egui_workbench::egui;

use super::{Host, Scene};

const KIND_LABELS: [&str; 4] = ["Workbook (XLSX)", "CSV", "TSV", "Plain text"];

/// One file per format, opened from embedded bytes so the demo runs in a browser too.
pub struct ViewerScene {
    viewer: FileViewer,
    kind: usize,
}

impl Default for ViewerScene {
    fn default() -> Self {
        Self {
            viewer: open(0),
            kind: 0,
        }
    }
}

fn open(kind: usize) -> FileViewer {
    let result = match kind {
        0 => FileViewer::from_bytes("sales.xlsx", include_bytes!("../../tests/fixtures/sales.xlsx")),
        1 => FileViewer::from_bytes(
            "measurements.csv",
            "sensor,time,value,ok\n\
             alpha,0.0,12.5,true\n\
             alpha,0.5,12.9,true\n\
             beta,0.0,8.1,false\n\
             beta,0.5,8.4,true\n\
             \"gamma, room 2\",1.0,10.0,true\n",
        ),
        2 => FileViewer::from_bytes(
            "matrix.tsv",
            "row\tcol\tvalue\n\
             1\t1\t0.5\n\
             1\t2\t0.8\n\
             2\t1\t0.2\n\
             2\t2\t0.9\n",
        ),
        _ => FileViewer::from_bytes(
            "notes.txt",
            "Workbench notes\n\
             ===============\n\n\
             Plain UTF-8 files open in a virtualized reader with line numbers.\n\
             Tabs expand to four spaces; long lines scroll horizontally.\n\n\
             1\tThe viewer never lays out a row that is not on screen.\n\
             2\tWorkbook tabs keep the active sheet until the file is closed.\n",
        ),
    };
    result.unwrap_or_else(|error| FileViewer::text("error.txt", format!("Could not open the demo: {error}")))
}

impl Scene for ViewerScene {
    fn scroll(&self) -> bool {
        false
    }

    fn ui(&mut self, ui: &mut egui::Ui, _host: &mut Host<'_>) {
        ui.horizontal_wrapped(|ui| {
            for (index, label) in KIND_LABELS.into_iter().enumerate() {
                if Button::new(label)
                    .size(Size::Sm)
                    .variant(ButtonVariant::Ghost)
                    .selected(index == self.kind)
                    .show(ui)
                    .clicked()
                    && index != self.kind
                {
                    self.kind = index;
                    self.viewer = open(index);
                }
            }
        });
        ui.add_space(6.0);
        self.viewer.show(ui);
    }
}
