use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageTier {
    Bram,
    Ddr,
    Nvme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierConfig {
    pub tier: StorageTier,
    pub capacity: usize,
}

pub struct CsrStore {
    tiers: Vec<TierConfig>,
    row_ptr: Vec<u32>,
    col_idx: Vec<u32>,
    values: Vec<Vec<u8>>,
}

impl CsrStore {
    pub fn new(tiers: Vec<TierConfig>) -> Self {
        Self {
            tiers,
            row_ptr: vec![0],
            col_idx: Vec::new(),
            values: Vec::new(),
        }
    }

    pub fn insert_edge(&mut self, _from: u32, to: u32, value: Vec<u8>) {
        self.col_idx.push(to);
        self.values.push(value);
        if let Some(last) = self.row_ptr.last_mut() {
            *last += 1;
        }
    }

    pub fn neighbors(&self, node: u32) -> &[u32] {
        let start = self.row_ptr.get(node as usize).copied().unwrap_or(0) as usize;
        let end = self
            .row_ptr
            .get(node as usize + 1)
            .copied()
            .unwrap_or(start as u32) as usize;
        &self.col_idx[start..end]
    }

    pub fn tier_capacity(&self, tier: StorageTier) -> usize {
        self.tiers
            .iter()
            .find(|t| t.tier == tier)
            .map(|t| t.capacity)
            .unwrap_or(0)
    }

    pub fn total_edges(&self) -> usize {
        self.col_idx.len()
    }
}
