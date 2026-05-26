use std::path::{Path, PathBuf};

use super::boundary::{rule_floquet_beta_boundary_curves, rule_floquet_boundary_candidates};
use super::dynamics::rule_floquet_grid_point_for;
use super::source::apply_rule_figure8_source_comparison;
use crate::models::rule::sweep::rule_sweep_params;
use crate::*;

pub(crate) fn default_rule_figure8_source_curve_file(out: &Path) -> PathBuf {
    out.parent()
        .unwrap_or_else(|| Path::new("."))
        .join("source-curves")
        .join("rule-2011-fig8-source-curves.json")
}

pub(crate) fn rule_figure8_wave_number_normalization(
    source_beta_per_model_cycle: f64,
    model_domain_points: usize,
) -> RuleFigure8WaveNumberNormalization {
    let scale = source_beta_per_model_cycle.max(1.0e-12);
    RuleFigure8WaveNumberNormalization {
        model: "zero_offset_domain_scale",
        decision: "source_beta = source_beta_per_model_cycle * model_beta_cycles",
        internal_wave_number_formula: "q_cell = 2*pi*model_beta_cycles/model_domain_points",
        source_beta_per_model_cycle: scale,
        model_cycles_per_source_beta: 1.0 / scale,
        source_beta_offset: 0.0,
        model_domain_points,
        note: "The affine beta fit remains diagnostic only; the calibrated source-axis comparison uses a zero-offset domain scale because beta=0 must map to zero wave number.",
    }
}

pub(crate) fn rule_source_beta_for_model_cycles(
    beta_cycles: f64,
    normalization: RuleFigure8WaveNumberNormalization,
) -> f64 {
    normalization.source_beta_per_model_cycle * beta_cycles + normalization.source_beta_offset
}

pub(crate) fn rule_model_cycles_for_source_beta(
    source_beta: f64,
    normalization: RuleFigure8WaveNumberNormalization,
) -> f64 {
    (source_beta - normalization.source_beta_offset) * normalization.model_cycles_per_source_beta
}

pub(crate) fn rule_source_betas_for_modes(
    mode_cycles: &[f64],
    normalization: RuleFigure8WaveNumberNormalization,
) -> Vec<f64> {
    mode_cycles
        .iter()
        .map(|cycles| rule_source_beta_for_model_cycles(*cycles, normalization))
        .collect()
}

pub(crate) fn source_beta_values_to_model_cycles(
    values: Vec<f64>,
    normalization: RuleFigure8WaveNumberNormalization,
) -> Vec<f64> {
    values
        .into_iter()
        .map(|source_beta| rule_model_cycles_for_source_beta(source_beta, normalization))
        .collect()
}

pub(crate) fn rule_floquet_evaluation(
    config: RuleFloquetEvaluationConfig<'_>,
) -> RuleFloquetEvaluation {
    let mut points = Vec::new();
    for period in &config.grid.periods {
        for amplitude in &config.grid.amplitudes {
            for stim_i_fraction in &config.grid.stim_i_fractions {
                let params = rule_sweep_params(
                    config.raw,
                    config.grid,
                    *period,
                    *amplitude,
                    *stim_i_fraction,
                );
                points.push(rule_floquet_grid_point_for(params, config.mode_cycles));
            }
        }
    }

    let boundary_candidates = rule_floquet_boundary_candidates(
        &points,
        &config.grid.periods,
        &config.grid.amplitudes,
        &config.grid.stim_i_fractions,
    );
    let mut boundary_curves = rule_floquet_beta_boundary_curves(
        &points,
        config.raw,
        config.grid,
        config.curve_refinement.tolerance,
        config.curve_refinement.max_steps,
    );
    let source_curve_comparison = apply_rule_figure8_source_comparison(
        &mut boundary_curves,
        config.wave_number_normalization,
        config.source_curves,
        config.source_curve_file,
    );

    RuleFloquetEvaluation {
        points,
        boundary_candidates,
        boundary_curves,
        source_curve_comparison,
    }
}

pub(crate) fn rule_floquet_mode_defaults() -> Vec<f64> {
    linspace_values(0.5, 4.0, 15)
}
