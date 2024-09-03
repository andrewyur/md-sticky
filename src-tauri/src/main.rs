// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;
use tauri::{AppHandle, CustomMenuItem, Manager, Menu, MenuItem, Submenu};

fn main() {
    // here `"quit".to_string()` defines the menu item id, and the second parameter is the menu item label.
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let new_note = CustomMenuItem::new("new_note".to_string(), "New Note");
    let submenu = Submenu::new("File", Menu::new().add_item(quit).add_item(new_note));
    let menu = Menu::new()
        .add_native_item(MenuItem::Copy)
        .add_item(CustomMenuItem::new("hide", "Hide"))
        .add_submenu(submenu);

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
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
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
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
    .title("Sticky Note")
    .resizable(true)
    .inner_size(400.0, 400.0)
    .build()
    .expect("Failed to create window");
}
