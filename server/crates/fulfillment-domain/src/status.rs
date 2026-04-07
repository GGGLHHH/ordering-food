use crate::DomainError;
use statig::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowStatus {
    PendingAcceptance,
    Accepted,
    Preparing,
    ReadyForPickup,
    Completed,
    CancelledByCustomer,
    RejectedByStore,
}

impl WorkflowStatus {
    pub fn parse(value: impl AsRef<str>) -> Result<Self, DomainError> {
        match value.as_ref().trim().to_ascii_lowercase().as_str() {
            "pending_acceptance" => Ok(Self::PendingAcceptance),
            "accepted" => Ok(Self::Accepted),
            "preparing" => Ok(Self::Preparing),
            "ready_for_pickup" => Ok(Self::ReadyForPickup),
            "completed" => Ok(Self::Completed),
            "cancelled_by_customer" => Ok(Self::CancelledByCustomer),
            "rejected_by_store" => Ok(Self::RejectedByStore),
            other => Err(DomainError::InvalidWorkflowStatus(other.to_string())),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PendingAcceptance => "pending_acceptance",
            Self::Accepted => "accepted",
            Self::Preparing => "preparing",
            Self::ReadyForPickup => "ready_for_pickup",
            Self::Completed => "completed",
            Self::CancelledByCustomer => "cancelled_by_customer",
            Self::RejectedByStore => "rejected_by_store",
        }
    }

    pub fn accept(self) -> Result<Self, DomainError> {
        WorkflowLifecycle::transition(self, WorkflowEvent::Accept)
    }

    pub fn start_preparing(self) -> Result<Self, DomainError> {
        WorkflowLifecycle::transition(self, WorkflowEvent::StartPreparing)
    }

    pub fn mark_ready(self) -> Result<Self, DomainError> {
        WorkflowLifecycle::transition(self, WorkflowEvent::MarkReady)
    }

    pub fn complete(self) -> Result<Self, DomainError> {
        WorkflowLifecycle::transition(self, WorkflowEvent::Complete)
    }

    pub fn cancel_by_customer(self) -> Result<Self, DomainError> {
        WorkflowLifecycle::transition(self, WorkflowEvent::CancelByCustomer)
    }

    pub fn reject_by_store(self) -> Result<Self, DomainError> {
        WorkflowLifecycle::transition(self, WorkflowEvent::RejectByStore)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorkflowEvent {
    Accept,
    StartPreparing,
    MarkReady,
    Complete,
    CancelByCustomer,
    RejectByStore,
}

impl WorkflowEvent {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Accept => "accept",
            Self::StartPreparing => "start_preparing",
            Self::MarkReady => "mark_ready",
            Self::Complete => "complete",
            Self::CancelByCustomer => "cancel_by_customer",
            Self::RejectByStore => "reject_by_store",
        }
    }
}

#[derive(Debug, Default)]
struct WorkflowLifecycle {
    handled: bool,
    invalid: bool,
}

impl WorkflowLifecycle {
    fn transition(
        current: WorkflowStatus,
        event: WorkflowEvent,
    ) -> Result<WorkflowStatus, DomainError> {
        let mut machine = Self::default().uninitialized_state_machine();
        *machine.state_mut() = current.into_state();
        let mut machine = machine.init();
        machine.handle(&event);

        if machine.inner().invalid || !machine.inner().handled {
            return Err(DomainError::InvalidTransition {
                event: event.as_str().to_string(),
                status: current.as_str().to_string(),
            });
        }

        Ok(WorkflowStatus::from_state(machine.state()))
    }

    fn transition_to(&mut self, next: WorkflowLifecycleState) -> Outcome<WorkflowLifecycleState> {
        self.handled = true;
        Transition(next)
    }

    fn invalid(&mut self) -> Outcome<WorkflowLifecycleState> {
        self.invalid = true;
        Handled
    }
}

