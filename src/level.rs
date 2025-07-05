// In AssetLoader:
// meshes: HashMap<MeshHandle, Mesh>;


pub struct AssetLoader;

pub struct ResourceManager {
    asset_loader: AssetLoader,
    meshes: HashMap<MeshHandle, Mesh>,
}

pub struct Renderer {
    render_data: HashMap<MeshHandle, RenderData>,
}

pub struct MeshHandle(pub usize);

pub struct MeshInstance {
    pub name: String,
    pub handle: MeshHandle,
}

pub struct Level {
    pub name: String,
    
    pub mesh_instances: Vec<MeshInstance>,
    pub material_instances: Vec<MeshInstance>,
}

pub struct World {
    pub current_level: usize,
    pub levels: Vec<Box<Level>>,
}
