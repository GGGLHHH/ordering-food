#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContextDescriptor {
    pub id: &'static str,
    pub depends_on: &'static [&'static str],
}
