pub(crate) fn run(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let command = args.get(1).map(String::as_str).unwrap_or("serve");
    match command {
        "calibrate" => crate::calibrate_command(&args[2..])?,
        "bressloff-geometry" | "figure-stills" => crate::bressloff_geometry_command(&args[2..])?,
        "export" => crate::export_command(&args[2..])?,
        "rule-report" | "rule-calibrate" => crate::rule_report_command(&args[2..])?,
        "rule-sweep" => crate::rule_sweep_command(&args[2..])?,
        "rule-floquet" => crate::rule_floquet_command(&args[2..])?,
        "rule-fit" | "rule-figure8-fit" => crate::rule_fit_command(&args[2..])?,
        "driven-registry" => crate::driven_registry_command(&args[2..])?,
        "mackay-report" => crate::mackay_report_command(&args[2..])?,
        "serve" => crate::server::serve_command(&args[2..])?,
        "--help" | "-h" => print_usage(),
        other => {
            eprintln!("unknown command: {other}");
            print_usage();
        }
    }
    Ok(())
}

fn print_usage() {
    println!(
        "usage:\n  bressloff-v1 serve [--host 127.0.0.1] [--port 8892] [--root .]\n  bressloff-v1 export [--out viewer/frames.json] [--paper-preset fig31_square_even] [--rule-preset rule_fig4_high_freq_stripes] [--export-orientations] [model params]\n  bressloff-v1 calibrate [--out reports/paper-calibration.json] [model params]\n  bressloff-v1 bressloff-geometry [--out reports/figure-targets/bressloff-generated-stills.json] [--preset-set figures29-36|all] [model params]\n  bressloff-v1 rule-report [--out reports/rule-2011-regimes.json] [model params]\n  bressloff-v1 rule-sweep [--out reports/rule-2011-sweep.json] [--preset-grid quick|paper|dense] [--periods 140,120,85,65,55] [--period-min 40 --period-max 160 --period-steps 13] [--amplitudes 0.65,0.8,1.0] [model params]\n  bressloff-v1 rule-floquet [--out reports/rule-2011-floquet.json] [--preset-grid quick|paper|dense] [--modes 0.5,0.75,...,4.0] [--source-beta-modes 0.2,0.4,...,1.0] [--figure8-beta-scale 0.4286845] [model params]\n  bressloff-v1 rule-fit [--out reports/rule-2011-fit-search.json] [--max-trials 25] [rule-floquet options] [model params]\n  bressloff-v1 driven-registry [--out reports/driven-neural-fields-registry.json]\n  bressloff-v1 mackay-report [--out reports/mackay-localized-input.json] [--n 128] [--iterations 60]"
    );
}
