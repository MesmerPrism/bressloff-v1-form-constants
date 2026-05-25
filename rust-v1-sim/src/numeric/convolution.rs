use rayon::prelude::*;

use crate::PI;

pub(crate) fn gaussian_weights(n: usize, dx: f64, sigma: f64) -> Vec<f64> {
    let radius = n.saturating_sub(1);
    let norm = 1.0 / ((2.0 * PI).sqrt() * sigma);
    (0..=(2 * radius))
        .map(|index| {
            let offset = index as isize - radius as isize;
            let x = offset as f64 * dx;
            norm * (-0.5 * (x / sigma).powi(2)).exp()
        })
        .collect()
}

pub(crate) fn gaussian_convolve_2d_into(
    field: &[f64],
    n: usize,
    weights: &[f64],
    dx: f64,
    scratch: &mut [f64],
    out: &mut [f64],
) {
    let radius = n.saturating_sub(1);
    scratch
        .par_chunks_mut(n)
        .enumerate()
        .for_each(|(row, chunk)| {
            for (col, target) in chunk.iter_mut().enumerate().take(n) {
                let mut sum = 0.0;
                for source_col in 0..n {
                    let offset = col as isize - source_col as isize;
                    let weight_index = (offset + radius as isize) as usize;
                    sum += weights[weight_index] * field[row * n + source_col] * dx;
                }
                *target = sum;
            }
        });

    out.par_chunks_mut(n).enumerate().for_each(|(row, chunk)| {
        for (col, target) in chunk.iter_mut().enumerate().take(n) {
            let mut sum = 0.0;
            for source_row in 0..n {
                let offset = row as isize - source_row as isize;
                let weight_index = (offset + radius as isize) as usize;
                sum += weights[weight_index] * scratch[source_row * n + col] * dx;
            }
            *target = sum;
        }
    });
}
