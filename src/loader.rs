use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use crate::{
    data::*,
    handles::{AssetHandle, MaterialHandle, MeshHandle, ShaderHandle, TextureHandle},
};
use crossbeam_channel::{unbounded, Receiver, Sender};
use gltf::{buffer::Source, Gltf, mesh::util::ReadColors};

pub fn load_gltf_full(path: &Path) -> Result<LoadedMesh, String> {
    let gltf = Gltf::open(path).map_err(|e| format!("GLTF open error: {:?}", e))?;

    let mut raw_buffers = Vec::new();
    let blob = gltf.blob.as_ref().cloned();

    // Load all buffers referenced by the GLTF:
    for buffer in gltf.buffers() {
        let data = match buffer.source() {
            Source::Uri(uri) => {
                let buf_path = path.parent().unwrap().join(uri);
                std::fs::read(&buf_path).map_err(|e| format!("Buffer read error: {:?}", e))?
            }
            Source::Bin => blob
                .clone()
                .ok_or_else(|| "GLB binary chunk missing".to_string())?,
        };
        raw_buffers.push(data);
    }

    let mut primitives = Vec::new();

    for mesh in gltf.meshes() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| {
                let index = buffer.index();
                raw_buffers.get(index).map(|v| v.as_slice())
            });

            let mut vertex_data = VertexData {
                positions: Vec::new(),
                normals: None,
                tangents: None,
                texcoords: Vec::new(),
                colors: Vec::new(),
                joints: None,
                weights: None,
            };

            // ----------- Mandatory positions -----------
            if let Some(position_iter) = reader.read_positions() {
                vertex_data.positions = position_iter.collect();
            } else {
                return Err("GLTF mesh is missing positions!".into());
            }

            let vertex_count = vertex_data.positions.len();

            // ----------- Optionals -----------
            if let Some(normals_iter) = reader.read_normals() {
                vertex_data.normals = Some(normals_iter.collect());
            }

            if let Some(tangents_iter) = reader.read_tangents() {
                vertex_data.tangents = Some(tangents_iter.collect());
            }

            if let Some(uv_sets) = reader.read_tex_coords(0) {
                let texcoords0 = uv_sets.into_f32().collect::<Vec<[f32; 2]>>();
                vertex_data.texcoords.push(Uv(texcoords0));
            }

            // Supports TEXCOORD_1 as second UV set:
            if let Some(uv_sets1) = reader.read_tex_coords(1) {
                let texcoords1 = uv_sets1.into_f32().collect::<Vec<[f32; 2]>>();
                vertex_data.texcoords.push(Uv(texcoords1));
            }

            if let Some(colors_reader) = reader.read_colors(0) {
                match colors_reader {
                    ReadColors::RgbU8(rgb) => {
                        vertex_data.colors.push(Color::Rgb(rgb.map(|c| [
                            c[0] as f32 / 255.0,
                            c[1] as f32 / 255.0,
                            c[2] as f32 / 255.0,
                        ]).collect()));
                    }
                    ReadColors::RgbaU8(rgba) => {
                        vertex_data.colors.push(Color::Rgba(rgba.map(|c| [
                            c[0] as f32 / 255.0,
                            c[1] as f32 / 255.0,
                            c[2] as f32 / 255.0,
                            c[3] as f32 / 255.0,
                        ]).collect()));
                    }
                    ReadColors::RgbF32(rgb) => {
                        vertex_data.colors.push(Color::Rgb(rgb.collect()));
                    }
                    ReadColors::RgbaF32(rgba) => {
                        vertex_data.colors.push(Color::Rgba(rgba.collect()));
                    }
                    ReadColors::RgbU16(_iter) => {
                        println!("TODO");
                    },
                    ReadColors::RgbaU16(_iter) => {
                        println!("TODO");
                    },
                }
            }

            if let Some(joints_iter) = reader.read_joints(0) {
                vertex_data.joints = Some(joints_iter.into_u16().collect());
            }

            if let Some(weights_iter) = reader.read_weights(0) {
                vertex_data.weights = Some(weights_iter.into_f32().collect());
            }

            // Indices:
            let indices = reader.read_indices().map(|idx| idx.into_u32().collect());

            // Material (optional):
            let material = primitive.material();
            let pbr = material.pbr_metallic_roughness();

            let loaded_material = Some(LoadedMaterial {
                base_color_texture: pbr.base_color_texture().and_then(|info| {
                    let image = info.texture().source();
                    match image.source() {
                        gltf::image::Source::Uri { uri, .. } => Some(PathBuf::from(uri)),
                        gltf::image::Source::View { .. } => None, // Embedded images not supported here yet
                    }
                }),
                metallic_roughness_texture: pbr.metallic_roughness_texture().and_then(|info| {
                    let image = info.texture().source();
                    match image.source() {
                        gltf::image::Source::Uri { uri, .. } => Some(PathBuf::from(uri)),
                        gltf::image::Source::View { .. } => None,
                    }
                }),
                normal_texture: material.normal_texture().and_then(|info| {
                    let image = info.texture().source();
                    match image.source() {
                        gltf::image::Source::Uri { uri, .. } => Some(PathBuf::from(uri)),
                        gltf::image::Source::View { .. } => None,
                    }
                }),
                occlusion_texture: material.occlusion_texture().and_then(|info| {
                    let image = info.texture().source();
                    match image.source() {
                        gltf::image::Source::Uri { uri, .. } => Some(PathBuf::from(uri)),
                        gltf::image::Source::View { .. } => None,
                    }
                }),
                emissive_texture: material.emissive_texture().and_then(|info| {
                    let image = info.texture().source();
                    match image.source() {
                        gltf::image::Source::Uri { uri, .. } => Some(PathBuf::from(uri)),
                        gltf::image::Source::View { .. } => None,
                    }
                }),
                base_color_factor: Color::Rgba(vec![pbr.base_color_factor()]),
                metallic_factor: pbr.metallic_factor(),
                roughness_factor: pbr.roughness_factor(),
                alpha_mode: matches!(material.alpha_mode(), gltf::material::AlphaMode::Blend),
                double_sided: material.double_sided(),
            });

            primitives.push(LoadedPrimitive {
                vertex_data,
                material: loaded_material,
                indices,
            });
        }
    }

    Ok(LoadedMesh {
        name: path.file_name().unwrap().to_string_lossy().into_owned(),
        path: path.to_path_buf(),
        primitives,
    })
}

