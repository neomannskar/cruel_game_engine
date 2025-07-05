use cgmath::Deg;
use glow::HasContext;

use crate::{camera::Camera, viewport::Viewport, world::SceneNode};

pub struct Renderer;

impl Renderer {
    pub fn render(
        context: &glow::Context,
        level: &SceneNode,
        camera: &mut dyn Camera,
        viewport: &Viewport
    ) /* -> OpenGL error maybe later */ {
        unsafe {
            context.clear(glow::DEPTH_BUFFER_BIT);
            context.enable(glow::CULL_FACE);
            context.enable(glow::DEPTH_TEST);
            context.depth_func(glow::LESS);
            
            // Makes sure that everything is renderered in the central panel of the ui (FIX THIS: Renders outside)
            context.viewport(viewport.x, viewport.y, viewport.width, viewport.height);
        }

        unsafe {
            // Very bad, just in place to make it run
            if level.textures.len() > 0 {
                context.bind_texture(
                    glow::TEXTURE_2D,
                    Some(level.textures.get(0).unwrap().texture),
                );
            }
            
            context.use_program(Some(level.default_program));

            context.active_texture(glow::TEXTURE0);

            let texture_uniform = context
                .get_uniform_location(level.default_program, "image")
                .expect("Could not find the uniform called 'image'");
            context.uniform_1_i32(Some(&texture_uniform), 0);
        }

        for static_mesh in &level.static_meshes {
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
                    .get_uniform_location(level.default_program, "camMatrix")
                    .expect("Could not find the uniform called 'camMatrix'");
                context.uniform_matrix_4_f32_slice(Some(&camera_matrix_uniform), false, mvp_array);
            }

            static_mesh.render(context);
        }

        for dynamic_mesh in &level.dynamic_meshes {
            dynamic_mesh.render(context);
        }
    }
}
