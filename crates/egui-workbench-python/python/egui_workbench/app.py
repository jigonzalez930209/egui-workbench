"""The Pythonic workbench: commands, menus, widgets, events, threads, and asyncio."""

from __future__ import annotations

import asyncio
import inspect
import json
import queue
import threading
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Callable, Iterable, Sequence

from . import _native
from .widgets import TreeNode, Widget

SPEC_VERSION = 1


@dataclass
class Command:
    """A command shown in a menu and dispatched to a Python handler."""

    id: str
    title: str = ""
    shortcut: str | None = None
    category: str = ""
    when: str | None = None
    enabled: bool = True
    checked: bool = False

    def __post_init__(self) -> None:
        if not self.title:
            self.title = self.id

    def to_spec(self) -> dict[str, Any]:
        return {
            "id": self.id,
            "title": self.title,
            "category": self.category,
            "shortcut": self.shortcut,
            "when": self.when,
            "enabled": self.enabled,
            "checked": self.checked,
        }


@dataclass
class ToastOptions:
    """One toast, used by ``app.toast`` and ``ctx.toast``."""

    kind: str
    title: str
    description: str | None = None


class Toast:
    """The ``ctx.toast`` facade."""

    def __init__(self, app: "Workbench") -> None:
        self._app = app

    def info(self, title: str, description: str | None = None) -> None:
        self._app._queue_toast("info", title, description)

    def success(self, title: str, description: str | None = None) -> None:
        self._app._queue_toast("success", title, description)

    def warning(self, title: str, description: str | None = None) -> None:
        self._app._queue_toast("warning", title, description)

    def error(self, title: str, description: str | None = None) -> None:
        self._app._queue_toast("error", title, description)

    def loading(self, title: str, description: str | None = None) -> None:
        self._app._queue_toast("loading", title, description)


class Context:
    """The object a handler receives: the event, data, and thread-safe actions."""

    def __init__(self, app: "Workbench", event: dict[str, Any]) -> None:
        self.app = app
        self.event = event
        self.toast = Toast(app)

    @property
    def value(self) -> Any:
        """The new value for a ``value_changed`` event, else ``None``."""
        return self.event.get("value")

    @property
    def key(self) -> str | None:
        """The command id, tree key, or custom kind for the event."""
        return self.event.get("id") or self.event.get("key") or self.event.get("kind")

    @property
    def kind(self) -> str:
        """The host event name, such as ``command`` or ``value_changed``."""
        return str(self.event.get("event", ""))

    def patch(self, patch: dict[str, Any]) -> None:
        """Queue one JSON-pointer patch for the next frame."""
        self.app._queue_patch(patch)

    def set(self, path: str, value: Any) -> None:
        self.patch({"op": "set", "path": path, "value": value})

    def replace(self, path: str, value: Any) -> None:
        self.patch({"op": "replace", "path": path, "value": value})

    def remove(self, path: str) -> None:
        self.patch({"op": "remove", "path": path})

    def append(self, path: str, values: Sequence[Any]) -> None:
        self.patch({"op": "append", "path": path, "values": list(values)})

    def batch(self, patches: Iterable[dict[str, Any]]) -> None:
        self.patch({"op": "batch", "patches": list(patches)})

    def title(self, text: str) -> None:
        """Change the window title."""
        if self.app._native is not None:
            self.app._native.set_title(text)

    def quit(self) -> None:
        """Close the window."""
        self.app.close()

    def wake(self) -> None:
        """Request another frame."""
        if self.app._native is not None:
            self.app._native.wake()

    def command(self, command_id: str) -> None:
        """Dispatch a command event to the local handlers."""
        self.app._dispatch({"event": "command", "id": command_id})


