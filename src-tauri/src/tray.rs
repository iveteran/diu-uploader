// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg(all(desktop, not(test)))]

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, Runtime,
};
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_shell::{process::CommandEvent, ShellExt};

const DIU_SITE: &str = "https://diu.matrixworks.cn";
//const DIU_PROGRAM : &str = "/home/yuu/DIU";  // absolute path
const DIU_PROGRAM: &str = "DIU"; // find in $PATH, could copy DIU program to $HOME/.local/bin

pub fn create_tray<R: Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<()> {
    let toggle_i = MenuItem::with_id(app, "toggle", "Hide", true, None::<&str>)?;
    let console_i = MenuItem::with_id(app, "open-console", "Console (Client)", true, None::<&str>)?;
    let browser_i =
        MenuItem::with_id(app, "open-browser", "Console (Browser)", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&toggle_i, &console_i, &browser_i, &quit_i])?;

    let _ = TrayIconBuilder::with_id("tray-1")
        .tooltip("DIU")
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "quit" => {
                app.exit(0);
            }
            "toggle" => {
                if let Some(window) = app.get_webview_window("main") {
                    let new_title = if window.is_visible().unwrap_or_default() {
                        let _ = window.hide();
                        "Show"
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                        "Hide"
                    };
                    toggle_i.set_text(new_title).unwrap();
                }
            }
            "open-console" => {
                // Refer: https://docs.rs/tauri-plugin-shell/latest/tauri_plugin_shell/process/struct.Command.html
                let handle = app.clone();
                tauri::async_runtime::spawn(async move {
                    let (mut rx, mut child) = handle
                        .shell()
                        .command(DIU_PROGRAM)
                        //.args(["arg1", "arg2"])
                        .spawn()
                        .expect("Failed to spawn DIU");

                    let mut i = 0;
                    while let Some(event) = rx.recv().await {
                        if let CommandEvent::Stdout(line) = event {
                            println!(
                                "DIU program got event: {}",
                                String::from_utf8(line).unwrap()
                            );
                            i += 1;
                            if i == 4 {
                                child.write("Message from Rust\n".as_bytes()).unwrap();
                                i = 0;
                            }
                        }
                    }
                });
            }
            "open-browser" => {
                let opener = app.opener();
                // Opens the URL in the default browser
                let _ = opener.open_url(DIU_SITE, None::<&str>);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app);

    Ok(())
}
