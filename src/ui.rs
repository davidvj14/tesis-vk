use egui::{TextEdit, TextStyle, Ui};
use egui_winit_vulkano::{Gui, GuiConfig};
use winit::event_loop::EventLoop;
use vulkano_util::{
    context::{VulkanoConfig, VulkanoContext},
    window::{VulkanoWindows, WindowDescriptor},
};
use vulkano::{
    format::Format,
    image::{ImageUsage, SampleCount},
};

use crate::rendering_pipeline::MSAAPipeline;

pub struct Application{
    pub context: VulkanoContext,
    pub windows: VulkanoWindows,
    pub pipeline: MSAAPipeline,
    pub gui: Gui,
    pub panel_width: f32,
}

impl Application{
    pub fn new(event_loop: &EventLoop<()>) -> Self{
        let context = VulkanoContext::new(VulkanoConfig::default());
        let mut windows = VulkanoWindows::default();
        
        windows.create_window(&event_loop, &context, &WindowDescriptor::default(), |ci|{
            ci.image_format = Some(Format::B8G8R8A8_SRGB);
            ci.image_usage = ImageUsage{
                transfer_dst: true,
                ..ci.image_usage
            };
        });

        let pipeline = MSAAPipeline::new(
            context.graphics_queue().clone(),
            windows.get_primary_renderer_mut().unwrap().swapchain_format(),
            context.memory_allocator(),
            SampleCount::Sample4
            );

        let gui = Gui::new_with_subpass(
            &event_loop,
            windows.get_primary_renderer_mut().unwrap().surface(),
            windows.get_primary_renderer_mut().unwrap().graphics_queue(),
            pipeline.gui_pass(),
            GuiConfig{
                preferred_format: Some(vulkano::format::Format::B8G8R8A8_SRGB),
                samples: SampleCount::Sample4,
                ..Default::default()
            },
            );

        let panel_width = windows.get_primary_window().unwrap().inner_size().width as f32 / 2.0;

        Application {
            context,
            windows,
            pipeline,
            gui,
            panel_width
        }
    }

    pub fn resize(&mut self){
        self.panel_width = self.windows.get_primary_window().unwrap().inner_size().width as f32 / 4.0;
    }

    pub fn gui_panel(panel_width: f32, code: &mut String, console: &mut String, gui: &mut Gui){
        let ctx = gui.context();
        egui::SidePanel::right("Panel").resizable(true).show(&ctx, |ui| {
            ui.set_width(panel_width);
            ui.add(TextEdit::multiline(code)
                .font(TextStyle::Monospace)
                .desired_width(panel_width)
                .desired_rows(25));
            ui.separator();
            Self::lower_panel(console, ui);
        });
    }

    fn lower_panel(console: &mut String, ui: &mut Ui){
        ui.columns(2, |columns| {
            columns[0].add(
                TextEdit::multiline(console)
                .font(TextStyle::Monospace)
                .desired_rows(24)
                );
            columns[1].add(
                TextEdit::multiline(&mut "err".to_string())
                .font(TextStyle::Monospace)
                .desired_rows(24)
                );
        });
    }
}
