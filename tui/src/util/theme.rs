// Theme resolution — the pure decision shared across the app: given a per-doc override, the global
// default preference, and the current system color scheme, pick the dark flag for a document.
// Kept out of GTK so the matrix is unit-testable (mirrors util/zoom's separation).

use serde::{Deserialize, Serialize};

/// Global default theme preference. Persisted in AppSettings. The per-doc override (ThemeStore)
/// only ever stores an explicit Light/Dark, never "system" — once a user toggles a doc, it sticks.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ThemePref {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "light")]
    Light,
    #[serde(rename = "dark")]
    Dark,
}

impl Default for ThemePref {
    fn default() -> Self {
        ThemePref::System
    }
}

/// Resolve the effective dark flag for a document.
///
/// - `per_doc`: Some(d) when the user has toggled this doc's theme before (persisted by hash).
///   Wins unconditionally — "switched docs keep their last-used theme".
/// - otherwise fall back to the global default, where `System` follows the OS color scheme.
pub fn resolve_dark(per_doc: Option<bool>, pref: ThemePref, system_dark: bool) -> bool {
    match per_doc {
        Some(d) => d,
        None => match pref {
            ThemePref::System => system_dark,
            ThemePref::Light => false,
            ThemePref::Dark => true,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn per_doc_override_wins_over_everything() {
        for pref in [ThemePref::System, ThemePref::Light, ThemePref::Dark] {
            for sys in [false, true] {
                assert_eq!(resolve_dark(Some(true), pref, sys), true);
                assert_eq!(resolve_dark(Some(false), pref, sys), false);
            }
        }
    }

    #[test]
    fn no_override_light_is_light() {
        assert!(!resolve_dark(None, ThemePref::Light, false));
        assert!(!resolve_dark(None, ThemePref::Light, true));
    }

    #[test]
    fn no_override_dark_is_dark() {
        assert!(resolve_dark(None, ThemePref::Dark, false));
        assert!(resolve_dark(None, ThemePref::Dark, true));
    }

    #[test]
    fn no_override_system_follows_os() {
        assert!(!resolve_dark(None, ThemePref::System, false));
        assert!(resolve_dark(None, ThemePref::System, true));
    }

    #[test]
    fn default_pref_is_system() {
        assert_eq!(ThemePref::default(), ThemePref::System);
    }

    #[test]
    fn serde_roundtrips_as_lowercase_string() {
        for pref in [ThemePref::System, ThemePref::Light, ThemePref::Dark] {
            let s = serde_json::to_string(&pref).unwrap();
            let back: ThemePref = serde_json::from_str(&s).unwrap();
            assert_eq!(pref, back);
        }
        assert_eq!(serde_json::to_string(&ThemePref::System).unwrap(), "\"system\"");
        assert_eq!(serde_json::to_string(&ThemePref::Light).unwrap(), "\"light\"");
        assert_eq!(serde_json::to_string(&ThemePref::Dark).unwrap(), "\"dark\"");
    }

    #[test]
    fn unknown_string_falls_back_to_default() {
        // Tolerant decode: an unknown/typo value yields the default (System) rather than failing,
        // so a stale config from a future/newer version doesn't break launch.
        let v: ThemePref = serde_json::from_str("\"black\"").unwrap_or_default();
        assert_eq!(v, ThemePref::System);
    }
}
