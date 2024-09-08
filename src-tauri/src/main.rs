// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::Mutex;
use tauri::{AppHandle, CustomMenuItem, Manager, Menu, MenuItem, Submenu};

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
            let path_buf = app.app_handle().path_resolver().app_data_dir().unwrap();

            let app_data_path = path_buf.as_path();

            if !app_data_path.exists() {
                fs::create_dir(app_data_path).unwrap();
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![add_color, get_colors])
        .manage(Mutex::new(0 as u32))
        .menu(menu)
        .on_menu_event(|event| match event.menu_item_id() {
            "quit" => {
                std::process::exit(0);
            }
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
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn create_new_sticky(handle: AppHandle) {
    let binding = handle.state::<Mutex<u32>>();
    let mut window_count = binding.lock().unwrap();
    *window_count += 1;
    let _local_window = tauri::WindowBuilder::new(
        &handle,
        format!("new_sticky_window_{}", window_count),
        tauri::WindowUrl::App("index.html".into()),
    )
    .decorations(false)
    .resizable(true)
    .inner_size(400.0, 400.0)
    .build()
    .expect("Failed to create window");
}

// - Yellow:
// - White:
// - Light Orange:
// - Olive:
// - Green:
// - Pastel Blue:
// - Aqua:
// - Blue:
// - Orange:
// - Pink:
// - Red:
// - Purple:
// default is yellow

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

    let file_content = if file_path.exists() {
        fs::read_to_string(file_path).map_err(|e| e.to_string())?
    } else {
        fs::File::create(file_path).unwrap();
        String::new()
    };

    let mut colors = if file_content.len() > 0 {
        serde_json::from_str(&file_content).map_err(|e| e.to_string())?
    } else {
        DEFAULT_COLORS
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
    };

    // Check if color already exists
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
