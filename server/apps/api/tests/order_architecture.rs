use std::{fs, path::Path};

#[test]
fn route_modules_do_not_reference_order_sqlx_infrastructure() {
    for relative_path in [
        "src/routes/api.rs",
        "src/routes/auth.rs",
        "src/routes/health.rs",
        "src/routes/identity.rs",
        "src/routes/menu.rs",
        "src/routes/orders.rs",
        "src/routes/mod.rs",
        "src/http.rs",
    ] {
        let source =
            fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path)).unwrap();
        assert!(!source.contains("ordering_food_order_infrastructure_sqlx"));
    }
}
