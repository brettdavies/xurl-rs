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
