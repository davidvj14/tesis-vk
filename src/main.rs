mod ui;
mod interpreter;
mod syntax;
mod parser;
mod rendering_pipeline;
use ui::{Application, AppInfo};
use winit::{
    event::{Event, WindowEvent}, event_loop::{ControlFlow, EventLoop},
};

fn main() {
    let event_loop = EventLoop::new();
    let mut code = CODE.to_string();
    let mut console = CONSOLE.to_string();
    let mut app = Application::new(&event_loop, &code);
    let win_size = app.windows.get_primary_window().unwrap().inner_size();
    let mut app_info = AppInfo::new(
        win_size.into(),
        win_size.width as f32 / 4.0,
        win_size.width as f32 / 8.0,
        app.windows.get_primary_window().unwrap().scale_factor(),
        );
    app.interpreter.interpret(&mut app.pipeline);
    event_loop.run(move |event, _, control_flow| {
        let renderer = app.windows.get_primary_renderer_mut().unwrap();
        match event{
            Event::WindowEvent { event, window_id} if window_id == renderer.window().id() => {
                let _pass = !app.gui.update(&event);

                match event{
                    WindowEvent::Resized(size) => {
                        renderer.resize();
                        app_info.resize(size.into());
                    }
                    WindowEvent::ScaleFactorChanged { scale_factor, new_inner_size } => {
                        renderer.resize();
                        app_info.rescale(scale_factor);
                        let (width, height) = (new_inner_size.width as f32, new_inner_size.height as f32);
                        app_info.resize([width, height]);
                    }
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => (),
                }
            }
            Event::RedrawRequested(window_id) if window_id == window_id => {
                app.gui.immediate_ui(|gui|{
                    Application::gui_panel(
                        &mut app.changed_input,
                        &mut app_info,
                        &mut app.pipeline.vk_ratio,
                        &mut code,
                        &mut console,
                        gui);
                    if app.changed_input{
                        app.interpreter.parser.source = code.clone();
                        app.interpreter.interpret(&mut app.pipeline);
                        app.changed_input = false;
                    }
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

const CODE: &str = 
r#"(config (drawing-mode triangle-list)
           (interpreting-mode manual))
(def-vertex v1
    (pos (x -0.75) (y -1) (z 1))
    (color #FF0000FF))
(def-vertex v2
    (pos (x -0.25) (y -0.5) (z 1))
    (color #00FF00FF))
(def-vertex v3
    (pos (x -0.75) (y -0.25) (z 1))
    (color #0000FFFF))
(def-vertex v4
    (pos (x 0.75) (y 1) (z 1))
    (color #FF0000FF))
(def-vertex v5
    (pos (x 0.25) (y 0.5) (z 1))
    (color #00FF00FF))
(def-vertex v6
    (pos (x 0.75) (y 0.25) (z 1))
    (color #0000FFFF))
(mk-vertex-buffer vb1
    (v1 v2 v3))
(mk-vertex-buffer vb2
    (v4 v5 v6))
(draw vb1)
(draw vb2)
"#;

const CONSOLE: &str = "vk-repl> ";
