#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::{
        extract::Extension,
        routing::{get, post},
        Router,
    };
    use axum_example::app::*;
    use axum_example::fileserv::file_and_error_handler;
    use leptos::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use std::sync::Arc;

    simple_logger::init_with_level(log::Level::Debug).expect("couldn't initialize logging");

    // Setting get_configuration(None) means we'll be using cargo-leptos's env values
    // For deployment these variables are:
    // <https://github.com/leptos-rs/start-axum#executing-a-server-on-a-remote-machine-without-the-toolchain>
    // Alternately a file can be specified such as Some("Cargo.toml")
    // The file would need to be included with the executable when moved to deployment
    let conf = get_configuration(None).await.unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(|cx| view! { cx, <App/> }).await;

    // build our application with a route
    let app = Router::new()
        .route("/api/*fn_name", post(leptos_axum::handle_server_fns))
        .route("/ws", get(websocket))
        .leptos_routes(leptos_options.clone(), routes, |cx| view! { cx, <App/> })
        .fallback(file_and_error_handler)
        .layer(Extension(Arc::new(leptos_options)));

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    log!("listening on http://{}", &addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for a purely client-side app
    // see lib.rs for hydration function instead
}

#[cfg(feature = "ssr")]
pub async fn websocket(ws: axum::extract::WebSocketUpgrade) -> axum::response::Response {
    ws.on_upgrade(handle_socket)
}

#[cfg(feature = "ssr")]
async fn handle_socket(socket: axum::extract::ws::WebSocket) {
    use std::{sync::Arc, time::Duration};

    use axum_example::app::Count;
    use leptos_server_signal::ServerSignal;
    use tokio::sync::Mutex;

    let websocket = Arc::new(Mutex::new(socket));
    let mut count = ServerSignal::<Count>::new(websocket);

    loop {
        tokio::time::sleep(Duration::from_millis(10)).await;
        if count.with(|count| count.value += 1).await.is_err() {
            break;
        }
    }
}