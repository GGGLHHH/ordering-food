use std::{fs, path::Path};

#[test]
fn application_scope_types_do_not_import_organization_published_models() {
    for relative_path in ["src/organization_scope.rs", "src/ports.rs"] {
        let source = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path))
            .unwrap();

        assert!(
            !source.contains("ordering_food_organization_published"),
            "{relative_path} unexpectedly imports organization published module"
        );
        assert!(
            !source.contains("StoreSummary"),
            "{relative_path} unexpectedly exposes StoreSummary"
        );
        assert!(
            !source.contains("BrandRef"),
            "{relative_path} unexpectedly exposes BrandRef"
        );
    }
}
