use ordering_food_fulfillment_domain::WorkflowStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "fulfillment.workflow_status", rename_all = "snake_case")]
pub enum DbWorkflowStatus {
    PendingAcceptance,
    Accepted,
    Preparing,
    ReadyForPickup,
    Completed,
    CancelledByCustomer,
    RejectedByStore,
}

impl From<WorkflowStatus> for DbWorkflowStatus {
    fn from(value: WorkflowStatus) -> Self {
        match value {
            WorkflowStatus::PendingAcceptance => Self::PendingAcceptance,
            WorkflowStatus::Accepted => Self::Accepted,
            WorkflowStatus::Preparing => Self::Preparing,
            WorkflowStatus::ReadyForPickup => Self::ReadyForPickup,
            WorkflowStatus::Completed => Self::Completed,
            WorkflowStatus::CancelledByCustomer => Self::CancelledByCustomer,
            WorkflowStatus::RejectedByStore => Self::RejectedByStore,
        }
    }
}

impl From<DbWorkflowStatus> for WorkflowStatus {
    fn from(value: DbWorkflowStatus) -> Self {
        match value {
            DbWorkflowStatus::PendingAcceptance => WorkflowStatus::PendingAcceptance,
            DbWorkflowStatus::Accepted => WorkflowStatus::Accepted,
            DbWorkflowStatus::Preparing => WorkflowStatus::Preparing,
            DbWorkflowStatus::ReadyForPickup => WorkflowStatus::ReadyForPickup,
            DbWorkflowStatus::Completed => WorkflowStatus::Completed,
            DbWorkflowStatus::CancelledByCustomer => WorkflowStatus::CancelledByCustomer,
            DbWorkflowStatus::RejectedByStore => WorkflowStatus::RejectedByStore,
        }
    }
}
