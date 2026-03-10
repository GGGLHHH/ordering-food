use ordering_food_identity_application::{Clock, IdGenerator};
use ordering_food_identity_domain::UserId;
use ordering_food_shared_kernel::Timestamp;
use uuid::Uuid;

#[derive(Debug, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Timestamp {
        Timestamp::now_utc()
    }
}

#[derive(Debug, Default)]
pub struct UuidV7UserIdGenerator;

impl IdGenerator for UuidV7UserIdGenerator {
    fn next_user_id(&self) -> UserId {
        UserId::new(Uuid::now_v7().to_string())
    }
}
