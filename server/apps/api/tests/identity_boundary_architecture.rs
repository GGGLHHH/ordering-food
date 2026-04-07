use std::{fs, path::Path};

fn read_source(relative_path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path)).unwrap()
}

#[test]
fn http_authentication_depends_on_identity_published_contract() {
    let http_source = read_source("src/http.rs");

    assert!(http_source.contains("ordering_food_identity_published"));
    assert!(http_source.contains("AccessTokenVerifier"));
    assert!(!http_source.contains("ordering_food_identity_application::TokenService"));
}

#[test]
fn ordering_and_fulfillment_contexts_consume_identity_published_verifier() {
    for relative_path in [
        "src/composition/contexts/ordering.rs",
        "src/composition/contexts/fulfillment.rs",
    ] {
        let source = read_source(relative_path);

        assert!(source.contains("ordering_food_identity_published"));
        assert!(source.contains("AccessTokenVerifier"));
        assert!(source.contains("IDENTITY_ACCESS_TOKEN_VERIFIER"));
        assert!(source.contains(".resolve::<Arc<dyn AccessTokenVerifier>>"));
        assert!(!source.contains("ordering_food_identity_application::TokenService"));
        assert!(!source.contains("JwtTokenService::new"));
    }
}

#[test]
fn identity_context_exports_verifier_for_http_shell() {
    let source = read_source("src/composition/contexts/identity.rs");

    assert!(source.contains("ordering_food_identity_published"));
    assert!(source.contains("ordering_food_identity_integration"));
    assert!(source.contains("AccessTokenVerifier"));
    assert!(source.contains("IDENTITY_ACCESS_TOKEN_VERIFIER"));
    assert!(source.contains("IDENTITY_SUBJECT_LOOKUP_GATEWAY"));
    assert!(source.contains(".publish("));
    assert!(source.contains("IdentityContextConfig"));
    assert!(source.contains("build_identity_context_runtime"));
    assert!(!source.contains("ordering_food_identity_application::TokenService"));
}

#[test]
fn identity_context_bootstraps_runtime_through_integration_boundary() {
    let source = read_source("src/composition/contexts/identity.rs");

    assert!(source.contains("ordering_food_identity_integration"));
    assert!(source.contains("build_identity_context_runtime"));
    assert!(!source.contains("ordering_food_identity_infrastructure_auth"));
    assert!(!source.contains("ordering_food_identity_infrastructure_sqlx"));
    assert!(!source.contains("Argon2PasswordHasher"));
    assert!(!source.contains("JwtTokenService"));
    assert!(!source.contains("RedisRefreshTokenStore"));
    assert!(!source.contains("build_identity_module"));
    assert!(!source.contains("build_subject_lookup_gateway"));
}

#[test]
fn identity_integration_exposes_runtime_and_config_boundary() {
    let source = read_source("../../crates/identity-integration/src/lib.rs");

    assert!(source.contains("pub struct IdentityContextConfig"));
    assert!(source.contains("pub struct IdentityContextRuntime"));
    assert!(source.contains("pub fn build_identity_context_runtime"));
    assert!(source.contains("AccessTokenVerifier"));
    assert!(source.contains("SubjectLookupGateway"));
    assert!(!source.contains("pub fn build_subject_lookup_gateway"));
}