#[derive(Debug)]
pub enum Asset {
    Texture(LoadedTexture),
    Mesh(LoadedMesh),
    Material(LoadedMaterial),
    Shader(CompiledShaderProgram),
    // ...
}

impl Asset {
    pub fn into_texture(self) -> Option<LoadedTexture> {
        if let Asset::Texture(texture) = self {
            Some(texture)
        } else {
            None
        }
    }

    pub fn into_mesh(self) -> Option<LoadedMesh> {
        if let Asset::Mesh(mesh) = self {
            Some(mesh)
        } else {
            None
        }
    }

    pub fn into_material(self) -> Option<LoadedMaterial> {
        if let Asset::Material(material) = self {
            Some(material)
        } else {
            None
        }
    }

    pub fn into_shader(self) -> Option<CompiledShaderProgram> {
        if let Asset::Shader(shader) = self {
            Some(shader)
        } else {
            None
        }
    }
}

pub enum AssetRequest {
    LoadTexture((PathBuf, String)),
    LoadMesh((PathBuf, String)),
    // ...
}

pub struct AssetLoader {
    request_tx: Sender<AssetRequest>,
    result_rx: Receiver<(AssetHandle, Asset)>,

    next_handle_id: Arc<Mutex<usize>>,

    pub loaded_texture_data: HashMap<TextureHandle, LoadedTexture>,
    pub loaded_mesh_data: HashMap<MeshHandle, LoadedMesh>,
    pub loaded_material_data: HashMap<MaterialHandle, LoadedMaterial>,
    pub compiled_shader_programs: HashMap<ShaderHandle, CompiledShaderProgram>,
}

