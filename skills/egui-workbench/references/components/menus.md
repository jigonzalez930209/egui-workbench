# Menus

> Text menus, menu rows, toggles, icon menu buttons, and navigator tabs. Live page: <https://jigonzalez930209.github.io/egui-workbench/components/menus.html>

## Text menus

`Menu` is a menu button with a popup. The closure draws compact menu contents; use
`MenuButton`, `MenuToggle`, or `MenuItem` inside it.

```rust
Menu::new("Appearance").show(ui, |ui| {
    if MenuToggle::new("Follow system", self.follow).show(ui) {
        self.toggle_follow();
    }
    ui.separator();
    Menu::new("Theme").show(ui, |ui| {
        for id in ThemeId::ALL {
            if MenuToggle::new(id.label(), id == self.theme).show(ui) {
                self.set_theme(id);
            }
        }
    });
});
```

The default popup width is 180 pt. Use `.width(220.0)` for long labels.

## MenuButton

A plain text row that closes the menu when clicked and returns `true`:

```rust
if MenuButton::new("Save").show(ui) {
    commands.push(Command::Save);
}
```

## MenuToggle

A selectable row with a check mark. It closes the menu and returns `true` when clicked:

```rust
if MenuToggle::new("Explorer", self.sidebar).show(ui) {
    commands.push(Command::ToggleSidebar);
}
```

## MenuItem

A 22 pt context-menu row with an icon, label, optional shortcut, and danger state.

```rust
if MenuItem::new("edit", "Rename…").shortcut("F2").show(ui, &mut icons) {
    rename();
}
if MenuItem::new("close", "Delete…").danger(true).show(ui, &mut icons) {
    delete();
}
```

`MenuItem::show` takes the icon cache and returns `bool`. Disabled rows are faded and
inert.

## MenuIconButton

The compact "…" button that opens a popup menu; the closure receives the icon cache.

```rust
MenuIconButton::new("more", "Tab actions")
    .width(210.0)
    .show(ui, &mut icons, |ui, _icons| {
        if MenuButton::new("Close tab").show(ui) {
            commands.push(Command::CloseTab);
        }
    });
```

## title_menu

`title_menu` is the title-bar variant: it paints the label with the title-bar typography
and a minimum popup width.

```rust
use egui_workbench::title_menu;

title_menu(ui, "File", 230.0, |ui| {
    if MenuButton::new("New Tab").show(ui) {
        commands.push(Command::NewTab);
    }
    if MenuButton::new("Save").show(ui) {
        commands.push(Command::Save);
    }
});
```

## NavTab

A full-width navigator row for sidebars and section lists.

```rust
if NavTab::new("Charts").active(section == Section::Charts).show(ui) {
    open(Section::Charts);
}
```

`NavTab` exposes `Role::Tab`; `.enabled(false)` fades it and ignores clicks.

## API summary

| Item | Type | Default | Description |
|---|---|---|---|
| `Menu::new(label)` | `Menu` | 180 pt | Popup menu; closure draws compact rows. |
| `MenuButton::new(label)` | `MenuButton` | — | Action row; true when clicked. |
| `MenuToggle::new(label, on)` | `MenuToggle` | — | Check row; true when toggled. |
| `MenuItem::new(icon, label)` | `MenuItem` | enabled | Icon row with shortcut and danger options. |
| `MenuIconButton::new(icon, tooltip)` | `MenuIconButton` | 180 pt | “…” button with a popup. |
| `NavTab::new(label)` | `NavTab` | inactive | Sidebar row with active and enabled states. |
| `title_menu(ui, label, width, body)` | `()` | — | Title-bar menu with the chrome typography. |

## Rules

- Every row produces a command; do not mutate state inside the menu closure.
- Put longer shortcuts first in the keymap, but display the conventional chord here.
- Use `ui.separator()` between logical groups.
- Call `ui.close()` only when you need to close a menu from a custom row; the builders
  already close on click.
- Icon-only menu buttons need a tooltip; it becomes their AccessKit name.
