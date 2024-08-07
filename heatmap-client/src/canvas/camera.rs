use cgmath::Vector2;
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
            aspect: config.width as f64 / config.height as f64,
            width: config.width as f64,
            height: config.height as f64,
            position: (0.0, 0.0).into(),
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

        CameraContext {
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group_layout,
            camera_bind_group,
        }
    }

    pub fn run_camera_logic(&mut self, input_state: &mut InputState) {
        // Updates the zoom of the camera
        self.update_camera(CameraEvent::Zoom(
            input_state.consume_scroll_delta() * self.camera.zoom * 0.001,
            (input_state.cursor_position.x, input_state.cursor_position.y).into(),
        ));
        // Updates the position of the camera
        let drag_delta = input_state.consume_drag_delta();
        self.update_camera(CameraEvent::Translate(
            self.mouse_coordinate_convert((drag_delta.x, drag_delta.y).into()),
        ));

        // Have to recalculate the view projection matrix for these changes to take effect
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
            }

            CameraEvent::AspectRatio(aspect) => {
                self.camera.aspect = aspect;
            }

            // Zooms the camera in and out, ensures the camera stays within the bounds of the heatmap
            CameraEvent::Zoom(mut zoom, mut pos) => {
                let camera_size =
                    cgmath::Vector2::<f64>::new(self.camera.width, self.camera.height)
                        / (self.camera.zoom + zoom);

                if camera_size.x > 360.0 {
                    zoom = self.camera.width / 360.0 - self.camera.zoom;
                }

                if camera_size.y > 170.0 {
                    zoom = zoom.max(self.camera.height / 170.0 - self.camera.zoom);
                }

                if self.camera.zoom + zoom < 0.0 {
                    zoom = -self.camera.zoom + 0.001;
                }

                let scale_factor = (self.camera.zoom + zoom) / self.camera.zoom;

                self.camera.zoom += zoom;
                pos = self.mouse_coordinate_convert(pos);
                self.update_camera(CameraEvent::Translate(pos - pos * scale_factor));
            }

            // Moves the camera around, ensures the camera stays within bounds of the heatmap
            CameraEvent::Translate(mut pos) => {
                let camera_upper_bounds: cgmath::Vector2<f64> = self.camera.position
                    + pos
                    + cgmath::Vector2::<f64>::new(
                        self.camera.width / self.camera.zoom,
                        -self.camera.height / self.camera.zoom,
                    );

                let camera_lower_bounds: cgmath::Vector2<f64> = self.camera.position + pos;

                if camera_upper_bounds.x > 180.0 {
                    pos.x = 0.0;
                    self.camera.position.x = 180.0 - self.camera.width / self.camera.zoom;
                }

                if camera_upper_bounds.y < -90.0 {
                    pos.y = 0.0;
                    self.camera.position.y = -90.0 + self.camera.height / self.camera.zoom;
                }

                if camera_lower_bounds.y > 80.0 {
                    pos.y = 0.0;
                    self.camera.position.y = 80.0;
                }

                if camera_lower_bounds.x < -180.0 {
                    pos.x = 0.0;
                    self.camera.position.x = -180.0;
                }

                self.camera.position += pos;
            }

            // Displays the entire heatmap, used to calculate max weight and export to png
            CameraEvent::EntireView => {
                self.camera.position = Vector2::new(-180.0, 90.0);

                self.camera.zoom = 5.0;

                self.update_camera(CameraEvent::Resize(1800, 900));

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
        )
    }
}

#[derive(Clone)]
pub struct Camera {
    pub aspect: f64,
    pub width: f64,
    pub height: f64,
    pub zoom: f64,
    pub position: cgmath::Vector2<f64>,
}

impl Camera {
    // This is the cool matrix math that makes this whole thing actually work!
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
