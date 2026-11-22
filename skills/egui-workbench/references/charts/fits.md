# Fits and smoothing

> Least-squares models, equations, R², moving averages, and selecting points inside a plot area. Live page: <https://jigonzalez930209.github.io/egui-workbench/charts/fits.html>

`Fit` performs a least-squares fit over `[[f64; 2]]` data and can produce a ready-to-draw
series with its equation and R².

```rust
use egui_workbench::components::plot::{Fit, FitKind, Plot, Series};

let fit = Fit::new(FitKind::Linear, &data);
if let Some(fit) = fit {
    self.status = format!("{} (R² = {:.4})", fit.equation(), fit.r_squared);
    let plot = Plot::new("fit")
        .series(Series::scatter("samples", &data))
        .series(fit.series("least squares", 200));
}
```

The `Plot::fit` shorthand adds the dashed curve for you:

```rust
Plot::new("fit")
    .series(Series::scatter("samples", &data))
    .fit(FitKind::Polynomial(3), &data);
```

Nothing is added when the fit fails (too few points, singular system).

## Models

| `FitKind` | Equation | Notes |
|---|---|---|
| `Linear` | `y = a + b·x` | Degree-1 polynomial |
| `Polynomial(n)` | `y = c0 + c1·x + … + cn·xⁿ` | Degree clamped to 1–8 |
| `Exponential` | `y = a·e^(b·x)` | Fitted on `ln y`; points with `y <= 0` skipped |
| `Power` | `y = a·x^b` | Fitted on `ln x`, `ln y`; points with `x <= 0` or `y <= 0` skipped |
| `Logarithmic` | `y = a + b·ln x` | Points with `x <= 0` skipped |
| `MovingAverage(n)` | Centered moving average over `n` points | Smoothing, not a model; no R² |

## Fit struct

```rust
pub struct Fit {
    pub kind: FitKind,
    pub coefficients: Vec<f64>,
    pub r_squared: f64,
    pub x_range: (f64, f64),
    // private smoothed points
}
```

| Method | Returns | Use |
|---|---|---|
| `Fit::new(kind, data)` | `Option<Fit>` | Fit the usable finite points |
| `.eval(x)` | `f64` | Model value; `NaN` outside the domain or outside the sampled range for a moving average |
| `.equation()` | `String` | `y = 2.01x + 0.498   R² = 0.998` or `moving average (5)` |
| `.series(name, samples)` | `Series<'static>` | Dashed 1.5 pt curve over `x_range`; the smoothed points for a moving average |

Coefficients are `[a, b]` for two-parameter models and `c0…cn` for polynomials.

## R²

`r_squared` is computed on the original finite points, not on the logged or smoothed
values. A value of 1 is a perfect fit; `NaN` means it is undefined (too few points or a
constant target). For a moving average it stays `NaN`.

The equation omits zero polynomial terms (except when all terms are zero) and uses Latin-1
superscripts `²` and `³`, so the default fonts can draw them.

## Selections

`points_in(data, bounds)` returns the indices of the points inside a `Bounds`:

```rust
use egui_workbench::components::plot::points_in;

let response = plot.show(ui, &mut icons, &mut state);
if response.selection_changed {
    if let Some(bounds) = state.selection {
        let selected = points_in(&data, &bounds);
        if selected.len() >= 2 {
            let subset: Vec<[f64; 2]> = selected.iter().map(|index| data[*index]).collect();
            if let Some(fit) = Fit::new(FitKind::Linear, &subset) {
                self.fit_label = fit.equation();
            }
        }
    }
}
```

`Bounds` stores `min_x`, `max_x`, `min_y`, and `max_y` in data coordinates;
`Bounds::from_corners(a, b)` builds one from two dragged corners, and `contains(point)`
tests a point.

## Fitting a selection

A useful pattern: fit the whole data set, then let the user select an area and refit the
selected points. Keep both the full fit and the selection fit as series; name them with
their equations, and call `.hide_from_legend()` on the raw samples if the legend gets
crowded.

## Caveats

- Polynomial degrees above 4 can oscillate on sparse or noisy data; the maximum is 8.
- Exponential and power fits need positive targets; the model silently skips invalid
  points, but R² is then computed on all finite points, including skipped ones.
- A moving average is centered: the first and last `n/2` points have no output.
- `Fit::new` returns `None` when the usable point count is not greater than the polynomial
  degree.
