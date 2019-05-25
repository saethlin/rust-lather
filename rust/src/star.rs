use std::f64::consts;

use bounds::Bounds;
use distributions::{Distribution, DistributionConfig};
use linspace::linspace;
use profile::Profile;

use rand::distributions::{LogNormal, Uniform};

const SOLAR_RADIUS: f64 = 6.96e8;
const DAYS_TO_SECONDS: f64 = 86400.0;

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
    pub minimum_fill_factor: Option<f64>,
    pub target_fill_factor: Option<f64>,
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
    pub equatorial_velocity: f64,
    pub target_fill_factor: f64,
    pub integrated_ccf: Vec<f64>,
    pub profile_spot: Profile,
    pub profile_quiet: Profile,
    pub latitude_distribution: Distribution,
    pub longitude_distribution: Distribution,
    pub fillfactor_distribution: Distribution,
    pub lifetime_distribution: Distribution,
    image: std::sync::Mutex<Option<Vec<u8>>>,
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
            .field("equatorial_velocity", &self.equatorial_velocity)
            .field("target_fill_factor", &self.target_fill_factor)
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
        let equatorial_velocity = edge_velocity * (config.inclination.to_radians()).sin();

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
            // Config units are degrees, but we use radians internally
            inclination: config.inclination.to_radians(),
            temperature: config.temperature,
            spot_temp_diff: config.spot_temp_diff,
            limb_linear: config.limb_linear,
            limb_quadratic: config.limb_quadratic,
            grid_size: config.grid_size,
            flux_quiet,
            equatorial_velocity,
            target_fill_factor: config
                .minimum_fill_factor
                .or(config.target_fill_factor)
                .unwrap_or(0.0),
            integrated_ccf,
            profile_spot: Profile::new(::solar_ccfs::RV.to_vec(), ::solar_ccfs::CCF_SPOT.to_vec()),
            profile_quiet,
            latitude_distribution,
            longitude_distribution,
            fillfactor_distribution,
            lifetime_distribution,
            image: std::sync::Mutex::new(None),
        }
    }

    pub fn limb_integral(&self, z_bounds: &Bounds, y: f64) -> f64 {
        limb_integral(z_bounds, y, self.limb_linear, self.limb_quadratic)
    }

    pub fn limb_brightness(&self, x: f64) -> f64 {
        1.0 - self.limb_linear * (1.0 - x) - self.limb_quadratic * (1.0 - x).powi(2)
    }

    pub fn draw_bgr(&self, image: &mut [u8]) {
        let mut cache = self.image.lock().unwrap();

        if let Some(ref cached) = *cache {
            image.copy_from_slice(&cached);
            return;
        }

        let color = match &TEMP_TO_RGB.binary_search_by(|k| k.0.cmp(&(self.temperature as u16))) {
            Ok(v) => TEMP_TO_RGB[*v].1,
            Err(v) => TEMP_TO_RGB[*v].1,
        };
        let color = [color[0] as f64, color[1] as f64, color[2] as f64];

        let mut i = 0;
        for z in linspace(1.0, -1.0, 1000) {
            for y in linspace(1.0, -1.0, 1000) {
                let intensity = if (y.powi(2) + z.powi(2)) <= 1.0 {
                    let x = f64::max(0.0, 1.0 - (z.powi(2) + y.powi(2)));
                    self.limb_brightness(x)
                } else {
                    0.0
                };
                // opencv wants these in BGR but the image array has them in RGB
                image[i + 0] = (color[2] * intensity) as u8;
                image[i + 1] = (color[1] * intensity) as u8;
                image[i + 2] = (color[0] * intensity) as u8;
                i += 3;
            }
        }

        *cache = Some(image.to_vec());
    }
}

