use std::sync::Arc;

use assets::Assets;
use gpui::{
    point, px, size, App, AppContext, Bounds, TitlebarOptions, WindowBounds, WindowOptions,
};
use reqwest_client::ReqwestClient;

fn main() {
    App::new()
        .with_assets(Assets)
        .with_http_client(Arc::new(ReqwestClient::new()))
        .run(|cx: &mut AppContext| {
            ui::init(cx);

            let window_options = window_options(cx);
            if let Err(e) = cx.open_window(window_options, dga::App::root) {
                eprintln!("{:?}", e);
            }
        });
}

fn window_options(cx: &mut AppContext) -> WindowOptions {
    let window_size = utils::read_window().unwrap_or(size(px(1000.0), px(700.0)));
    let window_bounds = Bounds::centered(None, window_size, cx);

    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(window_bounds)),
        titlebar: Some(TitlebarOptions {
            title: None,
            appears_transparent: true,
            traffic_light_position: Some(point(px(9.0), px(9.0))),
        }),
        window_min_size: Some(size(px(360.0), px(240.0))),
        ..Default::default()
    }
}
