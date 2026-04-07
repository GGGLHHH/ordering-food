use ordering_food_platform_kernel::Clock;
use ordering_food_shared_kernel::Timestamp;

#[derive(Debug, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Timestamp {
        Timestamp::now_utc()
    }
}
