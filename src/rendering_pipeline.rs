use egui_winit_vulkano::Gui;
use std::{
    sync::Arc,
    convert::TryFrom, collections::HashMap,
};
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer, TypedBufferAccess},
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder,
        CommandBufferUsage, CommandBufferInheritanceInfo, RenderPassBeginInfo, SubpassContents,
    },
    format::Format,
    device::{Device, Queue},
    image::{view::ImageView, AttachmentImage, SampleCount, ImageAccess, ImageViewAbstract},
    memory::allocator::StandardMemoryAllocator,
    pipeline::{
        graphics::{
            input_assembly::{InputAssemblyState, PrimitiveTopology},
            multisample::MultisampleState,
            vertex_input::BuffersDefinition,
            viewport::{Viewport, ViewportState},
        },
        GraphicsPipeline
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    sync::GpuFuture,
};
use vulkano_util::renderer::SwapchainImageView;
use crate::syntax::{Vertex, DrawingMode};

pub struct MSAAPipeline{
    allocator: Arc<StandardMemoryAllocator>,
    queue: Arc<Queue>,
    render_pass: Arc<RenderPass>,
    current_primitive: DrawingMode,
    pipelines: HashMap<DrawingMode, Arc<GraphicsPipeline>>,
    subpass: Subpass,
    intermediary: Arc<ImageView<AttachmentImage>>,
    pub vertex_buffers: Vec<Arc<CpuAccessibleBuffer<[Vertex]>>>,
    command_buffer_allocator: StandardCommandBufferAllocator,
    pub vk_ratio: f32,
}

impl MSAAPipeline{
    pub fn new(
        queue: Arc<Queue>,
        image_format: Format,
        allocator: &Arc<StandardMemoryAllocator>,
        sample_count: SampleCount,
        ) -> Self {
        let render_pass = Self::create_render_pass(queue.device().clone(), image_format, sample_count);

        let mut pipelines = HashMap::new();

        let (pipeline, subpass) =
            Self::create_pipeline(
                queue.device().clone(),
                render_pass.clone(),
                PrimitiveTopology::PointList);
        pipelines.insert(DrawingMode::Point, pipeline);

        let (pipeline, _) =
            Self::create_pipeline(
                queue.device().clone(),
                render_pass.clone(),
                PrimitiveTopology::LineList);
        pipelines.insert(DrawingMode::LineList, pipeline);

        let (pipeline, _) =
            Self::create_pipeline(
                queue.device().clone(),
                render_pass.clone(),
                PrimitiveTopology::LineStrip);
        pipelines.insert(DrawingMode::LineStrip, pipeline);

        let (pipeline, _) =
            Self::create_pipeline(
                queue.device().clone(),
                render_pass.clone(),
                PrimitiveTopology::TriangleStrip);
        pipelines.insert(DrawingMode::TriangleStrip, pipeline);

        let (pipeline, _) =
            Self::create_pipeline(
                queue.device().clone(),
                render_pass.clone(),
                PrimitiveTopology::TriangleList);
        pipelines.insert(DrawingMode::TriangleList, pipeline);

        let vertex_buffers: Vec<Arc<CpuAccessibleBuffer<[Vertex]>>> = Vec::new();

        let command_buffer_allocator = StandardCommandBufferAllocator::new(queue.device().clone(),  Default::default());
        let intermediary = ImageView::new_default(
            AttachmentImage::transient_multisampled(allocator, [1, 1], sample_count, image_format).unwrap(),
            ).unwrap();

        Self{
            allocator: allocator.clone(),
            queue,
            render_pass,
            current_primitive: DrawingMode::TriangleList,
            pipelines,
            subpass,
            intermediary,
            vertex_buffers,
            command_buffer_allocator,
            vk_ratio: 1.0,
        }
    }

    pub fn get_current_pipeline(&self) -> Arc<GraphicsPipeline> {
        if let Some(pipeline) = self.pipelines.get(&self.current_primitive){
            return pipeline.clone();
        }
        else {
            panic!("Couldn't get pipeline");
        }
    }

    pub fn set_vertex_buffer(&mut self, vbs: Vec<Vec<Vertex>>){
        self.vertex_buffers.clear();
        for vb in vbs{
            let buf = 
                CpuAccessibleBuffer::from_iter(
                    &self.allocator,
                    BufferUsage { vertex_buffer: true, ..BufferUsage::empty() },
                    false,
                    vb
                    .iter()
                    .cloned(),
                    )
                    .expect("failed to create buffer");
            self.vertex_buffers.push(buf);
        }
    }

