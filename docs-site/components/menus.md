---
title: Menus
description: Text menus, menu rows, toggles, icon menu buttons, and navigator tabs.
---

# Menus

<LiveDemo
  scene="menus"
  height="300"
  alt="A context menu under a tree row"
  caption="Dropdown menus with shortcuts and toggles, an overflow menu, and a context menu (right-click the box)."
/>

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

<ApiTable :rows="[
  { name: 'Menu::new(label)', type: 'Menu', default: '180 pt', description: 'Popup menu; closure draws compact rows.' },
  { name: 'MenuButton::new(label)', type: 'MenuButton', default: '—', description: 'Action row; true when clicked.' },
  { name: 'MenuToggle::new(label, on)', type: 'MenuToggle', default: '—', description: 'Check row; true when toggled.' },
  { name: 'MenuItem::new(icon, label)', type: 'MenuItem', default: 'enabled', description: 'Icon row with shortcut and danger options.' },
  { name: 'MenuIconButton::new(icon, tooltip)', type: 'MenuIconButton', default: '180 pt', description: '“…” button with a popup.' },
  { name: 'NavTab::new(label)', type: 'NavTab', default: 'inactive', description: 'Sidebar row with active and enabled states.' },
  { name: 'title_menu(ui, label, width, body)', type: '()', default: '—', description: 'Title-bar menu with the chrome typography.' }
]" />

## Rules

- Every row produces a command; do not mutate state inside the menu closure.
- Put longer shortcuts first in the keymap, but display the conventional chord here.
- Use `ui.separator()` between logical groups.
- Call `ui.close()` only when you need to close a menu from a custom row; the builders
  already close on click.
- Icon-only menu buttons need a tooltip; it becomes their AccessKit name.
