#![allow(clippy::needless_range_loop)]

//! Nonlinear least squares and the small linear-algebra helpers.

/// A nonlinear model `f(x, params)` for [`fit_model`].
pub type Model<'a> = dyn Fn(f64, &[f64]) -> f64 + 'a;

/// A fitted parameter set with standard errors.
#[derive(Clone, Debug, PartialEq)]
pub struct ModelFit {
    /// Fitted parameters.
    pub params: Vec<f64>,
    /// Standard errors, when the covariance could be estimated.
    pub errors: Option<Vec<f64>>,
    /// Root-mean-square residual.
    pub rmse: f64,
    /// Number of iterations used.
    pub iterations: usize,
}

/// Gauss–Newton fit with numerical Jacobians and Levenberg damping.
pub fn fit_model(x: &[f64], y: &[f64], initial: &[f64], model: &Model<'_>) -> Option<ModelFit> {
    if x.len() != y.len() || x.len() < initial.len() || initial.is_empty() {
        return None;
    }
    let mut params = initial.to_vec();
    let mut damping = 1e-3;
    let mut iterations = 0;
    let mut best = residual_sum(x, y, &params, model);
    for _ in 0..100 {
        iterations += 1;
        let mut jtj = vec![vec![0.0; params.len()]; params.len()];
        let mut jtr = vec![0.0; params.len()];
        for (sample, (x, y)) in x.iter().zip(y).enumerate() {
            let predicted = model(*x, &params);
            let residual = y - predicted;
            for column in 0..params.len() {
                let step = 1e-6 * params[column].abs().max(1.0);
                let mut bumped = params.clone();
                bumped[column] += step;
                let derivative = (model(*x, &bumped) - predicted) / step;
                jtr[column] += derivative * residual;
                for row in 0..params.len() {
                    jtj[row][column] += derivative * derivative;
                }
            }
            let _ = sample;
        }
        for (index, row) in jtj.iter_mut().enumerate() {
            row[index] += damping;
        }
        let Some(delta) = solve(&jtj, &jtr) else {
            damping *= 10.0;
            continue;
        };
        let candidate: Vec<f64> = params.iter().zip(&delta).map(|(param, delta)| param + delta).collect();
        let residual = residual_sum(x, y, &candidate, model);
        if residual < best {
            params = candidate;
            best = residual;
            damping = (damping * 0.3).max(1e-9);
        } else {
            damping *= 10.0;
        }
        if delta.iter().all(|value| value.abs() < 1e-8) {
            break;
        }
    }
    let rmse = (best / x.len() as f64).sqrt();
    let errors = covariance(x, y, &params, model, rmse).map(|matrix| {
        (0..params.len())
            .map(|index| matrix[index][index].max(0.0).sqrt())
            .collect()
    });
    Some(ModelFit {
        params,
        errors,
        rmse,
        iterations,
    })
}

fn residual_sum(x: &[f64], y: &[f64], params: &[f64], model: &Model<'_>) -> f64 {
    x.iter()
        .zip(y)
        .map(|(x, y)| {
            let residual = y - model(*x, params);
            residual * residual
        })
        .sum()
}

fn covariance(x: &[f64], _y: &[f64], params: &[f64], model: &Model<'_>, rmse: f64) -> Option<Vec<Vec<f64>>> {
    let size = params.len();
    let mut jtj = vec![vec![0.0; size]; size];
    for value in x {
        let predicted = model(*value, params);
        for column in 0..size {
            let step = 1e-6 * params[column].abs().max(1.0);
            let mut bumped = params.to_vec();
            bumped[column] += step;
            let derivative = (model(*value, &bumped) - predicted) / step;
            for row in 0..size {
                jtj[row][column] += derivative * derivative;
            }
        }
    }
    for (index, row) in jtj.iter_mut().enumerate() {
        row[index] += 1e-12;
    }
    let mut inverse = vec![vec![0.0; size]; size];
    for column in 0..size {
        let mut unit = vec![0.0; size];
        unit[column] = 1.0;
        let solution = solve(&jtj, &unit)?;
        for row in 0..size {
            inverse[row][column] = solution[row];
        }
    }
    let variance = rmse * rmse;
    Some(
        inverse
            .iter()
            .map(|row| row.iter().map(|value| value * variance).collect())
            .collect(),
    )
}

fn solve(matrix: &[Vec<f64>], rhs: &[f64]) -> Option<Vec<f64>> {
    let size = rhs.len();
    let mut a = matrix.to_vec();
    let mut b = rhs.to_vec();
    for pivot in 0..size {
        let mut best = pivot;
        for row in pivot + 1..size {
            if a[row][pivot].abs() > a[best][pivot].abs() {
                best = row;
            }
        }
        if a[best][pivot].abs() < 1e-12 {
            return None;
        }
        a.swap(pivot, best);
        b.swap(pivot, best);
        let divisor = a[pivot][pivot];
        for column in pivot..size {
            a[pivot][column] /= divisor;
        }
        b[pivot] /= divisor;
        for row in 0..size {
            if row != pivot {
                let factor = a[row][pivot];
                for column in pivot..size {
                    a[row][column] -= factor * a[pivot][column];
                }
                b[row] -= factor * b[pivot];
            }
        }
    }
    Some(b)
}
