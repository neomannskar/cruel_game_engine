use cgmath::SquareMatrix;
use egui::Pos2;

#[derive(Debug)]
pub struct PerspectiveCamera {
    pub name: String,

    pub view: cgmath::Matrix4<f32>,
    pub projection: cgmath::Matrix4<f32>,

    pub position: cgmath::Point3<f32>,
    pub orientation: cgmath::Vector3<f32>,
    pub up: cgmath::Vector3<f32>,

    pub fov: f32, // in deg
    pub aspect_ratio: f32,
    pub width: u32,
    pub height: u32,
    pub near_plane: f32,
    pub far_plane: f32,
    pub speed: f32,
    pub sensitivity: f32,
    first_click: bool,
    last_mouse_pos: Pos2,
}

pub trait Camera {
    fn get_view(&self) -> &cgmath::Matrix4<f32>;
    fn get_projection(&self) -> &cgmath::Matrix4<f32>;
    fn update_matrices(&mut self);

    fn get_position(&self) -> cgmath::Point3<f32>;
    fn set_position(&mut self, new: cgmath::Point3<f32>);
    fn get_orientation(&self) -> cgmath::Vector3<f32>;
    fn set_orientation(&mut self, new: cgmath::Vector3<f32>);
    fn get_speed(&self) -> f32;
    fn set_speed(&mut self, new: f32);
    fn get_sensitivity(&self) -> f32;
    fn set_sensitivity(&mut self, new: f32);

    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;

    fn get_up(&self) -> cgmath::Vector3<f32>;
    fn get_first_click(&self) -> bool;
    fn set_first_click(&mut self, value: bool);

    fn get_last_mouse_pos(&self) -> Pos2;
    fn set_last_mouse_pos(&mut self, new: Pos2);
}

impl PerspectiveCamera {
    pub fn new(
        name: String,
        position: cgmath::Point3<f32>,
        fov: f32,
        width: u32,
        height: u32,
        aspect_ratio: f32,
        near_plane: f32,
        far_plane: f32,
        speed: f32,
        sensitivity: f32,
    ) -> Self {
        Self {
            name,

            view: cgmath::Matrix4::identity(),
            projection: cgmath::Matrix4::identity(),

            position,
            orientation: cgmath::vec3(0.0, 0.0, -1.0),
            up: cgmath::vec3(0.0, 1.0, 0.0),

            fov,
            aspect_ratio,

            width,
            height,

            near_plane,
            far_plane,
            speed,
            sensitivity,
            first_click: false,
            last_mouse_pos: Pos2::new(0.0, 0.0),
        }
    }

    pub fn set_fov(&mut self, fov: f32) {
        self.fov = fov;
    }

    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio;
    }

    pub fn set_near_plane(&mut self, near_plane: f32) {
        self.near_plane = near_plane;
    }

    pub fn set_far_plane(&mut self, far_plane: f32) {
        self.far_plane = far_plane;
    }
}

impl Camera for PerspectiveCamera {
    fn get_view(&self) -> &cgmath::Matrix4<f32> {
        &self.view
    }

    fn get_projection(&self) -> &cgmath::Matrix4<f32> {
        &self.projection
    }

    fn update_matrices(&mut self) {
        let view =
            cgmath::Matrix4::look_at_rh(self.position, self.position + self.orientation, self.up);
        self.view = view;
        let proj = cgmath::perspective(
            cgmath::Deg(self.fov),
            self.aspect_ratio,
            self.near_plane,
            self.far_plane,
        );
        self.projection = proj;
    }

    fn get_position(&self) -> cgmath::Point3<f32> {
        self.position
    }

    fn set_position(&mut self, new: cgmath::Point3<f32>) {
        self.position = new
    }

    fn get_orientation(&self) -> cgmath::Vector3<f32> {
        self.orientation
    }

    fn set_orientation(&mut self, new: cgmath::Vector3<f32>) {
        self.orientation = new
    }

    fn get_speed(&self) -> f32 {
        self.speed
    }

    fn set_speed(&mut self, new: f32) {
        self.speed = new
    }

