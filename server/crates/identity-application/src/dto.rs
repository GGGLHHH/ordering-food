use ordering_food_shared_kernel::Timestamp;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserProfileReadModel {
    pub display_name: String,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserIdentityReadModel {
    pub identity_type: String,
    pub identifier_normalized: String,
    pub bound_at: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserReadModel {
    pub user_id: String,
    pub status: String,
    pub profile: UserProfileReadModel,
    pub identities: Vec<UserIdentityReadModel>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub deleted_at: Option<Timestamp>,
}
