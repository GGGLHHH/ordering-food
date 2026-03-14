use crate::{
    error::ErrorEnvelope,
    http::PageMeta,
    readiness::{LiveResponse, ReadyResponse},
    routes::api::{
        ExampleItemPath, ExampleItemResponse, ExamplePayload, ExamplePayloadResponse,
        ExampleSearchQuery, ExampleSearchResponse,
    },
    routes::auth::{AuthMeResponse, AuthResponse, LoginRequest},
    routes::identity::{
        BindIdentityUserIdentityRequest, CreateIdentityUserIdentityRequest,
        CreateIdentityUserRequest, IdentityUserIdentityResponse, IdentityUserPath,
        IdentityUserProfileResponse, IdentityUserResponse, UpdateIdentityUserProfileRequest,
    },
    routes::menu::{
        MenuCategoriesResponse, MenuCategoryResponse, MenuItemPath, MenuItemResponse,
        MenuItemsQuery, MenuItemsResponse, MenuStoreResponse,
    },
    routes::orders::{
        OrderItemResponse, OrderListItemResponse, OrderListResponse, OrderPath, OrderResponse,
        PlaceOrderItemRequest, PlaceOrderRequest,
    },
};
use anyhow::{Context, anyhow, ensure};
use std::{
    fs,
    path::{Path, PathBuf},
};
use ts_rs::{Config, TS};

const GENERATED_API_DIR_ENV_VAR: &str = "GENERATED_API_DIR";

pub fn export_bindings() -> anyhow::Result<PathBuf> {
    let output_dir = configured_output_dir()?;
    export_bindings_to(&output_dir)?;
    Ok(output_dir)
}

pub fn configured_output_dir() -> anyhow::Result<PathBuf> {
    resolve_output_dir(std::env::var(GENERATED_API_DIR_ENV_VAR).ok())
}

fn resolve_output_dir(env_value: Option<String>) -> anyhow::Result<PathBuf> {
    let raw_path = env_value.ok_or_else(|| {
        anyhow!("{GENERATED_API_DIR_ENV_VAR} is not set; configure it before exporting TS bindings")
    })?;
    let trimmed_path = raw_path.trim();
    ensure!(
        !trimmed_path.is_empty(),
        "{GENERATED_API_DIR_ENV_VAR} must not be empty"
    );

    Ok(PathBuf::from(trimmed_path))
}

pub fn export_bindings_to(output_dir: &Path) -> anyhow::Result<()> {
    reset_output_dir(output_dir)?;

    let config = Config::new()
        .with_out_dir(output_dir)
        .with_large_int("number");

    export_contract_types(&config)?;
    write_index_file(output_dir)?;

    Ok(())
}

fn export_contract_types(config: &Config) -> Result<(), ts_rs::ExportError> {
    ErrorEnvelope::export_all(config)?;
    LiveResponse::export_all(config)?;
    ReadyResponse::export_all(config)?;
    PageMeta::export_all(config)?;
    ExamplePayload::export_all(config)?;
    ExamplePayloadResponse::export_all(config)?;
    ExampleSearchQuery::export_all(config)?;
    ExampleSearchResponse::export_all(config)?;
    ExampleItemPath::export_all(config)?;
    ExampleItemResponse::export_all(config)?;
    BindIdentityUserIdentityRequest::export_all(config)?;
    CreateIdentityUserIdentityRequest::export_all(config)?;
    CreateIdentityUserRequest::export_all(config)?;
    UpdateIdentityUserProfileRequest::export_all(config)?;
    IdentityUserPath::export_all(config)?;
    IdentityUserIdentityResponse::export_all(config)?;
    IdentityUserProfileResponse::export_all(config)?;
    IdentityUserResponse::export_all(config)?;
    LoginRequest::export_all(config)?;
    AuthResponse::export_all(config)?;
    AuthMeResponse::export_all(config)?;
    MenuStoreResponse::export_all(config)?;
    MenuCategoryResponse::export_all(config)?;
    MenuCategoriesResponse::export_all(config)?;
    MenuItemsQuery::export_all(config)?;
    MenuItemPath::export_all(config)?;
    MenuItemResponse::export_all(config)?;
    MenuItemsResponse::export_all(config)?;
    PlaceOrderItemRequest::export_all(config)?;
    PlaceOrderRequest::export_all(config)?;
    OrderPath::export_all(config)?;
    OrderItemResponse::export_all(config)?;
    OrderListItemResponse::export_all(config)?;
    OrderListResponse::export_all(config)?;
    OrderResponse::export_all(config)?;
    Ok(())
}

fn reset_output_dir(output_dir: &Path) -> anyhow::Result<()> {
    if output_dir.exists() {
        fs::remove_dir_all(output_dir).with_context(|| {
            format!(
                "failed to clear existing TS bindings directory `{}`",
                output_dir.display()
            )
        })?;
    }

    fs::create_dir_all(output_dir).with_context(|| {
        format!(
            "failed to create TS bindings directory `{}`",
            output_dir.display()
        )
    })
}

