//! Domain language for the Access bounded context.

mod access_role;
mod access_scope;
mod subject_access_grant;

pub use access_role::AccessRole;
pub use access_scope::AccessScope;
pub use subject_access_grant::{InvalidSubjectAccessGrant, SubjectAccessGrant};
