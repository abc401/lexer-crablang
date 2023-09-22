use std::collections::HashMap;

#[derive(Debug)]
pub struct StringDecorator {
    decoration_indices: HashMap<String, u32>,
}

impl Default for StringDecorator {
    fn default() -> Self {
        Self {
            decoration_indices: Default::default(),
        }
    }
}

impl StringDecorator {
    pub fn decorate_and_increment(&mut self, string: String) -> String {
        let mut index: u32 = 0;

        let entry = self
            .decoration_indices
            .entry(string)
            .and_modify(|e| index = *e);

        let decorated_string = format!("{}_{}", entry.key(), index);
        entry.and_modify(|e| *e += 1).or_insert(1);
        return decorated_string;
    }

    pub fn decorate(&self, string: &str) -> String {
        let index = self.decoration_indices.get(string).unwrap_or(&0);
        return format!("{}_{}", string, index);
    }

    pub fn increment(&mut self, string: String) {
        self.decoration_indices
            .entry(string)
            .and_modify(|e| *e += 1)
            .or_insert(1);
    }
}
