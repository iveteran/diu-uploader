// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

#[cfg(desktop)]
mod tray;
#[cfg(desktop)]
mod uploader;

use tauri::{webview::PageLoadEvent, Emitter, Listener, Manager, RunEvent};

use serde::Serialize;

#[derive(Clone, Serialize)]
struct Reply {
    data: String,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[allow(unused_mut)]
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, uploader::my_upload,])
        .setup(move |app| {
            #[cfg(all(desktop, not(test)))]
            {
                let handle = app.handle();
                tray::create_tray(handle)?;
            }

            #[cfg(debug_assertions)]
            {
                let _webview = app.get_webview_window("main").unwrap();
                _webview.open_devtools();
            }

            Ok(())
        })
        .on_page_load(|webview, payload| {
            if payload.event() == PageLoadEvent::Finished {
                let _webview = webview.clone();
                webview.listen("js-event", move |event| {
                    println!("got js-event with message '{:?}'", event.payload());
                    let reply = Reply {
                        data: "something else".to_string(),
                    };
                    _webview
                        .emit("rust-event", Some(reply))
                        .expect("failed to emit");
                });
            }
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(move |_app_handle, _event| {
            #[cfg(all(desktop, not(test)))]
            match &_event {
                RunEvent::ExitRequested { api, code, .. } => {
                    // Keep the event loop running even if all windows are closed
                    // This allow us to catch tray icon events when there is no window
                    // if we manually requested an exit (code is Some(_)) we will let it go through
                    if code.is_none() {
                        api.prevent_exit();
                    }
                }
                RunEvent::WindowEvent {
                    event: tauri::WindowEvent::CloseRequested { api, .. },
                    label,
                    ..
                } => {
                    println!("closing window...");
                    // run the window destroy manually just for fun :)
                    // usually you'd show a dialog here to ask for confirmation or whatever
                    api.prevent_close();
                    let _ = _app_handle.get_webview_window(label).unwrap().hide();
                }
                _ => (),
            }
        })
}
