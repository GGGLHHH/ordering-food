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
pub struct UuidV4UserIdGenerator;

impl IdGenerator for UuidV4UserIdGenerator {
    fn next_user_id(&self) -> UserId {
        UserId::new(Uuid::new_v4().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::UuidV4UserIdGenerator;
    use ordering_food_identity_application::IdGenerator;
    use ordering_food_shared_kernel::Identifier;

    #[test]
    fn uuid_v4_user_id_generator_generates_uuid_v4_string() {
        let generator = UuidV4UserIdGenerator;
        let user_id = generator.next_user_id();
        let parsed = uuid::Uuid::parse_str(user_id.as_str()).unwrap();

        assert_eq!(parsed.get_version_num(), 4);
    }
}
