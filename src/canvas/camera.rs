use std::f64::consts::PI;

use cgmath::{EuclideanSpace, InnerSpace, Point3, Vector2, Vector3};
use wgpu::util::DeviceExt;

use super::input::InputState;
use super::render_context::RenderContext;

pub enum CameraEvent {
    Resize(u32, u32),
    Translate(cgmath::Vector2<f64>),
    AspectRatio(f64),
    Zoom(f64, cgmath::Vector2<f64>),
    EntireView,
}

// This is the camera that modifies the viewport that the renderpasses render
pub struct CameraContext {
    pub camera: Camera,
    pub camera_uniform: CameraUniform,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
    pub camera_bind_group: wgpu::BindGroup,
}

impl CameraContext {
    pub fn generate_camera_context(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> Self {
        // Create a default camera
        let camera = Camera {
            eye: Point3::new(0.0, 0.0, 50.0),
            target: Point3::new(0.0, 0.0, 0.0),
            up: Vector3::new(0.0, 1.0, 0.0),
            aspect: f64::from(config.width) / f64::from(config.height),
            width: f64::from(config.width),
            height: f64::from(config.height),
            zoom: 1.0,
        };

        // To access the camera from the inside a render pass we create a camera_uniform which is just a matrix!
        let mut camera_uniform = CameraUniform::new();

        // Does some cool Matrix Math to create the view projection matrix!
        camera_uniform.update_view_proj(&camera);

        // We have to store the camera in a uniform buffer so we can access it inside a render pass
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        Self {
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group_layout,
            camera_bind_group,
        }
    }

    pub fn run_camera_logic(&mut self, input_state: &mut InputState) {
        // Updates the zoom of the camera
        self.update_camera(&CameraEvent::Zoom(
            input_state.consume_scroll_delta() * self.camera.zoom * 0.001,
            (input_state.cursor_position.x, input_state.cursor_position.y).into(),
        ));
        // Updates the position of the camera
        let drag_delta = input_state.consume_drag_delta();
        self.update_camera(&CameraEvent::Translate((drag_delta.x, drag_delta.y).into()));

        // Have to recalculate the view projection matrix for these changes to take effect
        self.rebuild_view_matrix();
    }

    pub fn mouse_coordinate_convert(
        &self,
        mut coordinate: cgmath::Vector2<f64>,
    ) -> cgmath::Vector2<f64> {
        // convert to world scale
        coordinate / self.camera.zoom
    }

    pub fn update_camera(&mut self, camera_event: &CameraEvent) {
        match *camera_event {
            CameraEvent::Resize(width, height) => {
                let aspect = f64::from(width) / f64::from(height);
                self.update_camera(&CameraEvent::AspectRatio(aspect));
                self.camera.width = f64::from(width);
                self.camera.height = f64::from(height);
            }

            CameraEvent::AspectRatio(aspect) => {
                self.camera.aspect = aspect;
            }

            // Zooms the camera in and out, ensures the camera stays within the bounds of the heatmap
            CameraEvent::Zoom(mut zoom, mut pos) => {
                self.camera.zoom -= zoom;
                self.camera.zoom = self.camera.zoom.clamp(0.8, 10.0);
                let vec = self.camera.eye.to_vec();

                self.camera.eye =
                    Point3::from_vec(vec.normalize() * (self.camera.zoom * 5.0).log2());
            }

            // Moves the camera around, ensures the camera stays within bounds of the heatmap
            CameraEvent::Translate(diff) => {
                let radius = self.camera.eye.to_vec().magnitude();
                let mut theta = self.camera.eye.z.atan2(self.camera.eye.x);
                let mut phi = (self.camera.eye.y / radius).acos();
                theta += (diff.x / self.camera.width) * PI;
                phi -= (diff.y / self.camera.height) * PI * 0.5;
                let epsilon = 0.05;
                let phi = phi.clamp(epsilon, PI - epsilon);
                self.camera.eye.x = radius * phi.sin() * theta.cos();
                self.camera.eye.y = radius * phi.cos();
                self.camera.eye.z = radius * phi.sin() * theta.sin();
            }

            // Displays the entire heatmap, used to calculate max weight and export to png
            CameraEvent::EntireView => {
                self.camera.zoom = 1.0;

                self.update_camera(&CameraEvent::Resize(1800, 900));

                self.rebuild_view_matrix();
            }
        }
    }

    pub fn rebuild_view_matrix(&mut self) {
        self.camera_uniform.update_view_proj(&self.camera);
    }

    // Store the contents of camera_uniform in the buffer
    pub fn write_camera_buffer(&self, render_context: &RenderContext) {
        render_context.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }
}

#[derive(Clone)]
pub struct Camera {
    pub eye: Point3<f64>,
    pub target: Point3<f64>,
    pub up: Vector3<f64>,
    pub aspect: f64,
    pub width: f64,
    pub height: f64,
    pub zoom: f64,
}

impl Camera {
    // This is the cool matrix math that makes this whole thing actually work!
    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f64> {
        let pos = self.eye;
        let target = self.target;

        let view = cgmath::Matrix4::look_at_rh(pos, target, self.up);

        let proj = cgmath::perspective(cgmath::Deg(90.0), self.aspect, 0.1, 5000.0);

        proj * view
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    fn update_view_proj(&mut self, camera: &Camera) {
        let view_proj_f64: [[f64; 4]; 4] = camera.build_view_projection_matrix().into();

        self.view_proj = view_proj_f64.map(|x| x.map(|y| y as f32));
    }
}
