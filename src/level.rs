#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LevelHandle(pub usize);

pub struct Level {
    pub name: String,
    
    // Change to ECS later where instances are entities maybe?
    pub texture_instances: Vec<TextureInstance>,
    pub mesh_instances: Vec<MeshInstance>,
    pub material_instances: Vec<MaterialInstance>,
}
