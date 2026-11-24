"""Native, IDE-grade workbench windows from a Python script.

``Workbench`` builds a versioned host spec, opens a native window through PyO3 and
egui-workbench, and dispatches command and value events to Python handlers. The same
widgets, plots, and data grids are available without a Rust toolchain.
"""

from __future__ import annotations

from . import _native
from .app import Command, Context, Toast, ToastOptions, Workbench
from .widgets import (
    Button,
    Card,
    Checkbox,
    Column,
    Custom,
    DataGrid,
    Label,
    Plot,
    Row,
    Select,
    Separator,
    Series,
    Slider,
    Spacer,
    Switch,
    Tabs,
    TextField,
    Tree,
    TreeNode,
    Widget,
)

__version__ = _native.__version__
SPEC_VERSION = 1

__all__ = [
    "Button",
    "Card",
    "Checkbox",
    "Column",
    "Command",
    "Context",
    "Custom",
    "DataGrid",
    "Label",
    "Plot",
    "Row",
    "Select",
    "Separator",
    "Series",
    "Slider",
    "Spacer",
    "Switch",
    "Tabs",
    "TextField",
    "Toast",
    "ToastOptions",
    "Tree",
    "TreeNode",
    "Widget",
    "Workbench",
    "SPEC_VERSION",
    "__version__",
]
