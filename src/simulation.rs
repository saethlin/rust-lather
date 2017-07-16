extern crate std;
extern crate ini;
extern crate rand;
use std::iter;
use self::ini::Ini;
use self::rand::distributions::{Range, LogNormal, IndependentSample};
use std::rc::Rc;
use planck::planck_integral;
use fit_rv::fit_rv;
use compute_bisector::compute_bisector;

use star::Star;
use spot::Spot;

pub struct Observation {
    pub rv: f64,
    pub bisector: Vec<f64>,
}

pub struct Simulation {
    star: Rc<Star>,
    spots: Vec<Spot>,
    dynamic_fill_factor: f64,
    generator: rand::ThreadRng,
}

fn normalize(vector: &mut Vec<f64>) {
    let max = vector.iter().cloned().fold(std::f64::NAN, f64::max);
    for ref mut item in vector {
        **item /= max;
    }
}

impl Simulation {
    pub fn new(filename: &str) -> Simulation {
        let file = Ini::load_from_file(filename).unwrap();

        let star = file.section(Some("star")).unwrap();
        let radius = star.get("radius").unwrap().parse::<f64>().unwrap();
        let period = star.get("period").unwrap().parse::<f64>().unwrap();
        let inclination = star.get("inclination").unwrap().parse::<f64>().unwrap();
        let temperature = star.get("Tstar").unwrap().parse::<f64>().unwrap();
        let spot_temp_diff = star.get("Tdiff_spot").unwrap().parse::<f64>().unwrap();
        let limb_linear = star.get("limb1").unwrap().parse::<f64>().unwrap();
        let limb_quadratic = star.get("limb2").unwrap().parse::<f64>().unwrap();
        let dynamic_fill_factor = star.get("fillfactor").unwrap().parse::<f64>().unwrap();
        let grid_size = star.get("gridsize").unwrap().parse::<usize>().unwrap();

        let mut this = Simulation {
            star: Rc::new(Star::new(radius, period, inclination, temperature, spot_temp_diff, limb_linear, limb_quadratic, grid_size)),
            spots: Vec::new(),
            dynamic_fill_factor: dynamic_fill_factor,
            generator: rand::thread_rng(),
        };

        for spot in file
            .iter()
            .filter(|&(s, _)| s.to_owned().is_some())
            .filter(|&(s, _)| s.to_owned().unwrap().as_str().starts_with("spot"))
            .map(|(_, p)| p) {

            let latitude = spot.get("latitude").unwrap().parse::<f64>().unwrap();
            let longitude = spot.get("longitude").unwrap().parse::<f64>().unwrap();
            let size = spot.get("size").unwrap().parse::<f64>().unwrap();
            let plage = spot.get("plage").unwrap().parse::<bool>().unwrap();

            this.spots.push(Spot::new(this.star.clone(), latitude, longitude, size, plage, false));
        }

        this
    }

    fn check_fill_factor(&mut self, time: f64) {
        let mut current_fill_factor = self.spots.iter()
            .filter(|s| s.alive(time))
            .map(|s| (s.radius*s.radius)/2.0)
            .sum::<f64>();

        let fill_range = LogNormal::new(0.5, 4.0);
        let lat_range = Range::new(-30.0, 30.0);
        let long_range = Range::new(0.0, 360.0);

        while current_fill_factor < self.dynamic_fill_factor {

            let new_fill_factor = iter::repeat(())
                .map(|_| fill_range.ind_sample(&mut self.generator)*9.4e-6)
                .find(|v| *v < 0.001).unwrap();

            let mut new_spot = Spot::new(
                self.star.clone(),
                lat_range.ind_sample(&mut self.generator),
                long_range.ind_sample(&mut self.generator),
                new_fill_factor,
                false,
                true,
            );
            new_spot.time_appear += time;
            new_spot.time_disappear += time;

            let collides = self.spots.iter()
                .filter(|s| s.alive(new_spot.time_appear) || s.alive(new_spot.time_disappear))
                .any(|s| new_spot.collides_with(s));

            if !collides {
                current_fill_factor += (new_spot.radius*new_spot.radius)/2.0;
                self.spots.push(new_spot);
            }
        }
    }

    pub fn observe_flux(&mut self, time: Vec<f64>, wavelength_min: f64, wavelength_max: f64) -> Vec<f64> {
        let star_intensity = planck_integral(self.star.temperature, wavelength_min, wavelength_max);
        for spot in &mut self.spots {
            spot.intensity = planck_integral(spot.temperature, wavelength_min, wavelength_max) / star_intensity;
        }

        let mut flux: Vec<f64> = time.iter().map(|t| {
            self.check_fill_factor(*t);
            let spot_flux: f64 = self.spots.iter().map(|s| s.get_flux(*t)).sum();
            (self.star.flux_quiet - spot_flux) / self.star.flux_quiet
        }).collect();

        normalize(&mut flux);
        flux
    }

    pub fn observe_rv(&mut self, time: Vec<f64>, wavelength_min: f64, wavelength_max: f64) -> Vec<Observation> {
        let mut output = Vec::with_capacity(time.len());

        let star_intensity = planck_integral(self.star.temperature, wavelength_min, wavelength_max);
        for spot in &mut self.spots {
            spot.intensity = planck_integral(spot.temperature, wavelength_min, wavelength_max) / star_intensity;
        }

        for t in &time {
            self.check_fill_factor(*t);
        }

        let fit_guess = self.star.fit_result.clone();

        for t in time.iter() {
            let mut spot_profile = vec![0.0; self.star.profile_active.len()];
            for spot in self.spots.iter().filter(|s| s.alive(*t)) {
                let profile = spot.get_ccf(*t);
                for (total, this) in spot_profile.iter_mut().zip(profile.iter()) {
                    *total += *this;
                }
            }

            for (spot, star) in spot_profile.iter_mut().zip(self.star.integrated_ccf.iter()) {
                *spot = *star - *spot;
            }
            normalize(&mut spot_profile);
            let fit_result = fit_rv(&self.star.profile_quiet.rv, &spot_profile, &fit_guess);
            let rv = (fit_result.centroid - self.star.zero_rv)*1000.0; // TODO km/s

            let bisector: Vec<f64> = compute_bisector(&self.star.profile_quiet.rv, &spot_profile).iter()
                .map(|b| (b - self.star.zero_rv)*1000.0).collect(); // TODO: km/s

            output.push(Observation{rv: rv, bisector: bisector})
        }
        output
    }
}
