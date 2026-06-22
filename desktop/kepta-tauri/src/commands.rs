use tauri::State;
use uuid::Uuid;

use crate::TauriState;

#[tauri::command]
pub async fn sync_household(
    state: State<'_, TauriState>,
    household_id: String,
) -> Result<(), String> {
    let id = household_id.parse::<Uuid>().map_err(|e| e.to_string())?;
    state.sync.sync(id).await.map_err(|e| e.to_string())
}
