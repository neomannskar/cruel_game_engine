use rayon::prelude::*;
use std::ffi::CString;
use std::num::NonZeroU32;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use egui_glow::Painter;
use glutin::config::ConfigTemplate;
use glutin::context::{ContextAttributesBuilder, PossiblyCurrentContext};
use glutin::display::{Display, DisplayApiPreference};
use glutin::prelude::*;
use glutin::surface::Surface;
use glutin::surface::{SurfaceAttributesBuilder, WindowSurface};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::window::{Window, WindowId};

use egui_winit::State as EguiState;

mod graphics;

mod data;
mod handles;

mod shaders;

mod loader;
use loader::AssetLoader;

mod gui;
use gui::Gui;

mod textures;
use textures::Texture;

mod viewport;
use viewport::Viewport;

mod camera;
use camera::{Camera, PerspectiveCamera};
mod material;
mod mesh;
mod opengl;

mod scene_graph;
use scene_graph::SceneGraph;

use crate::camera::OrthographicCamera;
use crate::loader::{Asset /* AssetHandle */};
use crate::opengl::Layout;
use crate::scene_graph::SceneNode;

#[derive(PartialEq, Clone, Copy)]
enum CameraType {
    Perspective,
    Orthographic,
}

struct Timer {
    last_frame: std::time::Instant,
    delta_time: f64,
}

impl Timer {
    fn new(last_frame_time: std::time::Instant) -> Timer {
        let now = Instant::now();
        let mut timer = Timer {
            last_frame: last_frame_time,
            delta_time: now.duration_since(last_frame_time).as_secs_f64(),
        };

        timer.last_frame = now;

        timer
    }

    fn update(&mut self) {
        let now = Instant::now();
        self.delta_time = now.duration_since(self.last_frame).as_secs_f64();
        self.last_frame = now;
    }

    fn get_delta_time(&self) -> f64 {
        self.delta_time
    }
}

#[derive(Default)]
struct App {
    timer: Option<Timer>,

    window: Option<Window>,
    current_context: Option<PossiblyCurrentContext>,
    surface: Option<Surface<WindowSurface>>,

    asset_loader: Option<Arc<Mutex<AssetLoader>>>,

    context: Option<Arc<glow::Context>>,
    gui: Option<Gui>,
    active_editor_camera_type: Option<CameraType>,
    editor_cameras: Option<(Box<PerspectiveCamera>, Box<OrthographicCamera>)>,
    editor_cameras_updated: Option<bool>,

    scene_graph: Option<SceneGraph>,

