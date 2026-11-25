# Platform pixel testing

> The manual native matrix for Windows, macOS, Linux X11, Wayland compositors, hit testing, and the legacy support boundary. Live page: <https://jigonzalez930209.github.io/egui-workbench/reference/platform-testing.html>

Native pixels depend on the window backend, the compositor, DPI scaling, and the GPU
driver. The Linux snapshot suite is the fast gate; the native matrix is manual because a
GitHub-hosted runner cannot represent every desktop.

## Run it

```text
Actions → Platform pixels → Run workflow
```

The workflow builds one `platform_probe` example and uploads a PNG + JSON report per
runner. It does not run on every push.

Hosted coverage:

| Runner | What it covers |
|---|---|
| `windows-2019`, `windows-2022`, `windows-latest` | Windows 10/11-era APIs, DPI, undecorated windows |
| `macos-13`, `macos-14`, `macos-15` | Intel and Apple Silicon, old and current macOS hosted images |
| `ubuntu-22.04`, `ubuntu-24.04` | X11 with Xvfb |
| `ubuntu-24.04` + weston headless | Wayland protocol and EGL |

Enable **include_self_hosted** to run labels for the machines GitHub cannot host:

```text
[self-hosted, windows, win7]
[self-hosted, windows, win10]
[self-hosted, windows, win11]
[self-hosted, macOS, intel, old]
[self-hosted, macOS, modern]
[self-hosted, linux, x11, gnome]
[self-hosted, linux, wayland, kde]
[self-hosted, linux, wayland, sway]
```

The jobs stay queued until a runner with those labels is online. That is intentional:
the matrix spends no hosted minutes when you do not need it.

## The probe

```bash
cargo run --example platform_probe -- --screenshot artifacts/probe.png
```

It draws the real 34 pt title bar, a 6×8 button grid, the pointer crosshair, DPI, content
rect, and viewport rect. The JSON sidecar records:

```json
{
  "os": "linux",
  "pixels_per_point": 1.0,
  "content_rect": [0, 0, 900, 600],
  "screen_rect": [0, 0, 900, 600],
  "viewport_inner_rect": null,
  "pointer": [202.2, 513.9],
  "clicks": [["r5c1", [205.0, 510.0]]]
}
```

Compare the drawn button center with the recorded pointer. A vertical offset that grows
with row index points at a DPI/`pixels_per_point` mismatch; a constant offset points at a
window frame or title-bar inset. `ensure_undecorated` re-sends
`ViewportCommand::Decorations(false)` once on Windows to remove a native frame that can
survive the initial window creation.

## Support boundary

- **Windows 10 and 11** are supported by 1.x with Rust 1.95 and egui 0.37.
- **Windows 7/8.1** are not supported by this stack: Rust 1.78 raised the Windows minimum
  to Windows 10, and winit 0.30 uses Windows 10 APIs. A Win7 build needs a frozen legacy
  toolchain and egui version as a separate product, not a flag on 1.x. The self-hosted
  `win7` label exists so the failure is visible and measured, not assumed away.
- **macOS 13+** (Ventura or newer) is the supported line; `macos-13` is the oldest hosted
  image, with macOS 14/15 covering Apple Silicon and current APIs. Older macOS needs the
  same legacy-stack decision as Windows 7.
- **Linux:** X11 and Wayland (GNOME, KDE, Sway, weston) are the supported desktop
  surface.

If the probe fails on a self-hosted legacy machine, upload the artifact and use the JSON
offset to decide whether a compatibility fix is possible before changing the minimum.

## Hit-testing checklist

1. Click the center of a grid button in each row.
2. Confirm the recorded id is the same cell and the pointer inside its rect.
3. Repeat at 100%, 125%, 150%, and 200% scaling.
4. Maximize and restore, then repeat on the title bar buttons and the grid.
5. On macOS, repeat with the window on an external non-Retina display.
6. On Wayland, repeat under GNOME/KDE/Sway and compare against X11.
