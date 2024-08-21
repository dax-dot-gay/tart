pub mod term {
    use core::str;
    use std::{collections::HashMap, io::{BufRead, BufReader, Read, Write}, sync::{Arc, Mutex}, thread::{self, JoinHandle}};

    use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize, SlavePty};
    use serde::{Deserialize, Serialize};
    use tauri::{AppHandle, Emitter};
    use uuid::Uuid;
    use crossbeam_channel::{unbounded, Receiver, Sender, TrySendError};

    use crate::common::app_state::BackendEvent;

    #[derive(Serialize, Deserialize, Clone, Debug)]
    #[serde(tag = "type")]
    pub enum TerminalCommand {
        Kill,
        TerminalFailure{id: Uuid, reason: String},
        Write(String),
        Read(String),
        Error{scope: String, reason: String},

        #[serde(with = "PtySizeDef")]
        Resize{size: PtySize}
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct TerminalMessage {
        pub id: Uuid,
        pub command: TerminalCommand
    }

    impl TerminalMessage {
        pub fn new(command: TerminalCommand) -> Self {
            TerminalMessage {
                id: Uuid::new_v4(),
                command
            }
        }

        pub fn result(&self, command: TerminalCommand) -> Self {
            TerminalMessage {
                id: self.id,
                command
            }
        }
    }

    #[derive(Serialize, Deserialize)]
    #[serde(remote = "PtySize")]
    pub struct PtySizeDef {
        rows: u16,
        cols: u16,
        pixel_width: u16,
        pixel_height: u16
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct TerminalInfo {
        pub id: Uuid,
        pub command: String,
        pub args: Option<Vec<String>>,
        pub title: Option<String>,

        #[serde(with = "PtySizeDef")]
        pub size: PtySize
    }

    #[allow(dead_code)]
    pub struct TerminalContext {
        pub handle: Box<dyn MasterPty>,
        pub target: Box<dyn SlavePty>,
        pub reader: BufReader<Box<dyn Read + Send>>,
        pub writer: Box<dyn Write + Send>,
        pub process: Box<dyn Child>
    }

    #[derive(Clone, Debug)]
    pub struct Terminal {
        _info: TerminalInfo,
        pub commands: (Sender<TerminalMessage>, Receiver<TerminalMessage>),
        pub results: (Sender<TerminalMessage>, Receiver<TerminalMessage>)
    }

    impl Terminal {
        pub fn new(commands: (Sender<TerminalMessage>, Receiver<TerminalMessage>), results: (Sender<TerminalMessage>, Receiver<TerminalMessage>), command: String, args: Option<Vec<String>>, title: Option<String>) -> Self {
            Terminal { _info: TerminalInfo { id: Uuid::new_v4(), command, args, title, size: PtySize { rows: 24, cols: 80, pixel_width: 0, pixel_height: 0 } }, commands, results }
        }

        pub fn id(&self) -> Uuid {
            self._info.id
        }

        pub fn info(&self) -> TerminalInfo {
            self._info.clone()
        }

        pub fn send_event(&self, command: TerminalCommand) -> Result<(), TrySendError<TerminalMessage>> {
            self.results.0.try_send(TerminalMessage::new(command))
        }

        pub fn send_result(&self, message: TerminalMessage, command: TerminalCommand) -> Result<(), TrySendError<TerminalMessage>> {
            self.results.0.try_send(message.result(command))
        }

        pub fn recv_command(&self) -> Option<TerminalMessage> {
            if let Ok(recv) = self.commands.1.try_recv() {
                Some(recv)
            } else {
                None
            }
        }

        fn get_pty(&self) -> Result<TerminalContext, String> {
            let pty_system = native_pty_system();
            let info = self._info.clone();
            if let Ok(pair) = pty_system.openpty(info.size) {
                let mut cmd = CommandBuilder::new(info.command);
                if let Some(args) = info.args {
                    cmd.args(args);
                }
                if let Ok(child) = pair.slave.spawn_command(cmd) {
                    let reader = pair.master.try_clone_reader();
                    let writer = pair.master.take_writer();
                    if reader.is_ok() && writer.is_ok() {
                        Ok(TerminalContext { handle: pair.master, target: pair.slave, reader: BufReader::new(reader.unwrap()), writer: writer.unwrap(), process: child })
                    } else {
                        Err("Failed to get IO handles".to_string())
                    }
                } else {
                    Err("Failed to spawn command".to_string())
                }
            } else {
                Err("PTY open failed".to_string())
            }
        }

        pub fn run_loop(&self, app: AppHandle) -> () {
            match self.get_pty() {
                Ok(mut context) => {
                    loop {
                        if let Some(cmd) = self.recv_command() {
                            match cmd.clone().command {
                                TerminalCommand::Kill => {
                                    let _ = context.process.kill();
                                    break;
                                },
                                TerminalCommand::Write(data) => {
                                    if let Err(_) = context.writer.write_all(data.as_bytes()) {
                                        let _ = self.send_result(cmd.clone(), TerminalCommand::Error { scope: "pty".to_string(), reason: "write_failure".to_string() });
                                    }
                                },
                                TerminalCommand::Resize { size } => {
                                    if let Err(_) = context.handle.resize(size) {
                                        let _ = self.send_result(cmd.clone(), TerminalCommand::Error { scope: "pty".to_string(), reason: "resize_failure".to_string() });
                                    }
                                }
                                _ => ()
                            }
                        }

                        let data = {
                            if let Ok(data) = context.reader.fill_buf() {
                                if data.len() > 0 {
                                    if let Ok(parsed) = str::from_utf8(data) {
                                        Some(parsed.to_string())
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        };

                        if let Some(d) = data {
                            context.reader.consume(d.len());
                            let _ = app.emit("tart://internal", BackendEvent::TerminalRead { id: self.id(), data: d });
                        }
                    }
                },
                Err(reason) => {
                    let _ = self.send_event(TerminalCommand::TerminalFailure { id: self.id(), reason });
                }
            };
        }
    }

    #[derive(Clone, Debug)]
    pub struct TerminalManager {
        terminals: Arc<Mutex<HashMap<Uuid, Terminal>>>,
        threads: Arc<Mutex<HashMap<Uuid, JoinHandle<()>>>>
    }

    impl TerminalManager {
        pub fn new() -> Self {
            TerminalManager { terminals: Arc::new(Mutex::new(HashMap::new())), threads: Arc::new(Mutex::new(HashMap::new())) }
        }

        pub fn create_terminal(&mut self, app: AppHandle, command: String, args: Option<Vec<String>>, title: Option<String>) -> Result<Terminal, &str> {
            let commands = unbounded::<TerminalMessage>();
            let results = unbounded::<TerminalMessage>();
            let term = Terminal::new((commands.0.clone(), commands.1.clone()), (results.0.clone(), results.1.clone()), command, args, title);
            if let Ok(mut terminals) = self.terminals.lock() {
                if let Ok(mut threads) = self.threads.lock() {
                    let cloned = term.clone();
                    terminals.insert(term.clone().id(), term.clone());
                    threads.insert(term.clone().id(), thread::spawn(move || term.run_loop(app.clone())));
                    return Ok(cloned);
                }   
                return Err("Failed to lock threads mapping");
            }
            return Err("Failed to lock terminals mapping");
        }

        pub fn remove_terminal(&mut self, id: Uuid) -> () {
            if let Ok(mut terminals) = self.terminals.lock() {
                if let Ok(mut threads) = self.threads.lock() {
                    if let Some(sender) = self.sender(id) {
                        let _ = sender.send(TerminalMessage::new(TerminalCommand::Kill));
                        if let Some(thread) = threads.remove(&id) {
                            let _ = thread.join();
                        }
                        let _ = terminals.remove(&id);
                    }
                }   
            }
        }

        pub fn terminal(&self, id: Uuid) -> Option<TerminalInfo> {
            if let Ok(terminals) = self.terminals.lock() {
                if let Some(res) = terminals.get(&id) {
                    Some(res.info())
                } else {
                    None
                }
            } else {
                None
            }
        }

        pub fn list_terminals(&self) -> Vec<TerminalInfo> {
            if let Ok(terminals) = self.terminals.lock() {
                terminals.values().map(|v| v.info()).collect()
            } else {
                Vec::new()
            }
        }

        pub fn sender(&self, id: Uuid) -> Option<Sender<TerminalMessage>> {
            if let Ok(terminals) = self.terminals.lock() {
                if let Some(res) = terminals.get(&id) {
                    Some(res.commands.0.clone())
                } else {
                    None
                }
            } else {
                None
            }
        }

        pub fn receiver(&self, id: Uuid) -> Option<Receiver<TerminalMessage>> {
            if let Ok(terminals) = self.terminals.lock() {
                if let Some(res) = terminals.get(&id) {
                    Some(res.results.1.clone())
                } else {
                    None
                }
            } else {
                None
            }
        }
    }
}