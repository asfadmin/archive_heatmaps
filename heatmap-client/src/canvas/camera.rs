use wgpu::util::DeviceExt;

use super::input::InputState;
use super::render_context::RenderContext;

pub enum CameraEvent {
    Resize(u32, u32),
    Translate(cgmath::Vector2<f64>),
    AspectRatio(f64),
    Zoom(f64, cgmath::Vector2<f64>),
}

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
        let camera = Camera {
            aspect: config.width as f64 / config.height as f64,
            width: config.width as f64,
            height: config.height as f64,
            position: (0.0, 0.0).into(),
            zoom: 10.0,
        };

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

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

        CameraContext {
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group_layout,
            camera_bind_group,
        }
    }

    pub fn run_camera_logic(&mut self, input_state: &mut InputState) {
        self.update_camera(CameraEvent::Zoom(
            input_state.consume_scroll_delta() * self.camera.zoom * 0.001,
            (input_state.cursor_position.x, input_state.cursor_position.y).into(),
        ));

        let drag_delta = input_state.consume_drag_delta();
        self.update_camera(CameraEvent::Translate(
            self.mouse_coordinate_convert((drag_delta.x, drag_delta.y).into()),
        ));

        self.rebuild_view_matrix();
    }

    pub fn mouse_coordinate_convert(
        &self,
        mut coordinate: cgmath::Vector2<f64>,
    ) -> cgmath::Vector2<f64> {
        coordinate.x *= -1.0;

        // convert to world scale
        coordinate / self.camera.zoom
    }

    pub fn update_camera(&mut self, camera_event: CameraEvent) {
        match camera_event {
            CameraEvent::Resize(width, height) => {
                let aspect = width as f64 / height as f64;
                self.update_camera(CameraEvent::AspectRatio(aspect));
                self.camera.width = width as f64;
                self.camera.height = height as f64;

                web_sys::console::log_1(&format!("{:?}, {:?}", width, height).into());
            }

            CameraEvent::AspectRatio(aspect) => {
                self.camera.aspect = aspect;
            }

            CameraEvent::Zoom(zoom, mut pos) => {
                let scale_factor = (self.camera.zoom + zoom) / self.camera.zoom;

                self.camera.zoom += zoom;

                pos = self.mouse_coordinate_convert(pos);
                self.update_camera(CameraEvent::Translate(pos - pos * scale_factor));
            }

            CameraEvent::Translate(pos) => {
                self.camera.position += pos;
            }
        }
    }

    pub fn rebuild_view_matrix(&mut self) {
        self.camera_uniform.update_view_proj(&self.camera);
    }

    pub fn write_camera_buffer(&self, render_context: &RenderContext) {
        render_context.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        )
    }
}

pub struct Camera {
    pub aspect: f64,
    pub width: f64,
    pub height: f64,
    pub zoom: f64,
    pub position: cgmath::Vector2<f64>,
}

impl Camera {
    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f64> {
        let view = cgmath::Matrix4::from_scale(self.zoom)
            * cgmath::Matrix4::from_translation(-self.position.extend(0.0));

        let proj = cgmath::ortho(0.0, self.width, -self.height, 0.0, -1.0, 1.0);

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
