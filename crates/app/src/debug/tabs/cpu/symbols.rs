use std::collections::HashMap;

pub struct SymbolManager {
    labels: HashMap<u32, String>,
    inv_label_lookup: HashMap<String, u32>,
}

impl SymbolManager {
    pub fn new() -> Self {
        Self {
            labels: HashMap::new(),
            inv_label_lookup: HashMap::new(),
        }
    }

    pub fn get_address_label(&self, address: u32) -> Option<&str> {
        self.labels.get(&address).map(|x| x.as_str())
    }

    /// Sets the label for a given address. Fails if the label exists for another address.
    pub fn set_address_label(&mut self, address: u32, label: Option<String>) -> anyhow::Result<()> {
        if let Some(label) = &label {
            match self.inv_label_lookup.get(label) {
                Some(&addr) => {
                    if addr != address {
                        return Err(anyhow::anyhow!("label already exists for address ${:06X}", addr));
                    } else {
                        return Ok(());
                    }
                }
                _ => {}
            }
        }

        if let Some(old_label) = self.labels.get(&address) {
            self.inv_label_lookup.remove(old_label);
            self.labels.remove(&address);
        }

        if let Some(label) = label {
            self.labels.insert(address, label);
            let new_label = self.labels.get(&address).unwrap();
            self.inv_label_lookup.insert(new_label.clone(), address);
        }

        Ok(())
    }
}

pub struct LabelEditState {
    pub open: bool,
    pub address: u32,
    pub input: String,
    pub error: Option<String>,
}

impl LabelEditState {
    pub fn new() -> Self {
        Self {
            open: false,
            address: 0,
            input: String::new(),
            error: None,
        }
    }

    pub fn open_for(&mut self, address: u32, current_label: Option<&str>) {
        self.open = true;
        self.address = address;
        self.input = current_label.unwrap_or("").to_string();
        self.error = None;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.input.clear();
        self.error = None;
    }
}