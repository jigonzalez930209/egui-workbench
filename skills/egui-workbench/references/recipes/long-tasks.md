# Long-running tasks

> Run work off the UI thread and report progress with loading, success, and error toasts. Live page: <https://jigonzalez930209.github.io/egui-workbench/recipes/long-tasks.html>

## Pattern

1. Start the work on a worker thread.
2. Return a `ToastId` from `toasts.loading(...)`.
3. Poll the worker in `logic`; when a result arrives, `update` the toast in place.
4. Wake the UI with `request_repaint` when the worker finishes.

```rust
use std::sync::mpsc::{Receiver, TryRecvError};

enum Job {
    Export { receiver: Receiver<Result<std::path::PathBuf, String>> },
}

struct App {
    job: Option<Job>,
    toasts: Toasts,
    icons: Icons,
}
```

## Start

```rust
fn start_export(&mut self, ctx: &egui::Context, path: PathBuf, data: Vec<[f64; 2]>) {
    let (sender, receiver) = std::sync::mpsc::channel();
    let ctx = ctx.clone();
    std::thread::spawn(move || {
        let result = render_and_write(&path, &data);
        let _ = sender.send(result);
        ctx.request_repaint();
    });
    self.toasts.loading("Exporting chart…");
    self.job = Some(Job::Export { receiver });
}
```

`Toasts` stays on the UI thread; only plain data crosses the channel. `ToastId` is `Copy`
but the `Toasts` stack itself is not `Send`.

## Poll

```rust
fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    if let Some(Job::Export { receiver }) = &self.job {
        match receiver.try_recv() {
            Ok(result) => {
                let id = /* the id returned by loading */;
                match result {
                    Ok(path) => {
                        self.toasts.update(
                            id,
                            Toast::new(ToastKind::Success, "Exported")
                                .description(path.display().to_string()),
                        );
                    }
                    Err(error) => {
                        self.toasts.update(
                            id,
                            Toast::new(ToastKind::Error, "Export failed")
                                .description(error)
                                .action("Retry"),
                        );
                    }
                }
                self.job = None;
                ctx.request_repaint();
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                self.toasts.update(id, Toast::new(ToastKind::Error, "Worker stopped"));
                self.job = None;
            }
        }
    }
}
```

Store the `ToastId` next to the job; the template keeps it in the tab model.

## Progress

For a determinate bar, send progress messages:

```rust
enum Progress {
    Fraction(f32),
    Done(Result<(), String>),
}
```

Update a `ProgressDialog` or a toast description from `Fraction`; do not open a modal for
every step. A modal blocks the rest of the UI, which is correct for a short operation and
annoying for a long one.

## Cancellation

```rust
struct Job {
    receiver: Receiver<Progress>,
    cancel: std::sync::Arc<std::sync::atomic::AtomicBool>,
}
```

The worker checks `cancel.load(Ordering::Relaxed)` between chunks. On cancel, drop the
receiver; the worker's next `send` fails and it exits. The loading toast should be
dismissed or updated to "Cancelled".

## Multiple jobs

- Keep a small `Vec<Job>` or a map keyed by `ToastId`.
- `max_visible` queues extra toasts; a batch of 20 exports should show one summary toast,
  not 20.
- Resolve each toast exactly once; `update` returns `false` if it is already gone.
- Do not hold a `&mut Toasts` across the worker boundary.

## Errors

- Put the actionable reason in `.description(...)`.
- Use `.action("Retry")` for transient failures; handle `ToastEvent::Action(id)` in the
  `show` loop.
- Keep the error in the status bar too if the toast may disappear.
- Log the full error to stderr or your logger; the toast is for the user.

## Checklist

- [ ] Work off the UI thread; no blocking I/O in `ui`.
- [ ] One `loading` toast per job, updated in place.
- [ ] `request_repaint` from the worker or on the next poll.
- [ ] Cancellation actually stops the worker.
- [ ] Errors are actionable and do not get lost when the toast expires.
