mod ui;
mod rendering_pipeline;
use ui::Application;
use winit::{
    event::{Event, WindowEvent}, event_loop::{ControlFlow, EventLoop},
};

fn main() {
    let event_loop = EventLoop::new();
    let mut app = Application::new(&event_loop);

    let mut code = CODE.to_string();
    let mut console = CONSOLE.to_string();
    event_loop.run(move |event, _, control_flow| {
        let renderer = app.windows.get_primary_renderer_mut().unwrap();
        match event{
            Event::WindowEvent { event, window_id} if window_id == renderer.window().id() => {
                let _pass = !app.gui.update(&event);

                match event{
                    WindowEvent::Resized(_) => {
                        renderer.resize();
                        app.resize();
                    }
                    WindowEvent::ScaleFactorChanged { .. } => {
                        renderer.resize();
                        app.resize();
                    }
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => (),
                }
            }
            Event::RedrawRequested(window_id) if window_id == window_id => {
                app.gui.immediate_ui(|gui|{
                    Application::gui_panel(app.panel_width, &mut code, &mut console, gui);
                });
                let before_future = renderer.acquire().unwrap();
                let after_future = app.pipeline.render(before_future, renderer.swapchain_image_view(), &mut app.gui);
                renderer.present(after_future, true);
            }
            Event::MainEventsCleared => {
                renderer.window().request_redraw();
            }
            _ => (),
        }
    });
}

const CODE: &str = r#"
(a (b (c d)))
"#;

const CONSOLE: &str = "vk-repl> ";
