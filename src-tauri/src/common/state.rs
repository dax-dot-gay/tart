pub mod app_state {
    use std::sync::{Arc, Mutex};

    use serde::{Deserialize, Serialize};
    use tauri::{AppHandle, Emitter, Listener, Manager};
    use uuid::Uuid;

    use crate::common::term::TerminalManager;

    #[derive(Clone, Debug)]
    pub struct AppState {
        pub terminals: Arc<Mutex<TerminalManager>>
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    #[serde(tag = "type")]
    pub enum AppCommand {
        CreateTerminal{command: String, args: Option<Vec<String>>, title: Option<String>}
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    #[serde(tag = "type")]
    pub enum FrontendEvent {}

    #[derive(Serialize, Deserialize, Clone, Debug)]
    #[serde(tag = "type")]
    pub enum BackendEvent {
        TerminalRead{id: Uuid, data: String}
    }

    impl AppState {
        pub fn new() -> Self {
            AppState {terminals: Arc::new(Mutex::new(TerminalManager::new()))}
        }

        pub fn run_in_background(&self, app: &AppHandle) -> () {
            let handle = app.app_handle().clone();
            let internal_listener = app.listen("tart://internal", move |evt| {
                if let Ok(event) = serde_json::from_str::<BackendEvent>(evt.payload()) {
                    let state = handle.state::<AppState>();
                }
            });

            let handle = app.app_handle().clone();
            let command_listener = app.listen("tart://command", move |evt| {
                if let Ok(event) = serde_json::from_str::<AppCommand>(evt.payload()) {
                    let state = handle.state::<AppState>();
                    let mut terminals = state.terminals.lock().unwrap();
                    match event {
                        AppCommand::CreateTerminal { command, args, title } => {
                            terminals.create_terminal(handle.clone(), command, args, title);
                        }
                    }
                }
            });
        }
    }
}