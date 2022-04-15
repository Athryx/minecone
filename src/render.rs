use std::time::Duration;
use std::mem;

use nalgebra::{Point3, Vector3, Scale3, Matrix4, UnitQuaternion, Unit};
use winit::{
	window::Window,
	event::WindowEvent,
};
use wgpu::util::DeviceExt;

use crate::texture::Texture;
use crate::camera::{Camera, CameraController};
use crate::model::{Vertex, ModelVertex, Model, DrawModel};

#[derive(Debug)]
struct Instance {
	translation: Vector3<f32>,
	rotation: UnitQuaternion<f32>,
	scale: Scale3<f32>,
}

impl Instance {
	fn to_raw(&self) -> InstanceRaw {
		let translation = Matrix4::new_translation(&self.translation);
		let rotation = self.rotation.to_homogeneous();
		let scale = self.scale.to_homogeneous();
		InstanceRaw((translation * rotation * scale).into())
	}
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceRaw([[f32; 4]; 4]);

impl InstanceRaw {
	const ATTRIBS: [wgpu::VertexAttribute; 4] =
		wgpu::vertex_attr_array![5 => Float32x4, 6 => Float32x4, 7 => Float32x4, 8 => Float32x4];

	fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
		wgpu::VertexBufferLayout {
			array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Instance,
			attributes: &Self::ATTRIBS,
		}
	}
}

const INSTANCES_PER_ROW: u32 = 10;
const INSTANCE_DISPLACEMENT: Vector3<f32> = Vector3::new(INSTANCES_PER_ROW as f32 * 0.5, 0.0, INSTANCES_PER_ROW as f32 * 0.5);


#[derive(Debug)]
pub struct State {
	surface: wgpu::Surface,
	device: wgpu::Device,
	queue: wgpu::Queue,
	config: wgpu::SurfaceConfiguration,
	render_pipeline: wgpu::RenderPipeline,
	model: Model,
	// TODO: put instances in a data structure with model
	instances: Vec<Instance>,
	instance_buffer: wgpu::Buffer,
	depth_texture: Texture,
	// TODO: remove
	diffuse_texture: Texture,
	diffuse_bind_group: wgpu::BindGroup,
	camera: Camera,
	camera_controler: CameraController,
	camera_buffer: wgpu::Buffer,
	camera_bind_group: wgpu::BindGroup,
	pub size: winit::dpi::PhysicalSize<u32>,
}

impl State {
	// Creating some of the wgpu types requires async code
	pub async fn new(window: &Window) -> Self {
		let size = window.inner_size();

		let instance = wgpu::Instance::new(wgpu::Backends::all());
		let surface = unsafe { instance.create_surface(window) };

		let adapter = instance.request_adapter(
			&wgpu::RequestAdapterOptions {
				power_preference: wgpu::PowerPreference::default(),
				compatible_surface: Some(&surface),
				force_fallback_adapter: false,
			},
		).await.unwrap();

		let (device, queue) = adapter.request_device(
			&wgpu::DeviceDescriptor {
				features: wgpu::Features::empty(),
				limits: wgpu::Limits::default(),
				label: None,
			},
			None,
		).await.unwrap();

		let config = wgpu::SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: surface.get_preferred_format(&adapter).unwrap(),
			width: size.width,
			height: size.height,
			present_mode: wgpu::PresentMode::Fifo,
		};
		surface.configure(&device, &config);


		// load texture
		let diffuse_bytes = include_bytes!("happy-tree.png");
		let diffuse_texture = Texture::from_bytes(&device, &queue, diffuse_bytes, "diffuse_texture").unwrap();

		let texture_bind_group_layout = device.create_bind_group_layout(
			&wgpu::BindGroupLayoutDescriptor {
				label: Some("texture_bind_group_layout"),
				entries: &[
					wgpu::BindGroupLayoutEntry {
						binding: 0,
						visibility: wgpu::ShaderStages::FRAGMENT,
						ty: wgpu::BindingType::Texture {
							multisampled: false,
							view_dimension: wgpu::TextureViewDimension::D2,
							sample_type: wgpu::TextureSampleType::Float { filterable: true },
						},
						count: None,
					},
					wgpu::BindGroupLayoutEntry {
						binding: 1,
						visibility: wgpu::ShaderStages::FRAGMENT,
						ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
						count: None,
					},
				],
			}
		);

		let diffuse_bind_group = device.create_bind_group(
			&wgpu::BindGroupDescriptor {
				label: Some("diffuse_bind_group"),
				layout: &texture_bind_group_layout,
				entries: &[
					wgpu::BindGroupEntry {
						binding: 0,
						resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
					},
					wgpu::BindGroupEntry {
						binding: 1,
						resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
					}
				],
			}
		);

		let depth_texture = Texture::create_depth_texture(&device, &config, "depth texture");

		// render pipeline
		let camera = Camera::new(Point3::new(0.0, 1.0, 2.0), Point3::new(0.0, 0.0, 0.0), config.width as f32 / config.height as f32);
		let camera_uniform = camera.get_camera_uniform();
		let camera_controler = CameraController::new(0.6);

