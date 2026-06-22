#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

use kepta_localdb::SqliteStore;
use kepta_sync::SyncEngine;
use std::sync::Arc;

pub struct TauriState {
    pub sync: Arc<SyncEngine>,
}

fn main() {
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");

    let store = rt.block_on(async {
        let data_dir = tauri::api::path::data_dir()
            .unwrap_or_default()
            .join("kepta")
            .join("kepta.db");
        let path = format!("sqlite://{}", data_dir.display());
        Arc::new(SqliteStore::new(&path).await.expect("local db"))
    });

    tauri::Builder::default()
        .manage(TauriState {
            sync: Arc::new(SyncEngine::new(
                store,
                std::env::var("KEPTA_API_URL").unwrap_or_else(|_| "https://app.kepta.app".into()),
                String::new(),
            )),
        })
        .invoke_handler(tauri::generate_handler![
            commands::sync_household,
        ])
        .run(tauri::generate_context!())
        .expect("tauri app failed");
}
