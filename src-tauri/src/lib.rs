use common::app_state::{AppCommand, AppState, CommandResult};
use tauri::Manager;

mod common;

#[tauri::command]
async fn execute_command(
    app: tauri::AppHandle,
    command: AppCommand
) -> Result<CommandResult, String> {
    let state = app.state::<AppState>();
    Ok(state.run_command(&app.clone(), command).await)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(AppState::new());

            let state = app.state::<AppState>();
            state.run_in_background(app.handle());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![execute_command])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
