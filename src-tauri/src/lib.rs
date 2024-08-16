use common::{app_state::{ApplicationState, StateContainer}, terminal::Terminal};
use tauri::Manager;

mod common;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(StateContainer::new());
            app.state::<StateContainer>().state().add_terminal(Terminal::new(24, 80, "bash".to_string(), None, None));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