fn write_index_file(output_dir: &Path) -> anyhow::Result<()> {
    let mut export_stems = fs::read_dir(output_dir)
        .with_context(|| {
            format!(
                "failed to read TS bindings directory `{}`",
                output_dir.display()
            )
        })?
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            let extension = path.extension()?.to_str()?;
            let stem = path.file_stem()?.to_str()?;

            (extension == "ts" && stem != "index").then(|| stem.to_string())
        })
        .collect::<Vec<_>>();
    export_stems.sort();

    let index_contents = export_stems
        .into_iter()
        .map(|stem| format!("export * from \"./{stem}\";"))
        .collect::<Vec<_>>()
        .join("\n");
    let index_contents = if index_contents.is_empty() {
        String::new()
    } else {
        format!("{index_contents}\n")
    };

    fs::write(output_dir.join("index.ts"), index_contents).with_context(|| {
        format!(
            "failed to write `{}`",
            output_dir.join("index.ts").display()
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn configured_output_dir_requires_env_var() {
        let error = resolve_output_dir(None).unwrap_err();

        assert!(error.to_string().contains("GENERATED_API_DIR is not set"));
    }

    #[test]
    fn configured_output_dir_rejects_empty_env_var() {
        let error = resolve_output_dir(Some("   ".to_string())).unwrap_err();

        assert!(
            error
                .to_string()
                .contains("GENERATED_API_DIR must not be empty")
        );
    }

    #[test]
    fn configured_output_dir_uses_env_var_value() {
        let output_dir =
            resolve_output_dir(Some("../frontend/src/contracts/generated".to_string())).unwrap();

        assert_eq!(
            output_dir,
            PathBuf::from("../frontend/src/contracts/generated")
        );
    }

    #[test]
    fn export_bindings_writes_expected_contract_files() {
        let temp_root = std::env::temp_dir().join(format!(
            "ordering-food-ts-bindings-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        export_bindings_to(&temp_root).unwrap();

        let error_envelope = fs::read_to_string(temp_root.join("ErrorEnvelope.ts")).unwrap();
        let bind_identity_user_identity_request =
            fs::read_to_string(temp_root.join("BindIdentityUserIdentityRequest.ts")).unwrap();
        let create_identity_user_request =
            fs::read_to_string(temp_root.join("CreateIdentityUserRequest.ts")).unwrap();
        let ready_response = fs::read_to_string(temp_root.join("ReadyResponse.ts")).unwrap();
        let example_item_response =
            fs::read_to_string(temp_root.join("ExampleItemResponse.ts")).unwrap();
        let identity_user_response =
            fs::read_to_string(temp_root.join("IdentityUserResponse.ts")).unwrap();
        let menu_store_response =
            fs::read_to_string(temp_root.join("MenuStoreResponse.ts")).unwrap();
        let menu_items_query = fs::read_to_string(temp_root.join("MenuItemsQuery.ts")).unwrap();
        let place_order_request =
            fs::read_to_string(temp_root.join("PlaceOrderRequest.ts")).unwrap();
        let order_list_response =
            fs::read_to_string(temp_root.join("OrderListResponse.ts")).unwrap();
        let order_response = fs::read_to_string(temp_root.join("OrderResponse.ts")).unwrap();
        let update_identity_user_profile_request =
            fs::read_to_string(temp_root.join("UpdateIdentityUserProfileRequest.ts")).unwrap();
        let index = fs::read_to_string(temp_root.join("index.ts")).unwrap();

        assert!(error_envelope.contains("request_id?: string"));
        assert!(bind_identity_user_identity_request.contains("identity_type: string"));
        assert!(create_identity_user_request.contains("display_name: string"));
        assert!(ready_response.contains("checks: DependencyChecks"));
        assert!(example_item_response.contains("item_id: number"));
        assert!(identity_user_response.contains("deleted_at?: string"));
        assert!(menu_store_response.contains("currency_code: string"));
        assert!(menu_items_query.contains("category_slug?: string"));
        assert!(place_order_request.contains("items: Array<PlaceOrderItemRequest>"));
        assert!(order_list_response.contains("orders: Array<OrderListItemResponse>"));
        assert!(order_response.contains("status: string"));
        assert!(update_identity_user_profile_request.contains("display_name: string"));
        assert!(index.contains("export * from \"./ErrorEnvelope\";"));
        assert!(index.contains("export * from \"./BindIdentityUserIdentityRequest\";"));
        assert!(index.contains("export * from \"./CreateIdentityUserRequest\";"));
        assert!(index.contains("export * from \"./IdentityUserResponse\";"));
        assert!(index.contains("export * from \"./MenuStoreResponse\";"));
        assert!(index.contains("export * from \"./MenuItemsQuery\";"));
        assert!(index.contains("export * from \"./PlaceOrderRequest\";"));
        assert!(index.contains("export * from \"./OrderListResponse\";"));
        assert!(index.contains("export * from \"./OrderResponse\";"));
        assert!(index.contains("export * from \"./UpdateIdentityUserProfileRequest\";"));
        assert!(index.contains("export * from \"./ReadyResponse\";"));
        assert!(index.contains("export * from \"./ExampleItemResponse\";"));

        fs::remove_dir_all(temp_root).unwrap();
    }
}
