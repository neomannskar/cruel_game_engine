pub enum Asset {
    Texture(Texture),
    Mesh(Mesh),
    Material(Material),
    // Shader(ShaderProgram),
}

pub enum AssetHandle {
    Texture(TextureHandle),
    Mesh(MeshHande),
    Material(MaterialHandle),
    Shader(todo!())
}

pub struct AssetLoader {
    request_tx:  Sender<AssetRequest>,
    result_rx: Receiver<(AssetHandle, Asset)>,

    pub textures: HashMap<TextureHandle, PathBuf>,
    pub meshes: HashMap<MeshHandle, PathBuf>,
    pub materials: HashMap<MaterialHandle, PathBuf>,
}

impl AssetLoader {
    pub fn new() -> Self {
        let (request_tx, request_rx) = unbounded::<AssetRequest>();
        let (result_tx, result_rx) = unbounded::<(AssetHandle, Asset)>;

        let next_handle_id = Arc::new(Mutex::new(0usize));

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
            textures: HashMap::new(),
            meshes: HashMap::new(),
            materials: HashMap::new(),
        }
    }

    pub fn request_texture<P: AsRef<Path>>(&self, path: P, name: String) {
        let path_buf = path.as_ref().to_path_buf();
        if let Err(e) = self
            .request_tx
            .send(AssetRequest::LoadTexture((path_buf, name)))
        {
            eprintln!("AssetLoader: Failed to send load request: {:?}", e);
        }
    }

    pub fn request_mesh<P: AsRef<Path>>(&self, path: P, name: String) {
        let path_buf = path.as_ref().to_path_buf();
        if let Err(e) = self
            .request_tx
            .send(AssetRequest::LoadMesh((path_buf, name)))
        {
            eprintln!("AssetLoader: Failed to send mesh load request: {:?}", e);
        }
    }

    pub fn poll_loaded(&self) -> Vec<(AssetHandle, Asset)> {
        let mut loaded = Vec::new();
        while let Ok(asset) = self.result_rx.try_recv() {
            loaded.push(asset);
        }
        loaded
    }
}
