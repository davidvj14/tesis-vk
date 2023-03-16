use egui::{TextEdit, TextStyle, Ui, ScrollArea};
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
    pub min_width: f32,
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
        let min_width = panel_width / 4.0;

        Application {
            context,
            windows,
            pipeline,
            gui,
            panel_width,
            min_width,
        }
    }

    pub fn resize(&mut self){
        self.panel_width = self.windows.get_primary_window().unwrap().inner_size().width as f32 / 4.0;
    }

    pub fn gui_panel(min_width: f32, panel_width: &mut f32, window_width: f32, vk_ratio: &mut f32, code: &mut String, console: &mut String, gui: &mut Gui){
        let ctx = gui.context();
        egui::SidePanel::right("Panel").resizable(true).show(&ctx, |ui| {
            let font = egui::FontId { size: 14.0, family: egui::FontFamily::Monospace };
            let row_height = ui.fonts().row_height(&font);
            let editor_height = ui.available_height() / 2.0;
            let editor_rows = editor_height / row_height;
            ui.set_min_width(min_width);
            ScrollArea::vertical()
                .max_height(ui.available_height() / 2.0)
                .show_rows(ui, row_height, editor_rows as usize - 5, |ui, _| {
                    ui.add(TextEdit::multiline(code)
                        .font(font)
                        .desired_width(ui.available_width())
                        .desired_rows(editor_rows as usize)
                        );
                });
            ui.separator();
            Self::lower_panel(console, ui);
            if *panel_width != ui.available_width(){
                *panel_width = ui.available_width();
                *vk_ratio = 1.0 - (*panel_width / window_width) * 2.0;
            }
        });
    }

    fn lower_panel(console: &mut String, ui: &mut Ui){
        ui.columns(2, |columns| {
            columns[0].set_height(columns[0].available_height());
            columns[0].add(
                TextEdit::multiline(console)
                .font(TextStyle::Monospace)
                .desired_rows(columns[0].available_height() as usize / 14)
                );
            columns[1].add(
                TextEdit::multiline(&mut "err".to_string())
                .font(TextStyle::Monospace)
                .desired_rows(columns[1].available_height() as usize / 14)
                );
        });
        ui.set_height(ui.available_height());
    }
}
