use crate::language::types;
use egui_winit_vulkano::Gui;
use std::{
    collections::{BTreeMap, HashMap},
    convert::TryFrom,
    sync::Arc,
    io::Cursor,
};
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer, TypedBufferAccess},
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder,
        CommandBufferInheritanceInfo, CommandBufferUsage, RenderPassBeginInfo,
        SecondaryAutoCommandBuffer, SubpassContents,
    },
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator,
        layout::DescriptorSetLayoutBinding,
        pool::DescriptorPoolCreateInfo,
        PersistentDescriptorSet, WriteDescriptorSet,
    },
    device::{Device, Queue},
    format::Format,
    image::{view::ImageView, ImageDimensions, AttachmentImage, ImageAccess, ImageViewAbstract, SampleCount, ImmutableImage, MipmapsCount},
    memory::allocator::StandardMemoryAllocator,
    pipeline::{
        graphics::{
            input_assembly::{InputAssemblyState, PrimitiveTopology},
            multisample::MultisampleState,
            vertex_input::BuffersDefinition,
            viewport::{Viewport, ViewportState},
        },
        GraphicsPipeline, Pipeline,
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    shader::{ShaderStages, ShaderModule},
    sync::GpuFuture, sampler::{Sampler, SamplerCreateInfo, Filter, SamplerMipmapMode, SamplerAddressMode},
};
use vulkano_util::renderer::SwapchainImageView;
use png;

pub struct MSAAPipeline {
    pub allocator: Arc<StandardMemoryAllocator>,
    queue: Arc<Queue>,
    render_pass: Arc<RenderPass>,
    current_primitive: PrimitiveTopology,
    pipelines: HashMap<String, Arc<GraphicsPipeline>>,
    subpass: Subpass,
    intermediary: Arc<ImageView<AttachmentImage>>,
    pub models: Vec<types::Model>,
    pub vbs: Vec<Vec<types::Vertex>>,
    sampler: Arc<Sampler>,
    command_buffer_allocator: StandardCommandBufferAllocator,
    pub vk_ratio: f32,
}