    fn create_render_pass(
        device: Arc<Device>,
        format: Format,
        samples: SampleCount,
        ) -> Arc<RenderPass>{
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

    pub fn gui_pass(&self) -> Subpass{
        self.subpass.clone()
    }

    fn create_pipeline(
        device: Arc<Device>,
        render_pass: Arc<RenderPass>,
        topology: PrimitiveTopology
        ) -> (Arc<GraphicsPipeline>, Subpass){
        let vs = vs::load(device.clone()).expect("failed to create shader module");
        let fs = fs::load(device.clone()).expect("failed to create shader module");

        let subpass = Subpass::from(render_pass, 0).unwrap();

        (
        GraphicsPipeline::start()
            .vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
            .vertex_shader(vs.entry_point("main").unwrap(), ())
            .input_assembly_state(InputAssemblyState::new().topology(topology))
            .fragment_shader(fs.entry_point("main").unwrap(), ())
            .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
            .render_pass(subpass.clone())
            .multisample_state(MultisampleState{
                rasterization_samples: subpass.num_samples().unwrap(),
                ..Default::default()
            })
            .build(device)
            .unwrap(),
            subpass
        )
    }

    //TODO: make it actually do something
    pub fn change_topology(&mut self, mode: DrawingMode){
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
            ).unwrap();

        let dimensions = image.image().dimensions().width_height();
        let mut vk_dimensions = dimensions;
        vk_dimensions[0] = (vk_dimensions[0] as f32 * self.vk_ratio) as u32;

        if dimensions != self.intermediary.dimensions().width_height(){
            self.intermediary = ImageView::new_default(
                AttachmentImage::transient_multisampled(
                    &self.allocator,
                    dimensions,
                    self.subpass.num_samples().unwrap(),
                    image.image().format(),
                    ).unwrap(),
                )
                .unwrap();
        }

        let framebuffer = Framebuffer::new(self.render_pass.clone(), FramebufferCreateInfo{
            attachments: vec![self.intermediary.clone(), image],
            ..Default::default()
        }).unwrap();

        builder
            .begin_render_pass(
                RenderPassBeginInfo{
                    clear_values: vec![
                        Some([0.0, 0.0, 0.0, 1.0].into()),
                        Some([0.0, 0.0, 0.0, 1.0].into()),
                    ],
                    ..RenderPassBeginInfo::framebuffer(framebuffer)
                },
                SubpassContents::SecondaryCommandBuffers,
                ).unwrap();

        let mut secondary_builder = AutoCommandBufferBuilder::secondary(
            &self.command_buffer_allocator,
            self.queue.queue_family_index(),
            CommandBufferUsage::MultipleSubmit,
            CommandBufferInheritanceInfo{
                render_pass: Some(self.subpass.clone().into()),
                ..Default::default()
            },
        ).unwrap();

        for vb in &self.vertex_buffers{
            secondary_builder
                .bind_pipeline_graphics(self.get_current_pipeline().clone())
                .set_viewport(0, vec![Viewport{
                    origin: [0.0, 0.0],
                    dimensions: [vk_dimensions[0] as f32, vk_dimensions[1] as f32],
                    depth_range: 0.0..1.0,
                }])
                .bind_vertex_buffers(0, vb.clone())
                .draw(vb.len() as u32, 1, 0, 0)
                .unwrap();
        }

        let cb = secondary_builder.build().unwrap();
        builder.execute_commands(cb).unwrap();

        let cb = gui.draw_on_subpass_image(dimensions);
        builder.execute_commands(cb).unwrap();

        builder.end_render_pass().unwrap();

        let command_buffer = builder.build().unwrap();
        let after_future = before_future.then_execute(self.queue.clone(), command_buffer).unwrap();

        after_future.boxed()
    }
}


mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
#version 450
layout(location = 0) in vec3 position;
layout(location = 1) in vec4 color;
layout(location = 0) out vec4 v_color;
void main() {
    gl_Position = vec4(position, 1.0);
    v_color = color;
}"
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
#version 450
layout(location = 0) in vec4 v_color;
layout(location = 0) out vec4 f_color;
void main() {
    f_color = v_color;
}"
    }
}
