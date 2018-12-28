use std;
use std::f64::consts;

use bounds::Bounds;
use fit_rv::fit_rv;
use linspace::linspace;
use profile::Profile;

use rand::distributions::{LogNormal, StandardNormal, Uniform};
use rand::Rng;

static SOLAR_RADIUS: f64 = 6.96e8;
static DAYS_TO_SECONDS: f64 = 86400.0;

#[derive(Debug)]
pub enum Distribution {
    StandardNormal(StandardNormal),
    LogNormal(LogNormal),
    Uniform(Uniform<f64>),
}

impl Distribution {
    pub fn sample(&self, rng: &mut rand::rngs::StdRng) -> f64 {
        match self {
            Distribution::StandardNormal(ref inner) => rng.sample(inner),
            Distribution::LogNormal(ref inner) => rng.sample(inner),
            Distribution::Uniform(ref inner) => rng.sample(inner),
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "name")]
pub enum DistributionConfig {
    StandardNormal,
    LogNormal { mean: f64, std_dev: f64 },
    Uniform { min: f64, max: f64 },
}

impl From<DistributionConfig> for Distribution {
    fn from(c: DistributionConfig) -> Distribution {
        match c {
            DistributionConfig::StandardNormal => Distribution::StandardNormal(StandardNormal),
            DistributionConfig::LogNormal { mean, std_dev } => {
                Distribution::LogNormal(LogNormal::new(mean, std_dev))
            }
            DistributionConfig::Uniform { min, max } => {
                Distribution::Uniform(Uniform::new(min, max))
            }
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct StarConfig {
    pub grid_size: usize,
    pub radius: f64,
    pub period: f64,
    pub inclination: f64,
    pub temperature: f64,
    pub spot_temp_diff: f64,
    pub limb_linear: f64,
    pub limb_quadratic: f64,
    pub minimum_fill_factor: f64,
    pub latitude_distribution: Option<DistributionConfig>,
    pub longitude_distribution: Option<DistributionConfig>,
    pub fillfactor_distribution: Option<DistributionConfig>,
    pub lifetime_distribution: Option<DistributionConfig>,
}

/// A star that can host spots
pub struct Star {
    pub period: f64,
    pub inclination: f64,
    pub temperature: f64,
    pub spot_temp_diff: f64,
    pub limb_linear: f64,
    pub limb_quadratic: f64,
    pub grid_size: usize,
    pub flux_quiet: f64,
    pub zero_rv: f64,
    pub equatorial_velocity: f64,
    pub minimum_fill_factor: f64,
    pub integrated_ccf: Vec<f64>,
    pub profile_spot: Profile,
    pub profile_quiet: Profile,
    pub latitude_distribution: Distribution,
    pub longitude_distribution: Distribution,
    pub fillfactor_distribution: Distribution,
    pub lifetime_distribution: Distribution,
}

impl std::fmt::Debug for Star {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Star")
            .field("period", &self.period)
            .field("inclination", &self.inclination)
            .field("temperature", &self.temperature)
            .field("spot_temp_diff", &self.spot_temp_diff)
            .field("limb_linear", &self.limb_linear)
            .field("limb_quadratic", &self.limb_quadratic)
            .field("grid_size", &self.grid_size)
            .field("flux_quiet", &self.flux_quiet)
            .field("zero_rv", &self.zero_rv)
            .field("equatorial_velocity", &self.equatorial_velocity)
            .field("minimum_fill_factor", &self.minimum_fill_factor)
            .field("latitude_distribution", &self.latitude_distribution)
            .field("longitude_distribution", &self.longitude_distribution)
            .field("fillfactor_distribution", &self.fillfactor_distribution)
            .field("lifetime_distribution", &self.lifetime_distribution)
            .finish()
    }
}

impl Star {
    pub fn from_config(config: &StarConfig) -> Star {
        let sqrt = f64::sqrt;

        let edge_velocity =
            (2.0 * consts::PI * config.radius * SOLAR_RADIUS) / (config.period * DAYS_TO_SECONDS);
        let equatorial_velocity = edge_velocity * (config.inclination * consts::PI / 180.0).sin();

        let profile_quiet =
            Profile::new(::solar_ccfs::RV.to_vec(), ::solar_ccfs::CCF_QUIET.to_vec());
        let mut integrated_ccf = vec![0.0; profile_quiet.len()];
        let mut flux_quiet = 0.0;

        let mut ccf_shifted = vec![0.0; profile_quiet.len()];
        for y in linspace(-1.0, 1.0, config.grid_size) {
            profile_quiet.shift_into(y * equatorial_velocity, &mut ccf_shifted);
            let z_bound = sqrt(1.0 - y.powi(2));
            if z_bound < std::f64::EPSILON {
                continue;
            }
            let limb_integral = limb_integral(
                &Bounds::new(-z_bound, z_bound),
                y,
                config.limb_linear,
                config.limb_quadratic,
            );
            for (tot, shifted) in integrated_ccf.iter_mut().zip(ccf_shifted.iter()) {
                *tot += *shifted * limb_integral;
            }
            flux_quiet += limb_integral;
        }

        let latitude_distribution = config
            .latitude_distribution
            .clone()
            .map(|c| Distribution::from(c))
            .unwrap_or_else(|| Distribution::Uniform(Uniform::new(-30.0, 30.0)));

        let longitude_distribution = config
            .longitude_distribution
            .clone()
            .map(|c| Distribution::from(c))
            .unwrap_or_else(|| Distribution::Uniform(Uniform::new(0.0, 360.0)));

        let fillfactor_distribution = config
            .fillfactor_distribution
            .clone()
            .map(|c| Distribution::from(c))
            .unwrap_or_else(|| Distribution::LogNormal(LogNormal::new(0.5, 4.0)));

        let lifetime_distribution = config
            .lifetime_distribution
            .clone()
            .map(|c| Distribution::from(c))
            .unwrap_or_else(|| Distribution::Uniform(Uniform::new(10.0, 20.0)));

        Star {
            period: config.period,
            inclination: config.inclination * consts::PI / 180.0,
            temperature: config.temperature,
            spot_temp_diff: config.spot_temp_diff,
            limb_linear: config.limb_linear,
            limb_quadratic: config.limb_quadratic,
            grid_size: config.grid_size,
            flux_quiet,
            zero_rv: fit_rv(&::solar_ccfs::RV, &integrated_ccf),
            equatorial_velocity,
            minimum_fill_factor: config.minimum_fill_factor,
            integrated_ccf,
            profile_spot: Profile::new(::solar_ccfs::RV.to_vec(), ::solar_ccfs::CCF_SPOT.to_vec()),
            profile_quiet,
            latitude_distribution,
            longitude_distribution,
            fillfactor_distribution,
            lifetime_distribution,
        }
    }

    pub fn limb_integral(&self, z_bounds: &Bounds, y: f64) -> f64 {
        limb_integral(z_bounds, y, self.limb_linear, self.limb_quadratic)
    }

    pub fn limb_brightness(&self, x: f64) -> f64 {
        1.0 - self.limb_linear * (1.0 - x) - self.limb_quadratic * (1.0 - x).powi(2)
    }

    pub fn draw_rgba(&self) -> Vec<u8> {
        let mut output = Vec::with_capacity(4 * 1000 * 1000);
        for z in linspace(1.0, -1.0, 1000) {
            for y in linspace(1.0, -1.0, 1000) {
                let intensity = if (y.powi(2) + z.powi(2)) <= 1.0 {
                    let x = f64::max(0.0, 1.0 - (z.powi(2) + y.powi(2)));
                    self.limb_brightness(x)
                } else {
                    0.0
                };
                output.push((intensity * 255.) as u8);
                output.push((intensity * 157.) as u8);
                output.push((intensity * 63.) as u8);
                output.push(255);
            }
        }

        output
    }
}

pub fn min(a: f64, b: f64) -> f64 {
    if a < b {
        return a;
    }
    b
}

pub fn limb_integral(z_bounds: &Bounds, y: f64, limb_linear: f64, limb_quadratic: f64) -> f64 {
    let z_upper = z_bounds.upper;
    let z_lower = z_bounds.lower;

    let x_upper = (1.0 - min(z_upper * z_upper + y * y, 1.0)).sqrt();
    let x_lower = (1.0 - min(z_lower * z_lower + y * y, 1.0)).sqrt();

    1. / 6.
        * (z_upper
            * (3.0 * limb_linear * (x_upper - 2.0)
                + 2.0
                    * (limb_quadratic * (3.0 * x_upper + 3.0 * y * y + z_upper * z_upper - 6.0)
                        + 3.0))
            - 3.0
                * (y * y - 1.0)
                * (limb_linear + 2.0 * limb_quadratic)
                * (z_upper / x_upper).atan())
        - 1. / 6.
            * (z_lower
                * (3.0 * limb_linear * (x_lower - 2.0)
                    + 2.0
                        * (limb_quadratic
                            * (3.0 * x_lower + 3.0 * y * y + z_lower * z_lower - 6.0)
                            + 3.0))
                - 3.0
                    * (y * y - 1.0)
                    * (limb_linear + 2.0 * limb_quadratic)
                    * (z_lower / x_lower).atan())
}
