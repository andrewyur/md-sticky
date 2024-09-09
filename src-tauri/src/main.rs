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
    let new_note =
        CustomMenuItem::new("new_note".to_string(), "New Note").accelerator("CmdOrCtrl+N");
    let clear_colors = CustomMenuItem::new("clear_colors", "Clear Colors");
    let file_submenu = Submenu::new(
        "File",
        Menu::new()
            .add_item(new_note)
            .add_item(clear_colors)
            .add_item(quit),
    );

    let menu = Menu::new()
        .add_native_item(MenuItem::Copy)
        .add_item(CustomMenuItem::new("hide", "Hide"))
        .add_submenu(file_submenu);

    tauri::Builder::default()
        .setup(|app| {
            let path_buf = app.handle().path_resolver().app_data_dir().unwrap();

            let app_data_path = path_buf.as_path();

            if !app_data_path.exists() {
                fs::create_dir(app_data_path).unwrap();
            }

            let notes = read_contents(app.handle()).unwrap();

            notes.into_iter().for_each(|note| {
                let window = create_new_sticky(app.handle());
                let window_clone = window.clone();
                window.once("ready", move |_event| {
                    window_clone.emit("init", note).unwrap();
                });
            });

            let handle_clone = app.handle().clone();
            thread::spawn(move || loop {
                thread::sleep(Duration::from_secs(1));
                save_notes(&handle_clone)
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![add_color, get_colors])
        .manage(Mutex::new(0 as u32))
        .menu(menu)
        .on_menu_event(|event| match event.menu_item_id() {
            "quit" => std::process::exit(0),
            "new_note" => {
                std::thread::spawn(move || {
                    create_new_sticky(event.window().app_handle());
                });
            }
            "clear_colors" => {
                let path_buf = event
                    .window()
                    .app_handle()
                    .path_resolver()
                    .app_data_dir()
                    .unwrap()
                    .join("colors.json");

                let file_path = path_buf.as_path();

                fs::remove_file(file_path).unwrap();
            }
            _ => {}
        })
        .run(generate_context!())
        .expect("error while building tauri application")
}

fn create_new_sticky(handle: AppHandle) -> tauri::Window {
    let binding = handle.state::<Mutex<u32>>();
    let mut window_count = binding.lock().unwrap();
    *window_count += 1;
    tauri::WindowBuilder::new(
        &handle,
        format!("new_sticky_window_{}", window_count),
        tauri::WindowUrl::App("index.html".into()),
    )
    .decorations(false)
    .resizable(true)
    .inner_size(300.0, 250.0)
    .build()
    .expect("Failed to create window")
}

const DEFAULT_COLORS: [&str; 8] = [
    "#fff9b1", "#10e17a", "#a6ccf5", "#67c6c0", "#ff9d48", "#b384bb", "#ff6f61", "#d32f2f",
];

#[tauri::command]
fn add_color(color: &str, app_handle: tauri::AppHandle) -> Result<(), String> {
    let path_buf = app_handle
        .path_resolver()
        .app_data_dir()
        .unwrap()
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
        .unwrap()
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
        .unwrap()
        .join("notes.json");

    let file_path = path_buf.as_path();

    let file_content = if file_path.exists() {
        fs::read_to_string(file_path).map_err(|e| e.to_string())?
    } else {
        fs::File::create(file_path).unwrap();
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
        .unwrap()
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
fn save_notes(app_handle: &tauri::AppHandle) {
    let mut contents: Vec<Note> = Vec::new();

    let (tx, rx) = mpsc::channel();

    app_handle.windows().iter().for_each(|(_, window)| {
        if window.label() != "main" {
            window.emit("save-contents-request", {}).unwrap();

            let sender: Sender<Note> = tx.clone();
            window.once("save-contents-response", move |event| {
                sender
                    .send(serde_json::from_str(&event.payload().unwrap()).unwrap())
                    .unwrap();
            });
        }
    });

    for _ in 0..(app_handle.windows().len() - 1) {
        contents.push(rx.recv().unwrap());
    }

    save_contents(contents, app_handle).unwrap();
}
