extern crate cbindgen;

use std::env;
use std::fmt::Write;
use std::fs::File;
use std::io::Read;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut config = cbindgen::Config::default();
    config.language = cbindgen::Language::C;
    if let Ok(bindings) = cbindgen::generate_with_config(&crate_dir, config) {
        bindings.write_to_file("target/lather.h");
    }

    let ccf_file = std::fs::read_to_string(
        "resources/CCF_solar_spectrum_G2_FTS_reso_not_evenly_sampled_in_freq.rdb",
    ).unwrap();

    let mut rv = Vec::new();
    let mut ccf_quiet = Vec::new();
    let mut ccf_spot = Vec::new();

    for line in ccf_file.lines().skip(2) {
        let mut fields = line.split_whitespace().map(|s| s.parse::<f64>().unwrap());
        rv.push(fields.next().unwrap() * 1e3);
        ccf_quiet.push(fields.next().unwrap());
        ccf_spot.push(fields.next().unwrap());
    }

    // Write to a string then compare it to the previous contents of the file,
    // This prevents writing a new file every compilation which would cause cargo to recompile the
    // whole project
    let mut output = String::new();
    for &(ref name, ref array) in &[("RV", rv), ("CCF_QUIET", ccf_quiet), ("CCF_SPOT", ccf_spot)] {
        let _ = write!(output, "pub static {}: [f64; {}] = [", name, array.len());
        for val in array {
            let _ = write!(output, "{:.2e}, ", val);
        }
        // Remove the last trailing comma and space
        output.pop();
        output.pop();
        let _ = writeln!(output, "];");
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
