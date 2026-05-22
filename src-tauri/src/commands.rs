use crate::types::{Template, TeamSettings, TimerRecord};
use serde::{Deserialize, Serialize};
use std::fs::{File};
use std::io::{Read, Write};

#[derive(Debug, Serialize, Deserialize)]
struct TemplateStorageData {
    templates: Vec<Template>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TeamSettingsData {
    settings: TeamSettings,
}

#[derive(Debug, Serialize, Deserialize)]
struct TimerRecordsData {
    records: Vec<TimerRecord>,
}

fn get_data_dir() -> std::path::PathBuf {
    let dirs = directories::ProjectDirs::from("io", "hanzn", "dct").unwrap();
    let data_dir = dirs.data_dir();
    std::fs::create_dir_all(data_dir).unwrap();
    data_dir.to_path_buf()
}

fn get_templates_path() -> std::path::PathBuf {
    get_data_dir().join("templates.json")
}

fn get_team_settings_path() -> std::path::PathBuf {
    get_data_dir().join("team_settings.json")
}

fn get_timer_records_path() -> std::path::PathBuf {
    get_data_dir().join("timer_records.json")
}

fn load_timer_records_internal() -> Result<Vec<TimerRecord>, String> {
    let path = get_timer_records_path();
    if !path.exists() {
        return Ok(vec![]);
    }
    let mut file = File::open(&path).map_err(|e| e.to_string())?;
    let mut json = String::new();
    file.read_to_string(&mut json).map_err(|e| e.to_string())?;
    let data: TimerRecordsData = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    Ok(data.records)
}

#[tauri::command]
pub fn save_templates(templates: Vec<Template>) -> Result<(), String> {
    let path = get_templates_path();
    let data = TemplateStorageData { templates };
    let json = serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?;
    let mut file = File::create(&path).map_err(|e| e.to_string())?;
    file.write_all(json.as_bytes()).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn load_templates() -> Result<Vec<Template>, String> {
    let path = get_templates_path();
    if !path.exists() {
        return Ok(vec![]);
    }
    let mut file = File::open(&path).map_err(|e| e.to_string())?;
    let mut json = String::new();
    file.read_to_string(&mut json).map_err(|e| e.to_string())?;
    let data: TemplateStorageData = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    Ok(data.templates)
}

#[tauri::command]
pub fn save_team_settings(settings: TeamSettings) -> Result<(), String> {
    let path = get_team_settings_path();
    let data = TeamSettingsData { settings };
    let json = serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?;
    let mut file = File::create(&path).map_err(|e| e.to_string())?;
    file.write_all(json.as_bytes()).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn load_team_settings() -> Result<TeamSettings, String> {
    let path = get_team_settings_path();
    if !path.exists() {
        return Ok(TeamSettings::default());
    }
    let mut file = File::open(&path).map_err(|e| e.to_string())?;
    let mut json = String::new();
    file.read_to_string(&mut json).map_err(|e| e.to_string())?;
    let data: TeamSettingsData = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    Ok(data.settings)
}

#[tauri::command]
pub fn save_timer_record(record: TimerRecord) -> Result<(), String> {
    let path = get_timer_records_path();
    let mut records = load_timer_records_internal().unwrap_or_default();

    if let Some(existing) = records.iter_mut().find(|r| r.id == record.id) {
        *existing = record;
    } else {
        records.insert(0, record);
    }

    if records.len() > 100 {
        records.truncate(100);
    }

    let data = TimerRecordsData { records };
    let json = serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?;
    let mut file = File::create(&path).map_err(|e| e.to_string())?;
    file.write_all(json.as_bytes()).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn load_timer_records() -> Result<Vec<TimerRecord>, String> {
    load_timer_records_internal()
}

#[tauri::command]
pub fn delete_timer_record(id: String) -> Result<(), String> {
    let path = get_timer_records_path();
    let mut records = load_timer_records_internal().unwrap_or_default();
    records.retain(|r| r.id != id);

    let data = TimerRecordsData { records };
    let json = serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?;
    let mut file = File::create(&path).map_err(|e| e.to_string())?;
    file.write_all(json.as_bytes()).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn export_template(template: Template, path: String) -> Result<(), String> {
    let json = serde_json::to_string_pretty(&template).map_err(|e| e.to_string())?;
    let mut file = File::create(&path).map_err(|e| e.to_string())?;
    file.write_all(json.as_bytes()).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn import_template(path: String) -> Result<Template, String> {
    let mut file = File::open(&path).map_err(|e| e.to_string())?;
    let mut json = String::new();
    file.read_to_string(&mut json).map_err(|e| e.to_string())?;
    let mut template: Template = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    template.id = generate_id();
    template.created_at = chrono::Utc::now().to_rfc3339();
    template.updated_at = template.created_at.clone();
    Ok(template)
}

#[tauri::command]
pub fn generate_id() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let timestamp = chrono::Utc::now().timestamp().to_string();
    let random = rng.gen::<u64>().to_string();
    format!("{}_{}", timestamp, random)
}
