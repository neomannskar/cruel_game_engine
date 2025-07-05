use cgmath::SquareMatrix;
use glow::HasContext;

use crate::{
    data::{Color, DynamicPrimitiveInstance, StaticPrimitiveInstance, VertexData},
    handles::MeshHandle,
    loader::AssetLoader,
    opengl::{DynamicRenderData, Layout, StaticRenderData},
    viewport::Viewport,
};

#[derive(Debug, Clone)]
pub struct StaticMesh {
    pub name: String,                             // Nametag
    pub handle: MeshHandle,                       // Reference to loaded mesh asset
    pub primitives: Vec<StaticPrimitiveInstance>, // For multi-material meshes

    pub translation: cgmath::Vector3<f32>,
    pub rotation: cgmath::Vector3<f32>, // Later: cgmath::Quaternion<f32>,
    pub scale: cgmath::Vector3<f32>,
}

impl StaticMesh {
    pub fn new(
        context: &glow::Context,
        name: String,
        handle: MeshHandle,
        asset_loader: &AssetLoader,
    ) -> Self {
        let loaded_mesh = asset_loader
            .loaded_mesh_data
            .get(&handle)
            .expect("Mesh handle not found in asset loader");

        let mut primitives = Vec::new();

        for (i, primitive) in loaded_mesh.primitives.iter().enumerate() {
            let layouts = determine_layouts(&primitive.vertex_data);
            let stride = calculate_stride(&layouts);

            let interleaved_vertices = interleave_vertex_data(&primitive.vertex_data);

            let render_data = StaticRenderData::new(
                context,
                &interleaved_vertices,
                &primitive.indices.as_deref().unwrap_or(&[]),
                stride,
                layouts,
            );

            primitives.push(StaticPrimitiveInstance {
                primitive_index: i,
                render_data: Some(render_data),
            });
        }

        StaticMesh {
            name,
            handle,
            primitives,
            translation: cgmath::Vector3::new(0.0, 0.0, 0.0),
            rotation: cgmath::Vector3::new(0.0, 0.0, 0.0),
            scale: cgmath::Vector3::new(1.0, 1.0, 1.0),
        }
    }

