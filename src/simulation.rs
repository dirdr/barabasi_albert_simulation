use crate::gen::AttachementMethod;
use crate::gen::BarabasiAlbertGenConfig;
use petgraph::graph::UnGraph;

trait Gen {
    fn generate(&self) -> UnGraph<(), ()>;
}

struct Simulation {}

impl Simulation {
    pub fn barabasi_albert(config: BarabasiAlbertGenConfig) -> UnGraph<(), ()> {
        match config.attachement_method {
            AttachementMethod::Preferential => {
                if config.growth {
                    todo!()
                } else {
                    todo!()
                }
            }
            AttachementMethod::Random => {
                if !config.growth {
                    panic!("Cannot simulate Barabasi-Albert wit that configuration")
                }
                todo!()
            }
        }
    }
}

struct BarabasiAlbertClassic;
struct BarabasiAlbertNoGrowth;
struct BarabasiAlbertRandomAttachement;

impl Gen for BarabasiAlbertClassic {
    fn generate(&self) -> UnGraph<(), ()> {
        todo!()
    }
}

impl Gen for BarabasiAlbertNoGrowth {
    fn generate(&self) -> UnGraph<(), ()> {
        todo!()
    }
}

impl Gen for BarabasiAlbertRandomAttachement {
    fn generate(&self) -> UnGraph<(), ()> {
        todo!()
    }
}
