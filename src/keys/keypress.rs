//! Module contianing the KeyPress struct.
//! KeyPress is used to index keybindings.

use rustwlc::KeyMod;
use rustwlc::xkb::{Keysym, NameFlags};
use std::hash::{Hash, Hasher, SipHasher};

use std::fmt::Result as FmtResult;
use std::fmt::{Display, Formatter};

/// Structure containing keys which are pressed
/// to trigger a keybinding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyPress {
    /// The modifiers (ctrl, mod, shift, etc.)
    modifiers: KeyMod,
    /// The keys pressed
    key: Keysym
}

impl KeyPress {
    /// Creates a new KeyPress struct from a list of modifier and key names
    pub fn from_key_names(mods: &[&str], key: &str) -> Result<KeyPress, String> {
        super::keymod_from_names(mods).and_then(|mods| {
            let proper_key = if let Some(found_key) = super::NAME_MAPPING.get(key) {
                found_key
            }
            else {
                key
            };

            // Parse a keysym for each given key
            let key_sym = try!(
                if let Some(sym) = Keysym::from_name(key.to_string(),
                                                 NameFlags::None) {
                    Ok(sym)
                }
                // If lowercase cannot be parsed, try case insensitive
                else if let Some(sym) = Keysym::from_name(key.to_string(),
                                                      NameFlags::CaseInsensitive) {
                    Ok(sym)
                }
                else {
                    Err(format!("Invalid key: {}", key))
                });

            Ok(KeyPress { modifiers: mods, key: key_sym })
        })
    }

    /// Creates a KeyPress from the given keysyms and modifiers
    pub fn new(mods: KeyMod, key: Keysym) -> KeyPress {
        KeyPress { modifiers: mods, key: key }
    }

    /// Gets a String which can be used to index a Lua table.
    ///
    /// The hash value of KeyPress returns a u64 which cannot be used
    /// as a table index (Lua 5.2 uses f64 for numbers), and an unsafe
    /// transmute to f64 may cause rounding errors when used as a table
    /// index. Instead, the number is converted to a string.
    ///
    /// Switching to lua5.3-sys would improve this.
    pub fn get_lua_index_string(&self) -> String {
        let mut hasher = SipHasher::new();
        self.hash(&mut hasher);
        hasher.finish().to_string()
    }
}

impl Hash for KeyPress {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        hasher.write_u32(self.modifiers.bits());
        hasher.write_u32(self.key.get_code());
    }
}

impl Display for KeyPress {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.debug_tuple("Keypress")
            .field(&self.modifiers)
            .field(&self.key.get_name())
            .finish()
    }
}
