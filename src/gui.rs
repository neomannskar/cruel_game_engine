use std::{
    collections::VecDeque,
    io::Write,
    time::{Duration, Instant},
};

use super::Viewport;
use cgmath::{InnerSpace, Rotation3};
use crossbeam_channel::{unbounded, Receiver, Sender};
use egui::{Align, CornerRadius, Key, Layout, Pos2};
use glow::HasContext;
use winit::window::Window;

use clap::{Arg, Command};
use shell_words;

fn process_console_command(command: String) -> String {
    // Tokenize command line (split on whitespace) for clap
    let args = shell_words::split(&command).unwrap_or_else(|_| vec![]);

    let cli = Command::new("console")
        .subcommand(
            Command::new("echo")
                .about("Prints text")
                .arg(Arg::new("text").required(true).num_args(1..)),
        )
        .subcommand(
            Command::new("add")
                .about("Adds two numbers")
                .arg(Arg::new("a").required(true))
                .arg(Arg::new("b").required(true)),
        );

    match cli.try_get_matches_from(args) {
        Ok(matches) => match matches.subcommand() {
            Some(("echo", sub)) => {
                let text: Vec<_> = sub
                    .get_many::<String>("text")
                    .unwrap()
                    .map(|s| s.as_str())
                    .collect();
                format!("{}", text.join(" "))
            }
            Some(("add", sub)) => {
                let a: f64 = sub.get_one::<String>("a").unwrap().parse().unwrap_or(0.0);
                let b: f64 = sub.get_one::<String>("b").unwrap().parse().unwrap_or(0.0);
                format!("Result: {}", a + b)
            }
            _ => "Unknown command or syntax error".to_string(),
        },
        Err(e) => format!("Error parsing command: {}", e),
    }
}

#[derive(PartialEq)]
enum Choice {
    Console,
    ContentBrowser,
    Ide,
}

use crate::{
    camera::Camera, loader::AssetLoader, mesh::StaticMesh, world::{SceneGraph, SelectedObject}, CameraType
};

pub struct Gui {
    command_tx: Sender<String>,
    command_result_rx: Receiver<String>,

    mouse_delta_accumulator: Option<(f32, f32)>,

    choice: Choice,
    wireframe: bool,

    terminal_input: String,
    terminal_lines: VecDeque<String>,
    max_terminal_lines: usize,

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
        let (command_tx, command_rx) = unbounded();
        let (result_tx, command_result_rx) = unbounded();

        let gui = Self {
            command_tx,
            command_result_rx,

            mouse_delta_accumulator: None,

            choice: Choice::Console,
            wireframe: false,
            terminal_input: String::new(),
            terminal_lines: VecDeque::new(),
            max_terminal_lines: 100,

            viewport: None,
            frame_count: 0,
            accumulator: Duration::ZERO,
            last_frame_time: Instant::now(),
            fps: 0,

            selected_object: None, // Some(SelectedObject::StaticMesh(0)),
            selected_script: None,
            selected_material: None,
        };

        std::thread::spawn(move || {
            while let Ok(command) = command_rx.recv() {
                // Here you process the command:
                let output = process_console_command(command);
                let _ = result_tx.send(output);
            }
        });