    pub fn model_matrix(&self) -> cgmath::Matrix4<f32> {
        cgmath::Matrix4::from_translation(self.translation)
            * cgmath::Matrix4::from_angle_x(cgmath::Rad(self.rotation.x))
            * cgmath::Matrix4::from_angle_y(cgmath::Rad(self.rotation.y))
            * cgmath::Matrix4::from_angle_z(cgmath::Rad(self.rotation.z))
            * cgmath::Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z)
    }

    pub fn render(&self, context: &glow::Context) {
        unsafe {
            for primitive in &self.primitives {
                if let Some(render_data) = &primitive.render_data {
                    render_data.bind(context);

                    if render_data.ebo.is_some() {
                        context.draw_elements(
                            glow::TRIANGLES,
                            render_data.index_count,
                            glow::UNSIGNED_INT,
                            0,
                        );
                    } else {
                        context.draw_arrays(
                            glow::TRIANGLES,
                            0,
                            render_data.vertex_count,
                        );
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct DynamicMesh {
    pub name: String,                             // Nametag
    pub handle: MeshHandle,                       // Reference to loaded mesh asset
    pub primitives: Vec<DynamicPrimitiveInstance>, // For multi-material meshes

    pub translation: cgmath::Vector3<f32>,
    pub rotation: cgmath::Vector3<f32>, // Later: cgmath::Quaternion<f32>,
    pub scale: cgmath::Vector3<f32>,
}

impl DynamicMesh {
    pub fn new(
        context: &glow::Context,
        name: String,
        handle: MeshHandle,
        asset_loader: &AssetLoader,
    ) -> Self {
        let loaded_mesh = asset_loader
            .loaded_mesh_data
            .get(&handle)
            .expect("Mesh handle not found in asset loader");

        let mut primitives = Vec::new();

        for (i, primitive) in loaded_mesh.primitives.iter().enumerate() {
            let layouts = determine_layouts(&primitive.vertex_data);
            let stride = calculate_stride(&layouts);

            let interleaved_vertices = interleave_vertex_data(&primitive.vertex_data);

            let render_data = DynamicRenderData::new(
                context,
                &interleaved_vertices,
                &primitive.indices.as_deref().unwrap_or(&[]),
                stride,
                layouts,
            );

            primitives.push(DynamicPrimitiveInstance {
                primitive_index: i,
                render_data: Some(render_data),
            });
        }

        DynamicMesh {
            name,
            handle,
            primitives,
            translation: cgmath::Vector3::new(0.0, 0.0, 0.0),
            rotation: cgmath::Vector3::new(0.0, 0.0, 0.0),
            scale: cgmath::Vector3::new(1.0, 1.0, 1.0),
        }
    }

    pub fn update_vertices(&mut self, context: &glow::Context, new_vertices: &[f32]) {
        for primitive in &mut self.primitives {
            if let Some(rd) = &mut primitive.render_data {
                rd.update_vertices(context, new_vertices);
            }
        }
    }

    pub fn model_matrix(&self) -> cgmath::Matrix4<f32> {
        cgmath::Matrix4::from_translation(self.translation)
            * cgmath::Matrix4::from_angle_x(cgmath::Rad(self.rotation.x))
            * cgmath::Matrix4::from_angle_y(cgmath::Rad(self.rotation.y))
            * cgmath::Matrix4::from_angle_z(cgmath::Rad(self.rotation.z))
            * cgmath::Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z)
    }

    pub fn render(&self, context: &glow::Context) {
        unsafe {
            for primitive in &self.primitives {
                if let Some(render_data) = &primitive.render_data {
                    render_data.bind(context);

                    if render_data.ebo.is_some() {
                        context.draw_elements(
                            glow::TRIANGLES,
                            render_data.index_count,
                            glow::UNSIGNED_INT,
                            0,
                        );
                    } else {
                        context.draw_arrays(
                            glow::TRIANGLES,
                            0,
                            render_data.vertex_count,
                        );
                    }
                }
            }
        }
    }
}

/*

impl StaticMesh {
    pub fn new(name: String, vertices: Vec<f32>, indices: Vec<u32>) -> Self {
        Self {
            name,
            vertices,
            indices,
            render_data: None,
            translation: cgmath::Vector3::new(0.0, 0.0, 0.0),
            rotation: cgmath::Vector3::new(0.0, 0.0, 0.0),
            scale: cgmath::Vector3::new(1.0, 1.0, 1.0),
        }
    }

    pub fn from_loaded_data(
        context: &glow::Context,
        name: Option<String>,
        vertices: Vec<f32>,
        indices: Vec<u32>,
        layouts: Vec<Layout>,
        stride: i32,
    ) -> Self {
        let name = name.unwrap_or_else(|| "Unnamed".to_string());

        let mut mesh = StaticMesh::new(name, vertices.clone(), indices.clone());

        let render_data = StaticRenderData::new(
            context,
            &vertices,
            &indices,
            stride,
            layouts,
        );

        mesh.set_render_data(render_data);

        mesh
    }

    pub fn set_render_data(&mut self, render_data: StaticRenderData) {
        self.render_data = Some(render_data);
    }

    pub fn render(&self, context: &glow::Context,) {
        unsafe {
            if let Some(rd) = &self.render_data {
                rd.bind(context);
                if rd.ebo.is_some() {
                    context.draw_elements(glow::TRIANGLES, self.indices.len() as i32, glow::UNSIGNED_INT, 0);
                } else {
                    todo!("Static mesh rendering without EBO is not implemented yet");
                    // context.draw_arrays(glow::TRIANGLES, 0, 0);
                }
            } else {
                return;
            }
        }
    }
}

pub struct DynamicMesh {
    pub name: String,
    pub vertices: Vec<f32>,
    pub indices: Vec<u32>,

    pub render_data: Option<DynamicRenderData>,

    // Alternatively pub model_matrix: cgmath::Matrix4<f32>

    pub translation: cgmath::Vector3<f32>,
    pub rotation: cgmath::Vector3<f32>, // Change to cgmath::Quaternion<f32> later,
    pub scale: cgmath::Vector3<f32>,
}

impl DynamicMesh {
    pub fn new(name: String, vertices: Vec<f32>, indices: Vec<u32>) -> Self {
        Self {
            name,
            vertices,
            indices,
            render_data: None,
            translation: cgmath::Vector3::new(0.0, 0.0, 0.0),
            rotation: cgmath::Vector3::new(0.0, 0.0, 0.0),
            scale: cgmath::Vector3::new(1.0, 1.0, 1.0),
        }
    }

    pub fn from_loaded_data(
        context: &glow::Context,
        name: Option<String>,
        vertices: Vec<f32>,
        indices: Vec<u32>,
        layouts: Vec<Layout>,
        stride: i32,
    ) -> Self {
        let name = name.unwrap_or_else(|| "Unnamed".to_string());

        let mut mesh = DynamicMesh::new(name, vertices.clone(), indices.clone());

        let render_data = DynamicRenderData::new(
            context,
            &vertices,
            &indices,
            stride,
            layouts,
        );

        mesh.set_render_data(render_data);

        mesh
    }

    pub fn set_render_data(&mut self, render_data: DynamicRenderData) {
        self.render_data = Some(render_data);
    }

    pub fn render(&self, context: &glow::Context) {
        unsafe {
            if let Some(rd) = &self.render_data {
                rd.bind(context);
                if rd.ebo.is_some() {
                    context.draw_elements(glow::TRIANGLES, self.indices.len() as i32, glow::UNSIGNED_INT, 0);
                } else {
                    todo!("Static mesh rendering without EBO is not implemented yet");
                    // context.draw_arrays(glow::TRIANGLES, 0, 0);
                }
            } else {
                return;
            }
        }
    }
}

*/

pub fn determine_layouts(vertex_data: &VertexData) -> Vec<Layout> {
    let mut layouts = Vec::new();
    let mut offset = 0;

    // Position: always present
    layouts.push(Layout {
        index: 0,
        size: 3,
        gl_type: glow::FLOAT,
        normalized: false,
        offset,
    });
    offset += 3 * std::mem::size_of::<f32>();

    let mut attrib_index = 1;

    if vertex_data.normals.is_some() {
        layouts.push(Layout {
            index: attrib_index,
            size: 3,
            gl_type: glow::FLOAT,
            normalized: false,
            offset,
        });
        offset += 3 * std::mem::size_of::<f32>();
        attrib_index += 1;
    }

    if vertex_data.tangents.is_some() {
        layouts.push(Layout {
            index: attrib_index,
            size: 4,
            gl_type: glow::FLOAT,
            normalized: false,
            offset,
        });
        offset += 4 * std::mem::size_of::<f32>();
        attrib_index += 1;
    }

    for (i, uv) in vertex_data.texcoords.iter().enumerate() {
        layouts.push(Layout {
            index: attrib_index,
            size: 2,
            gl_type: glow::FLOAT,
            normalized: false,
            offset,
        });
        offset += 2 * std::mem::size_of::<f32>();
        attrib_index += 1;
    }

    for (i, color) in vertex_data.colors.iter().enumerate() {
        let size = match color {
            Color::Rgb(_) => 3,
            Color::Rgba(_) => 4,
        };
        layouts.push(Layout {
            index: attrib_index,
            size,
            gl_type: glow::FLOAT,
            normalized: false,
            offset,
        });
        offset += (size as usize * std::mem::size_of::<f32>()) as usize;
        attrib_index += 1;
    }

    if vertex_data.joints.is_some() {
        layouts.push(Layout {
            index: attrib_index,
            size: 4,
            gl_type: glow::UNSIGNED_SHORT,
            normalized: false,
            offset,
        });
        offset += 4 * std::mem::size_of::<u16>();
        attrib_index += 1;
    }

    if vertex_data.weights.is_some() {
        layouts.push(Layout {
            index: attrib_index,
            size: 4,
            gl_type: glow::FLOAT,
            normalized: false,
            offset,
        });
        offset += 4 * std::mem::size_of::<f32>();
    }

    layouts
}

pub fn calculate_stride(layouts: &[Layout]) -> i32 {
    if let Some(last) = layouts.last() {
        let size_in_bytes = match last.gl_type {
            glow::FLOAT => 4,
            glow::UNSIGNED_SHORT => 2,
            _ => panic!("Unsupported attribute type"),
        };
        (last.offset + (last.size as usize * size_in_bytes)) as i32
    } else {
        panic!("No layouts provided to calculate stride!");
    }
}

pub fn interleave_vertex_data(vertex_data: &VertexData) -> Vec<f32> {
    let vertex_count = vertex_data.positions.len();
    let mut interleaved = Vec::with_capacity(vertex_count * 20); // estimate; grows automatically

    for i in 0..vertex_count {
        // Always positions
        interleaved.extend_from_slice(&vertex_data.positions[i]);

        // Optional normals
        if let Some(normals) = &vertex_data.normals {
            interleaved.extend_from_slice(&normals[i]);
        }

        // Optional tangents
        if let Some(tangents) = &vertex_data.tangents {
            interleaved.extend_from_slice(&tangents[i]);
        }

        // Multiple texcoords
        for uv in &vertex_data.texcoords {
            interleaved.extend_from_slice(&uv.0[i]);
        }

        // Multiple colors
        for color in &vertex_data.colors {
            match color {
                Color::Rgb(colors) => interleaved.extend_from_slice(&colors[i]),
                Color::Rgba(colors) => interleaved.extend_from_slice(&colors[i]),
            }
        }

        // For joints + weights: NOTE
        // If you need support for u16 joints, you must make a separate buffer or cast them to f32,
        // because interleaving f32 + u16 into the same VBO is problematic.
        // For now: skip them or error until you handle them properly:
        if vertex_data.joints.is_some() || vertex_data.weights.is_some() {
            panic!("interleave_vertex_data: joints/weights not implemented yet (interleaving u16+f32 needs a different approach)");
        }
    }

    interleaved
}
