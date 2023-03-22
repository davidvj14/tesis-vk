mod ui;
mod interpreter;
mod syntax;
mod parser;
mod rendering_pipeline;
use ui::{Application, AppInfo};
use winit::{
    event::{Event, WindowEvent}, event_loop::{ControlFlow, EventLoop},
};
use parser::Parser;

use crate::interpreter::Interpreter;

fn main() {
    let event_loop = EventLoop::new();
    let mut app = Application::new(&event_loop);
    let win_size = app.windows.get_primary_window().unwrap().inner_size();
    let mut app_info = AppInfo::new(
        win_size.into(),
        win_size.width as f32 / 4.0,
        win_size.width as f32 / 8.0,
        app.windows.get_primary_window().unwrap().scale_factor(),
        );
    let mut code = CODE.to_string();
    let mut console = CONSOLE.to_string();
    let mut parser = Parser::new(CODE);
    parser.parse();
    println!("{:?}", parser.cmds);
    let mut interpreter = Interpreter::new(&parser.cmds, &mut app.pipeline);
    interpreter.interpret();
    println!("{:?} \n {:?}", interpreter.vertex_bindings, interpreter.vb_bindings);
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
                        &mut app_info,
                        &mut app.pipeline.vk_ratio,
                        &mut code,
                        &mut console,
                        gui);
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
(def-vertex v1
    (pos (x -0.75) (y -1) (z 1))
    (color #FF0000FF))
(def-vertex v2
    (pos (x 0.25) (y 0.5) (z 1))
    (color #00FF00FF))
(def-vertex v3
    (pos (x 0.75) (y 0.25) (z 1))
    (color #0000FFFF))
(mk-vertex-buffer vb1
    (v1 v2 v3))
(draw vb1)
"#;

const CONSOLE: &str = "vk-repl> ";
