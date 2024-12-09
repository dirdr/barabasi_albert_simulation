use core::fmt;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, value_parser= validate_n)]
    pub n: usize,

    #[arg(short, long, value_parser = validate_m)]
    pub m: usize,

    #[arg(short, long, default_value_t = 100000)]
    pub t_max: usize,

    #[arg(short, long, default_value_t = 100)]
    pub iterations: usize,

    #[arg(short, long, default_value_t, value_enum)]
    pub starting_graph: ArgsGraphType,

    #[arg(long, default_value_t, value_enum)]
    pub model: ArgsModelType,
}

#[derive(clap::ValueEnum, Debug, Clone, Default, Copy, PartialEq, Eq)]
pub enum ArgsGraphType {
    #[default]
    Complete,
    Star,
    Disconnected,
}

#[derive(clap::ValueEnum, Debug, Clone, Default, Copy)]
#[value(rename_all = "snake_case")]
pub enum ArgsModelType {
    #[default]
    GrowthPreferential,
    NoGrowthPreferential,
    GrowthRandom,
}

impl fmt::Display for ArgsModelType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArgsModelType::GrowthPreferential => write!(f, "growth_preferential"),
            ArgsModelType::GrowthRandom => write!(f, "growth_random"),
            ArgsModelType::NoGrowthPreferential => write!(f, "no_growth_preferential"),
        }
    }
}

fn validate_m(m: &str) -> Result<usize, String> {
    let parsed_n = m
        .parse::<usize>()
        .map_err(|_| "m must be a positive integer".to_string())?;
    if parsed_n >= 1 {
        Ok(parsed_n)
    } else {
        Err("Number of edges increment must be at least 1".to_string())
    }
}

fn validate_n(n: &str) -> Result<usize, String> {
    let parsed_n = n
        .parse::<usize>()
        .map_err(|_| "n must be a positive integer".to_string())?;
    if parsed_n >= 2 {
        Ok(parsed_n)
    } else {
        Err("Number of initial nodes must be at least 2".to_string())
    }
}
