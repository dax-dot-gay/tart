pub mod app_state {
    use std::sync::{Arc, Mutex};

    use portable_pty::PtySize;
    use serde::{Deserialize, Serialize};
    use serde_json::Value;
    use tauri::{AppHandle, Emitter, Listener, Manager};
    use uuid::Uuid;

    use crate::common::term::{PtySizeDef, TerminalCommand, TerminalInfo, TerminalManager, TerminalMessage};

    #[derive(Clone, Debug)]
    pub struct AppState {
        pub terminals: Arc<Mutex<TerminalManager>>
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    #[serde(tag = "type")]
    pub enum AppCommand {
        CreateTerminal{command: String, args: Option<Vec<String>>, title: Option<String>},
        RemoveTerminal{id: Uuid},
        WriteData{id: Uuid, data: String},
        GetTerminals{},
        Resize{
            id: Uuid, 

            #[serde(with = "PtySizeDef")]
            size: PtySize
        }
    }

    impl AppCommand {
        pub fn wrap(&self) -> AppCommandWrapper {
            AppCommandWrapper {
                id: Uuid::new_v4(),
                command: self.clone()
            }
        }
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct AppCommandWrapper {
        pub id: Uuid,
        pub command: AppCommand
    }

    impl AppCommandWrapper {
        pub fn emit_result(&self, app: &AppHandle, result: Result<impl Serialize, impl Serialize>) -> () {
            let serialized = match result {
                Ok(v) => Ok(serde_json::to_value(v).or::<()>(Ok(serde_json::to_value("Failed to serialize successful value.").unwrap())).unwrap()),
                Err(v) => Err(serde_json::to_value(v).or::<()>(Ok(serde_json::to_value("Failed to serialize error value.").unwrap())).unwrap())
            };

            let res = CommandResult {
                id: self.id.clone(),
                command: self.command.clone(),
                result: serialized
            };

            let _ = app.emit("tart://result", res);
        }
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    #[serde(tag = "type")]
    pub enum FrontendEvent {
        TerminalRead{id: Uuid, data: String},
        TerminalCreated{id: Uuid},
        TerminalRemoved{id: Uuid},
        TerminalResized{
            id: Uuid,
            
            #[serde(with = "PtySizeDef")]
            size: PtySize
        }
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    #[serde(tag = "type")]
    pub enum BackendEvent {
        TerminalRead{id: Uuid, data: String}
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct CommandResult {
        id: Uuid,
        command: AppCommand,
        result: Result<Value, Value>
    }

    impl AppState {
        pub fn new() -> Self {
            AppState {terminals: Arc::new(Mutex::new(TerminalManager::new()))}
        }

        pub fn emit_event(&self, app: &AppHandle, event: FrontendEvent) -> () {
            let _ = app.emit("tart://event", event);
        }

        pub fn run_in_background(&self, app: &AppHandle) -> () {
            let handle = app.app_handle().clone();
            app.listen("tart://internal", move |evt| {
                if let Ok(event) = serde_json::from_str::<BackendEvent>(evt.payload()) {
                    let _state = handle.state::<AppState>();

                    match event {
                        BackendEvent::TerminalRead { id, data } => {
                            let _ = handle.emit("tart://event", FrontendEvent::TerminalRead{id, data});
                        }
                    }
                }
            });

            let handle = app.app_handle().clone();
            app.listen("tart://command", move |evt| {
                if let Ok(event) = serde_json::from_str::<AppCommandWrapper>(evt.payload()) {
                    let state = handle.state::<AppState>();
                    let mut terminals = state.terminals.lock().unwrap();
                    match event.command {
                        AppCommand::CreateTerminal { ref command, ref args, ref title } => {
                            let result = terminals.create_terminal(handle.clone(), command.clone(), args.clone(), title.clone()).and_then(|t| Ok(t.info()));
                            event.emit_result(&handle, result.clone());
                            if let Ok(created) = result {
                                state.emit_event(&handle, FrontendEvent::TerminalCreated { id: created.id });
                            }

                        },
                        AppCommand::WriteData { id, ref data } => {
                            println!("Writing {:?} to {:?}", data.clone(), id);
                            match terminals.sender(id) {
                                Some(sender) => match sender.send(TerminalMessage::new(TerminalCommand::Write(data.clone()))) {
                                    Ok(_) => {
                                        event.emit_result(&handle, Ok::<&str, &str>("Wrote data"));
                                    },
                                    Err(_) => {
                                        event.emit_result(&handle, Err::<&str, &str>("Failed to send data to terminal instance"));
                                    }
                                },
                                None => {
                                    event.emit_result(&handle, Err::<&str, &str>("Unknown terminal ID"));
                                }
                            }
                        },
                        AppCommand::GetTerminals {  } => {event.emit_result(&handle, Ok::<Vec<TerminalInfo>, ()>(terminals.list_terminals()));},
                        AppCommand::Resize { id, size } => {
                            match terminals.sender(id) {
                                Some(sender) => match sender.send(TerminalMessage::new(TerminalCommand::Resize { size: size.clone() })) {
                                    Ok(_) => {
                                        event.emit_result(&handle, Ok::<&str, &str>("Resized terminal"));
                                        state.emit_event(&handle, FrontendEvent::TerminalResized { id, size });
                                    },
                                    Err(_) => event.emit_result(&handle, Err::<&str, &str>("Failed to send terminal resize command"))
                                },
                                None => event.emit_result(&handle, Err::<&str, &str>("Unknown terminal ID"))
                            }
                        },
                        AppCommand::RemoveTerminal { id } => {
                            if let Some(_) = terminals.terminal(id) {
                                terminals.remove_terminal(id);
                                event.emit_result(&handle, Ok::<&str, &str>("Removed terminal"));
                                state.emit_event(&handle, FrontendEvent::TerminalRemoved { id });
                            } else {
                                event.emit_result(&handle, Err::<&str, &str>("Unknown terminal ID"));
                            }
                        }
                    }
                }
            });
        }

        pub async fn run_command(&self, app: &AppHandle, command: AppCommand) -> CommandResult {
            let message = command.wrap();
            let (tx, mut rx) = tokio::sync::mpsc::channel::<CommandResult>(1);
            let msgclone = message.clone();
            app.listen("tart://result", move |evt| {
                if let Ok(result) = serde_json::from_str::<CommandResult>(evt.payload()) {
                    if result.id.to_string() == msgclone.id.to_string() {
                        let _ = tx.try_send(result);
                    }
                }
            });
            if let Err(_) = app.emit("tart://command", message.clone()) {
                return CommandResult {
                    id: message.clone().id,
                    command: message.clone().command,
                    result: Err(serde_json::to_value("Failed to emit event.").unwrap())
                };
            }

            if let Some(result) = rx.recv().await {
                result
            } else {
                CommandResult {
                    id: message.id,
                    command: message.command,
                    result: Err(serde_json::to_value("Unexpected error receiving response").unwrap())
                }
            }
        }
    }
}