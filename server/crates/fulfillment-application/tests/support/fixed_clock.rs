use ordering_food_fulfillment_application::Clock;
use ordering_food_shared_kernel::Timestamp;

pub struct FixedClock {
    pub now: Timestamp,
}

impl Clock for FixedClock {
    fn now(&self) -> Timestamp {
        self.now
    }
}
