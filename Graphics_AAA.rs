use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::window::Window;
use glam::{Mat4, Vec3};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

const CUBE_VERTICES: &[Vertex] = &[
    Vertex { position: [-0.5, -0.5,  0.5], color: [0.5, 0.5, 0.5] },
    Vertex { position: [ 0.5, -0.5,  0.5], color: [0.6, 0.6, 0.6] },
    Vertex { position: [ 0.5,  0.5,  0.5], color: [0.7, 0.7, 0.7] },
    Vertex { position: [-0.5,  0.5,  0.5], color: [0.5, 0.5, 0.5] },
    Vertex { position: [-0.5, -0.5, -0.5], color: [0.4, 0.4, 0.4] },
    Vertex { position: [ 0.5, -0.5, -0.5], color: [0.5, 0.5, 0.5] },
    Vertex { position: [ 0.5,  0.5, -0.5], color: [0.6, 0.6, 0.6] },
    Vertex { position: [-0.5,  0.5, -0.5], color: [0.4, 0.4, 0.4] },
];

const CUBE_INDICES: &[u16] = &[
    0, 1, 2,  2, 3, 0,
    1, 5, 6,  6, 2, 1,
    7, 6, 5,  5, 4, 7,
    4, 0, 3,  3, 7, 4,
    4, 5, 1,  1, 0, 4,
    3, 2, 6,  6, 7, 3,
];

const GRID_VERTICES: &[Vertex] = &[
    Vertex { position: [-20.0, -0.51, -20.0], color: [0.0, 0.0, 0.0] },
    Vertex { position: [ 20.0, -0.51, -20.0], color: [0.0, 0.0, 0.0] },
    Vertex { position: [ 20.0, -0.51,  20.0], color: [0.0, 0.0, 0.0] },
    Vertex { position: [-20.0, -0.51,  20.0], color: [0.0, 0.0, 0.0] },
];

const GRID_INDICES: &[u16] = &[
    0, 1, 2,  2, 3, 0,
];

pub struct GraphicsContext {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    grid_vertex_buffer: wgpu::Buffer,
    grid_index_buffer: wgpu::Buffer,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
}

impl GraphicsContext {
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window.clone()).unwrap();
        
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.unwrap();

        // الإصلاح الأول: تصحيح حقول الـ DeviceDescriptor لـ v29 ومسارات الـ Experimental / Trace
       // تصحيح الاستدعاء: إزالة الـ None الزائدة تماماً ليتوافق مع بارامتر wgpu v29 الوحيد
        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("Obito GPU Device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::default(),
            experimental_features: wgpu::ExperimentalFeatures::default(),
            trace: wgpu::Trace::default(),
        }).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let texture_format = surface_caps.formats[0];
        
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: texture_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2, 
        };
        surface.configure(&device, &config);

        let view = Mat4::look_at_rh(
            Vec3::new(3.0, 3.0, 5.0),
            Vec3::ZERO,
            Vec3::Y,
        );
        let proj = Mat4::perspective_rh(45.0f32.to_radians(), config.width as f32 / config.height as f32, 0.1, 100.0);
        let view_proj = proj * view;

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Uniform Buffer"),
            contents: bytemuck::cast_slice(&view_proj.to_cols_array()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
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

        let shader = device.create_shader_module(wgpu::include_wgsl!("Engine.wgsl"));

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[Some(&camera_bind_group_layout)],
            immediate_size: 0,
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Obito AAA Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3],
                }],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cube Vertex Buffer"),
            contents: bytemuck::cast_slice(CUBE_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cube Index Buffer"),
            contents: bytemuck::cast_slice(CUBE_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let grid_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Grid Vertex Buffer"),
            contents: bytemuck::cast_slice(GRID_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let grid_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Grid Index Buffer"),
            contents: bytemuck::cast_slice(GRID_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            surface,
            device,
            queue,
            config,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            grid_vertex_buffer,
            grid_index_buffer,
            camera_buffer,
            camera_bind_group,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    // الإصلاح الثاني: صياغة دالة رندر آمنة ومستقرة تماماً لمعايير v29 دون تفكيك معقد لـ CurrentSurfaceTexture
   
    // تأكد أن الدالة تبدأ هنا داخل بلوك impl GraphicsContext
  pub fn render(&mut self) -> Result<(), String> {
        // 1. استدعاء السطح للحصول على الـ CurrentSurfaceTexture
        let current_surface_frame = self.surface.get_current_texture();
        
        // 2. فك الحاوية الذكية للحصول على الـ SurfaceTexture الفعلي (الذي يحتوي على الحقول والدوال المطلوبة)
        // إذا كان الإصدار يعيدها كـ enum أو يحتاج unwrap() مباشر:
        // تفكيك الكائن بشكل متوافق تماماً مع الحالات المتاحة في v29
        let surface_texture = match current_surface_frame {
            wgpu::CurrentSurfaceTexture::Success(texture) => texture,
            wgpu::CurrentSurfaceTexture::Timeout => return Err("Surface timeout".to_string()),
            wgpu::CurrentSurfaceTexture::Outdated => return Err("Surface outdated".to_string()),
            wgpu::CurrentSurfaceTexture::Lost => return Err("Surface lost".to_string()),
            _ => return Err("Surface error occurred".to_string()), // معالجة أي حالة أخرى مثل نقص الذاكرة أو غيرها بشكل آمن
        };
        
        // 3. الآن يمكنك استدعاء الـ view من الـ texture الحقيقي بسلاسة
        let view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Obito Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.05, g: 0.05, b: 0.05, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

            render_pass.set_vertex_buffer(0, self.grid_vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.grid_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..GRID_INDICES.len() as u32, 0, 0..1);

            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..CUBE_INDICES.len() as u32, 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        
        // 4. استدعاء دالة الـ present من الـ SurfaceTexture الحقيقي بنجاح
        surface_texture.present();

        Ok(())
    }

} // إغلاق الـ impl الأساسي للملف