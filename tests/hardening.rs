//! Regression tests for untrusted input and file handling.

use egui_workbench::Localizer;
use egui_workbench::data::{CellValue, CsvOptions, CsvSource, DataSource, Query};

#[test]
fn csv_ignores_a_byte_order_mark_and_blank_lines() {
    let text = "\u{feff}a,b\n1,2\n\n3,4\n\n";
    let source = CsvSource::parse("t.csv", text, CsvOptions::csv()).expect("csv");
    assert_eq!(source.schema()[0].name, "a");
    assert_eq!(source.row_count(), 2);
    assert_eq!(source.data().cell(1, 1), CellValue::Int(4));
}

#[test]
fn a_deeply_nested_query_is_an_error_not_a_stack_overflow() {
    let open = "(".repeat(100_000);
    let close = ")".repeat(100_000);
    assert!(Query::parse(&format!("{open}x == 1{close}")).is_err());
    assert!(Query::parse(&format!("{}x == 1", "not ".repeat(100_000))).is_err());
    assert!(Query::parse("((x == 1) and not (y == 2))").is_ok());
}

#[test]
fn a_locale_cannot_choose_the_file_that_is_read() {
    let directory = std::env::temp_dir().join(format!("wb-locale-{}", std::process::id()));
    let inner = directory.join("inner");
    std::fs::create_dir_all(&inner).expect("dir");
    std::fs::write(directory.join("evil.ftl"), "app.save = leaked\n").expect("write");
    let localizer = Localizer::load(&inner, "../evil").expect("load");
    assert_ne!(localizer.text("app.save"), "leaked");
    let _ = std::fs::remove_dir_all(&directory);
}

#[test]
fn interpolated_values_are_not_expanded_again() {
    let directory = std::env::temp_dir().join(format!("wb-interp-{}", std::process::id()));
    std::fs::create_dir_all(&directory).expect("dir");
    std::fs::write(directory.join("en.ftl"), "greet = Hi { $a } and { $b }\n").expect("write");
    let localizer = Localizer::load(&directory, "en").expect("load");
    let text = localizer.translate("greet", &[("a", "{ $b }"), ("b", "second")]);
    assert_eq!(text, "Hi { $b } and second");
    let _ = std::fs::remove_dir_all(&directory);
}

#[cfg(feature = "sqlite")]
#[test]
fn sqlite_is_read_only_and_never_creates_files() {
    use egui_workbench::data::read_sqlite;
    let directory = std::env::temp_dir().join(format!("wb-sqlite-{}", std::process::id()));
    std::fs::create_dir_all(&directory).expect("dir");
    let missing = directory.join("missing.db");
    assert!(read_sqlite(&missing, "select 1").is_err());
    assert!(!missing.exists());
    let path = directory.join("a.db");
    {
        let connection = rusqlite::Connection::open(&path).expect("open");
        connection
            .execute_batch("create table t(n integer); insert into t values (1);")
            .expect("setup");
    }
    assert!(read_sqlite(&path, "drop table t").is_err());
    assert!(read_sqlite(&path, "insert into t values (2)").is_err());
    assert_eq!(read_sqlite(&path, "select n from t").expect("select").row_count(), 1);
    let _ = std::fs::remove_dir_all(&directory);
}

#[test]
fn a_file_larger_than_the_limit_is_refused() {
    // The limit is checked on bytes read, so a sparse file shows it without 512 MiB of disk.
    let path = std::env::temp_dir().join(format!("wb-big-{}.bin", std::process::id()));
    let file = std::fs::File::create(&path).expect("create");
    file.set_len(egui_workbench::data::MAX_FILE_BYTES + 1).expect("sparse");
    assert!(egui_workbench::data::read_limited(&path).is_err());
    let _ = std::fs::remove_file(&path);
}

#[test]
fn a_deeply_nested_when_expression_is_an_error() {
    let source = format!("{}a{}", "(".repeat(100_000), ")".repeat(100_000));
    assert!(egui_workbench::When::parse(&source).is_err());
    assert!(egui_workbench::When::parse(&"!".repeat(100_000)).is_err());
    assert!(egui_workbench::When::parse("!(a && b) || c").is_ok());
}

#[test]
fn ellipsizing_to_zero_width_does_not_underflow() {
    assert_eq!(egui_workbench::ellipsize_start("abcdef", 0), "");
    assert_eq!(egui_workbench::ellipsize_start("abcdef", 3), "…ef");
}

#[test]
fn vscode_removal_bindings_are_not_imported() {
    let json = r#"[{"key":"ctrl+s","command":"-workbench.action.files.save"},{"key":"ctrl+s","command":"file.save"}]"#;
    let file = egui_workbench::Keybindings::import_vscode(json).expect("import");
    assert_eq!(file.bindings.len(), 1);
    assert_eq!(file.bindings[0].command.as_str(), "file.save");
}

#[test]
fn an_empty_key_sequence_cannot_be_bound() {
    use egui_workbench::{Command, CommandId, CommandRegistry, KeySequence};
    let mut registry = CommandRegistry::new();
    registry.register(Command::new("file.save", "Save"));
    assert!(registry.bind(&CommandId::new("file.save"), KeySequence::new()).is_err());
}

#[test]
fn a_ring_pages_without_copying_the_whole_buffer() {
    use egui_workbench::data::{ColumnSchema, ColumnType, DataValue, RingBuffer};
    let mut ring = RingBuffer::new(100, [ColumnSchema::new("n", ColumnType::Int)]);
    for value in 0..50 {
        ring.push([DataValue::Int(value)]).expect("push");
    }
    let page = ring.read_range(10..13).expect("page");
    assert_eq!(page.row_count(), 3);
    assert_eq!(page.cell(0, 0), CellValue::Int(10));
    assert_eq!(page.cell(2, 0), CellValue::Int(12));
    assert!(ring.read_range(40..60).is_err());
}

#[test]
fn escape_does_not_answer_a_closed_choice_window() {
    let ctx = egui::Context::default();
    let mut open = false;
    let mut input = egui::RawInput::default();
    input.events.push(egui::Event::Key {
        key: egui::Key::Escape,
        physical_key: Some(egui::Key::Escape),
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    });
    let mut choice = None;
    let mut output = ctx.run_ui(input, |ui| {
        choice = egui_workbench::show_choice_window(ui.ctx(), "t", "m", &["a", "b"], &mut open);
    });
    output.textures_delta.clear();
    assert_eq!(choice, None);
}
