use std::collections::VecDeque;

pub struct Autocomplete {
    history: VecDeque<String>,
    history_index: usize,
}

impl Autocomplete {
    pub fn new() -> Self {
        Self {
            history: VecDeque::new(),
            history_index: 0,
        }
    }

    pub fn add_to_history(&mut self, command: String) {
        self.history.push_front(command);
        if self.history.len() > 100 {
            self.history.pop_back();
        }
        self.history_index = 0;
    }

    pub fn get_history_up(&mut self) -> Option<String> {
        if self.history.is_empty() {
            return None;
        }
        if self.history_index < self.history.len() - 1 {
            self.history_index += 1;
        }
        self.history.get(self.history_index).cloned()
    }

    pub fn get_history_down(&mut self) -> Option<String> {
        if self.history_index > 0 {
            self.history_index -= 1;
        }
        self.history.get(self.history_index).cloned()
    }

    pub fn suggest(&self, input: &str) -> Option<String> {
        let completions = self.complete(input);
        completions.first().cloned()
    }

    pub fn complete(&self, input: &str) -> Vec<String> {
        if input.starts_with('/') {
            let partial = input.trim_start_matches('/');
            let commands = vec![
                "search", "filter", "export", "goto", "bookmark", "save", "load", "help", "quit",
            ];
            commands
                .iter()
                .filter(|c| c.starts_with(&partial.to_lowercase()))
                .map(|c| format!("/{}", c))
                .collect()
        } else {
            Vec::new()
        }
    }
}

impl Default for Autocomplete {
    fn default() -> Self {
        Self::new()
    }
}
