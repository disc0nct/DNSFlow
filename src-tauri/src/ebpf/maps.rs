use aya::maps::HashMap as BpfHashMap;
use aya::Ebpf;
use std::collections::HashMap;

/// Kept for potential future use — map operations are currently inline in loader.rs
#[allow(dead_code)]
pub struct EbpfMapManager {
    pub maps: HashMap<String, EbpfMap>,
}

pub struct EbpfMap {
    pub name: String,
    pub map_type: EbpfMapType,
    pub max_entries: u32,
}

pub enum EbpfMapType {
    Hash,
    Array,
    PerCpuHash,
    PerCpuArray,
    RingBuf,
}

impl EbpfMapManager {
    pub fn new() -> Self {
        Self {
            maps: HashMap::new(),
        }
    }

    pub fn create_map(
        &mut self,
        name: &str,
        map_type: EbpfMapType,
        max_entries: u32,
    ) -> Result<(), String> {
        let map = EbpfMap {
            name: name.to_string(),
            map_type,
            max_entries,
        };
        self.maps.insert(name.to_string(), map);
        Ok(())
    }

    pub fn get_map(&self, name: &str) -> Option<&EbpfMap> {
        self.maps.get(name)
    }

    pub fn delete_map(&mut self, name: &str) -> bool {
        self.maps.remove(name).is_some()
    }
}

impl Default for EbpfMapManager {
    fn default() -> Self {
        Self::new()
    }
}

pub struct RuleManager {}

impl RuleManager {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update_rules(
        &mut self,
        bpf: &mut Ebpf,
        rules: Vec<(u32, u32)>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut map: BpfHashMap<_, u32, u32> = bpf
            .map_mut("RULES")
            .ok_or("RULES map not found")?
            .try_into()?;
        for (pid, server_id) in rules {
            map.insert(pid, server_id, 0)?;
        }
        Ok(())
    }
}

impl Default for RuleManager {
    fn default() -> Self {
        Self::new()
    }
}
