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

        Application {
            context,
            windows,
            pipeline,
            gui,
        }
    }

    pub fn gui_panel(app_info: &mut AppInfo, vk_ratio: &mut f32, code: &mut String, console: &mut String, gui: &mut Gui){
        let ctx = gui.context();
        egui::SidePanel::right("Panel").resizable(true).show(&ctx, |ui| {
            let font = egui::FontId { size: 14.0, family: egui::FontFamily::Monospace };
            let row_height = ui.fonts().row_height(&font);
            let editor_height = ui.available_height() / 1.5;
            let editor_rows = editor_height / row_height;
            ui.set_min_width(app_info.min_width);
            ScrollArea::vertical()
                .max_height(ui.available_height() / 1.5)
                .show_rows(ui, row_height, editor_rows as usize - 5, |ui, _| {
                    ui.set_height(ui.available_height());
                    ui.add(TextEdit::multiline(code)
                        .font(font)
                        .desired_width(ui.available_width())
                        .desired_rows(editor_rows as usize)
                        );
                });
            ui.separator();
            Self::lower_panel(console, ui);
            if app_info.panel_width != ui.available_width() + 20.0 {
                app_info.panel_width = ui.available_width() + 20.0;
                *vk_ratio = 1.0 - (app_info.panel_width / app_info.window_size[0]) * app_info.scale_factor as f32;
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
            columns[1].set_height(columns[1].available_height());
            columns[1].add(
                TextEdit::multiline(&mut "err".to_string())
                .font(TextStyle::Monospace)
                .desired_rows(columns[1].available_height() as usize / 14)
                );
        });
        ui.set_height(ui.available_height() / 2.0);
    }
}

pub struct AppInfo{
    window_size: [f32; 2],
    panel_width: f32,
    min_width: f32,
    scale_factor: f64,
}

impl AppInfo{
    pub fn new(window_size: [f32; 2], panel_width: f32, min_width: f32, scale_factor: f64) -> Self {
        AppInfo{
            window_size,
            panel_width,
            min_width,
            scale_factor,
        }
    }

    pub fn resize(&mut self, window_size: [f32; 2]){
        self.window_size = window_size;
        self.panel_width = window_size[0] as f32 / 4.0;
    }

    pub fn rescale(&mut self, scale_factor: f64){
        self.scale_factor = scale_factor;
    }
}
