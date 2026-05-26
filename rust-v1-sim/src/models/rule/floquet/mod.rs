mod boundary;
mod command;
mod dynamics;
mod normalization;
mod source;

pub(crate) use command::rule_floquet_command;
pub(crate) use dynamics::rule_floquet_report;
pub(crate) use normalization::{
    default_rule_figure8_source_curve_file, rule_figure8_wave_number_normalization,
    rule_floquet_evaluation, rule_floquet_mode_defaults, rule_source_betas_for_modes,
    source_beta_values_to_model_cycles,
};
pub(crate) use source::load_rule_figure8_source_curves;

#[cfg(test)]
pub(crate) use boundary::{
    empty_rule_floquet_curve_fit, refine_scalar_sign_change, rule_floquet_boundary_candidates,
    rule_floquet_branch_label, rule_floquet_branch_periodicity,
};
#[cfg(test)]
pub(crate) use dynamics::{
    floquet_mode_from_matrix, floquet_mode_margin, rule_floquet_grid_point_for,
    rule_wave_number_for_cycles,
};
#[cfg(test)]
pub(crate) use source::apply_rule_figure8_source_comparison;
