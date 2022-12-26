// Copyright 2020-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use objc::*;
// needed so we can get the .webview() and .ns_window() which is an NSView: https://github.com/tauri-apps/wry/blob/dev/src/webview/mod.rs#L856
use wry::webview::WebviewExtMacOS;
use cocoa::base::id;
use cocoa::appkit::{NSView, NSViewHeightSizable, NSViewWidthSizable};
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
    .with_decorations(false)
    // There are actually three layer of background color when creating webview window.
    // The first is window background...
    .with_transparent(true)
    .build(&event_loop)
    .unwrap();

  // setup the webview first because it sets a new contentView
  let webview = WebViewBuilder::new(window)?
    // The second is on webview...
    .with_transparent(true)
    // And the last is in html.
    .with_html(
      r#"
            <!doctype html>
            <html>
              <body style="background-color:rgba(87,87,87,0.5);">hello</body>
              <script>
                window.onload = function() {
                  document.body.innerText = `hello, ${navigator.userAgent}`;
                };
              </script>
            </html>"#,
    )?
    .with_devtools(true)
    /*.with_url( 
      //"https://app.strem.io/shell-v4.4/#/"
      "http://127.0.0.1:11470/#/"
    )?*/
    .build()?;

  // Setup MPV
  unsafe {
    let window_id = webview.ns_window();
    let content_view: id = msg_send![window_id, contentView];
    let player_view: id = msg_send![class!(NSView), alloc];
    let frame: CGRect = msg_send![content_view, bounds];
    let _: () = msg_send![player_view, initWithFrame:frame];
    // This next line is actually done in wry: https://github.com/tauri-apps/wry/blob/dev/src/webview/wkwebview/mod.rs#L748
    let _: () = msg_send![player_view, setAutoresizingMask:NSViewHeightSizable | NSViewWidthSizable];
    let webview_view = webview.webview();
    // option 1
    //let _: () = msg_send![content_view, insertSubview:player_view at:0 as u32];    
    // option 2: instead of addSubview, we use insertSubview because the webview is already inserted
    //let _: () = msg_send![content_view, insertSubview:player_view belowSubview:webview.webview()];
    // option 3
    let _: () = msg_send![content_view, addSubview:player_view];
    //let _: () = msg_send![content_view, bringSubviewToFront:webview_view];


    //let subviews: id = msg_send![content_view, subviews];
    //let first_subview: id = msg_send![subviews, objectAtIndex: 0 as u32];
    // dbg!(first_subview, webview_view); // those are equal
  
    dbg!(webview_view, content_view, window_id, player_view);
    // ??     win.native("contentView")("setAutoresizesSubviews", $.YES); ??


    // we need a new thread here anyway
    let player_view_id = player_view as i64;

    //paradox spiral
    let mpv = libmpv::Mpv::new().unwrap();
    mpv.set_property("volume", 100).unwrap();
    mpv.set_property("terminal", "yes").unwrap();
    mpv.set_property("msg-level", "all=v").unwrap();
    mpv.set_property("wid", player_view_id).unwrap();
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
    NSRect frame = [[self->w contentView] bounds];
    self->wrapper = [[NSView alloc] initWithFrame:frame];
    [self->wrapper setAutoresizingMask:NSViewWidthSizable|NSViewHeightSizable];
    [[self->w contentView] addSubview:self->wrapper];
    [self->wrapper release];
    
    OR

    var playerView = $.NSView.extend("playerView");
    // not defininig playerView.isOpaque and .drawRect, although we may have to

    playerView.register();

    var size = win.native("contentView")("frame").size;
    var view = playerView("alloc")("initWithFrame", $.NSMakeRect(0,0,size.width,size.height));
    win.native("contentView")("addSubview", view);
    win.native("contentView")("setAutoresizesSubviews", $.YES);
    view("setAutoresizingMask", $.NSViewHeightSizable | $.NSViewWidthSizable);
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