class Workbench:
    """A native window described in Python.

    >>> app = Workbench(title="My App", theme="github-dark")  # doctest: +SKIP
    >>> app.menu("File", [Command("save", "Save", shortcut="Ctrl+S")])  # doctest: +SKIP
    >>> @app.on("save")  # doctest: +SKIP
    ... def save(ctx):
    ...     ctx.toast.success("Saved")
    >>> app.run()  # doctest: +SKIP
    """

    def __init__(
        self,
        title: str = "Workbench",
        *,
        theme: str = "github-dark",
        width: float = 980.0,
        height: float = 720.0,
        resizable: bool = True,
        custom_title_bar: bool = True,
        sidebar: bool = True,
        inspector: bool = False,
        bottom: bool = False,
        status: bool = True,
        follow_system: bool = False,
        reduced_motion: bool = False,
    ) -> None:
        self.title = title
        self.theme = theme
        self.width = width
        self.height = height
        self.resizable = resizable
        self.custom_title_bar = custom_title_bar
        self.panels = {"sidebar": sidebar, "inspector": inspector, "bottom": bottom, "status": status}
        self.follow_system = follow_system
        self.reduced_motion = reduced_motion
        self._widgets: list[Widget | dict[str, Any]] = []
        self._commands: dict[str, Command] = {}
        self._menus: list[dict[str, Any]] = []
        self._explorer: list[dict[str, Any]] = []
        self._status: dict[str, list[str]] = {"left": [], "right": []}
        self._dialogs: list[dict[str, Any]] = []
        self._toasts: list[ToastOptions] = []
        self._handlers: dict[str, list[Callable[[Context], Any]]] = {}
        self._datasets: dict[str, Any] = {}
        self._posted: queue.SimpleQueue[Callable[[], Any]] = queue.SimpleQueue()
        self._native: _native.NativeWorkbench | None = None
        self._loop: asyncio.AbstractEventLoop | None = None
        self._loop_thread: threading.Thread | None = None
        self._running = False
        self._closed = False
        self.toast = Toast(self)

    # -- structure -------------------------------------------------------

    def add(self, *widgets: Widget | dict[str, Any]) -> "Workbench":
        """Append widgets to the content tree."""
        self._widgets.extend(widgets)
        self._reset_native()
        return self

    def extend(self, widgets: Iterable[Widget | dict[str, Any]]) -> "Workbench":
        """Append an iterable of widgets."""
        return self.add(*widgets)

    def clear(self) -> "Workbench":
        """Remove every content widget."""
        self._widgets.clear()
        self._reset_native()
        return self

    def command(self, command: Command) -> Command:
        """Register a command without putting it in a menu."""
        self._commands[command.id] = command
        return command

    def menu(self, title: str, items: Sequence[Command | None | str]) -> "Workbench":
        """Add a top-level menu. ``None`` or ``\"-\"`` draws a separator."""
        rows = []
        for item in items:
            if item is None or item == "-":
                rows.append({"label": "", "command": None, "separator": True})
            else:
                command = item if isinstance(item, Command) else Command(item, item)
                self._commands[command.id] = command
                rows.append({"label": command.title, "command": command.id, "separator": False})
        self._menus.append({"title": title, "items": rows})
        self._reset_native()
        return self

    def explorer(self, nodes: Sequence[TreeNode | dict[str, Any]]) -> "Workbench":
        """Replace the explorer tree."""
        self._explorer = [node if isinstance(node, dict) else node.to_spec() for node in nodes]
        self._reset_native()
        return self

    def status(self, left: Sequence[str] = (), right: Sequence[str] = ()) -> "Workbench":
        """Set the status bar items."""
        self._status = {"left": list(left), "right": list(right)}
        self._reset_native()
        return self

    def dialog(
        self,
        id: str,
        title: str,
        message: str = "",
        confirm: str = "OK",
        cancel: str = "Cancel",
        *,
        open: bool = True,
    ) -> "Workbench":
        """Declare a dialog; its buttons emit ``<id>.confirm`` and ``<id>.cancel``."""
        self._dialogs.append(
            {
                "id": id,
                "title": title,
                "message": message,
                "confirm": confirm,
                "cancel": cancel,
                "open": open,
            }
        )
        self._reset_native()
        return self

    # -- events ----------------------------------------------------------

    def on(self, key: str) -> Callable[[Callable[[Context], Any]], Callable[[Context], Any]]:
        """Decorate a handler for a command id, event name, or ``value:<id>`` key."""

        def register(handler: Callable[[Context], Any]) -> Callable[[Context], Any]:
            self._handlers.setdefault(key, []).append(handler)
            return handler

        return register

    def post(self, callback: Callable[[], Any]) -> None:
        """Run ``callback`` on the UI thread. Safe from any thread."""
        native = self._native
        if native is not None:
            native.post(callback)
        else:
            self._posted.put(callback)

    def step(self, frames: int = 1) -> list[dict[str, Any]]:
        """Render headless frames, dispatch their events, and return them.

        This is the test and conformance path. It needs no display server.
        """
        native = self._ensure_native()
        raw = native.step(self._dispatcher, frames)
        return [json.loads(event) for event in raw]

    # -- run -------------------------------------------------------------

    def run(self) -> None:
        """Open the native window and run until it closes."""
        if self._running:
            raise RuntimeError("this workbench is already running")
        self._running = True
        self._closed = False
        native = self._ensure_native()
        try:
            native.run(self._dispatcher)
        finally:
            self._running = False
            self._closed = True
            self._stop_loop()

    async def run_async(self) -> None:
        """Run the window from an asyncio program.

        The native event loop still needs a thread; this is the convenient notebook and
        test spelling. Async event handlers get their own loop while the window runs.
        """
        await asyncio.to_thread(self.run)

    def close(self) -> None:
        """Ask a running window to close."""
        self._closed = True
        if self._native is not None:
            self._native.close()

    def emit(self, command_id: str) -> None:
        """Dispatch a command event as if its button or menu row was clicked."""
        self._ensure_native()
        self._dispatch({"event": "command", "id": command_id})

    def dataset_rows(self, name: str) -> int | None:
        """The row count of a registered dataset, when it exists."""
        if self._native is None:
            return None
        return self._native.dataset_rows(name)

    def current_spec(self) -> dict[str, Any]:
        """The spec the native host is rendering, including applied patches."""
        return json.loads(self._ensure_native().spec_json())

    # -- data ------------------------------------------------------------

    def dataset(self, name: str, data: Any) -> "Workbench":
        """Register a dataset from NumPy, pandas, Polars, PyArrow, a mapping, or rows.

        Arrow tables use the Arrow C data interface; NumPy arrays are encoded from their
        dtype and buffer. Both feed the same ``Dataset`` that grids and plots read.
        """
        self._datasets[name] = data
        if self._native is not None:
            _register_dataset(self._native, name, data)
        return self

    # -- spec ------------------------------------------------------------

    def spec(self) -> dict[str, Any]:
        """The current host spec as a JSON-compatible dictionary."""
        return {
            "version": SPEC_VERSION,
            "window": {
                "title": self.title,
                "width": self.width,
                "height": self.height,
                "resizable": self.resizable,
                "custom_title_bar": self.custom_title_bar,
            },
            "theme": {
                "id": self.theme,
                "follow_system": self.follow_system,
                "reduced_motion": self.reduced_motion,
            },
            "commands": [command.to_spec() for command in self._commands.values()],
            "menus": self._menus,
            "explorer": self._explorer,
            "layout": {"groups": [{"id": "main", "tabs": [], "active": 0}], "active": 0, "direction": "horizontal"},
            "panels": self.panels,
            "status": self._status,
            "widgets": [widget if isinstance(widget, dict) else widget.to_spec() for widget in self._widgets],
            "dialogs": self._dialogs,
            "toasts": [
                {"kind": toast.kind, "title": toast.title, "description": toast.description}
                for toast in self._toasts
            ],
            "metadata": {},
        }

    def to_json(self, *, indent: int | None = 2) -> str:
        """The spec as JSON."""
        return json.dumps(self.spec(), indent=indent)

    def save(self, path: str | Path) -> None:
        """Write the spec to a file (the sidecar can run it without Python)."""
        Path(path).write_text(self.to_json(), encoding="utf-8")

    @staticmethod
    def schema() -> dict[str, Any]:
        """The JSON Schema generated from the Rust host types."""
        return json.loads(_native.NativeWorkbench.schema())

    # -- internals -------------------------------------------------------

    def _reset_native(self) -> None:
        if not self._running:
            self._native = None

    def _ensure_native(self) -> _native.NativeWorkbench:
        if self._native is None:
            native = _native.NativeWorkbench(self.to_json(indent=None))
            for name, data in self._datasets.items():
                _register_dataset(native, name, data)
            while not self._posted.empty():
                native.post(self._posted.get())
            self._native = native
        return self._native

    def _dispatcher(self, raw: str) -> None:
        self._dispatch(json.loads(raw))

    def _dispatch(self, event: dict[str, Any]) -> None:
        kind = str(event.get("event", ""))
        keys = [kind]
        if kind == "command":
            keys.append(str(event.get("id", "")))
        elif kind == "value_changed":
            keys.extend([f"value:{event.get('id', '')}", "value"])
        elif kind == "tab_changed":
            keys.extend([f"tab:{event.get('id', '')}", "tab"])
        elif kind == "tree_selected":
            keys.extend([f"tree:{event.get('id', '')}", "tree"])
        elif kind == "custom":
            keys.extend([f"custom:{event.get('kind', '')}", "custom"])
        seen: set[int] = set()
        for key in keys:
            for handler in self._handlers.get(key, ()):
                if id(handler) in seen:
                    continue
                seen.add(id(handler))
                context = Context(self, event)
                result = handler(context)
                if inspect.isawaitable(result):
                    self._submit(result)

    def _submit(self, awaitable: Any) -> None:
        loop = self._ensure_loop()

        def done(future: asyncio.Future[Any]) -> None:
            try:
                future.result()
            except Exception as error:  # pragma: no cover - user code
                import traceback

                traceback.print_exception(type(error), error, error.__traceback__)

        asyncio.run_coroutine_threadsafe(awaitable, loop).add_done_callback(done)

    def _ensure_loop(self) -> asyncio.AbstractEventLoop:
        if self._loop is None:
            self._loop = asyncio.new_event_loop()
            self._loop_thread = threading.Thread(target=self._loop.run_forever, name="egui-workbench-asyncio", daemon=True)
            self._loop_thread.start()
        return self._loop

    def _stop_loop(self) -> None:
        loop = self._loop
        thread = self._loop_thread
        self._loop = None
        self._loop_thread = None
        if loop is None:
            return
        loop.call_soon_threadsafe(loop.stop)
        if thread is not None:
            thread.join(timeout=1.0)
        if not loop.is_running():
            loop.close()

    def _queue_patch(self, patch: dict[str, Any]) -> None:
        if self._native is not None:
            self._native.queue_patch(json.dumps(patch))

    def _queue_toast(self, kind: str, title: str, description: str | None) -> None:
        if self._native is not None:
            self._native.queue_toast(kind, title, description)
        else:
            self._toasts.append(ToastOptions(kind, title, description))


