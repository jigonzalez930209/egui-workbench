"""The Pythonic layer and the native step/Arrow paths."""

from __future__ import annotations

import asyncio
import json
import threading
import time
from pathlib import Path

import pytest

from egui_workbench import (
    Command,
    Label,
    Button,
    DataGrid,
    TextField,
    Workbench,
)
from egui_workbench import _native


def test_spec_widgets_and_schema() -> None:
    app = Workbench(title="Spec", width=640, height=480)
    app.add(Label("Hello", strong=True), Button("go", "Go"), TextField("name", "Name"))
    app.menu("File", [Command("save", "Save", shortcut="Ctrl+S")])
    spec = app.spec()
    assert spec["version"] == 1
    assert spec["window"]["title"] == "Spec"
    assert spec["widgets"][0]["kind"] == "label"
    assert spec["widgets"][0]["strong"] is True
    assert spec["commands"][0]["shortcut"] == "Ctrl+S"
    assert spec["menus"][0]["items"][0]["command"] == "save"
    schema = app.schema()
    assert schema["$schema"].startswith("http://json-schema.org/")
    assert "HostSpec" in json.dumps(schema)


def test_commands_patch_the_tree_and_show_toasts() -> None:
    app = Workbench(title="Commands")
    app.add(Label("before"), Button("go", "Go"))

    @app.on("go")
    def go(ctx):
        ctx.set("/widgets/0/text", "after")
        ctx.toast.success("Clicked", "one time")

    app.step(1)
    app.emit("go")
    assert app.step(1) == []
    assert app.current_spec()["widgets"][0]["text"] == "after"


def test_numpy_and_pandas_datasets() -> None:
    numpy = pytest.importorskip("numpy")
    app = Workbench(title="Data")
    app.add(DataGrid("grid", "trace", height=200))
    app.dataset(
        "trace",
        {
            "x": numpy.array([1.0, 2.0, 3.0]),
            "n": numpy.array([1, 2, 3], dtype=numpy.int32),
            "flag": numpy.array([True, False, True]),
            "name": numpy.array(["a", "b", "c"]),
        },
    )
    app.step(1)
    assert app.dataset_rows("trace") == 3
    pandas = pytest.importorskip("pandas")
    app.dataset("frame", pandas.DataFrame({"x": [1.0, 2.0], "name": ["a", "b"]}))
    app.step(1)
    assert app.dataset_rows("frame") == 2


def test_arrow_c_data_interface() -> None:
    pyarrow = pytest.importorskip("pyarrow")
    app = Workbench(title="Arrow")
    table = pyarrow.table({"x": [1.0, 2.0, 3.0], "name": ["a", "b", "c"]})
    app.dataset("table", table)
    app.step(1)
    assert app.dataset_rows("table") == 3
    array = pyarrow.array([4.0, 5.0])
    app.dataset("array", array)
    app.step(1)
    assert app.dataset_rows("array") == 2
    batch = pyarrow.record_batch({"x": [1.0, 2.0]})
    app.dataset("batch", batch)
    app.step(1)
    assert app.dataset_rows("batch") == 2


def test_the_host_conformance_fixture() -> None:
    path = Path(__file__).resolve().parents[2] / "egui-workbench-host" / "conformance" / "basic.json"
    fixture = json.loads(path.read_text())
    native = _native.NativeWorkbench(json.dumps(fixture["spec"]))
    for patch in fixture["patches"]:
        native.apply_patch(json.dumps(patch))
    native.step(lambda _event: None, 1)
    spec = json.loads(native.spec_json())
    assert spec["widgets"][0]["value"] == "Ada"
    assert spec["widgets"][2]["kind"] == "label"
    assert spec["widgets"][2]["text"] == "Hello, Ada"


def test_async_handler_and_post() -> None:
    app = Workbench(title="Async")
    app.add(Label("start"))
    calls: list[str] = []

    @app.on("go")
    async def go(ctx):
        await asyncio.sleep(0)
        calls.append(ctx.key or "")

    app.step(1)
    app.emit("go")
    deadline = time.time() + 2.0
    while not calls and time.time() < deadline:
        time.sleep(0.01)
    app._stop_loop()
    assert calls == ["go"]

    app.post(lambda: calls.append("posted"))
    app.step(1)
    assert calls == ["go", "posted"]


def test_post_is_safe_from_a_thread() -> None:
    app = Workbench(title="Threads")
    app.add(Label("x"))
    app.step(1)
    calls: list[int] = []
    thread = threading.Thread(target=lambda: app.post(lambda: calls.append(1)))
    thread.start()
    thread.join()
    app.step(1)
    assert calls == [1]


def test_save_and_unsupported_dtype() -> None:
    app = Workbench(title="Save")
    app.add(Label("x"))
    path = Path("conformance.json")
    app.save(path)
    try:
        assert json.loads(path.read_text())["version"] == 1
    finally:
        path.unlink(missing_ok=True)
    app.dataset("bad", {"x": [object()]})
    with pytest.raises(Exception):
        app.step(1)
