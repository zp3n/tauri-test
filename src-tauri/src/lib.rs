// Learn more about Tauri commands at
// https://tauri.app/develop/calling-rust/

mod audio;

use std::sync::Mutex;
use tauri::State;

use audio::AudioEngine;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

struct AppState {
    audio: Mutex<AudioEngine>,
}

#[tauri::command]
fn start_monitor(
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut audio = state
        .audio
        .lock()
        .map_err(|_| "AudioEngine lock error")?;

    audio.start()
}

#[tauri::command]
fn stop_monitor(
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut audio = state
        .audio
        .lock()
        .map_err(|_| "AudioEngine lock error")?;

    audio.stop();

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            audio: Mutex::new(AudioEngine::new()),
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            start_monitor,
            stop_monitor
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

