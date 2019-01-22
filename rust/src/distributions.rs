use rand::distributions::{LogNormal, Normal, StandardNormal, Uniform};
use rand::Rng;

#[derive(Deserialize, Serialize, Clone)]
#[serde(tag = "name")]
pub enum DistributionConfig {
    #[serde(rename = "standard_normal")]
    StandardNormal,
    #[serde(rename = "lognormal")]
    LogNormal { mean: f64, std_dev: f64 },
    #[serde(rename = "uniform")]
    Uniform { min: f64, max: f64 },
    #[serde(rename = "normal")]
    Normal { mean: f64, std_dev: f64 },
}

#[derive(Debug)]
pub enum Distribution {
    StandardNormal(StandardNormal),
    LogNormal(LogNormal),
    Uniform(Uniform<f64>),
    Normal(Normal),
}

impl Distribution {
    pub fn sample(&self, rng: &mut rand::rngs::StdRng) -> f64 {
        match self {
            Distribution::StandardNormal(ref inner) => rng.sample(inner),
            Distribution::LogNormal(ref inner) => rng.sample(inner),
            Distribution::Uniform(ref inner) => rng.sample(inner),
            Distribution::Normal(ref inner) => rng.sample(inner),
        }
    }
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
            DistributionConfig::Normal { mean, std_dev } => {
                Distribution::Normal(Normal::new(mean, std_dev))
            }
        }
    }
}