impl MSAAPipeline {
    pub fn new(
        queue: Arc<Queue>,
        image_format: Format,
        allocator: &Arc<StandardMemoryAllocator>,
        sample_count: SampleCount,
    ) -> Self {
        let render_pass =
            Self::create_render_pass(queue.device().clone(), image_format, sample_count);

        let mut pipelines = HashMap::new();

        let vs = vs::load(queue.device().clone()).expect("failed to load shader module");
        let fs = fs::load(queue.device().clone()).expect("failed to load shader module");

        let (pipeline, _) = Self::create_pipeline(
            queue.device().clone(),
            render_pass.clone(),
            PrimitiveTopology::PointList,
            vs.clone(), fs.clone()
        );
        pipelines.insert("RESERVED_POINT_LIST".to_string(), pipeline);

        let (pipeline, _) = Self::create_pipeline(
            queue.device().clone(),
            render_pass.clone(),
            PrimitiveTopology::LineList,
            vs.clone(), fs.clone()
        );
        pipelines.insert("RESERVED_LINE_LIST".to_string(), pipeline);

        let (pipeline, _) = Self::create_pipeline(
            queue.device().clone(),
            render_pass.clone(),
            PrimitiveTopology::LineStrip,
            vs.clone(), fs.clone()
        );
        pipelines.insert("RESERVED_LINE_STRIP".to_string(), pipeline);

        let (pipeline, _) = Self::create_pipeline(
            queue.device().clone(),
            render_pass.clone(),
            PrimitiveTopology::TriangleStrip,
            vs.clone(), fs.clone()
        );
        pipelines.insert("RESERVED_TRIANGLE_STRIP".to_string(), pipeline);

        let (pipeline, _) = Self::create_pipeline(
            queue.device().clone(),
            render_pass.clone(),
            PrimitiveTopology::TriangleList,
            vs.clone(), fs.clone()
        );
        pipelines.insert("RESERVED_TRIANGLE_LIST".to_string(), pipeline);

        let (pipeline, subpass) = Self::create_tex_pipeline(
            queue.device().clone(),
            render_pass.clone(),
            PrimitiveTopology::PointList,
        );
        pipelines.insert("RESERVED_POINT_LIST_TEX".to_string(), pipeline);

        let (pipeline, _) = Self::create_tex_pipeline(
            queue.device().clone(),
            render_pass.clone(),
            PrimitiveTopology::LineList,
        );
        pipelines.insert("RESERVED_LINE_LIST_TEX".to_string(), pipeline);

        let (pipeline, _) = Self::create_tex_pipeline(
            queue.device().clone(),
            render_pass.clone(),
            PrimitiveTopology::LineStrip,
        );
        pipelines.insert("RESERVED_LINE_STRIP_TEX".to_string(), pipeline);

        let (pipeline, _) = Self::create_tex_pipeline(
            queue.device().clone(),
            render_pass.clone(),
            PrimitiveTopology::TriangleStrip,
        );
        pipelines.insert("RESERVED_TRIANGLE_STRIP_TEX".to_string(), pipeline);

        let (pipeline, _) = Self::create_tex_pipeline(
            queue.device().clone(),
            render_pass.clone(),
            PrimitiveTopology::TriangleList,
        );
        pipelines.insert("RESERVED_TRIANGLE_LIST_TEX".to_string(), pipeline);

        let command_buffer_allocator =
            StandardCommandBufferAllocator::new(queue.device().clone(), Default::default());
        let intermediary = ImageView::new_default(
            AttachmentImage::transient_multisampled(allocator, [1, 1], sample_count, image_format)
                .unwrap(),
        )
        .unwrap();

        let sampler = Sampler::new(
            queue.device().clone(),
            SamplerCreateInfo {
                mag_filter: Filter::Linear,
                min_filter: Filter::Linear,
                mipmap_mode: SamplerMipmapMode::Nearest,
                address_mode: [SamplerAddressMode::Repeat; 3],
                mip_lod_bias: 0.0,
                ..Default::default()
            },
            ).unwrap();

        Self {
            allocator: allocator.clone(),
            queue,
            render_pass,
            current_primitive: PrimitiveTopology::TriangleList,
            pipelines,
            subpass,
            intermediary,
            models: Vec::new(),
            vbs: Vec::new(),
            sampler,
            command_buffer_allocator,
            vk_ratio: 1.0,
        }
    }

