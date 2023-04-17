mod language;
mod rendering_pipeline;
mod tvk_glm;
mod ui;
use std::collections::HashMap;

use ui::{AppInfo, Application};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

fn main() {
    let event_loop = EventLoop::new();
    let mut code = CODE.to_string();
    let mut console = CONSOLE.to_string();
    let mut app = Application::new(&event_loop);
    let win_size = app.windows.get_primary_window().unwrap().inner_size();
    let mut app_info = AppInfo::new(
        win_size.into(),
        win_size.width as f32 / 2.5,
        win_size.width as f32 / 5.0,
        app.windows.get_primary_window().unwrap().scale_factor(),
    );
    let mut nu_pars = language::parser::Parser::new(&code);
    let exprs = nu_pars.parse();
    let mut nu_int = language::interpreter::Interpreter {
        bindings: HashMap::new(),
    };
    for e in &exprs {
        nu_int.eval(e, &mut app.pipeline);
    }
    event_loop.run(move |event, _, control_flow| {
        let renderer = app.windows.get_primary_renderer_mut().unwrap();
        match event {
            Event::WindowEvent { event, window_id } if window_id == renderer.window().id() => {
                let _pass = !app.gui.update(&event);

                match event {
                    WindowEvent::Resized(size) => {
                        renderer.resize();
                        app_info.resize(size.into());
                    }
                    WindowEvent::ScaleFactorChanged {
                        scale_factor,
                        new_inner_size,
                    } => {
                        renderer.resize();
                        app_info.rescale(scale_factor);
                        let (width, height) =
                            (new_inner_size.width as f32, new_inner_size.height as f32);
                        app_info.resize([width, height]);
                    }
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => (),
                }
            }
            Event::RedrawRequested(window_id) if window_id == window_id => {
                app.gui.immediate_ui(|gui| {
                    Application::gui_panel(
                        &mut app.changed_input,
                        &mut app_info,
                        &mut app.pipeline.vk_ratio,
                        &mut code,
                        &mut console,
                        gui,
                    );
                    if app.changed_input {
                        app.pipeline.models.clear();
                        app.pipeline.vbs.clear();
                        app.changed_input = false;
                        let mut nu_pars = language::parser::Parser::new(&code);
                        let exprs = nu_pars.parse();
                        let mut nu_int = language::interpreter::Interpreter {
                            bindings: HashMap::new(),
                        };
                        for e in &exprs {
                            nu_int.eval(e, &mut app.pipeline);
                        }
                    }
                });
                let before_future = renderer.acquire().unwrap();
                let after_future = app.pipeline.render(
                    before_future,
                    renderer.swapchain_image_view(),
                    &mut app.gui,
                );
                renderer.present(after_future, true);
            }
            Event::MainEventsCleared => {
                renderer.window().request_redraw();
            }
            _ => (),
        }
    });
}

const CODE: &str = r#"(config (primitive triangle-list)
(interpreting-mode manual))
(def p1 (position
  (x -0.5) (y -0.5) (z -0.5)))
(def p2 (position
  (x 0.5) (y -0.5) (z -0.5)))
(def p3 (position
  (x 0.5) (y -0.5) (z 0.5)))
(def p4 (position
  (x -0.5) (y -0.5) (z 0.5)))
(def p5 (position
  (x -0.5) (y 0.5) (z -0.5)))
(def p6 (position
  (x 0.5) (y 0.5) (z -0.5)))
(def p7 (position
  (x 0.5) (y 0.5) (z 0.5)))
(def p8 (position
  (x -0.5) (y 0.5) (z 0.5)))
(def red (color #FF0000FF))
(def green (color #00FF00FF))
(def blue (color #0000FFFF))
(def v1 (vertex
  (position p1) (color red)))
(def v2 (vertex
  (position p2) (color green)))
(def v3 (vertex
  (position p3) (color blue)))
(def v4 (vertex
  (position p4) (color red)))
(def v5 (vertex
  (position p5) (color red)))
(def v6 (vertex 
  (position p6) (color green)))
(def v7 (vertex
  (position p7) (color blue)))
(def v8 (vertex
  (position p8) (color red)))
(def vb (vertex-buffer
  (v1 v2 v3 v4 v5 v6 v7 v8)))
(def ib (index-buffer
  (0 1 2 3 0
   4 5 1 5 6 2
   6 7 3 7 4)))
(def pers (perspective
  (fovy 0.75)
  (z-near 0.0001)
  (z-far 1000.0)))
(def cam1 (camera
  (position (x 2) (y 3) (z 4))
  (center (x 0.0) (y 0.0) (z 0.0))
  (up (x 0.0) (y 1.0) (z 0.0))
  (perspective pers)))
(def cam2 (camera
  (position (x 5) (y 6) (z 7))
  (center (x 0.0) (y 0.0) (z 0.0))
  (up (x 0.0) (y 1.0) (z 0.0))
  (perspective pers)))
(def m1 (model
  vb ib (topology line-strip)
  (transform default) cam1))
(draw m1)"#;

const CONSOLE: &str = "vk-repl> ";
