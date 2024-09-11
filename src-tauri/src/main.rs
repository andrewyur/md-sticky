// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// use serde::{Deserialize, Serialize};
use serde::{Deserialize, Serialize};
use std::cmp;
use std::collections::HashSet;
use std::fs;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{
    generate_context, AppHandle, CustomMenuItem, Manager, Menu, PhysicalPosition, Submenu, Window,
};
use window_shadows;

const QUIT: &str = "quit";
const CLOSE_NOTE: &str = "close_note";
const NEW_NOTE: &str = "new_note";
const CLEAR_COLORS: &str = "clear_colors";

const SNAP_UP: &str = "snap_up";
const SNAP_DOWN: &str = "snap_down";
const SNAP_LEFT: &str = "snap_left";
const SNAP_RIGHT: &str = "snap_right";
const NEXT_WINDOW: &str = "next_window";
const PREV_WINDOW: &str = "past_window";
const FIT_TEXT: &str = "fit_text";

const CUT: &str = "copy";
const COPY: &str = "cut";
const SELECT_ALL: &str = "select_all";
const PASTE: &str = "paste";

const MAIN: &str = "main";

#[cfg(any(windows, target_os = "macos"))]
fn main() {
    // here `"quit".to_string()` defines the menu item id, and the second parameter is the menu item label.
    let quit = CustomMenuItem::new(QUIT, "Quit").accelerator("CmdOrCtrl+Q");
    let close_note =
        CustomMenuItem::new(CLOSE_NOTE, "Close Current Note").accelerator("CmdOrCtrl+W");
    let new_note = CustomMenuItem::new(NEW_NOTE, "New Note").accelerator("CmdOrCtrl+N");
    let clear_colors = CustomMenuItem::new(CLEAR_COLORS, "Clear Colors");
    let file_submenu = Submenu::new(
        "File",
        Menu::new()
            .add_item(new_note)
            .add_item(close_note)
            .add_item(clear_colors)
            .add_item(quit),
    );

    let snap_up = CustomMenuItem::new(SNAP_UP, "Snap Up").accelerator("CmdOrCtrl+Alt+Up");
    let snap_down = CustomMenuItem::new(SNAP_DOWN, "Snap Down").accelerator("CmdOrCtrl+Alt+Down");
    let snap_left = CustomMenuItem::new(SNAP_LEFT, "Snap Left").accelerator("CmdOrCtrl+Alt+Left");
    let snap_right =
        CustomMenuItem::new(SNAP_RIGHT, "Snap Right").accelerator("CmdOrCtrl+Alt+Right");
    let next_window =
        CustomMenuItem::new(NEXT_WINDOW, "Next Window").accelerator("CmdOrCtrl+Slash");
    let prev_window =
        CustomMenuItem::new(PREV_WINDOW, "Past Window").accelerator("CmdOrCtrl+Alt+Slash");
    let fit_text = CustomMenuItem::new(FIT_TEXT, "Fit Text").accelerator("CmdOrCtrl+F");
    let window_submenu = Submenu::new(
        "Window",
        Menu::new()
            .add_item(snap_up)
            .add_item(snap_down)
            .add_item(snap_left)
            .add_item(snap_right)
            .add_item(next_window)
            .add_item(prev_window)
            .add_item(fit_text),
    );

    let copy = CustomMenuItem::new(COPY, "Copy").accelerator("CmdOrCtrl+C");
    let paste = CustomMenuItem::new(PASTE, "Paste").accelerator("CmdOrCtrl+V");
    let cut = CustomMenuItem::new(CUT, "Cut").accelerator("CmdOrCtrl+X");
    let select_all = CustomMenuItem::new(SELECT_ALL, "Select All").accelerator("CmdOrCtrl+A");
    let edit_submenu = Submenu::new(
        "Edit",
        Menu::new()
            .add_item(copy)
            .add_item(paste)
            .add_item(cut)
            .add_item(select_all),
    );

    let menu = Menu::new()
        .add_submenu(file_submenu)
        .add_submenu(edit_submenu)
        .add_submenu(window_submenu);

    tauri::Builder::default()
        .setup(|app| {
            let path_buf = app
                .handle()
                .path_resolver()
                .app_data_dir()
                .expect("Could not resolve appdata directory");

            let app_data_path = path_buf.as_path();

            if !app_data_path.exists() {
                fs::create_dir(app_data_path).expect("Could not create the appdata directory");
            }

            let notes = read_contents(app.handle()).expect("Could not read save file");

            notes.into_iter().for_each(|note| {
                let window = create_new_sticky(app.handle());
                let window_clone = window.clone();
                window.once("ready", move |_event| {
                    window_clone
                        .emit("init", note)
                        .expect("Error emitting init event");
                });
            });

            let handle_clone = app.handle().clone();
            let mut save_fail_ct = 0;
            thread::spawn(move || loop {
                thread::sleep(Duration::from_millis(500));
                if let Err(_) = save_notes(&handle_clone) {
                    save_fail_ct += 1;
                } else {
                    save_fail_ct = 0;
                }
                if save_fail_ct > 2 {
                    handle_clone.restart();
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            add_color,
            get_colors,
            remove_window
        ])
        .manage(Mutex::new(0 as u32))
        .manage(Mutex::new(Vec::<String>::new()))
        .menu(menu)
        .on_menu_event(|event| match event.menu_item_id() {
            QUIT => std::process::exit(0),
            NEW_NOTE => {
                std::thread::spawn(move || {
                    create_new_sticky(event.window().app_handle());
                });
            }
            CLOSE_NOTE => {
                if let Some(focused_window) = event.window().get_focused_window() {
                    let window_label = focused_window.label();

                    if window_label != MAIN {
                        remove_window(window_label.to_string(), event.window().app_handle())
                    }
                }
            }
            CLEAR_COLORS => {
                let path_buf = event
                    .window()
                    .app_handle()
                    .path_resolver()
                    .app_data_dir()
                    .expect("could not resolve app data directory")
                    .join("colors.json");

                let file_path = path_buf.as_path();

                fs::remove_file(file_path).expect("Could not remove colors save file");
            }
            m if [CUT, COPY, PASTE, SELECT_ALL].contains(&m) => {
                if let Some(focused_window) = event.window().get_focused_window() {
                    focused_window
                        .emit(m, {})
                        .expect("could not send copy event");
                }
            }
            m if [SNAP_DOWN, SNAP_UP, SNAP_LEFT, SNAP_RIGHT].contains(&m) => {
                if let Some(focused_window) = event.window().get_focused_window() {
                    snap_window(focused_window, m);
                }
            }
            NEXT_WINDOW => {
                if let Some(focused_window) = event.window().get_focused_window() {
                    let mut collect = event
                        .window()
                        .app_handle()
                        .windows()
                        .into_iter()
                        .collect::<Vec<(String, Window)>>();

                    collect.sort_by(|a, b| a.0.cmp(&b.0));

                    let mut next = false;
                    for (label, window) in collect.iter().cycle() {
                        if next && label != MAIN {
                            window.set_focus().expect("Could not set focused");
                            break;
                        }
                        if window.label() == focused_window.label() {
                            next = true;
                        }
                    }
                }
            }
            PREV_WINDOW => {
                if let Some(focused_window) = event.window().get_focused_window() {
                    let mut prev_window: Option<Window> = None;

                    let mut collect = event
                        .window()
                        .app_handle()
                        .windows()
                        .into_iter()
                        .collect::<Vec<(String, Window)>>();

                    collect.sort_by(|a, b| a.0.cmp(&b.0));
                    for (label, window) in collect.iter().cycle() {
                        if window.label() == focused_window.label() && prev_window.is_some() {
                            prev_window
                                .unwrap()
                                .set_focus()
                                .expect("Could not set focused");
                            break;
                        }

                        if label != MAIN {
                            prev_window = Some(window.clone());
                        }
                    }
                }
            }
            FIT_TEXT => {
                if let Some(focused_window) = event.window().get_focused_window() {
                    focused_window
                        .emit(FIT_TEXT, {})
                        .expect("Could not emit event!")
                }
            }
            _ => {}
        })
        .on_window_event(|event| match event.event() {
            tauri::WindowEvent::Focused(focused) => {
                window_shadows::set_shadow(event.window(), *focused).unwrap();
            }
            _ => {}
        })
        .run(generate_context!())
        .expect("error while building tauri application")
}

fn snap_window(window: Window, direction: &str) {
    let window_position = window.outer_position().unwrap();
    let window_size = window.outer_size().unwrap();
    let current_monitor = window
        .current_monitor()
        .unwrap()
        .expect("monitor could not be detected");

    window
        .set_position(match direction {
            SNAP_LEFT => PhysicalPosition {
                x: window
                    .app_handle()
                    .windows()
                    .iter()
                    .filter(|(label, _)| *label != MAIN && *label != window.label())
                    .filter_map(|(_label, window)| {
                        let position = window.outer_position().unwrap();
                        let size = window.outer_size().unwrap();

                        if window_overlap(
                            position.y,
                            size.height as i32,
                            window_position.y,
                            window_size.height as i32,
                        ) {
                            Some(position.x + size.width as i32)
                        } else {
                            None
                        }
                    })
                    .filter(|edge| *edge < window_position.x as i32)
                    .max()
                    .unwrap_or(0)
                    + 20,
                y: window_position.y,
            },

            SNAP_UP => PhysicalPosition {
                x: window_position.x,
                y: window
                    .app_handle()
                    .windows()
                    .iter()
                    .filter(|(label, _)| *label != MAIN && *label != window.label())
                    .filter_map(|(_label, window)| {
                        let position = window.outer_position().unwrap();
                        let size = window.outer_size().unwrap();

                        if window_overlap(
                            position.x,
                            size.width as i32,
                            window_position.x,
                            window_size.width as i32,
                        ) {
                            Some(position.y + size.height as i32)
                        } else {
                            None
                        }
                    })
                    .filter(|edge| *edge < window_position.y as i32)
                    .max()
                    .unwrap_or(0)
                    + 20,
            },

            SNAP_RIGHT => PhysicalPosition {
                x: window
                    .app_handle()
                    .windows()
                    .iter()
                    .filter(|(label, _)| *label != MAIN && *label != window.label())
                    .filter_map(|(_label, window)| {
                        let position = window.outer_position().unwrap();
                        let size = window.outer_size().unwrap();

                        if window_overlap(
                            position.y,
                            size.height as i32,
                            window_position.y,
                            window_size.height as i32,
                        ) {
                            Some(position.x as i32)
                        } else {
                            None
                        }
                    })
                    .filter(|edge| *edge > window_position.x as i32)
                    .max()
                    .unwrap_or((current_monitor.size().width - window_size.width) as i32)
                    - 20,
                y: window_position.y,
            },

            SNAP_DOWN => PhysicalPosition {
                x: window_position.x,
                y: window
                    .app_handle()
                    .windows()
                    .iter()
                    .filter(|(label, _)| *label != MAIN && *label != window.label())
                    .filter_map(|(_label, window)| {
                        let position = window.outer_position().unwrap();
                        let size = window.outer_size().unwrap();

                        if window_overlap(
                            position.x,
                            size.width as i32,
                            window_position.x,
                            window_size.width as i32,
                        ) {
                            Some(position.y as i32)
                        } else {
                            None
                        }
                    })
                    .filter(|edge| *edge > window_position.y as i32)
                    .max()
                    .unwrap_or((current_monitor.size().height - window_size.height) as i32)
                    - 20,
            },

            _ => PhysicalPosition { x: 0, y: 0 },
        })
        .expect("Could not set window position")
}

fn window_overlap(start_1: i32, len_1: i32, start_2: i32, len_2: i32) -> bool {
    let end_1 = start_1 + len_1;
    let end_2 = start_2 + len_2;

    let overlap_start = cmp::max(start_1, start_2);
    let overlap_end = cmp::min(end_1, end_2);
    overlap_end - overlap_start > 20 // 20px is the gap value
}

fn create_new_sticky(handle: AppHandle) -> tauri::Window {
    let wc_binding = handle.state::<Mutex<u32>>();
    let mut window_count = wc_binding
        .lock()
        .expect("error obtaining lock for window count mutex");
    *window_count += 1;

    let window_label = format!("new_sticky_window_{}", window_count);

    let window = tauri::WindowBuilder::new(
        &handle,
        window_label.clone(),
        tauri::WindowUrl::App("index.html".into()),
    )
    .decorations(false)
    .resizable(true)
    .visible(false)
    .inner_size(300.0, 250.0)
    .build()
    .expect("Failed to create window");

    let handle_clone = handle.clone();
    let window_label_clone = window_label.clone();
    window.once("ready", move |_| {
        let wr_binding = handle_clone.state::<Mutex<Vec<String>>>();
        let mut windows_ready = wr_binding
            .lock()
            .expect("error obtaining lock for windows ready mutex");

        windows_ready.push(window_label_clone);
    });

    window
}

const DEFAULT_COLORS: [&str; 7] = [
    "#fff9b1", "#81B7DD", "#65A65B", "#AAD2CA", "#98C260", "#E1A1B1", "#B98CB3",
];

#[tauri::command]
fn add_color(color: &str, app_handle: tauri::AppHandle) -> Result<(), String> {
    let path_buf = app_handle
        .path_resolver()
        .app_data_dir()
        .expect("could not resolve app data directory")
        .join("colors.json");

    let file_path = path_buf.as_path();

    let mut colors = get_colors(app_handle).map_err(|e| e.to_string())?;

    if !colors.iter().any(|c| *c == color) {
        colors.push(color.to_string());
        fs::write(
            file_path,
            serde_json::to_string(&colors).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
fn get_colors(app_handle: tauri::AppHandle) -> Result<Vec<String>, String> {
    let path_buf = app_handle
        .path_resolver()
        .app_data_dir()
        .expect("could not resolve app data directory")
        .join("colors.json");

    let file_path = path_buf.as_path();

    if file_path.exists() {
        serde_json::from_str(&fs::read_to_string(file_path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())
    } else {
        Ok(DEFAULT_COLORS
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>())
    }
}

#[derive(Clone, Deserialize, Serialize)]
struct Note {
    color: String,
    contents: String,
    x: u32,
    y: u32,
    height: u32,
    width: u32,
    label: String,
}

#[tauri::command]
fn read_contents(app_handle: tauri::AppHandle) -> Result<Vec<Note>, String> {
    let path_buf = app_handle
        .path_resolver()
        .app_data_dir()
        .expect("could not resolve app data directory")
        .join("notes.json");

    let file_path = path_buf.as_path();

    let file_content = if file_path.exists() {
        fs::read_to_string(file_path).map_err(|e| e.to_string())?
    } else {
        fs::File::create(file_path).expect("could not create a notes save file");
        String::new()
    };

    let notes: Vec<Note> = if file_content.len() > 0 {
        serde_json::from_str(&file_content).map_err(|e| e.to_string())?
    } else {
        Vec::new()
    };

    Ok(notes)
}

fn save_contents(mut notes: Vec<Note>, app_handle: &tauri::AppHandle) -> Result<(), String> {
    let path_buf = app_handle
        .path_resolver()
        .app_data_dir()
        .expect("could not resolve app data directory")
        .join("notes.json");

    let file_path = path_buf.as_path();

    notes = notes.into_iter().filter(|n| n.label != "main").collect();

    fs::write(
        file_path,
        serde_json::to_string(&notes).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

// so many unwraps... this is bad code...
fn save_notes(app_handle: &tauri::AppHandle) -> Result<(), String> {
    let mut contents: Vec<Note> = Vec::new();

    let (tx, rx) = mpsc::channel();

    // have to have a mutex bc if we get the list of windows through the app_handle, messages are sometimes sent to windows that dont have listeners attached yet
    let binding = app_handle.state::<Mutex<Vec<String>>>();
    let windows_ready = binding
        .lock()
        .expect("could not obtain lock for windows ready mutex");

    let responded_windows = Arc::new(Mutex::new(HashSet::new()));

    windows_ready.iter().for_each(|window_label| {
        let window = app_handle
            .get_window(window_label)
            .expect("could get the current window from window label");

        let sender = tx.clone();
        let window_clone = window.clone();
        let window_label_clone = window_label.clone();
        let responded_windows_clone = Arc::clone(&responded_windows);

        window.listen("save-contents-response", move |event| {
            // sometimes this event gets called multiple times, not sure why...
            window_clone.unlisten(event.id());

            let mut binding = responded_windows_clone
                .lock()
                .expect("could not obtain lock on responded windows mutex");

            if binding.contains(&window_label_clone) {
                return; // Skip if this window has already responded
            }

            binding.insert(window_label_clone.clone());
            if let Err(_) = sender.send(
                serde_json::from_str(
                    &event
                        .payload()
                        .expect("could not extract payload from event"),
                )
                .expect("Could not decode save-contents-response payload"),
            ) {}
        });

        window
            .emit("save-contents-request", {})
            .expect("could not emit save-contents-request to the window");
    });

    for _ in 0..windows_ready.len() {
        contents.push(
            rx.recv_timeout(Duration::from_millis(100))
                .map_err(|_| String::from("Timeout failed"))?,
        );
    }

    save_contents(contents, app_handle).expect("could not save contents");

    Ok(())
}

#[tauri::command]
// sometimes after closing a window, tauri wry's webview tree does not get updated properly, and will crash on the next menu event. I cannot do anything to fix this unfortunately...
fn remove_window(label: String, app_handle: tauri::AppHandle) {
    // block scope to unlock mutex before closing window
    {
        let wr_binding = app_handle.state::<Mutex<Vec<String>>>();
        let mut windows_ready = wr_binding
            .lock()
            .expect("could not obtain lock on windows_ready mutex");

        windows_ready.retain(|s| *s != label);
    }

    app_handle
        .get_window(&label)
        .expect("could not get window from label")
        .close()
        .expect("could not close window");
}
