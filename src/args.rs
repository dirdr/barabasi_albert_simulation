use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, default_value_t = 10, value_parser= validate_n)]
    pub n: usize,

    #[arg(short, long, default_value_t = 5, value_parser = validate_m)]
    pub m: usize,

    #[arg(short, long, default_value_t = 100000)]
    pub barabasi_end_time: usize,

    #[arg(short, long, default_value_t, value_enum)]
    pub barabasi_starting_graph_type: ArgsGraphType,

    #[arg(short, long, default_value_t = 100)]
    pub iteration_number: usize,
}

#[derive(clap::ValueEnum, Debug, Clone, Default)]
pub enum ArgsGraphType {
    #[default]
    Complete,
    Star,
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
