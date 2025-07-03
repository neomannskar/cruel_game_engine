use std::{io::Write, time::{Duration, Instant}};

use super::Viewport;
use cgmath::{InnerSpace, Rotation3};
use egui::{Align, CornerRadius, Key, Layout, Pos2};
use glow::HasContext;
use winit::window::Window;

#[derive(PartialEq)]
enum Choice {
    Console,
    ContentBrowser,
    Ide,
}

use crate::{camera::Camera, scene_graph::{SceneGraph, SelectedObject}, CameraType};

pub struct Gui {
    choice: Choice,
    terminal_io: (String, String),

    viewport: Option<Viewport>,
    
    frame_count: u32,
    
    accumulator: Duration,
    last_frame_time: Instant,
    fps: u32,

    selected_object: Option<SelectedObject>,
    selected_script: Option<usize>,
    selected_material: Option<usize>,
}

impl Gui {
    pub fn new() -> Self {
        Self {
            choice: Choice::Console,
            terminal_io: (String::new(), String::new()),
            
            viewport: None,
            frame_count: 0,
            accumulator: Duration::ZERO,
            last_frame_time: Instant::now(),
            fps: 0,

            selected_object: Some(SelectedObject::StaticMesh(0)),
            selected_script: None,
            selected_material: None,
        }
    }

    pub fn clear(&self, context: &glow::Context) {
        unsafe {
            context.clear_color(0.0, 0.0, 0.0, 1.0);
            context.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
        }
    }

    pub fn get_viewport(&self, window: &Window) -> Option<Viewport> {
        if let Some(viewport) = &self.viewport {
            let window_height = window.inner_size().height;
            Some(Viewport::new(
                viewport.x,
                // Reverse the y since OpenGL uses a different origin
                window_height as i32 - viewport.y - viewport.height,
                viewport.width,
                viewport.height,
            ))
        } else {
            None
        }
    }

