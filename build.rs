use spirv_builder::{MetadataPrintout, SpirvBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    SpirvBuilder::new("src/shaders/test_shader", "spirv-unknown-vulkan1.0")
        .print_metadata(MetadataPrintout::Full)
        .build()?;
    Ok(())
}