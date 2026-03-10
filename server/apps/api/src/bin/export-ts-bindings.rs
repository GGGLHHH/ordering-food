use ordering_food_api::ts_bindings::export_bindings;

fn main() -> anyhow::Result<()> {
    let output_dir = export_bindings()?;
    println!("generated TypeScript bindings at {}", output_dir.display());
    Ok(())
}
