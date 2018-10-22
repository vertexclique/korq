#[derive(Debug, Clone, Copy)]
pub enum PodStatus {
    PodReachable,
    PodUnreachable,
}

#[derive(Debug, Clone, Copy)]
pub struct PodData {
    pub name: &'static str,
    pub status: PodStatus
}
