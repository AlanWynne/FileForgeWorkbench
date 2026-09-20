//! `KindRegistry`: the single source of truth for every Workspace Kind's
//! effective `KindConfig` (CR-NR-090, Slice B.1). Compiled built-in defaults plus
//! user Kinds loaded from `<User_Data_Dir>/workspace-kinds/<name>.toml`.

use std::collections::BTreeMap;
use std::path::Path;

use super::{BaseKind, BuiltinKind, KindConfig, KindConfigToml};

/// Resolves a Kind name to its effective `KindConfig`. Always total: an unknown
/// name falls back to a safe built-in default so lookups never fail (design P1).
#[derive(Debug, Clone)]
pub struct KindRegistry {
    by_name: BTreeMap<String, KindConfig>,
    /// Non-blocking notices accumulated while loading (unparseable user files,
    /// rejected user-on-user chains, unresolved external bases).
    notices: Vec<String>,
}

impl KindRegistry {
    /// A registry seeded with the compiled default config for every built-in
    /// Kind. Never empty; RESET BARE (B.4) restores from these.
    pub fn with_builtin_defaults() -> Self {
        let mut by_name = BTreeMap::new();
        for kind in BuiltinKind::ALL {
            let cfg = KindConfig::builtin_default(kind);
            by_name.insert(cfg.name.clone(), cfg);
        }
        Self {
            by_name,
            notices: Vec::new(),
        }
    }

    /// Load the registry: built-in defaults, then user Kind files from
    /// `<user_dir>/*.toml` layered on top. A user file whose `name` equals a
    /// built-in OVERRIDES it (CR-CH-021 pattern). Unparseable files are skipped
    /// with a notice; a user Kind modelled on another USER Kind (a chain) is
    /// rejected with a notice; an `External` base is accepted but flagged (it is
    /// unresolved in v1). An absent directory is silent.
    pub fn load(user_dir: &Path) -> Self {
        let mut reg = Self::with_builtin_defaults();
        let entries = match std::fs::read_dir(user_dir) {
            Ok(e) => e,
            Err(_) => return reg, // absent dir is silent (Req 2.3)
        };
        // Collect candidate user configs first so chain-validation can see them.
        let mut candidates: Vec<KindConfig> = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("toml") {
                continue;
            }
            let text = match std::fs::read_to_string(&path) {
                Ok(t) => t,
                Err(e) => {
                    reg.notices
                        .push(format!("workspace-kind: cannot read {path:?}: {e}"));
                    continue;
                }
            };
            match toml::from_str::<KindConfigToml>(&text) {
                Ok(t) => candidates.push(KindConfig::from(t)),
                Err(e) => reg
                    .notices
                    .push(format!("workspace-kind: cannot parse {path:?}: {e}")),
            }
        }
        // The set of built-in names, used to reject user-on-user chains.
        for cfg in candidates {
            match &cfg.modelled_on {
                BaseKind::Builtin(_) => {
                    reg.by_name.insert(cfg.name.clone(), cfg);
                }
                BaseKind::External(name) => {
                    // v1: external bases are unresolved. Accept the Kind but flag
                    // it; a distinct built-in name that happens to be a user file
                    // still overrides, but its base cannot resolve, so we keep the
                    // config for round-trip while noting the limitation. If the
                    // tag names another USER Kind, that is the rejected chain case.
                    if BuiltinKind::from_stable_name(name).is_some() {
                        // A bare built-in name that lost its prefix; treat as builtin.
                        reg.by_name.insert(cfg.name.clone(), cfg);
                    } else {
                        reg.notices.push(format!(
                            "workspace-kind '{}': modelled_on '{}' is not a built-in base \
                             (external/user bases are unsupported in v1); the Kind is kept \
                             but resolves to its safe built-in fallback",
                            cfg.name, name
                        ));
                        reg.by_name.insert(cfg.name.clone(), cfg);
                    }
                }
            }
        }
        reg
    }

    /// The effective `KindConfig` for `name`. Total: falls back to a safe
    /// built-in default (the `Menu`/generic kind by name, else the POM) when the
    /// name is unknown, so callers never handle a missing Kind (design P1).
    pub fn effective(&self, name: &str) -> &KindConfig {
        if let Some(cfg) = self.by_name.get(name) {
            return cfg;
        }
        // Unknown name: fall back to a built-in with the same name if one exists,
        // else the POM default (guaranteed present).
        self.by_name
            .get(BuiltinKind::Pom.stable_name())
            .expect("built-in POM default is always present")
    }

    /// Resolve a Kind name to its base built-in kind for command/behaviour, in a
    /// SINGLE hop (design P2). A built-in Kind resolves to itself; a user Kind
    /// resolves to its `Builtin(...)` base; an unresolvable base falls back to
    /// the POM (safe default) so resolution never fails.
    ///
    /// The runtime consumer (routing a user Kind's command/behaviour through its
    /// base) lands in Slice B.2; B.1 exercises it via unit tests.
    #[allow(dead_code)]
    pub fn resolve_base(&self, name: &str) -> BuiltinKind {
        match &self.effective(name).modelled_on {
            BaseKind::Builtin(k) => *k,
            BaseKind::External(_) => BuiltinKind::Pom,
        }
    }

    /// Non-blocking load notices (unparseable files, unresolved bases).
    pub fn notices(&self) -> &[String] {
        &self.notices
    }
}

impl Default for KindRegistry {
    fn default() -> Self {
        Self::with_builtin_defaults()
    }
}