def _is_arrow(data: Any) -> bool:
    return hasattr(data, "__arrow_c_stream__") or hasattr(data, "__arrow_c_array__")


def _encode_column(value: Any) -> Any:
    """Turn one column into ``(dtype, bytes)`` or a Python list."""
    if hasattr(value, "to_numpy"):
        value = value.to_numpy()
    if hasattr(value, "dtype") and hasattr(value, "tobytes"):
        kind = getattr(value.dtype, "kind", "")
        if kind in "biuf":
            import numpy as np

            array = np.ascontiguousarray(value)
            if array.ndim != 1:
                raise ValueError("a dataset column must be one-dimensional")
            return (array.dtype.str, array.tobytes())
        if kind == "M":
            import numpy as np

            return (np.dtype("int64").str, np.ascontiguousarray(value).view("int64").tobytes())
        if kind == "c":
            raise ValueError("complex columns are not supported")
        return _clean_sequence(value.tolist())
    return _clean_sequence(list(value))


def _clean_sequence(values: Sequence[Any]) -> list[Any]:
    cleaned = []
    for value in values:
        if value is None or type(value).__name__ in ("NAType", "NaTType"):
            cleaned.append(None)
        elif isinstance(value, bytes):
            cleaned.append(value.decode("utf-8"))
        else:
            cleaned.append(value)
    return cleaned


