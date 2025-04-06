// Copyright 2020-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT


// TO COMPILE, USE  export RUSTFLAGS='-L/opt/homebrew/Cellar/mpv/0.39.0/lib/'

use std::ffi::c_void;
use glium::{
    glutin::{
        dpi::LogicalSize,
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
        ContextBuilder,
    },
    Display,
};
use libmpv2::{
    render::{OpenGLInitParams, RenderContext, RenderParam, RenderParamApiType},
    Mpv,
};
use wry::{
    application::{
        event_loop::EventLoop as WryEventLoop,
        window::WindowBuilder as WryWindowBuilder,
    },
    webview::WebViewBuilder,
};


#[derive(Debug)]
enum UserEvent {
    MpvEventAvailable,
    RedrawRequested,
}

fn main() -> wry::Result<()> {
    // Create the event loop for MPV
    let events_loop = EventLoop::<UserEvent>::with_user_event();
    let wb = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(1024.0, 768.0))
        .with_title("libmpv-rs OpenGL Example");
    let cb = ContextBuilder::new()
        .with_gl_profile(glium::glutin::GlProfile::Core)
        .with_gl(glium::glutin::GlRequest::Specific(glium::glutin::Api::OpenGl, (3, 3)));
    let display = Display::new(wb, cb, &events_loop).unwrap();

    // Create WRY window and webview
    let wry_event_loop = WryEventLoop::new();
    let wry_window = WryWindowBuilder::new()
        .with_decorations(true)
        .with_transparent(true)
        .build(&wry_event_loop)
        .unwrap();

    let webview = WebViewBuilder::new(wry_window)?
        .with_transparent(true)
        .with_devtools(true)
        .with_url("https://app.strem.io/shell-v4.4/#/")?
        .build()?;

    // Create MPV instance and render context
    let mut mpv = Mpv::with_initializer(|init| {
        init.set_property("vo", "libmpv")?;

        init.set_property("terminal", "yes")?;
        init.set_property("msg-level", "all=v")?;
        Ok(())
    }).expect("Failed to create MPV instance");

    let mut render_context = RenderContext::new(
        unsafe { mpv.ctx.as_mut() },
        vec![
            RenderParam::ApiType(RenderParamApiType::OpenGl),
            RenderParam::InitParams(OpenGLInitParams {
                get_proc_address: |display: &Display, name: &str| {
                    display.gl_window().context().get_proc_address(name) as *mut c_void
                },
                ctx: display.clone(),
            }),
        ],
    )
    .expect("Failed creating render context");

    // Setup event callbacks
    mpv.event_context_mut().disable_deprecated_events().unwrap();
    let event_proxy = events_loop.create_proxy();
    render_context.set_update_callback(move || {
        event_proxy.send_event(UserEvent::RedrawRequested).unwrap();
    });
    // we need a new one because we moved the previous one
    let event_proxy = events_loop.create_proxy();
    mpv.event_context_mut().set_wakeup_callback(move || {
        event_proxy.send_event(UserEvent::MpvEventAvailable).unwrap();
    });
    // Load video
    mpv.command("loadfile", &["https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4", "replace"]).unwrap();

    // Run the event loop
    events_loop.run(move |event, _target, control_flow| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::UserEvent(UserEvent::RedrawRequested) => {
                display.gl_window().window().request_redraw();
            }
            Event::UserEvent(UserEvent::MpvEventAvailable) => loop {
                match mpv.event_context_mut().wait_event(0.0) {
                    Some(Ok(libmpv2::events::Event::EndFile(_))) => {
                        *control_flow = ControlFlow::Exit;
                        break;
                    }
                    Some(Ok(mpv_event)) => {
                        eprintln!("MPV event: {:?}", mpv_event);
                    }
                    Some(Err(err)) => {
                        eprintln!("MPV Error: {}", err);
                        *control_flow = ControlFlow::Exit;
                        break;
                    }
                    None => {
                        *control_flow = ControlFlow::Wait;
                        break;
                    }
                }
            },
            Event::RedrawRequested(_) => {
                let (width, height) = display.get_framebuffer_dimensions();
                render_context.render::<Display>(0, width as _, height as _, true)
                    .expect("Failed to draw on glutin window");
                display.swap_buffers().unwrap();
                *control_flow = ControlFlow::Wait;
            }
            Event::LoopDestroyed => {
                // @TODO
                // drop(render_context); // Properly drop the render context before the mpv player
            }
            _ => {
                *control_flow = ControlFlow::Wait;
            }
        }
    });
}
