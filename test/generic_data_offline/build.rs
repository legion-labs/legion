fn main() -> Result<(), Box<dyn std::error::Error>> {
    legion_data_codegen::data_container_gen!["/data.rs"];
    Ok(())
}
