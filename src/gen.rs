#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum AttachementMethod {
    Preferential,
    Random,
}

pub struct BarabasiAlbertGenConfig {
    pub growth: bool,
    pub attachement_method: AttachementMethod,
}

impl BarabasiAlbertGenConfig {
    pub fn new(growth: bool, attachement_method: AttachementMethod) -> Self {
        // Having no groth + random preferential attachement is not valid configuration and should
        // not be constructed
        assert!(growth || attachement_method == AttachementMethod::Preferential);
        Self {
            growth,
            attachement_method,
        }
    }
}