    fn get_up(&self) -> cgmath::Vector3<f32> {
        self.up
    }

    fn get_first_click(&self) -> bool {
        self.first_click
    }

    fn set_first_click(&mut self, value: bool) {
        self.first_click = value
    }

    fn get_sensitivity(&self) -> f32 {
        self.sensitivity
    }

    fn set_sensitivity(&mut self, new: f32) {
        self.sensitivity = new
    }

    fn get_width(&self) -> u32 {
        self.width
    }

    fn get_height(&self) -> u32 {
        self.height
    }

    fn get_last_mouse_pos(&self) -> Pos2 {
        self.last_mouse_pos
    }

    fn set_last_mouse_pos(&mut self, new: Pos2) {
        self.last_mouse_pos = new
    }
}

#[derive(Debug)]
pub struct OrthographicCamera {
    pub name: String,
    pub view: cgmath::Matrix4<f32>,
    pub projection: cgmath::Matrix4<f32>,
    pub position: cgmath::Point3<f32>,
    pub orientation: cgmath::Vector3<f32>,
    pub up: cgmath::Vector3<f32>,

    pub width: u32,
    pub height: u32,
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
    pub near_plane: f32,
    pub far_plane: f32,
    pub speed: f32,
    pub sensitivity: f32,
    first_click: bool,
    last_mouse_pos: Pos2,
}

impl OrthographicCamera {
    pub fn new(
        name: String,
        position: cgmath::Point3<f32>,
        width: u32,
        height: u32,
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near_plane: f32,
        far_plane: f32,
        speed: f32,
        sensitivity: f32,
    ) -> Self {
        Self {
            name,
            view: cgmath::Matrix4::identity(),
            projection: cgmath::Matrix4::identity(),
            position,
            orientation: cgmath::vec3(0.0, 0.0, -1.0),
            up: cgmath::vec3(0.0, 1.0, 0.0),
            width,
            height,
            left: -10.0,
            right: 10.0,
            bottom: -10.0,
            top: 10.0,
            near_plane: 0.1,
            far_plane: 100.0,
            speed: 0.4,
            sensitivity: 100.0,
            first_click: false,
            last_mouse_pos: Pos2::new(0.0, 0.0),
        }
    }
}

impl Camera for OrthographicCamera {
    fn get_view(&self) -> &cgmath::Matrix4<f32> {
        &self.view
    }
    fn get_projection(&self) -> &cgmath::Matrix4<f32> {
        &self.projection
    }
    fn update_matrices(&mut self) {
        self.view =
            cgmath::Matrix4::look_at_rh(self.position, self.position + self.orientation, self.up);
        self.projection = cgmath::ortho(
            self.left,
            self.right,
            self.bottom,
            self.top,
            self.near_plane,
            self.far_plane,
        );
    }

    fn get_position(&self) -> cgmath::Point3<f32> {
        self.position
    }

    fn set_position(&mut self, new: cgmath::Point3<f32>) {
        self.position = new
    }

    fn get_orientation(&self) -> cgmath::Vector3<f32> {
        self.orientation
    }

    fn set_orientation(&mut self, new: cgmath::Vector3<f32>) {
        self.orientation = new
    }

    fn get_speed(&self) -> f32 {
        self.speed
    }

    fn set_speed(&mut self, new: f32) {
        self.speed = new
    }

    fn get_up(&self) -> cgmath::Vector3<f32> {
        self.up
    }

    fn get_first_click(&self) -> bool {
        self.first_click
    }

    fn set_first_click(&mut self, value: bool) {
        self.first_click = value
    }

    fn get_sensitivity(&self) -> f32 {
        self.sensitivity
    }

    fn set_sensitivity(&mut self, new: f32) {
        self.sensitivity = new
    }

    fn get_width(&self) -> u32 {
        self.width
    }

    fn get_height(&self) -> u32 {
        self.height
    }

    fn get_last_mouse_pos(&self) -> Pos2 {
        self.last_mouse_pos
    }

    fn set_last_mouse_pos(&mut self, new: Pos2) {
        self.last_mouse_pos = new
    }
}
