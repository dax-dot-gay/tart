pub mod app_state {
    use std::collections::HashMap;
    use uuid::Uuid;

    use crate::common::terminal::Terminal;

    pub struct ApplicationState {
        pub terminals: HashMap<Uuid, Terminal>,
        pub selected: Option<Uuid>
    }

    impl ApplicationState {
        pub fn new() -> Self {
            ApplicationState {
                terminals: HashMap::<Uuid, Terminal>::new(),
                selected: None
            }
        }

        pub fn add_terminal(&mut self, terminal: Terminal) -> () {
            self.terminals.insert(terminal.id(), terminal);
        }

        pub fn remove_terminal(&mut self, id: Uuid) -> () {
            self.terminals.remove(&id);
        }

        pub fn set_terminal(&mut self, id: Option<Uuid>) -> Option<&Terminal> {
            self.selected = id.clone();
            if let Some(uuid) = id {
                self.terminals.get(&uuid)
            } else {
                None
            }
        }

        pub fn get_selected(&mut self) -> Option<&Terminal> {
            if let Some(id) = self.selected {
                self.terminals.get(&id)
            } else {
                None
            }
        }

        pub fn get_terminal(&mut self, id: Uuid) -> Option<&Terminal> {
            self.terminals.get(&id)
        }

        pub fn iter_terminals(&mut self) -> std::collections::hash_map::ValuesMut<Uuid, Terminal> {
            self.terminals.values_mut()
        }
    }
}