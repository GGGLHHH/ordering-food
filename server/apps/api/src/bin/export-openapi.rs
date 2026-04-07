use ordering_food_api::openapi_export::build_merged_openapi_document;
use std::{fs, path::PathBuf};

fn main() -> anyhow::Result<()> {
    let contracts_dir = std::env::var("GENERATED_API_DIR")
        .map(|dir| {
            let p = PathBuf::from(dir);
            // GENERATED_API_DIR points to contracts/generated/, go up to contracts/
            p.parent().unwrap_or(&p).to_path_buf()
        })
        .unwrap_or_else(|_| PathBuf::from("../frontend/src/contracts"));

    let openapi_dir = contracts_dir.join("openapi");
    fs::create_dir_all(&openapi_dir)?;

    let openapi = build_merged_openapi_document();
    let json = serde_json::to_string_pretty(&openapi)?;
    let output_path = openapi_dir.join("openapi.json");
    fs::write(&output_path, &json)?;

    println!("exported OpenAPI spec to {}", output_path.display());
    Ok(())
}
