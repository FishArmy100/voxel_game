use spirv_builder::{MetadataPrintout, SpirvBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    SpirvBuilder::new("resources/voxel_raytracer", "spirv-unknown-vulkan1.2")
            .print_metadata(MetadataPrintout::Full)
            .build()?;
    SpirvBuilder::new("resources/terrain_gen", "spirv-unknown-vulkan1.2")
            .print_metadata(MetadataPrintout::Full)
            .build()?;
    Ok(())
}