def _mapping_from(data: Any) -> dict[str, Any]:
    if isinstance(data, dict):
        return data
    if hasattr(data, "columns") and hasattr(data, "__getitem__"):
        return {str(column): data[column] for column in data.columns}
    if hasattr(data, "to_dict"):
        try:
            return data.to_dict(as_series=False)
        except TypeError:
            pass
    if hasattr(data, "column_names") and hasattr(data, "column"):
        return {name: data.column(name) for name in data.column_names}
    if hasattr(data, "dtype") and hasattr(data, "ndim"):
        if data.ndim == 1:
            return {"value": data}
        if data.ndim == 2:
            return {f"column_{index}": data[:, index] for index in range(data.shape[1])}
    if isinstance(data, Sequence) and data and isinstance(data[0], dict):
        columns = list(data[0].keys())
        return {column: [row.get(column) for row in data] for column in columns}
    raise TypeError("dataset() accepts a mapping, rows, a NumPy array, or an Arrow table")


def _register_dataset(native: _native.NativeWorkbench, name: str, data: Any) -> None:
    if _is_arrow(data):
        try:
            native.set_arrow(name, data)
            return
        except Exception:
            if not (hasattr(data, "columns") or hasattr(data, "to_dict") or isinstance(data, dict)):
                raise
    mapping = _mapping_from(data)
    native.set_dataset(name, {str(column): _encode_column(value) for column, value in mapping.items()})
