#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubjectStatus {
    Active,
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubjectRef {
    subject_id: String,
    status: SubjectStatus,
}

impl SubjectRef {
    pub fn new(subject_id: impl Into<String>, status: SubjectStatus) -> Self {
        Self {
            subject_id: subject_id.into(),
            status,
        }
    }

    pub fn subject_id(&self) -> &str {
        &self.subject_id
    }

    pub fn status(&self) -> SubjectStatus {
        self.status
    }
}