pub fn min(a: f64, b: f64) -> f64 {
    if a < b {
        a
    } else {
        b
    }
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

static TEMP_TO_RGB: [(u16, [u8; 3]); 130] = [
    (1990, [255, 233, 154]),
    (2000, [255, 198, 108]),
    (2167, [255, 198, 109]),
    (2180, [255, 167, 97]),
    (2333, [255, 197, 111]),
    (2370, [255, 165, 97]),
    (2500, [255, 195, 112]),
    (2560, [255, 178, 121]),
    (2667, [255, 204, 111]),
    (2750, [255, 197, 124]),
    (2833, [255, 201, 127]),
    (2940, [255, 206, 127]),
    (3000, [255, 206, 129]),
    (3130, [255, 200, 119]),
    (3167, [255, 196, 131]),
    (3320, [255, 198, 118]),
    (3333, [255, 204, 142]),
    (3500, [255, 195, 139]),
    (3510, [255, 200, 121]),
    (3640, [255, 202, 157]),
    (3700, [255, 203, 132]),
    (3717, [255, 206, 140]),
    (3725, [255, 205, 135]),
    (3733, [255, 208, 142]),
    (3750, [255, 206, 139]),
    (3780, [255, 209, 174]),
    (3800, [255, 211, 146]),
    (3900, [255, 216, 167]),
    (3920, [255, 199, 142]),
    (4000, [255, 221, 175]),
    (4057, [255, 223, 181]),
    (4060, [255, 205, 152]),
    (4114, [255, 227, 190]),
    (4171, [255, 231, 196]),
    (4200, [255, 210, 161]),
    (4229, [255, 231, 199]),
    (4286, [255, 234, 207]),
    (4340, [255, 216, 181]),
    (4343, [255, 236, 215]),
    (4400, [255, 236, 211]),
    (4480, [255, 222, 195]),
    (4620, [255, 227, 196]),
    (4669, [255, 243, 233]),
    (4760, [255, 224, 188]),
    (4900, [255, 238, 221]),
    (4937, [255, 243, 233]),
    (5010, [255, 239, 221]),
    (5120, [255, 237, 222]),
    (5206, [255, 243, 233]),
    (5230, [255, 244, 235]),
    (5340, [255, 244, 235]),
    (5450, [255, 244, 234]),
    (5474, [255, 243, 233]),
    (5560, [255, 241, 229]),
    (5670, [255, 243, 236]),
    (5743, [255, 242, 233]),
    (5780, [255, 245, 242]),
    (5890, [255, 247, 248]),
    (6000, [255, 248, 252]),
    (6011, [255, 246, 233]),
    (6140, [255, 247, 252]),
    (6280, [255, 247, 252]),
    (6420, [246, 243, 255]),
    (6520, [255, 243, 250]),
    (6560, [244, 241, 255]),
    (6700, [248, 247, 255]),
    (6760, [255, 234, 252]),
    (6840, [224, 226, 255]),
    (6980, [230, 233, 255]),
    (7000, [219, 225, 255]),
    (7120, [236, 239, 255]),
    (7193, [227, 231, 255]),
    (7260, [230, 234, 255]),
    (7387, [236, 237, 255]),
    (7400, [224, 229, 255]),
    (7580, [244, 243, 255]),
    (7650, [219, 224, 255]),
    (7773, [223, 229, 255]),
    (7900, [213, 222, 255]),
    (7967, [202, 215, 255]),
    (8150, [200, 213, 255]),
    (8160, [206, 218, 255]),
    (8353, [210, 221, 255]),
    (8400, [199, 212, 255]),
    (8547, [215, 223, 255]),
    (8650, [202, 215, 255]),
    (8740, [219, 226, 255]),
    (8900, [197, 211, 255]),
    (8933, [223, 229, 255]),
    (9127, [215, 224, 255]),
    (9150, [191, 207, 255]),
    (9320, [207, 219, 255]),
    (9400, [187, 203, 255]),
    (9513, [199, 214, 255]),
    (9650, [181, 199, 255]),
    (9707, [214, 223, 255]),
    (9900, [185, 201, 255]),
    (11710, [181, 198, 255]),
    (13520, [177, 195, 255]),
    (15330, [173, 191, 255]),
    (17140, [172, 189, 255]),
    (18950, [170, 191, 255]),
    (20160, [187, 203, 255]),
    (20760, [164, 184, 255]),
    (21370, [175, 194, 255]),
    (22570, [165, 185, 255]),
    (22580, [177, 196, 255]),
    (23790, [168, 193, 255]),
    (24380, [160, 180, 255]),
    (25000, [161, 189, 255]),
    (26190, [160, 182, 255]),
    (27600, [164, 185, 255]),
    (28000, [156, 178, 255]),
    (30200, [154, 178, 255]),
    (32400, [157, 177, 255]),
    (32800, [160, 181, 255]),
    (34600, [157, 177, 255]),
    (35400, [157, 178, 255]),
    (36800, [162, 184, 255]),
    (38000, [155, 176, 255]),
    (39000, [155, 176, 255]),
    (40400, [153, 174, 255]),
    (41200, [153, 174, 255]),
    (42800, [151, 172, 255]),
    (43400, [151, 172, 255]),
    (45200, [148, 170, 255]),
    (45600, [148, 170, 255]),
    (47600, [146, 168, 255]),
    (47800, [146, 168, 255]),
    (50000, [144, 166, 255]),
];
