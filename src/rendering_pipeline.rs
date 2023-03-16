use bytemuck::{Pod, Zeroable};
use egui_winit_vulkano::Gui;
use std::{
    sync::Arc,
    convert::TryFrom,
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
            input_assembly::InputAssemblyState,
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

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, Zeroable, Pod)]
struct Vertex{
    position: [f32; 2],
    color: [f32; 4],
}
vulkano::impl_vertex!(Vertex, position, color);

pub struct MSAAPipeline{
    allocator: Arc<StandardMemoryAllocator>,
    queue: Arc<Queue>,
    render_pass: Arc<RenderPass>,
    pipeline: Arc<GraphicsPipeline>,
    subpass: Subpass,
    intermediary: Arc<ImageView<AttachmentImage>>,
    vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,
    command_buffer_allocator: StandardCommandBufferAllocator,
    pub viewport_width: f32,
}

impl MSAAPipeline{
    pub fn new(
        queue: Arc<Queue>,
        image_format: Format,
        allocator: &Arc<StandardMemoryAllocator>,
        sample_count: SampleCount,
        ) -> Self {
        let render_pass = Self::create_render_pass(queue.device().clone(), image_format, sample_count);
        let (pipeline, subpass) = Self::create_pipeline(queue.device().clone(), render_pass.clone());

        let vertex_buffer = {
            CpuAccessibleBuffer::from_iter(
                allocator,
                BufferUsage { vertex_buffer: true, ..BufferUsage::empty() },
                false,
                [
                    Vertex { position: [-0.5, -0.25], color: [1.0, 0.0, 0.0, 1.0] },
                    Vertex { position: [0.0, 0.5], color: [0.0, 1.0, 0.0, 1.0] },
                    Vertex { position: [0.25, -0.1], color: [0.0, 0.0, 1.0, 1.0] },
                ]
                .iter()
                .cloned(),
                )
                .expect("failed to create buffer")
        };

        let command_buffer_allocator = StandardCommandBufferAllocator::new(queue.device().clone(),  Default::default());
        let intermediary = ImageView::new_default(
            AttachmentImage::transient_multisampled(allocator, [1, 1], sample_count, image_format).unwrap(),
            ).unwrap();

        Self{
            allocator: allocator.clone(),
            queue,
            render_pass,
            pipeline,
            subpass,
            intermediary,
            vertex_buffer,
            command_buffer_allocator,
            viewport_width: 0.75,
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
        ) -> (Arc<GraphicsPipeline>, Subpass){
        let vs = vs::load(device.clone()).expect("failed to create shader module");
        let fs = fs::load(device.clone()).expect("failed to create shader module");

        let subpass = Subpass::from(render_pass, 0).unwrap();

        (
        GraphicsPipeline::start()
            .vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
            .vertex_shader(vs.entry_point("main").unwrap(), ())
            .input_assembly_state(InputAssemblyState::new())
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
        vk_dimensions[0] = (vk_dimensions[0] as f32 * self.viewport_width) as u32;

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

        secondary_builder
            .bind_pipeline_graphics(self.pipeline.clone())
            .set_viewport(0, vec![Viewport{
                origin: [0.0, 0.0],
                dimensions: [vk_dimensions[0] as f32, dimensions[1] as f32],
                depth_range: 0.0..1.0,
            }])
            .bind_vertex_buffers(0, self.vertex_buffer.clone())
            .draw(self.vertex_buffer.len() as u32, 1, 0, 0)
            .unwrap();
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
layout(location = 0) in vec2 position;
layout(location = 1) in vec4 color;
layout(location = 0) out vec4 v_color;
void main() {
    gl_Position = vec4(position, 0.0, 1.0);
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
