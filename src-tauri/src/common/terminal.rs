pub mod term {
    use std::{collections::HashMap, sync::{Arc, Mutex}};

    use serde::{Deserialize, Serialize};
    use uuid::Uuid;
    use crossbeam_channel::{unbounded, Receiver, Sender, TrySendError};

    #[derive(Serialize, Deserialize, Clone, Debug)]
    #[serde(tag = "type")]
    pub enum TerminalCommand {
        Kill
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

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct TerminalInfo {
        pub id: Uuid,
        pub command: String,
        pub args: Option<Vec<String>>,
        pub title: Option<String>,
    }

    #[derive(Clone, Debug)]
    pub struct Terminal {
        _info: TerminalInfo,
        pub commands: (Sender<TerminalMessage>, Receiver<TerminalMessage>),
        pub results: (Sender<TerminalMessage>, Receiver<TerminalMessage>)
    }

    impl Terminal {
        pub fn new(commands: (Sender<TerminalMessage>, Receiver<TerminalMessage>), results: (Sender<TerminalMessage>, Receiver<TerminalMessage>), command: String, args: Option<Vec<String>>, title: Option<String>) -> Self {
            Terminal { _info: TerminalInfo { id: Uuid::new_v4(), command, args, title }, commands, results }
        }

        pub fn id(&self) -> Uuid {
            self._info.id
        }

        pub fn info(&self) -> TerminalInfo {
            self._info.clone()
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
    }

    #[derive(Clone, Debug)]
    pub struct TerminalManager {
        terminals: Arc<Mutex<HashMap<Uuid, Terminal>>>,
    }

    impl TerminalManager {
        pub fn new() -> Self {
            TerminalManager { terminals: Arc::new(Mutex::new(HashMap::new())) }
        }

        pub fn create_terminal(&mut self, command: String, args: Option<Vec<String>>, title: Option<String>) -> () {
            let commands = unbounded::<TerminalMessage>();
            let results = unbounded::<TerminalMessage>();
            let term = Terminal::new((commands.0.clone(), commands.1.clone()), (results.0.clone(), results.1.clone()), command, args, title);
            if let Ok(mut terminals) = self.terminals.lock() {
                terminals.insert(term.id(), term);
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