// `XurlError`'s largest variant (`AuthMethodMismatch`) carries multiple
// `String` and `Vec<String>` fields so agents can pattern-match on the
// envelope structure. Boxing the variant would change the public
// construction surface and break consumer code; allow the lint instead.
#![allow(clippy::result_large_err)]

pub mod api;
pub mod auth;
pub mod cli;
pub mod config;
pub mod envelope;
pub mod error;
pub mod output;
pub mod skill_install;
pub mod store;
