use crate::DomainError;
use statig::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    PendingAcceptance,
    Accepted,
    Preparing,
    ReadyForPickup,
    Completed,
    CancelledByCustomer,
    RejectedByStore,
}

impl OrderStatus {
    pub fn parse(value: impl AsRef<str>) -> Result<Self, DomainError> {
        match value.as_ref().trim().to_ascii_lowercase().as_str() {
            "pending_acceptance" => Ok(Self::PendingAcceptance),
            "accepted" => Ok(Self::Accepted),
            "preparing" => Ok(Self::Preparing),
            "ready_for_pickup" => Ok(Self::ReadyForPickup),
            "completed" => Ok(Self::Completed),
            "cancelled_by_customer" => Ok(Self::CancelledByCustomer),
            "rejected_by_store" => Ok(Self::RejectedByStore),
            other => Err(DomainError::InvalidOrderStatus(other.to_string())),
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
        OrderLifecycle::transition(self, OrderEvent::Accept)
    }

    pub fn start_preparing(self) -> Result<Self, DomainError> {
        OrderLifecycle::transition(self, OrderEvent::StartPreparing)
    }

    pub fn mark_ready(self) -> Result<Self, DomainError> {
        OrderLifecycle::transition(self, OrderEvent::MarkReady)
    }

    pub fn complete(self) -> Result<Self, DomainError> {
        OrderLifecycle::transition(self, OrderEvent::Complete)
    }

    pub fn cancel_by_customer(self) -> Result<Self, DomainError> {
        OrderLifecycle::transition(self, OrderEvent::CancelByCustomer)
    }

    pub fn reject_by_store(self) -> Result<Self, DomainError> {
        OrderLifecycle::transition(self, OrderEvent::RejectByStore)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OrderEvent {
    Accept,
    StartPreparing,
    MarkReady,
    Complete,
    CancelByCustomer,
    RejectByStore,
}

impl OrderEvent {
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
struct OrderLifecycle {
    handled: bool,
    invalid: bool,
}

impl OrderLifecycle {
    fn transition(current: OrderStatus, event: OrderEvent) -> Result<OrderStatus, DomainError> {
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

        Ok(OrderStatus::from_state(machine.state()))
    }

    fn transition_to(&mut self, next: OrderLifecycleState) -> Outcome<OrderLifecycleState> {
        self.handled = true;
        Transition(next)
    }

    fn invalid(&mut self) -> Outcome<OrderLifecycleState> {
        self.invalid = true;
        Handled
    }
}

#[state_machine(
    initial = "OrderLifecycleState::pending_acceptance()",
    state(name = "OrderLifecycleState", derive(Debug, Clone, PartialEq, Eq))
)]
impl OrderLifecycle {
    #[state]
    fn pending_acceptance(&mut self, event: &OrderEvent) -> Outcome<OrderLifecycleState> {
        match event {
            OrderEvent::Accept => self.transition_to(OrderLifecycleState::accepted()),
            OrderEvent::CancelByCustomer => {
                self.transition_to(OrderLifecycleState::cancelled_by_customer())
            }
            OrderEvent::RejectByStore => {
                self.transition_to(OrderLifecycleState::rejected_by_store())
            }
            _ => self.invalid(),
        }
    }

    #[state]
    fn accepted(&mut self, event: &OrderEvent) -> Outcome<OrderLifecycleState> {
        match event {
            OrderEvent::StartPreparing => self.transition_to(OrderLifecycleState::preparing()),
            OrderEvent::CancelByCustomer => {
                self.transition_to(OrderLifecycleState::cancelled_by_customer())
            }
            OrderEvent::RejectByStore => {
                self.transition_to(OrderLifecycleState::rejected_by_store())
            }
            _ => self.invalid(),
        }
    }

    #[state]
    fn preparing(&mut self, event: &OrderEvent) -> Outcome<OrderLifecycleState> {
        match event {
            OrderEvent::MarkReady => self.transition_to(OrderLifecycleState::ready_for_pickup()),
            _ => self.invalid(),
        }
    }

    #[state]
    fn ready_for_pickup(&mut self, event: &OrderEvent) -> Outcome<OrderLifecycleState> {
        match event {
            OrderEvent::Complete => self.transition_to(OrderLifecycleState::completed()),
            _ => self.invalid(),
        }
    }

    #[state]
    fn completed(&mut self, event: &OrderEvent) -> Outcome<OrderLifecycleState> {
        let _ = event;
        self.invalid()
    }

    #[state]
    fn cancelled_by_customer(&mut self, event: &OrderEvent) -> Outcome<OrderLifecycleState> {
        let _ = event;
        self.invalid()
    }

    #[state]
    fn rejected_by_store(&mut self, event: &OrderEvent) -> Outcome<OrderLifecycleState> {
        let _ = event;
        self.invalid()
    }
}

impl OrderStatus {
    fn into_state(self) -> OrderLifecycleState {
        match self {
            Self::PendingAcceptance => OrderLifecycleState::pending_acceptance(),
            Self::Accepted => OrderLifecycleState::accepted(),
            Self::Preparing => OrderLifecycleState::preparing(),
            Self::ReadyForPickup => OrderLifecycleState::ready_for_pickup(),
            Self::Completed => OrderLifecycleState::completed(),
            Self::CancelledByCustomer => OrderLifecycleState::cancelled_by_customer(),
            Self::RejectedByStore => OrderLifecycleState::rejected_by_store(),
        }
    }

    fn from_state(state: &OrderLifecycleState) -> Self {
        match state {
            OrderLifecycleState::PendingAcceptance {} => Self::PendingAcceptance,
            OrderLifecycleState::Accepted {} => Self::Accepted,
            OrderLifecycleState::Preparing {} => Self::Preparing,
            OrderLifecycleState::ReadyForPickup {} => Self::ReadyForPickup,
            OrderLifecycleState::Completed {} => Self::Completed,
            OrderLifecycleState::CancelledByCustomer {} => Self::CancelledByCustomer,
            OrderLifecycleState::RejectedByStore {} => Self::RejectedByStore,
        }
    }
}