    egui_context: Option<egui::Context>,
    egui_painter: Option<Painter>,
    egui_state: Option<EguiState>,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self::default();
        app.asset_loader = Some(Arc::new(Mutex::new(AssetLoader::new())));
        app
    }

    pub fn request_texture<P: AsRef<std::path::Path>>(&self, path: P, name: String) {
        if let Some(asset_loader) = &self.asset_loader {
            asset_loader
                .lock()
                .unwrap()
                .request_texture(path, name);
        } else {
            eprintln!("Asset loader not initialized when requesting texture!");
        }
    }

    pub fn request_textures_parallel(&self, requests: &[(String, String)]) {
        if let Some(asset_loader) = &self.asset_loader {
            let asset_loader = Arc::clone(asset_loader);
            requests.par_iter().for_each(|(path, name)| {
                let loader = asset_loader.lock().unwrap();
                loader.request_texture(path, name.clone());
            });
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create a new window and store it in self.window
        self.window = Some(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );

        let window = self.window.as_ref().unwrap();

        // Get platform-specific handles to the display and window
        let display_handle = window.display_handle().unwrap();
        let window_handle = window.window_handle().unwrap();

        // Create a WGL (Windows OpenGL) display using the handles
        let display = unsafe {
            Display::new(
                display_handle.into(),
                DisplayApiPreference::Wgl(Some(window_handle.into())),
            )
            .expect("Failed to create Wgl display")
        };

        // Create a default OpenGL configuration
        let config_template = ConfigTemplate::default();
        let config = unsafe {
            display
                .find_configs(config_template)
                .unwrap()
                .next()
                .unwrap()
        };

        // Get the window dimensions
        let physical_size = window.inner_size();
        let width = NonZeroU32::new(physical_size.width).unwrap();
        let height = NonZeroU32::new(physical_size.height).unwrap();

        // Create attributes for the window surface
        let surface_attributes = SurfaceAttributesBuilder::<WindowSurface>::build(
            SurfaceAttributesBuilder::new(),
            window_handle.into(),
            width,
            height,
        );

        // Create context attributes (e.g., OpenGL version, flags)
        let context_attributes = ContextAttributesBuilder::new().build(Some(window_handle.into()));

        // Create the OpenGL window surface using the display and attributes
        let surface = unsafe {
            display
                .create_window_surface(&config, &surface_attributes)
                .unwrap()
        };

        // Create a non current OpenGL context
        let non_current_context = unsafe {
            display
                .create_context(&config, &context_attributes)
                .unwrap()
        };

        // Make the context current
        let current_context = non_current_context.make_current(&surface).unwrap();

        surface
            .set_swap_interval(
                &current_context,
                glutin::surface::SwapInterval::Wait(NonZeroU32::new(1).unwrap()),
            )
            .expect("Failed to set vsync");

        // Create the glow context
        let gl = unsafe {
            Arc::new(glow::Context::from_loader_function(|s| {
                let c_str = CString::new(s).unwrap();
                display.get_proc_address(&c_str) as *const _
            }))
        };

        self.surface = Some(surface);
        self.current_context = Some(current_context);
        self.context = Some(gl);

        // self.graphics_example = Some(GraphicsExample::new(self.gl.as_ref().unwrap()));

        let cube_vertices: Vec<f32> = vec![
            // Each: x, y, z, u, v, r, g, b

            // Front face (+Z)
            -1.0, -1.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, // 0
            1.0, -1.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0, // 1
            1.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 1.0, // 2
            -1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 0.0, // 3
            // Back face (-Z)
            1.0, -1.0, -1.0, 0.0, 0.0, 1.0, 0.0, 1.0, // 4
            -1.0, -1.0, -1.0, 1.0, 0.0, 0.0, 1.0, 1.0, // 5
            -1.0, 1.0, -1.0, 1.0, 1.0, 1.0, 1.0, 0.0, // 6
            1.0, 1.0, -1.0, 0.0, 1.0, 0.0, 1.0, 0.5, // 7
            // Left face (-X)
            -1.0, -1.0, -1.0, 0.0, 0.0, 0.5, 0.5, 1.0, // 8
            -1.0, -1.0, 1.0, 1.0, 0.0, 0.5, 1.0, 0.5, // 9
            -1.0, 1.0, 1.0, 1.0, 1.0, 0.5, 0.5, 0.5, //10
            -1.0, 1.0, -1.0, 0.0, 1.0, 0.5, 0.0, 0.5, //11
            // Right face (+X)
            1.0, -1.0, 1.0, 0.0, 0.0, 1.0, 0.5, 0.0, //12
            1.0, -1.0, -1.0, 1.0, 0.0, 1.0, 0.0, 0.5, //13
            1.0, 1.0, -1.0, 1.0, 1.0, 0.0, 0.5, 1.0, //14
            1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 0.0, //15
            // Bottom face (-Y)
            -1.0, -1.0, -1.0, 0.0, 0.0, 1.0, 0.0, 0.0, //16
            1.0, -1.0, -1.0, 1.0, 0.0, 0.0, 1.0, 0.0, //17
            1.0, -1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 1.0, //18
            -1.0, -1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 0.0, //19
            // Top face (+Y)
            -1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.0, //20
            1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 0.0, 1.0, //21
            1.0, 1.0, -1.0, 1.0, 1.0, 1.0, 1.0, 1.0, //22
            -1.0, 1.0, -1.0, 0.0, 1.0, 0.0, 0.0, 0.0, //23
        ];

        let cube_indices: Vec<u32> = vec![
            0, 1, 2, 2, 3, 0, // Front
            4, 5, 6, 6, 7, 4, // Back
            8, 9, 10, 10, 11, 8, // Left
            12, 13, 14, 14, 15, 12, // Right
            16, 17, 18, 18, 19, 16, // Bottom
            20, 21, 22, 22, 23, 20, // Top
        ];

        /* Commented out for now

        let mut cube = StaticMesh::from(
            "Cube".to_string(),
            cube_vertices,
            cube_indices,
        );

        let stride = (8 * std::mem::size_of::<f32>()) as i32;
        let layouts = vec![
            Layout::new(0, 3, glow::FLOAT, false, 0),
            Layout::new(1, 2, glow::FLOAT, false, 3 * std::mem::size_of::<f32>()),
            Layout::new(2, 3, glow::FLOAT, false, 5 * std::mem::size_of::<f32>()),
        ];

        let render_data = StaticRenderData::from(
            self.context.as_ref().unwrap(),
            &cube.vertices,
            &cube.indices,
            stride,
            layouts,
        );

        cube.set_render_data(render_data);
        */

        let scene = SceneNode::new("Main Scene", &self.context.as_ref().unwrap());

        // scene.add_static_mesh(cube);

        let mut asset_loader = self.asset_loader.as_ref().unwrap().lock().unwrap();
        let loaded_assets = asset_loader.poll_loaded();
        for (handle, asset) in loaded_assets {
            match asset {
                Asset::Mesh(loaded_mesh) => {
                    asset_loader
                        .loaded_mesh_data
                        .insert(handle.as_mesh_handle().unwrap(), loaded_mesh);
                }
                Asset::Texture(loaded_texture) => {
                    asset_loader
                        .loaded_texture_data
                        .insert(handle.as_texture_handle().unwrap(), loaded_texture);
                }
                _ => unimplemented!(),
            }
        }

        self.scene_graph = Some(SceneGraph::new());
        self.scene_graph
            .as_mut()
            .unwrap()
            .scenes
            .push(Box::new(scene));

        self.gui = Some(Gui::new());

        self.active_editor_camera_type = Some(CameraType::Perspective);

        self.egui_context = Some(egui::Context::default());
        self.egui_painter = Some(
            Painter::new(self.context.as_ref().unwrap().clone(), "", None, false)
                .expect("Failed to create egui_glow painter"),
        );
        self.egui_state = Some(EguiState::new(
            self.egui_context.as_ref().unwrap().clone(),
            self.egui_context.as_ref().unwrap().viewport_id(),
            &window,
            Some(window.scale_factor() as f32),
            None,
            None,
        ));

        self.editor_cameras = Some((
            Box::new(PerspectiveCamera::new(
                "Editor Perspective Camera".to_string(),
                cgmath::point3(0.0, 0.0, 3.0),
                45.0,
                window.inner_size().width,
                window.inner_size().height,
                (16.0 / 9.0) as f32,
                0.1,
                100.0,
                2.4,
                100.0,
            )),
            Box::new(OrthographicCamera::new(
                "Editor Orthograhic Camera".to_string(),
                cgmath::point3(0.0, 0.0, 3.0),
                window.inner_size().width,
                window.inner_size().height,
                -10.0,
                10.0,
                -10.0,
                10.0,
                0.1,
                100.0,
                2.4,
                100.0,
            )),
        ));

        self.editor_cameras_updated = Some(false);

        // Move to "new" function: self.asset_loader = Some(AssetLoader::new());

        self.timer = Some(Timer::new(Instant::now()));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let window = self.window.as_ref().unwrap();
        window.set_title("Cruel Engine v0.1");

        // give egui any winit events
        _ = self
            .egui_state
            .as_mut()
            .unwrap()
            .on_window_event(window, &event);

        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // Clear the framebuffer
                self.gui
                    .as_ref()
                    .unwrap()
                    .clear(self.context.as_ref().unwrap());

                let active_camera: &mut dyn Camera = match &mut self.editor_cameras {
                    Some((persp, ortho)) => match self.active_editor_camera_type {
                        Some(CameraType::Perspective) => persp.as_mut(),
                        Some(CameraType::Orthographic) => ortho.as_mut(),
                        None => panic!("Editor cameras not initialized!"),
                    },
                    None => panic!("Editor cameras not initialized!"),
                };

                // Run the UI code
                let full_output = self.gui.as_mut().unwrap().update(
                    self.egui_state.as_mut().unwrap().take_egui_input(window),
                    self.egui_context.as_ref().unwrap(),
                    self.active_editor_camera_type.as_mut().unwrap(),
                    active_camera,
                    self.scene_graph.as_mut().unwrap(),
                    self.timer.as_ref().unwrap().delta_time,
                );

                // Handle the platform output (like copy/paste)
                self.egui_state
                    .as_mut()
                    .unwrap()
                    .handle_platform_output(window, full_output.platform_output);

                // Get the triangles from egui's UI
                let clipped_primitives = self
                    .egui_context
                    .as_ref()
                    .unwrap()
                    .tessellate(full_output.shapes, full_output.pixels_per_point);

                // Paint the egui UI
                let physical_size = window.inner_size();
                self.egui_painter
                    .as_mut()
                    .unwrap()
                    .paint_and_update_textures(
                        [physical_size.width, physical_size.height],
                        full_output.pixels_per_point,
                        &clipped_primitives,
                        &full_output.textures_delta,
                    );

                // let v = self.gui.as_ref().unwrap().get_viewport(window).unwrap();
                // self.editor_cameras.as_mut().unwrap().0.fov = (v.width / v.height) as f32;

                let active_camera: &mut dyn Camera = match &mut self.editor_cameras {
                    Some((persp, ortho)) => match self.active_editor_camera_type {
                        Some(CameraType::Perspective) => persp.as_mut(),
                        Some(CameraType::Orthographic) => ortho.as_mut(),
                        None => panic!("Editor cameras not initialized!"),
                    },
                    None => panic!("Editor cameras not initialized!"),
                };

                active_camera.update_matrices();

                // Render the scene
                if let Some(sg) = self.scene_graph.as_mut() {
                    if let Some(scene) = sg.current_scene() {
                        scene.update(active_camera);
                        scene.render(self.context.as_ref().unwrap(), active_camera, &self.gui.as_ref().unwrap().get_viewport(window).expect(
                        "Viewport not present, make sure to update the ui before calling this",
                        ),);
                    }
                }

                self.timer.as_mut().unwrap().update();

                // Swap the frame buffers
                self.surface
                    .as_ref()
                    .unwrap()
                    .swap_buffers(self.current_context.as_ref().unwrap())
                    .unwrap();

                window.request_redraw();
            }
            _ => (),
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.egui_painter.as_mut().unwrap().destroy();
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();

    // ControlFlow::Wait pauses the event loop if no events are available to process.
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();

    // Add entities, components and systems to the app here
    app.request_texture("assets/texture.jpg", "sigma.jpg".to_string());

    // Run the app when behaviour is defined
    event_loop.run_app(&mut app).unwrap();
}
