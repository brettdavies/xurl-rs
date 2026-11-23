//! The process environment, read once per run by the binary.
//!
//! The library's [`EnvOverrides::from_env`] reads the variables an X API
//! client needs. The ones the CLI alone consumes are read here, so no
//! library code path touches `XURL_OUTPUT`, `HOME`, `XURL_TOKEN_STORE`,
//! `NO_COLOR`, `XURL_SKILL_HOME`, a skill host's config- or base-directory
//! variable, or the `PATH` and `PATHEXT` a forward program is found by.

use std::ffi::OsString;

use xdk::config::EnvOverrides;

use crate::cli::skill_install::{BASE_DIR_VARS, CONFIG_DIR_VARS, SkillEnv};

/// Every variable `xr` reads, as data.
#[must_use]
pub fn from_process() -> EnvOverrides {
    let mut overrides = EnvOverrides::from_env();
    overrides.output = std::env::var("XURL_OUTPUT").ok();
    overrides.home = std::env::var("HOME").ok();
    overrides.token_store = std::env::var("XURL_TOKEN_STORE").ok();
    overrides.no_color = std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty());
    overrides
}

/// The variables of the two settings `xr` resolves by layer, other than
/// `XURL_OUTPUT`, which [`EnvOverrides::output`] carries. Each is the string
/// as the process held it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FlagEnv {
    /// `XURL_JSON`.
    pub json: Option<String>,
    /// `XURL_JSONL`.
    pub jsonl: Option<String>,
    /// `XURL_CURSOR`.
    pub cursor: Option<String>,
    /// `XURL_AFTER`.
    pub after: Option<String>,
}

impl FlagEnv {
    /// The value held for the variable named `name`, `None` for a name this
    /// value does not carry.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&str> {
        match name {
            "XURL_JSON" => self.json.as_deref(),
            "XURL_JSONL" => self.jsonl.as_deref(),
            "XURL_CURSOR" => self.cursor.as_deref(),
            "XURL_AFTER" => self.after.as_deref(),
            _ => None,
        }
    }
}

/// The four variables of [`FlagEnv`], as the process holds them.
#[must_use]
pub fn flags_from_process() -> FlagEnv {
    FlagEnv {
        json: std::env::var("XURL_JSON").ok(),
        jsonl: std::env::var("XURL_JSONL").ok(),
        cursor: std::env::var("XURL_CURSOR").ok(),
        after: std::env::var("XURL_AFTER").ok(),
    }
}

/// The value of the variable named `name`, as clap reads it for an `env`
/// binding. A value that is not UTF-8 is held by no one.
#[must_use]
pub fn variable(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

/// Where the OS looks for a program by name: the directories of `PATH`, and
/// on Windows the extensions of `PATHEXT`. They are search paths, not `xr`
/// settings.
#[derive(Debug, Clone, Default)]
pub struct ProgramSearch {
    /// `PATH`, as the OS holds it.
    pub path: Option<OsString>,
    /// `PATHEXT`, the extensions Windows tries on a name that has none.
    pub pathext: Option<String>,
}

/// The variables a forward program is found by, as data.
#[must_use]
pub fn program_search() -> ProgramSearch {
    ProgramSearch {
        path: std::env::var_os("PATH"),
        pathext: std::env::var("PATHEXT").ok(),
    }
}

/// The skill-destination variables, as data: `HOME` from `overrides`, plus
/// `XURL_SKILL_HOME` and each host config- or base-directory variable that is
/// set.
#[must_use]
pub fn skill_from_process(overrides: &EnvOverrides) -> SkillEnv {
    SkillEnv {
        home: overrides.home.clone(),
        skill_home: std::env::var("XURL_SKILL_HOME").ok(),
        config_dirs: CONFIG_DIR_VARS
            .iter()
            .chain(BASE_DIR_VARS)
            .filter_map(|var| {
                std::env::var(var)
                    .ok()
                    .map(|value| ((*var).to_string(), value))
            })
            .collect(),
    }
}
