mod commands;
mod types;

pub use types::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::save_templates,
            commands::load_templates,
            commands::save_team_settings,
            commands::load_team_settings,
            commands::save_timer_record,
            commands::load_timer_records,
            commands::delete_timer_record,
            commands::export_template,
            commands::import_template,
            commands::generate_id
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