        gui
    }

    fn append_terminal(&mut self, text: impl Into<String>) {
        self.terminal_lines.push_back(text.into());
        while self.terminal_lines.len() > self.max_terminal_lines {
            self.terminal_lines.pop_front();
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

    pub fn update(
        &mut self,
        raw_input: egui::RawInput,
        ctx: &egui::Context,
        context: &glow::Context,
        active_camera_type: &mut CameraType,
        camera: &mut dyn Camera,
        scene_graph: &mut SceneGraph,
        asset_loader: &AssetLoader,
        delta_time: f64,
    ) -> egui::FullOutput {
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

        let current_scene = scene_graph.current_scene_mut().unwrap();

        while let Ok(line) = self.command_result_rx.try_recv() {
            self.append_terminal(line);
        }

        if let Some((mut acc_dx, mut acc_dy)) = self.mouse_delta_accumulator.take() {
            // Clamp the maximum delta movement so camera doesn't spin uncontrollably
            let max_delta = 75.0; // adjust as needed for your sensitivity
            acc_dx = acc_dx.clamp(-max_delta, max_delta);
            acc_dy = acc_dy.clamp(-max_delta, max_delta);
                    
            let rot_x = camera.get_sensitivity() * (acc_dy as f32) / camera.get_height() as f32;
            let rot_y = camera.get_sensitivity() * (acc_dx as f32) / camera.get_width() as f32;

            let right = camera.get_orientation().cross(camera.get_up()).normalize();
            let pitch_quat = cgmath::Quaternion::from_axis_angle(right, cgmath::Deg(-rot_x));

            let new_orientation = pitch_quat * camera.get_orientation();

            let up_dot = new_orientation.dot(camera.get_up());
            if up_dot.abs() < 0.99 {
                camera.set_orientation(new_orientation);
            }

            let yaw_quat = cgmath::Quaternion::from_axis_angle(
                camera.get_up(),
                cgmath::Deg(-rot_y),
            );
            camera.set_orientation(yaw_quat * camera.get_orientation());
        }

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
                        ui.selectable_value(
                            &mut self.choice,
                            Choice::ContentBrowser,
                            "Content Browser",
                        );
                        if let Some(script) = self.selected_script {
                            let mut x = current_scene.scripts.get(script).unwrap().clone();
                            x.push_str(".rs");
                            ui.selectable_value(&mut self.choice, Choice::Ide, x);
                        } else {
                            ui.selectable_value(&mut self.choice, Choice::Ide, "IDE");
                        }
                    });

                    ui.separator();

                    if self.choice == Choice::Console {
                        use egui::{Key, ScrollArea, TextEdit};

                        // Output area: scrollable multiline, read-only
                        ScrollArea::vertical()
                            .max_height(100.0)
                            .auto_shrink([false; 2])
                            .stick_to_bottom(true)
                            .show(ui, |ui| {
                                ui.set_min_width(ui.available_width());
                                for line in &self.terminal_lines {
                                    ui.monospace(line);
                                }
                            });

                        // Input area: single-line editable input
                        let enter_pressed = {
                            let input = &mut self.terminal_input;
                            ui.add(TextEdit::singleline(input).hint_text("Enter command"))
                                .lost_focus()
                                && ui.input(|i: &egui::InputState| i.key_pressed(Key::Enter))
                        };

                        if enter_pressed {
                            let mut input = self.terminal_input.clone();
                            let command = input.trim();
                            if !command.is_empty() {
                                self.append_terminal(format!("> {}", command));
                                let _ = self.command_tx.send(command.to_string());
                                input.clear();
                            }
                        }
                    } else if self.choice == Choice::Ide {
                        use egui::TextEdit;

                        if self.selected_script == None {
                            let mut file_content =
                                String::from("fn main() {\n    println!(\"Hello World!\");\n}");
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
                                        self.append_terminal("Saving script ...");
                                        match file.write_all(file_content.as_bytes()) {
                                            Ok(_) => {
                                                self.append_terminal("Saved script!");
                                            }
                                            Err(e) => {
                                                self.append_terminal(format!(
                                                    "ERROR: Failed to Saved script!\n\t{}",
                                                    e
                                                ));
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("Error: {}", e);
                                    }
                                }
                            }
                        } else {
                            let script_path = current_scene
                                .scripts
                                .get(self.selected_script.unwrap().clone())
                                .unwrap();
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
                                let path = script_path.clone();
                                let data = file_content.clone();
                                rayon::spawn(move || {
                                    if let Err(e) = std::fs::write(&path, data) {
                                        eprintln!("Error saving {}: {}", path, e);
                                    } else {
                                        println!("Saved script: {}", path);
                                    }
                                });
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
                                let mesh = current_scene
                                    .static_meshes
                                    .get_mut(*index)
                                    .expect("Static mesh not found");

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
                                            ui.add(
                                                egui::DragValue::new(&mut mesh.translation.z)
                                                    .speed(0.05),
                                            );
                                            ui.add(
                                                egui::DragValue::new(&mut mesh.translation.y)
                                                    .speed(0.05),
                                            );
                                            ui.add(
                                                egui::DragValue::new(&mut mesh.translation.x)
                                                    .speed(0.05),
                                            );
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
                                            ui.add(
                                                egui::DragValue::new(&mut mesh.rotation.z)
                                                    .speed(1.0),
                                            );
                                            ui.add(
                                                egui::DragValue::new(&mut mesh.rotation.y)
                                                    .speed(1.0),
                                            );
                                            ui.add(
                                                egui::DragValue::new(&mut mesh.rotation.x)
                                                    .speed(1.0),
                                            );
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
                                            ui.add(
                                                egui::DragValue::new(&mut mesh.scale.z).speed(0.01),
                                            );
                                            ui.add(
                                                egui::DragValue::new(&mut mesh.scale.y).speed(0.01),
                                            );
                                            ui.add(
                                                egui::DragValue::new(&mut mesh.scale.x).speed(0.01),
                                            );
                                        },
                                    );
                                });
                            }
                            SelectedObject::DynamicMesh(index) => {
                                ui.label(format!("Selected Dynamic Mesh: {}", index));
                            }
                            SelectedObject::PerspectiveCamera(index) => {
                                ui.label(format!("Selected Perspective Camera: {}", index));
                            } // Add more cases as needed
                        }
                    } else {
                        ui.label("No object selected");
                    }
                });

            egui::CentralPanel::default().show(ctx, |ui| {
                ui.input(|input| {
                    if input.pointer.button_down(egui::PointerButton::Primary) {
                        if camera.get_first_click() {
                            if let Some(pos) = input.pointer.hover_pos() {
                                camera.set_last_mouse_pos(pos);
                            }
                            camera.set_first_click(false);
                        }

                        if let Some(pos) = input.pointer.hover_pos() {
                            let dx = pos.x - camera.get_last_mouse_pos().x;
                            let dy = pos.y - camera.get_last_mouse_pos().y;

                            // Accumulate deltas
                            self.mouse_delta_accumulator = Some(
                                self.mouse_delta_accumulator
                                    .map(|(acc_dx, acc_dy)| (acc_dx + dx, acc_dy + dy))
                                    .unwrap_or((dx, dy)),
                            );

                            // Store last mouse pos for next delta calculation
                            camera.set_last_mouse_pos(pos);
                        }
                    } else {
                        camera.set_first_click(true);
                    }
                    
                    if input.key_down(egui::Key::W) {
                        camera.set_position(
                            camera.get_position()
                                + camera.get_speed() * camera.get_orientation() * delta_time as f32,
                        );
                    }
                    if input.key_down(egui::Key::A) {
                        camera.set_position(
                            camera.get_position()
                                + camera.get_speed()
                                    * -cgmath::Vector3::normalize(cgmath::Vector3::cross(
                                        camera.get_orientation(),
                                        camera.get_up(),
                                    ))
                                    * delta_time as f32,
                        );
                    }
                    if input.key_down(egui::Key::S) {
                        camera.set_position(
                            camera.get_position()
                                + camera.get_speed()
                                    * -camera.get_orientation()
                                    * delta_time as f32,
                        );
                    }
                    if input.key_down(egui::Key::D) {
                        camera.set_position(
                            camera.get_position()
                                + camera.get_speed()
                                    * cgmath::Vector3::normalize(cgmath::Vector3::cross(
                                        camera.get_orientation(),
                                        camera.get_up(),
                                    ))
                                    * delta_time as f32,
                        );
                    }
                    if input.key_down(egui::Key::Space) {
                        camera.set_position(
                            camera.get_position()
                                + camera.get_speed() * camera.get_up() * delta_time as f32,
                        );
                    }
                    if input.key_down(egui::Key::ArrowDown) {
                        camera.set_position(
                            camera.get_position()
                                + camera.get_speed() * -camera.get_up() * delta_time as f32,
                        );
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

                egui::TopBottomPanel::bottom("Toolbar")
                    .resizable(false)
                    .show_inside(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Tools:");

                            if ui.button("▶ Play").clicked() {
                                println!("Todo!");
                            }

                            ui.menu_button("Add", |ui| {
                                ui.menu_button("Mesh", |ui| {
                                    ui.menu_button("Static Mesh", |ui| {
                                        for (handle, loaded_mesh) in &asset_loader.loaded_mesh_data {
                                            let mesh_name = loaded_mesh.name.as_str();

                                            if ui.button(mesh_name).clicked() {
                                                let static_mesh = StaticMesh::new(
                                                    context,
                                                    mesh_name.to_string(),
                                                    *handle,
                                                    asset_loader,
                                                );

                                                current_scene.add_static_mesh(static_mesh);

                                                self.append_terminal(format!("Added Static Mesh: {}", mesh_name));
                                                ui.close_menu();
                                            }
                                        }
                                    });


                                    if ui.button("Static Mesh").clicked() {
                                        // current_scene.add_static_mesh();
                                        self.append_terminal("Add Static Mesh ... TODO");

                                        

                                        ui.close_menu();
                                    }
                                    if ui.button("Dynamic Mesh").clicked() {
                                        // current_scene.add_static_mesh();

                                        self.append_terminal("Add Dynamic Mesh ... TODO");
                                        ui.close_menu();
                                    }
                                });

                                ui.menu_button("Camera", |ui| {
                                    if ui.button("Perspective Camera").clicked() {
                                        self.append_terminal("Add Perspective Camera!");
                                        ui.close_menu();
                                    }
                                    if ui.button("Orthographic Camera").clicked() {
                                        self.append_terminal("Add Orthographic Camera!");
                                        ui.close_menu();
                                    }
                                });

                                ui.menu_button("Light", |ui| {
                                    if ui.button("Point Light").clicked() {
                                        self.append_terminal("Add Point Light!");
                                        ui.close_menu();
                                    }

                                    if ui.button("Spot Light").clicked() {
                                        self.append_terminal("Add Ambient Light!");
                                        ui.close_menu();
                                    }

                                    if ui.button("Ambient Light").clicked() {
                                        self.append_terminal("Add Ambient Light!");
                                        ui.close_menu();
                                    }
                                });
                            });

                            if ui.button("Perspective").clicked() {
                                *active_camera_type = CameraType::Perspective;
                            }
                            if ui.button("Orthographic").clicked() {
                                *active_camera_type = CameraType::Orthographic;
                            }
                        });

                        ui.checkbox(&mut self.wireframe, "Wireframe");

                        if self.wireframe {
                            unsafe {
                                context.polygon_mode(glow::FRONT_AND_BACK, glow::LINE);
                            }
                        } else {
                            unsafe {
                                context.polygon_mode(glow::FRONT_AND_BACK, glow::FILL);
                            }
                        }
                    });
            });
        })
    }
}