		let camera_buffer = device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: Some("camera buffer"),
				contents: bytemuck::cast_slice(&[camera_uniform]),
				usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			}
		);

		let camera_bind_group_layout = device.create_bind_group_layout(
			&wgpu::BindGroupLayoutDescriptor {
				label: Some("camera_bind_group_layout"),
				entries: &[
					wgpu::BindGroupLayoutEntry {
						binding: 0,
						visibility: wgpu::ShaderStages::VERTEX,
						ty: wgpu::BindingType::Buffer {
							ty: wgpu::BufferBindingType::Uniform,
							has_dynamic_offset: false,
							min_binding_size: None,
						},
						count: None,
					}
				],
			}
		);

		let camera_bind_group = device.create_bind_group(
			&wgpu::BindGroupDescriptor {
				label: Some("camera_bind_group"),
				layout: &camera_bind_group_layout,
				entries: &[
					wgpu::BindGroupEntry {
						binding: 0,
						resource: camera_buffer.as_entire_binding(),
					},
				],
			}
		);

		let shader = device.create_shader_module(&wgpu::include_wgsl!("shader.wgsl"));
		let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("Render Pipeline Layout"),
			bind_group_layouts: &[
				&texture_bind_group_layout,
				&camera_bind_group_layout,
			],
			push_constant_ranges: &[],
		});

		let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("Render Pipeline"),
			layout: Some(&render_pipeline_layout),
			vertex: wgpu::VertexState {
				module: &shader,
				entry_point: "vs_main",
				buffers: &[
					ModelVertex::desc(),
					InstanceRaw::desc(),
				],
			},
			fragment: Some(wgpu::FragmentState {
				module: &shader,
				entry_point: "fs_main",
				targets: &[wgpu::ColorTargetState {
					format: config.format,
					blend: Some(wgpu::BlendState::REPLACE),
					write_mask: wgpu::ColorWrites::ALL,
				}],
			}),
			primitive: wgpu::PrimitiveState {
				topology: wgpu::PrimitiveTopology::TriangleList,
				strip_index_format: None,
				front_face: wgpu::FrontFace::Ccw,
				cull_mode: Some(wgpu::Face::Back),
				// Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
				polygon_mode: wgpu::PolygonMode::Fill,
				// Requires Features::DEPTH_CLIP_CONTROL
				unclipped_depth: false,
				// Requires Features::CONSERVATIVE_RASTERIZATION
				conservative: false,
			},
			depth_stencil: Some(wgpu::DepthStencilState {
				format: Texture::DEPTH_FORMAT,
				depth_write_enabled: true,
				depth_compare: wgpu::CompareFunction::Less,
				stencil: wgpu::StencilState::default(),
				bias: wgpu::DepthBiasState::default(),
			}),
			multisample: wgpu::MultisampleState {
				count: 1,
				mask: !0,
				alpha_to_coverage_enabled: false,
			},
			multiview: None,
		});

		let model = Model::load_from_file("cube.obj", &device, &queue, &texture_bind_group_layout).unwrap();

		const SPACE_BETWEEN: f32 = 3.0;
		let instances = (0..INSTANCES_PER_ROW).flat_map(|z| {
			(0..INSTANCES_PER_ROW).map(move |x| {
				let x = SPACE_BETWEEN * (x as f32 - INSTANCES_PER_ROW as f32 / 2.0);
				let z = SPACE_BETWEEN * (z as f32 - INSTANCES_PER_ROW as f32 / 2.0);
				let translation = Vector3::new(x, 0.0, z);

				let rotation = if translation == Vector3::zeros() {
					UnitQuaternion::identity()
				} else {
					UnitQuaternion::from_axis_angle(&Unit::new_normalize(translation), std::f32::consts::FRAC_PI_4)
				};

				let scale = Scale3::identity();

				Instance {
					translation,
					rotation,
					scale,
				}
			})
		}).collect::<Vec<_>>();

		let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
		let instance_buffer = device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: Some("instance buffer"),
				contents: bytemuck::cast_slice(&instance_data),
				usage: wgpu::BufferUsages::VERTEX,
			}
		);

		Self {
			surface,
			device,
			queue,
			config,
			render_pipeline,
			model,
			instances,
			instance_buffer,
			depth_texture,
			diffuse_texture,
			diffuse_bind_group,
			camera,
			camera_controler,
			camera_buffer,
			camera_bind_group,
			size,
		}
	}

	pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
		if new_size.width > 0 && new_size.height > 0 {
			self.size = new_size;
			self.config.width = new_size.width;
			self.config.height = new_size.height;
			self.surface.configure(&self.device, &self.config);
			self.depth_texture = Texture::create_depth_texture(&self.device, &self.config, "depth texture");
		}
	}

	// returns true if the event has been processed, so we can skip processing the event in the rest of the code
	pub fn input(&mut self, event: &WindowEvent) -> bool {
		self.camera_controler.process_events(event)
	}

	pub fn update(&mut self, time_delta: Duration) {
		self.camera_controler.update_camera(&mut self.camera, time_delta);
		self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera.get_camera_uniform()]));
	}

	pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
		let output = self.surface.get_current_texture()?;
		let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

		let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
			label: Some("Render Encoder"),
		});

		{
			let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: Some("render pass"),
				color_attachments: &[wgpu::RenderPassColorAttachment {
					view: &view,
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(wgpu::Color {
							r: 0.1,
							g: 0.2,
							b: 0.3,
							a: 1.0,
						}),
						store: true,
					}
				}],
				depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
					view: &self.depth_texture.view,
					depth_ops: Some(wgpu::Operations {
						load: wgpu::LoadOp::Clear(1.0),
						store: true,
					}),
					stencil_ops: None,
				}),
			});

			// TEMP
			render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

			render_pass.set_pipeline(&self.render_pipeline);
			render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
			render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
			render_pass.draw_mesh_instanced(&self.model.meshes[0], 0..self.instances.len() as u32);
		}


		self.queue.submit(std::iter::once(encoder.finish()));
		output.present();

		Ok(())
	}
}
