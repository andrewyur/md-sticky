// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// use serde::{Deserialize, Serialize};
use serde::{Deserialize, Serialize};
use std::sync::mpsc::Sender;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use std::{fs, sync::mpsc};
use tauri::{generate_context, AppHandle, CustomMenuItem, Manager, Menu, MenuItem, Submenu};

fn main() {
    // here `"quit".to_string()` defines the menu item id, and the second parameter is the menu item label.
    let quit = CustomMenuItem::new("quit".to_string(), "Quit").accelerator("CmdOrCtrl+Q");
    let close_note = CustomMenuItem::new("close_note".to_string(), "Close Current Note")
        .accelerator("CmdOrCtrl+W");
    let new_note =
        CustomMenuItem::new("new_note".to_string(), "New Note").accelerator("CmdOrCtrl+N");
    let clear_colors = CustomMenuItem::new("clear_colors", "Clear Colors");
    let file_submenu = Submenu::new(
        "File",
        Menu::new()
            .add_item(new_note)
            .add_item(close_note)
            .add_item(clear_colors)
            .add_item(quit),
    );

    let menu = Menu::new()
        .add_native_item(MenuItem::Copy)
        .add_item(CustomMenuItem::new("hide", "Hide"))
        .add_submenu(file_submenu);

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
            thread::spawn(move || loop {
                thread::sleep(Duration::from_millis(500));
                println!("{}", "Calling save notes!");
                save_notes(&handle_clone)
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
            "quit" => std::process::exit(0),
            "new_note" => {
                std::thread::spawn(move || {
                    create_new_sticky(event.window().app_handle());
                });
            }
            "close_note" => {
                if let Some(focused_window) = event.window().get_focused_window() {
                    let window_label = focused_window.label();

                    if window_label != "main" {
                        remove_window(window_label.to_string(), event.window().app_handle())
                    }
                }
            }
            "clear_colors" => {
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
            _ => {}
        })
        .run(generate_context!())
        .expect("error while building tauri application")
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
    // .visible(false)
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

const DEFAULT_COLORS: [&str; 8] = [
    "#fff9b1", "#10e17a", "#a6ccf5", "#67c6c0", "#ff9d48", "#b384bb", "#ff6f61", "#d32f2f",
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

    println!(
        "Saving! Current notes: {}, Current time: {:#?}",
        notes.len(),
        std::time::SystemTime::now()
    );

    fs::write(
        file_path,
        serde_json::to_string(&notes).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

// so many unwraps... this is bad code...
fn save_notes(app_handle: &tauri::AppHandle) {
    let mut contents: Vec<Note> = Vec::new();

    let (tx, rx) = mpsc::channel();

    // have to have a mutex bc if we get the list of windows through the app_handle, messages are sometimes sent to windows that dont have listeners attached yet
    let binding = app_handle.state::<Mutex<Vec<String>>>();
    let windows_ready = binding
        .lock()
        .expect("could not obtain lock for windows ready mutex");

    windows_ready.iter().for_each(|window_label| {
        let window = app_handle
            .get_window(window_label)
            .expect("could get the current window from window label");

        window
            .emit("save-contents-request", {})
            .expect("could not emit save-contents-request to the window");

        let sender: Sender<Note> = tx.clone();
        window.once("save-contents-response", move |event| {
            sender
                .send(
                    serde_json::from_str(
                        &event
                            .payload()
                            .expect("could not extract payload from event"),
                    )
                    .expect("Could not decode save-contents-response payload"),
                )
                .expect("could not send notes contents to main thread");
        });
    });

    for i in 0..windows_ready.len() {
        contents.push(
            rx.recv()
                .expect("could not recieve response contents from reciever"),
        );
        println!("Recieving save {} of {}", i + 1, windows_ready.len());
    }

    save_contents(contents, app_handle).expect("could not save contents");
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