    pub fn update(&mut self, raw_input: egui::RawInput, ctx: &egui::Context, active_camera_type: &mut CameraType, camera: &mut dyn Camera, scene_graph: &mut SceneGraph, delta_time: f64) -> egui::FullOutput {
        // Calculate the delta time
        let now = Instant::now();
        let dt = now - self.last_frame_time;
        self.last_frame_time = now;

        // Update the time accumulator and frame count
        self.accumulator += dt;
        self.frame_count += 1;

        // If 0.1 seconds have passed then update the fps indicator
        if self.accumulator >= Duration::from_secs_f32(0.1) {
            self.fps = (self.frame_count as f32 / self.accumulator.as_secs_f32()) as u32;
            self.accumulator = Duration::ZERO;
            self.frame_count = 0;
        }

        let current_scene = scene_graph.current_scene().unwrap();

        ctx.run(raw_input, |ctx| {
            egui::SidePanel::left("Hierarchy")
                .min_width(150.0)
                .resizable(true)
                .show(ctx, |ui| {
                    ui.collapsing(current_scene.name.clone(), |ui| {
                        ui.collapsing("Static Meshes", |ui| {
                            for (i, sm) in current_scene.static_meshes.iter().enumerate() {
                                if ui.button(sm.name.clone()).clicked() {
                                    self.selected_object = Some(SelectedObject::StaticMesh(i))
                                }
                            }
                        });

                        ui.collapsing("Dynamic Meshes", |ui| {
                            for sm in &current_scene.dynamic_meshes {
                                ui.label(sm.name.clone());
                            }
                        });

                        ui.collapsing("Perspective Cameras", |ui| {
                            for sm in &current_scene.perspective_cameras {
                                ui.label(sm.name.clone());
                            }
                        });

                        ui.collapsing("Textures", |ui| {
                            for t in &current_scene.textures {
                                ui.label(t.name.clone());
                            }
                        });

                        ui.collapsing("Materials", |ui| {
                            for m in &current_scene.materials {
                                ui.label(m.name.clone());
                            }
                        });

                        ui.collapsing("Scripts", |ui| {
                            for s in &current_scene.scripts {
                                ui.label(s.clone());
                            }
                        });
                    });
                });

            egui::TopBottomPanel::bottom("Bottom panel")
                .min_height(105.0)
                .resizable(true)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.visuals_mut().widgets.inactive.corner_radius = CornerRadius::same(0);
                        ui.visuals_mut().widgets.hovered.corner_radius = CornerRadius::same(5);
                        ui.visuals_mut().widgets.active.corner_radius = CornerRadius::same(5);
                        ui.selectable_value(&mut self.choice, Choice::Console, "Console");
                        ui.selectable_value(&mut self.choice, Choice::ContentBrowser, "Content Browser");
                        if let Some(script) = self.selected_script{
                            let mut x = current_scene.scripts.get(script).unwrap().clone();
                            x.push_str(".rs");
                            ui.selectable_value(&mut self.choice, Choice::Ide, x);
                        
                        } else {
                            ui.selectable_value(&mut self.choice, Choice::Ide, "IDE");
                        }
                    });

                    ui.separator();

                    if self.choice == Choice::Console {
                        use egui::{ScrollArea, TextEdit, Key};

                        // Output area: scrollable multiline, read-only
                        ScrollArea::vertical()
                            .max_height(100.0)
                            .auto_shrink([false; 2])
                            .stick_to_bottom(true)
                            .show(ui, |ui| {
                                ui.set_min_width(ui.available_width());
                                ui.add(
                                    TextEdit::multiline(&mut self.terminal_io.1)
                                        .font(egui::TextStyle::Monospace)
                                        .code_editor()
                                        .desired_width(ui.available_width())
                                        .desired_rows(10)
                                        .interactive(false), // read-only
                                );
                            });

                        // Input area: single-line editable input
                        let input = &mut self.terminal_io.0;
                        let enter_pressed = ui
                            .add(TextEdit::singleline(input).hint_text("Enter command"))
                            .lost_focus()
                            && ui.input(|i| i.key_pressed(Key::Enter));

                        if enter_pressed {
                            let command = input.trim();
                            if !command.is_empty() {
                                // Append prompt + command to output
                                self.terminal_io.1.push_str(&format!("> {}\n", command));

                                // TODO: process the command, for now just echo back:
                                self.terminal_io.1.push_str(&format!("You typed: {}\n", command));

                                input.clear();
                            }
                        }
                    } else if self.choice == Choice::Ide {
                        use egui::TextEdit;

                        if self.selected_script == None {
                            let mut file_content = String::from("fn main() {\n    println!(\"Hello World!\");\n}");
                            ui.add(
                            TextEdit::multiline(&mut file_content)
                                .font(egui::TextStyle::Monospace)
                                .code_editor()
                                .desired_width(ui.available_width())
                                .desired_rows(20),
                            );

                            // Save button
                            if ui.button("Save").clicked() {
                                match std::fs::File::create_new("scripts/script1.rs") {
                                    Ok(mut file) => {
                                        file.write_all(file_content.as_bytes()).unwrap();
                                        self.terminal_io.1.push_str("Saving script!");
                                        println!("Saving script!");
                                    },
                                    Err(e) => {
                                        eprintln!("Error: {}", e);
                                    },
                                }
                            }
                        } else {
                            let script_path = current_scene.scripts.get(self.selected_script.unwrap().clone()).unwrap();
                            let mut file_content = std::fs::read_to_string(script_path).unwrap();
                            ui.add(
                            TextEdit::multiline(&mut file_content)
                                .font(egui::TextStyle::Monospace)
                                .code_editor()
                                .desired_width(ui.available_width())
                                .desired_rows(20),
                            );

                            // Save button
                            if ui.button("Save").clicked() {
                                let mut f = std::fs::File::create(script_path).unwrap();
                                f.write_all(file_content.as_bytes()).unwrap();
                                self.terminal_io.1.push_str("Saving script!");
                                println!("Saving script!"); // : \n{}", self.ide_content);
                                // TODO: write to disk
                            }
                        }
                    } else {
                        ui.heading("Content Browser");

                        ui.horizontal(|ui| {
                            ui.add(
                                egui::Image::new(egui::include_image!("../assets/texture.jpg"))
                                    .max_width(200.0)
                                    .corner_radius(10),
                            );
                        });
                    }

                    // To allow for resizing
                    ui.allocate_space(ui.available_size());
                });

            egui::SidePanel::right("Properties")
                .min_width(220.0)
                .resizable(true)
                .show(ctx, |ui| {
                    if let Some(selected) = &mut self.selected_object {
                        match selected {
                            SelectedObject::StaticMesh(index) => {   
                                let mesh = current_scene.static_meshes.get_mut(*index).expect("Static mesh not found");
                                
                                ui.label(format!("Selected Static Mesh: {}", index));
                                ui.horizontal(|ui| {
                                    ui.label("Name");
                                    // Adds space between the text and input
                                    ui.allocate_ui_with_layout(
                                        ui.available_size(),
                                        Layout::right_to_left(Align::Center),
                                        |ui| {
                                            ui.text_edit_singleline(&mut mesh.name);
                                        },
                                    );
                                });

                                ui.heading("Transform");

                                ui.horizontal(|ui| {
                                    ui.label("Translate");
                                    // Adds space between the text and inputs
                                    ui.allocate_ui_with_layout(
                                        ui.available_size(),
                                        Layout::right_to_left(Align::Center),
                                        |ui| {
                                            // The inputs are in the reverse order
                                            ui.add(egui::DragValue::new(&mut mesh.translation.z).speed(0.05));
                                            ui.add(egui::DragValue::new(&mut mesh.translation.y).speed(0.05));
                                            ui.add(egui::DragValue::new(&mut mesh.translation.x).speed(0.05));
                                        },
                                    );
                                });
                                
                                ui.horizontal(|ui| {
                                    ui.label("Rotate");
                                    // Adds space between the text and inputs
                                    ui.allocate_ui_with_layout(
                                        ui.available_size(),
                                        Layout::right_to_left(Align::Center),
                                        |ui| {
                                            // The inputs are in the reverse order
                                            ui.add(egui::DragValue::new(&mut mesh.rotation.z).speed(1.0));
                                            ui.add(egui::DragValue::new(&mut mesh.rotation.y).speed(1.0));
                                            ui.add(egui::DragValue::new(&mut mesh.rotation.x).speed(1.0));
                                        },
                                    );
                                });

                                ui.horizontal(|ui| {
                                    ui.label("Scale");
                                    // Adds space between the text and inputs
                                    ui.allocate_ui_with_layout(
                                        ui.available_size(),
                                        Layout::right_to_left(Align::Center),
                                        |ui| {
                                            // The inputs are in the reverse order
                                            ui.add(egui::DragValue::new(&mut mesh.scale.z).speed(0.01));
                                            ui.add(egui::DragValue::new(&mut mesh.scale.y).speed(0.01));
                                            ui.add(egui::DragValue::new(&mut mesh.scale.x).speed(0.01));
                                        },
                                    );
                                });
                            }
                            SelectedObject::DynamicMesh(index) => {
                                ui.label(format!("Selected Dynamic Mesh: {}", index));
                            }
                            SelectedObject::PerspectiveCamera(index) => {
                                ui.label(format!("Selected Perspective Camera: {}", index));
                            }
                            // Add more cases as needed
                        }
                    } else {
                        ui.label("No object selected");
                    }
                });

            egui::CentralPanel::default().show(ctx, |ui| {
                egui::TopBottomPanel::top("Toolbar")
                    .resizable(false)
                    .show_inside(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Tools:");

                            if ui.button("▶ Play").clicked() {
                                println!("Todo!");
                            }

                            ui.menu_button("Add", |ui| {
                                if ui.button("Static Mesh").clicked() {
                                    // current_scene.add_static_mesh();

                                    self.terminal_io.1.push_str("Add Static Mesh!");
                                    println!("Add Static Mesh!");
                                    ui.close_menu();
                                }
                                if ui.button("Dynamic Mesh").clicked() {
                                    // current_scene.add_static_mesh();

                                    self.terminal_io.1.push_str("Add Dynamic Mesh!");
                                    println!("Add Dynamic Mesh!");
                                    ui.close_menu();
                                }
                                if ui.button("Perspective Camera").clicked() {
                                    println!("Add Perspective Camera!");
                                    ui.close_menu();
                                }
                                if ui.button("Light").clicked() {
                                    println!("Add Light!");
                                    ui.close_menu();
                                }
                            });

                            if ui.button("Perspective").clicked() {
                                *active_camera_type = CameraType::Perspective;
                            }
                            if ui.button("Orthographic").clicked() {
                                *active_camera_type = CameraType::Orthographic;
                            }
                        });
                    });
                
                ui.input(|input| {
                    if input.key_pressed(egui::Key::W) {
                        self.terminal_io.1.push_str("W: Pressed\n");
                        camera.set_position(camera.get_position() + camera.get_speed() * camera.get_orientation() * delta_time as f32);
                    }
                    if input.key_pressed(egui::Key::A) {
                        self.terminal_io.1.push_str("A: Pressed\n");
                        camera.set_position(camera.get_position() + camera.get_speed() * -cgmath::Vector3::normalize(cgmath::Vector3::cross(camera.get_orientation(), camera.get_up())) * delta_time as f32);
                    }
                    if input.key_pressed(egui::Key::S) {
                        self.terminal_io.1.push_str("S: Pressed\n");
                        camera.set_position(camera.get_position() + camera.get_speed() * -camera.get_orientation() * delta_time as f32);
                    }
                    if input.key_pressed(egui::Key::D) {
                        self.terminal_io.1.push_str("D: Pressed\n");
                        camera.set_position(camera.get_position() + camera.get_speed() * cgmath::Vector3::normalize(cgmath::Vector3::cross(camera.get_orientation(), camera.get_up())) * delta_time as f32);
                    }
                    if input.key_pressed(egui::Key::Space) {
                        self.terminal_io.1.push_str("Space: Pressed\n");
                        camera.set_position(camera.get_position() + camera.get_speed() * camera.get_up() * delta_time as f32);
                    }
                    if input.key_pressed(egui::Key::ArrowDown) {
                        self.terminal_io.1.push_str("ArrowDown: Pressed\n");
                        camera.set_position(camera.get_position() + camera.get_speed() * -camera.get_up() * delta_time as f32);
                    }
                    if input.pointer.button_down(egui::PointerButton::Primary) {
                        self.terminal_io.1.push_str("Mouse button 1: Pressed\n");

                        if camera.get_first_click() {
                            if let Some(pos) = input.pointer.hover_pos() {
                                camera.set_last_mouse_pos(pos); // store initial pos
                            }
                            camera.set_first_click(false);
                        }

                        if let Some(pos) = input.pointer.hover_pos() {
                            // Calculate delta since last frame
                            let delta_x = pos.x - camera.get_last_mouse_pos().x;
                            let delta_y = pos.y - camera.get_last_mouse_pos().y;

                            let rot_x = camera.get_sensitivity() * (delta_y as f32) / camera.get_height() as f32;
                            let rot_y = camera.get_sensitivity() * (delta_x as f32) / camera.get_width() as f32;

                            let right = camera.get_orientation().cross(camera.get_up()).normalize();
                            let pitch_quat = cgmath::Quaternion::from_axis_angle(right, cgmath::Deg(-rot_x));

                            let new_orientation = pitch_quat * camera.get_orientation();

                            let up_dot = new_orientation.dot(camera.get_up());
                            if up_dot.abs() < 0.99 {
                                camera.set_orientation(new_orientation);
                            }

                            let yaw_quat = cgmath::Quaternion::from_axis_angle(camera.get_up(), cgmath::Deg(-rot_y));
                            camera.set_orientation(yaw_quat * camera.get_orientation());

                            // Update last mouse pos
                            camera.set_last_mouse_pos( pos);
                        }
                    } else {
                        camera.set_first_click(true);
                    }
                    
                });

                ui.horizontal(|ui| {
                    ui.heading(current_scene.name.clone());
                    ui.hyperlink_to("Cruel Engine homepage", "https://www.cruelengine.com");
                    ui.allocate_ui_with_layout(
                        ui.available_size(),
                        Layout::right_to_left(Align::Center),
                        |ui| {
                            ui.label(format!("FPS: {}", self.fps));
                        },
                    );
                });

                let rect = ui.max_rect();
                let (x, y) = rect.min.into();
                let (width, height) = rect.size().into();

                let pixels_per_point = ctx.pixels_per_point();

                // Set the viewport which the custom graphics will render in
                self.viewport = Some(Viewport::new(
                    (x * pixels_per_point) as i32,
                    (y * pixels_per_point) as i32,
                    (width * pixels_per_point) as i32,
                    (height * pixels_per_point) as i32,
                ));
            });
        })
    }
}
