extern crate lather;
extern crate png;
extern crate rayon;
extern crate structopt;

use lather::{linspace, Simulation};
use png::HasParameters;
use rayon::prelude::*;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "Lather visualizer")]
struct Opt {
    /// Path to a simulation config file to run
    #[structopt(short = "c", long = "config", parse(from_os_str))]
    config_path: Option<PathBuf>,

    /// Frames per second for the output mp4
    #[structopt(short = "fps", default_value = "60")]
    fps: u64,

    /// Duration (days) for the simulation to run (defaults to the star's rotational period)
    #[structopt(short = "d", long = "duration")]
    duration: Option<f64>,

    /// Frames in the simulation run
    #[structopt(short = "frames", default_value = "1000")]
    frames: u64,
}

fn main() {
    let opt = Opt::from_args();

    let mut sim = if let Some(config_path) = opt.config_path {
        Simulation::from_config(config_path.to_str().unwrap()).unwrap()
    } else {
        Simulation::sun()
    };

    let times = linspace(
        0.0,
        opt.duration.unwrap_or(sim.star.period),
        opt.frames as usize,
    )
    .enumerate()
    .collect::<Vec<(usize, f64)>>();

    for (_, time) in &times {
        sim.check_fill_factor(*time);
    }

    // I'm sorry this is gory, but that's what must be done to make this run in parallel
    times.par_iter().for_each(|(t, time)| {
        let image = sim.draw_rgba(*time);
        save_png(&image, &format!("{:04}.png", t));
    });

    // Find a filename that isn't taken
    let mut i = 0;
    let mut output_path = std::path::PathBuf::from(format!("lather{}.mp4", i));
    while output_path.exists() {
        i += 1;
        output_path = std::path::PathBuf::from(format!("lather{}.mp4", i));
    }

    // Launch ffmpeg
    let mut command = std::process::Command::new("ffmpeg");
    let fps_string = opt.fps.to_string();
    let args = [
        "-r",
        &fps_string,
        "-f",
        "image2",
        "-s",
        "1920x1080",
        "-i",
        "%04d.png",
        "-vcodec",
        "libx264",
        "-crf",
        "25",
        "-pix_fmt",
        "yuv420p",
        &output_path.to_str().unwrap(),
    ];
    command.args(&args);
    let mut handle = command.spawn().unwrap();

    // Try to be helpful if ffmpeg barfs
    let print_err = || {
        println!("Tried to run ffmpeg to make an mp4 video from the PNGs produced, but something went wrong");
        println!(
            "Here's the ffmpeg command I tried to run:\nffmpeg {}",
            args.join(" ")
        );
    };

    match handle.wait() {
        Err(_) => print_err(),
        Ok(r) => {
            if !r.success() {
                print_err();
            }
        }
    }
}

fn save_png(image: &[u8], filename: &str) {
    let file = ::std::fs::File::create(filename).unwrap();
    let w = &mut ::std::io::BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, 1000, 1000);
    encoder.set(png::ColorType::RGBA).set(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();

    writer.write_image_data(image).unwrap();
}