#[state_machine(
    initial = "WorkflowLifecycleState::pending_acceptance()",
    state(name = "WorkflowLifecycleState", derive(Debug, Clone, PartialEq, Eq))
)]
impl WorkflowLifecycle {
    #[state]
    fn pending_acceptance(&mut self, event: &WorkflowEvent) -> Outcome<WorkflowLifecycleState> {
        match event {
            WorkflowEvent::Accept => self.transition_to(WorkflowLifecycleState::accepted()),
            WorkflowEvent::CancelByCustomer => {
                self.transition_to(WorkflowLifecycleState::cancelled_by_customer())
            }
            WorkflowEvent::RejectByStore => {
                self.transition_to(WorkflowLifecycleState::rejected_by_store())
            }
            _ => self.invalid(),
        }
    }

    #[state]
    fn accepted(&mut self, event: &WorkflowEvent) -> Outcome<WorkflowLifecycleState> {
        match event {
            WorkflowEvent::StartPreparing => {
                self.transition_to(WorkflowLifecycleState::preparing())
            }
            WorkflowEvent::CancelByCustomer => {
                self.transition_to(WorkflowLifecycleState::cancelled_by_customer())
            }
            WorkflowEvent::RejectByStore => {
                self.transition_to(WorkflowLifecycleState::rejected_by_store())
            }
            _ => self.invalid(),
        }
    }

    #[state]
    fn preparing(&mut self, event: &WorkflowEvent) -> Outcome<WorkflowLifecycleState> {
        match event {
            WorkflowEvent::MarkReady => {
                self.transition_to(WorkflowLifecycleState::ready_for_pickup())
            }
            _ => self.invalid(),
        }
    }

    #[state]
    fn ready_for_pickup(&mut self, event: &WorkflowEvent) -> Outcome<WorkflowLifecycleState> {
        match event {
            WorkflowEvent::Complete => self.transition_to(WorkflowLifecycleState::completed()),
            _ => self.invalid(),
        }
    }

    #[state]
    fn completed(&mut self, event: &WorkflowEvent) -> Outcome<WorkflowLifecycleState> {
        let _ = event;
        self.invalid()
    }

    #[state]
    fn cancelled_by_customer(&mut self, event: &WorkflowEvent) -> Outcome<WorkflowLifecycleState> {
        let _ = event;
        self.invalid()
    }

    #[state]
    fn rejected_by_store(&mut self, event: &WorkflowEvent) -> Outcome<WorkflowLifecycleState> {
        let _ = event;
        self.invalid()
    }
}

impl WorkflowStatus {
    fn into_state(self) -> WorkflowLifecycleState {
        match self {
            Self::PendingAcceptance => WorkflowLifecycleState::pending_acceptance(),
            Self::Accepted => WorkflowLifecycleState::accepted(),
            Self::Preparing => WorkflowLifecycleState::preparing(),
            Self::ReadyForPickup => WorkflowLifecycleState::ready_for_pickup(),
            Self::Completed => WorkflowLifecycleState::completed(),
            Self::CancelledByCustomer => WorkflowLifecycleState::cancelled_by_customer(),
            Self::RejectedByStore => WorkflowLifecycleState::rejected_by_store(),
        }
    }

    fn from_state(state: &WorkflowLifecycleState) -> Self {
        match state {
            WorkflowLifecycleState::PendingAcceptance {} => Self::PendingAcceptance,
            WorkflowLifecycleState::Accepted {} => Self::Accepted,
            WorkflowLifecycleState::Preparing {} => Self::Preparing,
            WorkflowLifecycleState::ReadyForPickup {} => Self::ReadyForPickup,
            WorkflowLifecycleState::Completed {} => Self::Completed,
            WorkflowLifecycleState::CancelledByCustomer {} => Self::CancelledByCustomer,
            WorkflowLifecycleState::RejectedByStore {} => Self::RejectedByStore,
        }
    }
}
