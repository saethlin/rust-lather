extern crate cbindgen;

use std::env;
use std::fmt::Write;
use std::fs::File;
use std::io::Read;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut config: cbindgen::Config = Default::default();
    config.language = cbindgen::Language::C;
    if let Ok(bindings) = cbindgen::generate_with_config(&crate_dir, config) {
        bindings.write_to_file("target/lather.h");
    }

    let mut ccf_file = String::new();
    File::open("resources/CCF_solar_spectrum_G2_FTS_reso_not_evenly_sampled_in_freq.rdb")
        .unwrap()
        .read_to_string(&mut ccf_file)
        .unwrap();

    let mut rv = Vec::new();
    let mut ccf_quiet = Vec::new();
    let mut ccf_spot = Vec::new();

    for line in ccf_file.lines().skip(2) {
        let fields: Vec<f64> = line
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        rv.push(fields[0] * 1e3);
        ccf_quiet.push(fields[1]);
        ccf_spot.push(fields[2]);
    }

    let mut output = String::new();
    for &(ref name, ref array) in &[("rv", rv), ("ccf_quiet", ccf_quiet), ("ccf_spot", ccf_spot)] {
        writeln!(output, "macro_rules! {} {{", name).unwrap();
        write!(output, "    () => ({{ vec![").unwrap();
        for val in array {
            write!(output, "{:.2e}, ", val).unwrap();
        }
        writeln!(output, "] }})\n}}").unwrap();
    }

    let mut old_output = String::new();
    if let Ok(mut f) = File::open("src/solar_ccfs.rs") {
        f.read_to_string(&mut old_output).unwrap();
    }

    if old_output != output {
        use std::io::Write;
        let mut output_file = File::create("src/solar_ccfs.rs").unwrap();
        write!(output_file, "{}", output).unwrap();
    }
}
