use std::collections::HashMap;

pub struct OverlayQuery {
    overlays: HashMap<String, HashMap<u32, Vec<u8>>>,
}

impl OverlayQuery {
    pub fn new() -> Self {
        Self {
            overlays: HashMap::new(),
        }
    }

    pub fn set(&mut self, overlay: &str, node_id: u32, data: Vec<u8>) {
        self.overlays
            .entry(overlay.to_string())
            .or_default()
            .insert(node_id, data);
    }

    pub fn get(&self, overlay: &str, node_id: u32) -> Option<&[u8]> {
        self.overlays
            .get(overlay)
            .and_then(|m| m.get(&node_id))
            .map(|v| v.as_slice())
    }

    pub fn overlay_names(&self) -> Vec<&str> {
        self.overlays.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for OverlayQuery {
    fn default() -> Self {
        Self::new()
    }
}
