"""Dataclass-like widgets that serialize to the host spec.

The native host renders a tree of JSON widget descriptions. These classes are a thin,
typed Python spelling of that tree; every widget has ``to_spec()`` and the workbench
calls it when a window is built.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, Iterable, Sequence


def _specs(widgets: Iterable["Widget | dict[str, Any]"]) -> list[dict[str, Any]]:
    return [widget if isinstance(widget, dict) else widget.to_spec() for widget in widgets]


class Widget:
    """Base class for widgets that can become a spec node."""

    def to_spec(self) -> dict[str, Any]:  # pragma: no cover - abstract
        raise NotImplementedError


@dataclass
class Label(Widget):
    text: str
    size: float = 13.0
    tone: str = "default"
    strong: bool = False

    def to_spec(self) -> dict[str, Any]:
        return {
            "kind": "label",
            "text": self.text,
            "size": self.size,
            "tone": self.tone,
            "strong": self.strong,
        }


@dataclass
class Button(Widget):
    id: str
    label: str = ""
    variant: str = "primary"
    enabled: bool = True

    def __post_init__(self) -> None:
        if not self.label:
            self.label = self.id

    def to_spec(self) -> dict[str, Any]:
        return {
            "kind": "button",
            "id": self.id,
            "label": self.label,
            "variant": self.variant,
            "enabled": self.enabled,
        }


@dataclass
class TextField(Widget):
    id: str
    label: str = ""
    value: str = ""
    hint: str = ""

    def to_spec(self) -> dict[str, Any]:
        return {
            "kind": "text_field",
            "id": self.id,
            "label": self.label,
            "value": self.value,
            "hint": self.hint,
        }


@dataclass
class Checkbox(Widget):
    id: str
    label: str = ""
    checked: bool = False

    def to_spec(self) -> dict[str, Any]:
        return {"kind": "checkbox", "id": self.id, "label": self.label, "checked": self.checked}


@dataclass
class Switch(Widget):
    id: str
    label: str = ""
    on: bool = False

    def to_spec(self) -> dict[str, Any]:
        return {"kind": "switch", "id": self.id, "label": self.label, "on": self.on}


@dataclass
class Select(Widget):
    id: str
    label: str = ""
    options: Sequence[str] = field(default_factory=tuple)
    selected: int = 0

    def to_spec(self) -> dict[str, Any]:
        return {
            "kind": "select",
            "id": self.id,
            "label": self.label,
            "options": list(self.options),
            "selected": self.selected,
        }


@dataclass
class Slider(Widget):
    id: str
    label: str = ""
    min: float = 0.0
    max: float = 1.0
    value: float = 0.0
    step: float = 0.01

    def to_spec(self) -> dict[str, Any]:
        return {
            "kind": "slider",
            "id": self.id,
            "label": self.label,
            "min": self.min,
            "max": self.max,
            "value": self.value,
            "step": self.step,
        }


@dataclass
class Separator(Widget):
    def to_spec(self) -> dict[str, Any]:
        return {"kind": "separator"}


@dataclass
class Spacer(Widget):
    height: float = 8.0

    def to_spec(self) -> dict[str, Any]:
        return {"kind": "spacer", "height": self.height}


@dataclass
class Row(Widget):
    children: Sequence[Widget | dict[str, Any]] = field(default_factory=tuple)
    gap: float = 8.0

    def to_spec(self) -> dict[str, Any]:
        return {"kind": "row", "children": _specs(self.children), "gap": self.gap}


@dataclass
class Column(Widget):
    children: Sequence[Widget | dict[str, Any]] = field(default_factory=tuple)
    gap: float = 8.0

    def to_spec(self) -> dict[str, Any]:
        return {"kind": "column", "children": _specs(self.children), "gap": self.gap}


@dataclass
class Card(Widget):
    title: str = ""
    children: Sequence[Widget | dict[str, Any]] = field(default_factory=tuple)

    def to_spec(self) -> dict[str, Any]:
        return {"kind": "card", "title": self.title, "children": _specs(self.children)}


@dataclass
class Tabs(Widget):
    id: str
    tabs: Sequence[str] = field(default_factory=tuple)
    active: int = 0
    children: Sequence[Sequence[Widget | dict[str, Any]]] = field(default_factory=tuple)

    def to_spec(self) -> dict[str, Any]:
        children = [_specs(content) for content in self.children]
        while len(children) < len(self.tabs):
            children.append([])
        return {"kind": "tabs", "id": self.id, "tabs": list(self.tabs), "active": self.active, "children": children}


@dataclass
class TreeNode:
    key: str
    label: str = ""
    icon: str | None = None
    expanded: bool = False
    children: Sequence["TreeNode"] = field(default_factory=tuple)

    def __post_init__(self) -> None:
        if not self.label:
            self.label = self.key

    def to_spec(self) -> dict[str, Any]:
        return {
            "key": self.key,
            "label": self.label,
            "icon": self.icon,
            "expanded": self.expanded,
            "children": [child.to_spec() for child in self.children],
        }


@dataclass
class Tree(Widget):
    id: str
    nodes: Sequence[TreeNode] = field(default_factory=tuple)
    selected: str | None = None

    def to_spec(self) -> dict[str, Any]:
        return {
            "kind": "tree",
            "id": self.id,
            "nodes": [node.to_spec() for node in self.nodes],
            "selected": self.selected,
        }


@dataclass
class DataGrid(Widget):
    id: str
    dataset: str
    height: float = 280.0

    def to_spec(self) -> dict[str, Any]:
        return {"kind": "data_grid", "id": self.id, "dataset": self.dataset, "height": self.height}


@dataclass
class Series:
    name: str
    points: Sequence[Sequence[float]] = field(default_factory=tuple)
    dataset: str | None = None
    x: str | None = None
    y: str | None = None
    style: str = "line"
    color: str | None = None
    width: float | None = None

    def to_spec(self) -> dict[str, Any]:
        return {
            "name": self.name,
            "style": self.style,
            "points": [list(point) for point in self.points],
            "dataset": self.dataset,
            "x": self.x,
            "y": self.y,
            "color": self.color,
            "width": self.width,
        }


@dataclass
class Plot(Widget):
    id: str
    title: str = ""
    series: Sequence[Series] = field(default_factory=tuple)
    height: float = 260.0
    renderer: str = "cpu"

    def to_spec(self) -> dict[str, Any]:
        return {
            "kind": "plot",
            "id": self.id,
            "title": self.title,
            "series": [series.to_spec() for series in self.series],
            "height": self.height,
            "renderer": self.renderer,
        }


@dataclass
class Custom(Widget):
    """A native Rust widget registered with ``Host::register_custom``."""

    widget: str
    props: dict[str, Any] = field(default_factory=dict)

    def to_spec(self) -> dict[str, Any]:
        return {"kind": "custom", "widget": self.widget, "props": self.props}
