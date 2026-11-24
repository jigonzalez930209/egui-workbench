//! Typed datasets, CSV, queries, paging, the ring/channel sources, and the grid state.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use egui_workbench::components::{DataGridState, SortDirection};
use egui_workbench::data::{
    ChannelSource, Column, ColumnSchema, ColumnType, CsvOptions, CsvSource, DataSource, Dataset, MemorySource, Query,
    RingBuffer,
};

fn csv() -> String {
    let mut text = String::from("t,voltage,cycle,state\n");
    for row in 0..10_000 {
        text.push_str(&format!(
            "{row},{},{},{}\n",
            (row as f64 * 0.001) % 1.0,
            row % 5,
            if row % 2 == 0 { "idle" } else { "run" }
        ));
    }
    text
}

#[test]
fn typed_csv_query_and_paging() {
    let source = CsvSource::parse("trace.csv", &csv(), CsvOptions::csv()).expect("csv");
    assert_eq!(source.row_count(), 10_000);
    assert_eq!(source.schema()[0].kind, ColumnType::Int);
    assert_eq!(source.schema()[1].kind, ColumnType::Float);
    let page = source.read_range(100..200).expect("page");
    assert_eq!(page.row_count(), 100);

    let data = source.data();
    let query = Query::parse("voltage > 0.5 && cycle in 2..5").expect("query");
    let rows = query.matching_rows(data);
    assert!(!rows.is_empty());
    assert!(
        rows.iter()
            .all(|row| data.cell(*row, 2).to_text().parse::<i64>().unwrap() >= 2)
    );

    let memory = MemorySource::new(data.clone());
    assert_eq!(memory.schema().len(), 4);
    assert!(memory.read_range(0..data.row_count()).is_ok());
}

#[test]
fn a_grid_sorts_filters_and_copies() {
    let data = Dataset::from_columns([
        Column::floats("voltage", vec![0.4, 0.1, 0.3]),
        Column::text("name", ["b", "a", "c"]),
    ]);
    let state = DataGridState {
        sort: Some((0, SortDirection::Ascending)),
        query: "voltage >= 0.1".to_owned(),
        ..DataGridState::default()
    };
    assert_eq!(state.visible_rows(&data).expect("rows"), [1, 2, 0]);
    let tsv = state.copy_tsv(&data);
    assert!(tsv.starts_with("voltage\tname\n"));
    assert!(tsv.contains("0.1\ta"));
}

#[test]
fn the_ring_and_channel_sources_are_data_sources() {
    let mut ring = RingBuffer::new(2, [ColumnSchema::new("v", ColumnType::Float)]);
    ring.push([egui_workbench::data::DataValue::Float(1.0)]).expect("push");
    ring.push([egui_workbench::data::DataValue::Float(2.0)]).expect("push");
    ring.push([egui_workbench::data::DataValue::Float(3.0)]).expect("push");
    assert_eq!(ring.row_count(), 2);
    assert_eq!(ring.snapshot().cell(0, 0).to_text(), "2");

    let (sender, receiver) = std::sync::mpsc::channel();
    let mut channel = ChannelSource::new(ring.snapshot());
    channel.attach(receiver);
    sender.send(ring.snapshot()).expect("send");
    assert!(channel.poll());
    assert_eq!(channel.row_count(), 2);
}