impl AssetLoader {
    pub fn new() -> Self {
        let (request_tx, request_rx) = unbounded::<AssetRequest>();
        let (result_tx, result_rx) = unbounded::<(AssetHandle, Asset)>();

        let next_handle_id = Arc::new(Mutex::new(0usize));

        // Pass next_handle_id to the loader thread so it can generate handles.
        let thread_next_handle_id = Arc::clone(&next_handle_id);

        std::thread::spawn(move || {
            for request in request_rx {
                match request {
                    AssetRequest::LoadTexture((path, name)) => {
                        println!("Loader thread: Loading texture {:?}", path);

                        let img = match image::open(&path) {
                            Ok(i) => i.flipv().to_rgba8(),
                            Err(e) => {
                                eprintln!("Failed to load image {:?}: {:?}", path, e);
                                continue;
                            }
                        };

                        let (width, height) = img.dimensions();
                        let data = img.into_raw();

                        let loaded_texture = LoadedTexture {
                            path: path.clone(),
                            name,
                            width,
                            height,
                            data,
                        };

                        let texture_handle = {
                            let mut id = thread_next_handle_id.lock().unwrap();
                            let handle = TextureHandle(*id as usize);
                            *id += 1;
                            handle
                        };

                        if let Err(e) = result_tx.send((
                            AssetHandle::Texture(texture_handle),
                            Asset::Texture(loaded_texture),
                        )) {
                            eprintln!("Failed to send loaded texture: {:?}", e);
                            break;
                        }
                    }

                    AssetRequest::LoadMesh((path, name)) => {
                        println!("Loader thread: Loading mesh {:?}", path);

                        match load_gltf_full(&path) {
                            Ok(mut loaded_mesh) => {
                                loaded_mesh.name = name;

                                let mesh_handle = {
                                    let mut id = thread_next_handle_id.lock().unwrap();
                                    let handle = MeshHandle(*id as usize);
                                    *id += 1;
                                    handle
                                };

                                if let Err(e) = result_tx.send((
                                    AssetHandle::Mesh(mesh_handle),
                                    Asset::Mesh(loaded_mesh),
                                )) {
                                    eprintln!("Failed to send loaded mesh: {:?}", e);
                                    break;
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to load mesh {:?}: {:?}", path, e);
                            }
                        }
                    }
                }
            }
        });

        Self {
            request_tx,
            result_rx,
            next_handle_id,
            loaded_texture_data: HashMap::new(),
            loaded_mesh_data: HashMap::new(),
            loaded_material_data: HashMap::new(),
            compiled_shader_programs: HashMap::new(),
        }
    }

    fn generate_texture_handle(&mut self) -> TextureHandle {
        let mut id = self.next_handle_id.lock().unwrap();
        let handle = TextureHandle(*id);
        *id += 1;
        handle
    }

    fn generate_mesh_handle(&mut self) -> MeshHandle {
        let mut id = self.next_handle_id.lock().unwrap();
        let handle = MeshHandle(*id);
        *id += 1;
        handle
    }

    /// Request an async load of a texture.
    pub fn request_texture<P: AsRef<std::path::Path>>(&self, path: P, name: String) {
        let path_buf = path.as_ref().to_path_buf();
        if let Err(e) = self
            .request_tx
            .send(AssetRequest::LoadTexture((path_buf, name)))
        {
            eprintln!("AssetLoader: Failed to send load request: {:?}", e);
        }
    }

    pub fn request_mesh<P: AsRef<std::path::Path>>(&self, path: P, name: String) {
        let path_buf = path.as_ref().to_path_buf();
        if let Err(e) = self
            .request_tx
            .send(AssetRequest::LoadMesh((path_buf, name)))
        {
            eprintln!("AssetLoader: Failed to send mesh load request: {:?}", e);
        }
    }

    /// Poll to see if any assets have been loaded.
    pub fn poll_loaded(&self) -> Vec<(AssetHandle, Asset)> {
        let mut loaded = Vec::new();
        while let Ok(asset) = self.result_rx.try_recv() {
            loaded.push(asset);
        }
        loaded
    }
}