    fn primitive_to_str(primitive: &PrimitiveTopology) -> &'static str {
        match primitive {
            PrimitiveTopology::PointList => "RESERVED_POINT_LIST",
            PrimitiveTopology::LineList => "RESERVED_LINE_LIST",
            PrimitiveTopology::LineStrip => "RESERVED_LINE_LIST",
            PrimitiveTopology::TriangleList => "RESERVED_TRIANGLE_LIST",
            PrimitiveTopology::TriangleStrip => "RESERVED_TRIANGLE_LIST",
            _ => "",
        }
    }

    pub fn get_current_pipeline(&self) -> Arc<GraphicsPipeline> {
        if let Some(pipeline) = self.pipelines.get(Self::primitive_to_str(&self.current_primitive)) {
            return pipeline.clone();
        } else {
            panic!("Couldn't get pipeline");
        }
    }

    pub fn get_specific_pipeline(&self, primitive: &String) -> Arc<GraphicsPipeline> {
        if let Some(pipeline) = self.pipelines.get(primitive) {
            return pipeline.clone();
        } else {
            panic!("Couldn't get pipeline");
        }
    }

    pub fn receive_vertex_buffer(&mut self, vb: Vec<types::Vertex>) {
        self.vbs.push(vb);
    }

    pub fn receive_model(&mut self, model: types::Model) {
        self.models.push(model);
    }

    fn create_render_pass(
        device: Arc<Device>,
        format: Format,
        samples: SampleCount,
    ) -> Arc<RenderPass> {
        vulkano::single_pass_renderpass!(
        device,
        attachments: {
            intermediary: {
                load: Clear,
                store: DontCare,
                format: format,
                samples: samples,
            },

            color: {
                load: DontCare,
                store: Store,
                format: format,
                samples: 1,
            }
        },
        pass: {
            color: [intermediary],
            depth_stencil: {},
            resolve: [color],
        }
        )
        .unwrap()
    }

    fn create_tex_render_pass(
        device: Arc<Device>,
        format: Format,
        samples: SampleCount,
    ) -> Arc<RenderPass> {
        vulkano::single_pass_renderpass!(
        device,
        attachments: {
            intermediary: {
                load: Clear,
                store: DontCare,
                format: format,
                samples: samples,
            },

            color: {
                load: DontCare,
                store: Store,
                format: format,
                samples: 1,
            }

        },
        pass: {
            color: [intermediary],
            depth_stencil: {},
            resolve: [color],
        }
        )
        .unwrap()
    }

    pub fn gui_pass(&self) -> Subpass {
        self.subpass.clone()
    }

    fn create_pipeline(
        device: Arc<Device>,
        render_pass: Arc<RenderPass>,
        topology: PrimitiveTopology,
        vertex_module: Arc<ShaderModule>,
        frag_module: Arc<ShaderModule>,
    ) -> (Arc<GraphicsPipeline>, Subpass) {
        let subpass = Subpass::from(render_pass, 0).unwrap();

        (
            GraphicsPipeline::start()
                .vertex_input_state(BuffersDefinition::new().vertex::<types::Vertex>())
                .vertex_shader(vertex_module.entry_point("main").unwrap(), ())
                .input_assembly_state(InputAssemblyState::new().topology(topology))
                .fragment_shader(frag_module.entry_point("main").unwrap(), ())
                .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
                .render_pass(subpass.clone())
                .multisample_state(MultisampleState {
                    rasterization_samples: subpass.num_samples().unwrap(),
                    ..Default::default()
                })
                .build(device)
                .unwrap(),
            subpass,
        )
    }

    fn create_tex_pipeline(
        device: Arc<Device>,
        render_pass: Arc<RenderPass>,
        topology: PrimitiveTopology,
        ) -> (Arc<GraphicsPipeline>, Subpass) {
        let subpass = Subpass::from(render_pass, 0).unwrap();

        let vs = vs_tex::load(device.clone()).expect("failed to load shader module");
        let fs = fs_tex::load(device.clone()).expect("failed to load shader module");

        (
            GraphicsPipeline::start()
                .vertex_input_state(BuffersDefinition::new().vertex::<types::TextureVertex>())
                .vertex_shader(vs.entry_point("main").unwrap(), ())
                .input_assembly_state(InputAssemblyState::new().topology(topology))
                .fragment_shader(fs.entry_point("main").unwrap(), ())
                .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
                .render_pass(subpass.clone())
                .multisample_state(MultisampleState {
                    rasterization_samples: subpass.num_samples().unwrap(),
                    ..Default::default()
                })
                .build(device)
                .unwrap(),
            subpass,
        )
    }

    pub fn change_topology(&mut self, mode: PrimitiveTopology) {
        self.current_primitive = mode;
    }

    pub fn render(
        &mut self,
        before_future: Box<dyn GpuFuture>,
        image: SwapchainImageView,
        gui: &mut Gui,
    ) -> Box<dyn GpuFuture> {
        let mut builder = AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator,
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        for mut model in &mut self.models {
            let mut texture: Option<Arc<ImageView<_>>> = None;

            let texture = texture.get_or_insert_with(|| {
                let image = ImmutableImage::from_iter(
                    &self.allocator,
                    model.texture_data.0.iter().cloned(),
                    model.texture_data.1,
                    MipmapsCount::One,
                    Format::R8G8B8A8_SRGB,
                    &mut builder,
                    )
                    .unwrap();
                ImageView::new_default(image).unwrap()
            });
            model.texture = Some(texture.clone());
        }

        let dimensions = image.image().dimensions().width_height();
        let mut vk_dimensions = dimensions;
        vk_dimensions[0] = (vk_dimensions[0] as f32 * self.vk_ratio) as u32;

        if dimensions != self.intermediary.dimensions().width_height() {
            self.intermediary = ImageView::new_default(
                AttachmentImage::transient_multisampled(
                    &self.allocator,
                    dimensions,
                    self.subpass.num_samples().unwrap(),
                    image.image().format(),
                )
                .unwrap(),
            )
            .unwrap();
        }

        let framebuffer = Framebuffer::new(
            self.render_pass.clone(),
            FramebufferCreateInfo {
                attachments: vec![self.intermediary.clone(), image],
                ..Default::default()
            },
        )
        .unwrap();

        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![
                        Some([0.0, 0.0, 0.0, 1.0].into()),
                        Some([0.0, 0.0, 0.0, 1.0].into()),
                    ],
                    ..RenderPassBeginInfo::framebuffer(framebuffer)
                },
                SubpassContents::SecondaryCommandBuffers,
            )
            .unwrap();

        let mut secondary_builder = AutoCommandBufferBuilder::secondary(
            &self.command_buffer_allocator,
            self.queue.queue_family_index(),
            CommandBufferUsage::MultipleSubmit,
            CommandBufferInheritanceInfo {
                render_pass: Some(self.subpass.clone().into()),
                ..Default::default()
            },
        )
        .unwrap();

        self.record_vb_cmd(&mut secondary_builder, vk_dimensions);
        self.record_model_cmd(&mut secondary_builder, vk_dimensions);

        let cb = secondary_builder.build().unwrap();
        builder.execute_commands(cb).unwrap();

        let cb = gui.draw_on_subpass_image(dimensions);
        builder.execute_commands(cb).unwrap();

        builder.end_render_pass().unwrap();

        let command_buffer = builder.build().unwrap();
        let after_future = before_future
            .then_execute(self.queue.clone(), command_buffer)
            .unwrap();

        after_future.boxed()
    }

    fn record_vb_cmd(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>,
        vk_dimensions: [u32; 2],
    ) {
        for vb in &self.vbs {
            let vertex_buffer = CpuAccessibleBuffer::from_iter(
                &self.allocator,
                BufferUsage {
                    vertex_buffer: true,
                    ..BufferUsage::empty()
                },
                false,
                vb.iter().cloned(),
            )
            .expect("failed to create buffer");
            builder
                .bind_pipeline_graphics(self.get_current_pipeline().clone())
                .set_viewport(
                    0,
                    vec![Viewport {
                        origin: [0.0, 0.0],
                        dimensions: [vk_dimensions[0] as f32, vk_dimensions[1] as f32],
                        depth_range: 0.0..1.0,
                    }],
                )
                .bind_vertex_buffers(0, vertex_buffer)
                .draw(vb.len() as u32, 1, 0, 0)
                .unwrap();
        }
    }

    fn record_model_cmd(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>,
        vk_dimensions: [u32; 2],
    ) {
        for model in &self.models {
            let vertex_buffer = CpuAccessibleBuffer::from_iter(
                &self.allocator,
                BufferUsage {
                    vertex_buffer: true,
                    ..BufferUsage::empty()
                },
                false,
                model.vertices.iter().cloned(),
            )
            .expect("failed to create buffer");
            let index_buffer = CpuAccessibleBuffer::from_iter(
                &self.allocator,
                BufferUsage {
                    index_buffer: true,
                    ..BufferUsage::empty()
                },
                false,
                model.indices.iter().cloned(),
            )
            .expect("failed to create buffer");
            let ubo_matrix = model.generate_mvp_mats(vk_dimensions);

            let unibuffer = CpuAccessibleBuffer::from_data(
                &self.allocator,
                BufferUsage {
                    uniform_buffer: true,
                    ..BufferUsage::empty()
                },
                false,
                [ubo_matrix],
            )
            .unwrap();
            let desc_alloca = StandardDescriptorSetAllocator::new(self.queue.device().clone());


            let desc_set = PersistentDescriptorSet::new(
                &desc_alloca,
                self.get_specific_pipeline(&"RESERVED_TRIANGLE_LIST_TEX".to_string()).layout().set_layouts().get(0).unwrap().clone(),
                [
                WriteDescriptorSet::buffer(0, unibuffer.clone()),
                WriteDescriptorSet::image_view_sampler(1,
                    model.texture.as_ref().unwrap().clone(), self.sampler.clone())
                ],
            )
            .unwrap();
            builder
                .bind_pipeline_graphics(self.get_specific_pipeline(&"RESERVED_TRIANGLE_LIST_TEX".to_string()).clone())
                .set_viewport(
                    0,
                    vec![Viewport {
                        origin: [0.0, 0.0],
                        dimensions: [vk_dimensions[0] as f32, vk_dimensions[1] as f32],
                        depth_range: 0.0..1.0,
                    }],
                )
                .bind_vertex_buffers(0, vertex_buffer.clone())
                .bind_index_buffer(index_buffer.clone())
                .bind_descriptor_sets(
                    vulkano::pipeline::PipelineBindPoint::Graphics,
                    self.get_specific_pipeline(&"RESERVED_TRIANGLE_LIST_TEX".to_string()).layout().clone(),
                    0,
                    desc_set.clone(),
                )
                .draw_indexed(index_buffer.len() as u32, 1, 0, 0, 0)
                .unwrap();
        }
    }


    pub fn load_texture_image(&self, path: &str) -> (Vec<u8>, ImageDimensions) {
        let png_bytes = std::fs::read(path).expect("error loading texture");
        let cursor = Cursor::new(png_bytes);
        let decoder = png::Decoder::new(cursor);
        let mut reader = decoder.read_info().unwrap();
        let info = reader.info();
        let image_dimensions = ImageDimensions::Dim2d {
            width: info.width,
            height: info.height,
            array_layers: 1,
        };
        let mut image_data = Vec::new();
        let depth: u32 = match info.bit_depth {
            png::BitDepth::One => 1,
            png::BitDepth::Two => 2,
            png::BitDepth::Four => 4,
            png::BitDepth::Eight => 8,
            png::BitDepth::Sixteen => 16,
        };
        image_data.resize((info.width * info.height * depth) as usize, 0);
        reader.next_frame(&mut image_data).unwrap();
        (image_data, image_dimensions)
    }
}

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
#version 450

layout(binding = 0) uniform UniformBufferObject {
    mat4 model;
    mat4 view;
    mat4 proj;
} ubo;

layout(location = 0) in vec3 position;
layout(location = 1) in vec4 color;

layout(location = 0) out vec4 frag_color;

void main() {
    gl_Position = ubo.proj * ubo.view * ubo.model * vec4(position, 1.0);
    frag_color = color;
}"
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
#version 450
layout(location = 0) in vec4 color;
layout(location = 0) out vec4 f_color;

void main() {
    f_color = color;
}"
    }
}

mod vs_tex {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
#version 450

layout(binding = 0) uniform UniformBufferObject {
    mat4 model;
    mat4 view;
    mat4 proj;
} ubo;

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 uv;

layout(location = 0) out vec2 tex_coords;

void main() {
    gl_Position = ubo.proj * ubo.view * ubo.model * vec4(position, 1.0);
    tex_coords = uv;
}"
    }
}

mod fs_tex {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
#version 450
layout(location = 0) in vec2 tex_coords;
layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 1) uniform sampler2D tex;

void main() {
    f_color = texture(tex, tex_coords);
}"
    }
}

