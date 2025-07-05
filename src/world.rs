use std::fs;

use crate::{
    camera::{Camera, PerspectiveCamera},
    material::Material,
    mesh::{DynamicMesh, StaticMesh},
    textures::Texture,
    viewport::Viewport,
};
use cgmath::{Deg, Matrix, Rad, Rotation3};
use egui::*;
use glow::HasContext;

pub enum SelectedObject {
    StaticMesh(usize),
    DynamicMesh(usize),
    PerspectiveCamera(usize),
    // Material(usize),
}

pub struct SceneNode {
    pub name: String,
    pub perspective_cameras: Vec<PerspectiveCamera>,
    pub static_meshes: Vec<StaticMesh>,
    pub dynamic_meshes: Vec<DynamicMesh>,
    // pub stream_meshes: Vec<StreamMesh>,
    pub textures: Vec<Texture>,
    pub materials: Vec<Material>,
    // pub shaders: Vec<ShaderProgram>,
    pub scripts: Vec<String>,

    pub default_program: glow::NativeProgram,
    // pub children: Vec<SceneNode>,
}

impl SceneNode {
    pub fn new<T: ToString>(name: T, context: &glow::Context) -> Self {
        Self {
            name: name.to_string(),
            perspective_cameras: Vec::new(),
            static_meshes: Vec::new(),
            dynamic_meshes: Vec::new(),
            textures: Vec::new(),
            materials: Vec::new(),
            scripts: Vec::new(),
            default_program: Self::create_shader_program(
                context,
                "shaders/vertex.glsl",
                "shaders/fragment.glsl",
            ),
        }
    }

    pub fn add_static_mesh(&mut self, mesh: StaticMesh) {
        self.static_meshes.push(mesh);
    }

    pub fn add_dynamic_mesh(&mut self, mesh: DynamicMesh) {
        self.dynamic_meshes.push(mesh);
    }

    pub fn add_texture(&mut self, texture: Texture) {
        self.textures.push(texture);
    }

    pub fn add_perspective_camera(&mut self, camera: PerspectiveCamera) {
        self.perspective_cameras.push(camera);
    }

    pub fn create_shader_program(
        gl: &glow::Context,
        vertex_shader_path: &str,
        fragment_shader_path: &str,
    ) -> glow::NativeProgram {
        unsafe {
            let shader_source = fs::read_to_string(vertex_shader_path).unwrap();
            let vertex_shader = gl.create_shader(glow::VERTEX_SHADER).unwrap();
            gl.shader_source(vertex_shader, &shader_source);
            gl.compile_shader(vertex_shader);

            if !gl.get_shader_compile_status(vertex_shader) {
                panic!(
                    "Error compiling vertex shader: {}",
                    gl.get_shader_info_log(vertex_shader)
                );
            }

            let shader_source = fs::read_to_string(fragment_shader_path).unwrap();
            let fragment_shader = gl.create_shader(glow::FRAGMENT_SHADER).unwrap();
            gl.shader_source(fragment_shader, &shader_source);
            gl.compile_shader(fragment_shader);

            if !gl.get_shader_compile_status(fragment_shader) {
                panic!(
                    "Error compiling fragment shader: {}",
                    gl.get_shader_info_log(fragment_shader)
                );
            }

            let shader_program = gl.create_program().unwrap();
            gl.attach_shader(shader_program, vertex_shader);
            gl.attach_shader(shader_program, fragment_shader);
            gl.link_program(shader_program);

            gl.delete_shader(vertex_shader);
            gl.delete_shader(fragment_shader);

            if !gl.get_program_link_status(shader_program) {
                panic!(
                    "Shader link error: {}",
                    gl.get_program_info_log(shader_program)
                );
            }

            shader_program
        }
    }

    pub fn update(&mut self, camera: &mut dyn Camera) {
        camera.update_matrices();
    }

    pub fn render(&self, context: &glow::Context, camera: &mut dyn Camera, viewport: &Viewport) {
        // Simple rendering logic, later the ecs will query the entities with a render system material and mesh's

        unsafe {
            context.clear(glow::DEPTH_BUFFER_BIT);
            context.enable(glow::CULL_FACE);
            context.enable(glow::DEPTH_TEST);
            context.depth_func(glow::LESS);
            // Makes sure that everything is renderered in the central panel of the ui
            context.viewport(viewport.x, viewport.y, viewport.width, viewport.height);
        }

        unsafe {
            // Very bad, just in place to make it run
            if self.textures.len() > 0 {
                context.bind_texture(
                    glow::TEXTURE_2D,
                    Some(self.textures.get(0).unwrap().texture),
                );
            }
            
            context.use_program(Some(self.default_program));

            context.active_texture(glow::TEXTURE0);

            let texture_uniform = context
                .get_uniform_location(self.default_program, "image")
                .expect("Could not find the uniform called 'image'");
            context.uniform_1_i32(Some(&texture_uniform), 0);
        }

        for static_mesh in &self.static_meshes {
            let model_matrix = cgmath::Matrix4::from_translation(static_mesh.translation)
                * cgmath::Matrix4::from_angle_x(Deg(static_mesh.rotation.x))
                * cgmath::Matrix4::from_angle_y(Deg(static_mesh.rotation.y))
                * cgmath::Matrix4::from_angle_z(Deg(static_mesh.rotation.z))
                * cgmath::Matrix4::from_nonuniform_scale(
                    static_mesh.scale.x,
                    static_mesh.scale.y,
                    static_mesh.scale.z,
                );

            let mvp_matrix = camera.get_projection() * camera.get_view() * model_matrix;

            // Very bad way to convert the matrix to a slice, but it works for now
            // Later we can use a more efficient way to convert the matrix to a slice
            let mvp_array: &[f32; 16] = unsafe { std::mem::transmute(&mvp_matrix) };

            unsafe {
                let camera_matrix_uniform = context
                    .get_uniform_location(self.default_program, "camMatrix")
                    .expect("Could not find the uniform called 'camMatrix'");
                context.uniform_matrix_4_f32_slice(Some(&camera_matrix_uniform), false, mvp_array);
            }

            static_mesh.render(context);
        }

        for dynamic_mesh in &self.dynamic_meshes {
            dynamic_mesh.render(context);
        }
    }
}

pub struct SceneGraph {
    pub current_scene: usize,
    pub scenes: Vec<Box<SceneNode>>,
}

impl SceneGraph {
    pub fn new() -> Self {
        Self {
            current_scene: 0,
            scenes: Vec::new(),
        }
    }

    pub fn current_scene_mut(&mut self) -> Option<&mut Box<SceneNode>> {
        self.scenes.get_mut(self.current_scene)
    }
}
