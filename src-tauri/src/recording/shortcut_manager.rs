use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use global_hotkey::hotkey::HotKey;
use global_hotkey::GlobalHotKeyManager;

use crate::models::slot::Slot;

pub struct ShortcutManager {
    hotkey_manager: GlobalHotKeyManager,
    registered_shortcuts: HashMap<String, Slot>,
    registered_hotkeys: HashMap<String, HotKey>,
    hotkey_to_slot: HashMap<u32, String>,
    enabled: bool,
}

impl ShortcutManager {
    pub fn new() -> Result<Self, String> {
        let hotkey_manager = GlobalHotKeyManager::new()
            .map_err(|err| format!("failed to create hotkey manager: {err}"))?;

        Ok(Self {
            hotkey_manager,
            registered_shortcuts: HashMap::new(),
            registered_hotkeys: HashMap::new(),
            hotkey_to_slot: HashMap::new(),
            enabled: false,
        })
    }

    pub fn sync_slots(&mut self, slots: &[Slot]) -> Result<(), String> {
        let mut next_registered_shortcuts: HashMap<String, Slot> = HashMap::new();
        let mut next_registered_hotkeys: HashMap<String, HotKey> = HashMap::new();
        let mut next_hotkey_to_slot: HashMap<u32, String> = HashMap::new();
        let mut used_hotkey_ids: HashSet<u32> = HashSet::new();

        for slot in slots {
            next_registered_shortcuts.insert(slot.id.clone(), slot.clone());

            let shortcut = slot.shortcut.trim();
            if shortcut.is_empty() {
                continue;
            }

            let hotkey = HotKey::from_str(shortcut)
                .map_err(|err| format!("invalid shortcut \"{}\": {err}", slot.shortcut))?;

            if !used_hotkey_ids.insert(hotkey.id()) {
                return Err(format!("shortcut conflict detected for: {}", slot.shortcut));
            }

            next_hotkey_to_slot.insert(hotkey.id(), slot.id.clone());
            next_registered_hotkeys.insert(slot.id.clone(), hotkey);
        }

        if self.enabled && !same_hotkey_bindings(&self.registered_hotkeys, &next_registered_hotkeys) {
            self.unregister_all();
            self.register_all(&next_registered_hotkeys)?;
        }

        self.registered_shortcuts = next_registered_shortcuts;
        self.registered_hotkeys = next_registered_hotkeys;
        self.hotkey_to_slot = next_hotkey_to_slot;

        Ok(())
    }

    pub fn set_enabled(&mut self, enabled: bool) -> Result<(), String> {
        if self.enabled == enabled {
            return Ok(());
        }

        if enabled {
            self.register_all(&self.registered_hotkeys)?;
            self.enabled = true;
        } else {
            self.unregister_all();
            self.enabled = false;
        }

        Ok(())
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn handle_shortcut(&self, hotkey_id: u32) -> Option<Slot> {
        if !self.enabled {
            return None;
        }

        let slot_id = self.hotkey_to_slot.get(&hotkey_id)?;
        self.registered_shortcuts.get(slot_id).cloned()
    }

    fn register_all(&self, hotkeys: &HashMap<String, HotKey>) -> Result<(), String> {
        for hotkey in hotkeys.values() {
            self.hotkey_manager
                .register(*hotkey)
                .map_err(|err| format!("failed to register shortcut {hotkey}: {err}"))?;
        }

        Ok(())
    }

    fn unregister_all(&self) {
        for hotkey in self.registered_hotkeys.values() {
            if let Err(err) = self.hotkey_manager.unregister(*hotkey) {
                tracing::debug!(?err, "failed to unregister hotkey");
            }
        }
    }
}

fn same_hotkey_bindings(
    current: &HashMap<String, HotKey>,
    next: &HashMap<String, HotKey>,
) -> bool {
    if current.len() != next.len() {
        return false;
    }

    current
        .iter()
        .all(|(slot_id, hotkey)| next.get(slot_id).map(HotKey::id) == Some(hotkey.id()))
}
