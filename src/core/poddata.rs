#[derive(Debug, Clone, Copy)]
pub enum PodStatus {
    PodReachable,
    PodUnreachable,
}

#[derive(Debug, Clone, Copy)]
pub struct PodData {
    pub name: &'static str,
    pub container: &'static str,
    pub status: PodStatus
}

pub trait PodDataImpl {
    fn new() -> Self;
}

impl PodDataImpl for PodData {
    fn new() -> PodData {
        PodData {
            name: "",
            container: "",
            status: PodStatus::PodUnreachable
        }
    }
}
