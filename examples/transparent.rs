// Copyright 2020-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT


// TO COMPILE, USE  export RUSTFLAGS='-L/opt/homebrew/Cellar/mpv/0.39.0/lib/'

use objc::*;
// needed so we can get the .webview() and .ns_window() which is an NSView: https://github.com/tauri-apps/wry/blob/dev/src/webview/mod.rs#L856
use wry::webview::WebviewExtMacOS;
use cocoa::base::id;
use cocoa::appkit::{NSView, NSViewHeightSizable, NSViewWidthSizable};
use cocoa::appkit::NSWindowOrderingMode;
use core_graphics::geometry::CGRect;

fn main() -> wry::Result<()> {
  use wry::{
    application::{
      event::{Event, StartCause, WindowEvent},
      event_loop::{ControlFlow, EventLoop},
      window::WindowBuilder,
    },
    webview::WebViewBuilder,
  };

  // WindowBuilder, Window is from Tao
  let event_loop = EventLoop::new();
  let window = WindowBuilder::new()
    .with_decorations(true)
    // There are actually three layer of background color when creating webview window.
    // The first is window background...
    .with_transparent(true)
    .build(&event_loop)
    .unwrap();

  // setup the webview first because it sets a new contentView
  let webview = WebViewBuilder::new(window)?
    // The second is on webview...
    .with_transparent(true)
    .with_devtools(true)
    .with_url( 
      "https://app.strem.io/shell-v4.4/#/"
      // "http://127.0.0.1:11470/#/"
    )?
    .build()?;

  // Setup MPV
  // @TODO get rid of the unsafe
  unsafe {
    let window_id = webview.ns_window();
    let content_view: id = msg_send![window_id, contentView];
    let player_view: id = msg_send![class!(NSView), alloc];
    let frame: CGRect = msg_send![content_view, bounds];
    let _: () = msg_send![player_view, initWithFrame:frame];
    // This next line is actually done in wry: https://github.com/tauri-apps/wry/blob/dev/src/webview/wkwebview/mod.rs#L748
    // this line triggers a segfault when resizing the window
    // sometimes it doesn't crash so we need to debug this
    let _: () = msg_send![player_view, setAutoresizingMask:NSViewHeightSizable | NSViewWidthSizable];
    let webview_view = webview.webview();
    let _: () = msg_send![content_view, addSubview:player_view positioned:NSWindowOrderingMode::NSWindowBelow relativeTo:webview_view];
    // this line seems to not do anything
    //let _: () = msg_send![content_view, setAutoresizesSubviews:cocoa::base::YES];
    dbg!("all window IDs", webview_view, content_view, window_id, player_view);

    let player_view_id = player_view as i64;

    //paradox spiral
    let mpv = libmpv::Mpv::new().unwrap();
    mpv.set_property("terminal", "yes").unwrap();
    mpv.set_property("msg-level", "all=v").unwrap();
    mpv.set_property("wid", player_view_id).unwrap();
    mpv.set_property("volume", 100).unwrap();
    // For use with libmpv direct embedding. As a special case, on macOS it is used like a normal VO within mpv (cocoa-cb). Otherwise useless in any other contexts. (See <mpv/render.h>.)
    // This also supports many of the options the gpu VO has, depending on the backend.
    // "window embedding"!!!! not "direct embedding"
    mpv.set_property("vo", "swift").unwrap();
    // mpv.set_property("hwdec", "auto").unwrap();
    // mpv.set_property("gpu-context", "macvk").unwrap();
    // yeah it uses the GPU, like vo=gpu and vo=libmpv does. vo=libmpv is basically just a wrapper around vo=gpu with a public API and driving your own render loop.
    // @TODO
//     check_error(mpv_set_option_string(mpv, "vo", "gpu-next"));
// check_error(mpv_set_option_string(mpv, "gpu-api", "vulkan"));
// check_error(mpv_set_option_string(mpv, "gpu-context", "macvk"));
    // we need a new thread here anyway for the event loop
      std::thread::spawn(move || {
        let mut ev_ctx = mpv.create_event_context();
        ev_ctx.disable_deprecated_events().unwrap();
    
        mpv.playlist_load_files(&[(
          "https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4",
          libmpv::FileState::AppendPlay,
          None
        )]);
        loop {
          let ev = ev_ctx.wait_event(600.);
          dbg!(&ev);
        }
      });
  // end of paradoxapiral

    /*
    // not defininig playerView.isOpaque and .drawRect, although we may have to
    // not for drawRect, it's defined here: https://github.com/mpv-player/mpv/blob/master/video/out/cocoa/video_view.m
    */
  }
  // end setup MPV


  event_loop.run(move |event, _, control_flow| {
    *control_flow = ControlFlow::Wait;

    match event {
      Event::NewEvents(StartCause::Init) => println!("Wry has started!"),
      Event::WindowEvent {
        event: WindowEvent::CloseRequested,
        ..
      } => *control_flow = ControlFlow::Exit,
      _ => {}
    }
  });
}
