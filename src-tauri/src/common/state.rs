pub mod app_state {
    use std::{collections::HashMap, sync::{Arc, Mutex, MutexGuard}};
    use uuid::Uuid;

    use crate::common::terminal::Terminal;

    pub struct ApplicationState {
        _terminals: Arc<Mutex<HashMap<Uuid, Arc<Terminal>>>>,
        _selected: Mutex<Option<Uuid>>
    }

    impl ApplicationState {
        pub fn new() -> Self {
            ApplicationState {
                _terminals: Arc::new(Mutex::new(HashMap::<Uuid, Arc<Terminal>>::new())),
                _selected: Mutex::new(None)
            }
        }

        pub fn terminals(&mut self) -> MutexGuard<HashMap<Uuid, Arc<Terminal>>> {
            self._terminals.lock().unwrap()
        }

        pub fn selected(&mut self) -> MutexGuard<Option<Uuid>> {
            self._selected.lock().unwrap()
        }

        pub fn add_terminal(&mut self, terminal: Terminal) -> () {
            self.terminals().insert(terminal.id(), Arc::new(terminal));
        }

        pub fn remove_terminal(&mut self, id: Uuid) -> () {
            self.terminals().remove(&id);
        }

        pub fn set_terminal(&mut self, id: Option<Uuid>) -> () {
            *self.selected() = id.clone();
        }

        pub fn get_selected(&mut self) -> Option<Arc<Terminal>> {
            let selected = *self.selected();
            if let Some(id) = selected {
                self.get_terminal(id)
            } else {
                None
            }
        }

        pub fn get_terminal(&mut self, id: Uuid) -> Option<Arc<Terminal>> {
            let term = self.terminals();
            term.get(&id).map(|t| t.clone())
        }
    }

    #[derive(Clone)]
    pub struct StateContainer {
        _state: Arc<Mutex<ApplicationState>>
    }

    impl StateContainer {
        pub fn new() -> Self {
            StateContainer { _state: Arc::new(Mutex::new(ApplicationState::new())) }
        }

        pub fn state(&self) -> MutexGuard<ApplicationState> {
            self._state.lock().unwrap()
        }
    }
}