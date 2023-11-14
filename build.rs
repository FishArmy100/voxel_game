use spirv_builder::{MetadataPrintout, SpirvBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    SpirvBuilder::new("src/shaders/test_shader", "spirv-unknown-vulkan1.2")
            .print_metadata(MetadataPrintout::Full)
            .build()?;
    SpirvBuilder::new("src/shaders/screen_shader", "spirv-unknown-vulkan1.2")
            .print_metadata(MetadataPrintout::Full)
            .build()?;
    SpirvBuilder::new("src/shaders/raytracing_shader", "spirv-unknown-vulkan1.2")
            .print_metadata(MetadataPrintout::Full)
            .build()?;
    Ok(())
}