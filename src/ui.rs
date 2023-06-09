use egui::{ScrollArea, TextEdit, TextStyle, Ui};
use egui_extras::{TableBuilder, Column};
use egui_winit_vulkano::{Gui, GuiConfig};
use vulkano::device::Features;
use vulkano::format::Format;
use vulkano::image::{ImageUsage, SampleCount};
use vulkano_util::context::{VulkanoConfig, VulkanoContext};
use vulkano_util::window::{VulkanoWindows, WindowDescriptor};
use winit::event_loop::EventLoop;

use crate::rendering_pipeline::MSAAPipeline;

pub struct Application {
    pub context: VulkanoContext,
    pub windows: VulkanoWindows,
    pub pipeline: MSAAPipeline,
    pub changed_input: bool,
    pub gui: Gui,
}

impl Application {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        let mut vk_config = VulkanoConfig::default();
        vk_config.device_features = Features {
            multi_draw_indirect: true,
            ..Features::empty()
        };
        let context = VulkanoContext::new(vk_config);
        let mut windows = VulkanoWindows::default();

        windows.create_window(&event_loop, &context, &WindowDescriptor::default(), |ci| {
            ci.image_format = Some(Format::B8G8R8A8_SRGB);
            ci.image_usage = ImageUsage {
                transfer_dst: true,
                ..ci.image_usage
            };
        });

        let pipeline = MSAAPipeline::new(
            context.graphics_queue().clone(),
            windows
                .get_primary_renderer_mut()
                .unwrap()
                .swapchain_format(),
            context.memory_allocator(),
            SampleCount::Sample4,
        );

        let gui = Gui::new_with_subpass(
            &event_loop,
            windows.get_primary_renderer_mut().unwrap().surface(),
            windows.get_primary_renderer_mut().unwrap().graphics_queue(),
            pipeline.gui_pass(),
            GuiConfig {
                preferred_format: Some(vulkano::format::Format::B8G8R8A8_SRGB),
                samples: SampleCount::Sample4,
                ..Default::default()
            },
        );

        Application {
            context,
            windows,
            pipeline,
            changed_input: false,
            gui,
        }
    }

    pub fn gui_panel(
        changed: &mut bool,
        app_info: &mut AppInfo,
        vk_ratio: &mut f32,
        code: &mut String,
        console: &mut String,
        gui: &mut Gui,
    ) {
        let ctx = gui.context();
        egui::SidePanel::right("Panel")
            .resizable(true)
            .show(&ctx, |ui| {
                let font = egui::FontId {
                    size: 14.0,
                    family: egui::FontFamily::Monospace,
                };
                let row_height = ui.fonts().row_height(&font);
                let editor_height = ui.available_height() / 1.75;
                let editor_rows = editor_height / row_height;
                let editor = TextEdit::multiline(code)
                    .font(font)
                    .desired_width(ui.available_width())
                    .desired_rows(editor_rows as usize);
                ui.set_min_width(app_info.min_width);
                ScrollArea::vertical()
                    .max_height(ui.available_height() / 1.75)
                    .show_rows(ui, row_height, editor_rows as usize - 5, |ui, _| {
                        ui.set_height(ui.available_height());
                        *changed = ui.add(editor).changed();
                    });
                ui.separator();
                Self::lower_panel(app_info.panel_width * 0.86, ui);
                if app_info.panel_width != ui.available_width() + 20.0 {
                    app_info.panel_width = ui.available_width() + 20.0;
                    *vk_ratio = 1.0
                        - (app_info.panel_width / app_info.window_size[0])
                            * app_info.scale_factor as f32;
                }
            });
    }

    fn lower_panel(width: f32, ui: &mut Ui) {
        ui.columns(2, |columns| {
            columns[0].push_id(0, |ui|{
                TableBuilder::new(ui)
                    .column(Column::exact(width / 4.0).resizable(false))
                    .column(Column::exact(width / 4.0).resizable(false))
                    .header(16.0, |mut header| {
                        header.col(|ui| {
                            ui.heading("Name");
                        });
                        header.col(|ui| {
                            ui.heading("Type");
                        });
                    })
                .body(|mut body| {
                    body.row(26.0, |mut row| {
                        row.col(|ui| {
                            if ui.button("object").clicked() {
                                println!("object");
                            }
                        });
                        row.col(|ui| {
                            ui.label("NaN");
                        });
                    });
                });
            });
        });
    }
}

pub struct AppInfo {
    window_size: [f32; 2],
    panel_width: f32,
    min_width: f32,
    scale_factor: f64,
}

impl AppInfo {
    pub fn new(window_size: [f32; 2], panel_width: f32, min_width: f32, scale_factor: f64) -> Self {
        AppInfo {
            window_size,
            panel_width,
            min_width,
            scale_factor,
        }
    }

    pub fn resize(&mut self, window_size: [f32; 2]) {
        self.window_size = window_size;
        self.panel_width = window_size[0] as f32 / 4.0;
    }

    pub fn rescale(&mut self, scale_factor: f64) {
        self.scale_factor = scale_factor;
    }
}
