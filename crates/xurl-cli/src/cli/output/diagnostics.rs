//! Renders the library's `tracing` events the way `xr` prints them: the
//! `--verbose` wire lines and legacy-vocabulary notes, the media status
//! lines, and warnings.
//!
//! The library emits structured events and never touches a terminal. This
//! subscriber, installed by the runner around one dispatch, is where the
//! `> GET url`, `< 200 OK`, and `< key: value` lines and their colours live,
//! gated exactly as the flags gate every other stderr line.

use std::fmt;
use std::io::Write as _;
use std::sync::atomic::{AtomicU64, Ordering};

use tracing::field::{Field, Visit};
use tracing::{Event, Level, Metadata, Subscriber, span};

use super::OutputConfig;
use xdk::api::{MEDIA_TARGET, VOCABULARY_TARGET, WIRE_TARGET};

/// One dispatch's diagnostics renderer.
pub(crate) struct Diagnostics {
    out: OutputConfig,
    next_span: AtomicU64,
}

impl Diagnostics {
    pub(crate) fn new(out: OutputConfig) -> Self {
        Self {
            out,
            next_span: AtomicU64::new(1),
        }
    }

    /// The `--verbose` gate: text mode, not quiet.
    fn verbose_enabled(&self) -> bool {
        self.out.verbose && !self.out.quiet && !self.out.format.is_structured()
    }

    fn render(&self, event: &Event<'_>) -> Option<String> {
        let mut fields = Fields::default();
        event.record(&mut fields);
        let meta = event.metadata();
        let colour = self.out.use_color;

        match *meta.level() {
            Level::ERROR => return Some(format!("error: {}", fields.message.unwrap_or_default())),
            Level::WARN => {
                return Some(format!("warning: {}", fields.message.unwrap_or_default()));
            }
            _ => {}
        }
        match meta.target() {
            WIRE_TARGET => {
                if !self.verbose_enabled() {
                    return None;
                }
                wire_line(&fields, colour)
            }
            MEDIA_TARGET if *meta.level() == Level::INFO => {
                if self.out.quiet || self.out.format.is_structured() {
                    return None;
                }
                let message = fields.message.unwrap_or_default();
                Some(if colour {
                    format!("\x1b[32m{message}\x1b[0m")
                } else {
                    message
                })
            }
            MEDIA_TARGET => {
                if !self.verbose_enabled() {
                    return None;
                }
                fields.message
            }
            VOCABULARY_TARGET => {
                if !self.verbose_enabled() {
                    return None;
                }
                vocabulary_line(&fields)
            }
            _ => None,
        }
    }
}

/// One wire event as `xr --verbose` prints it.
fn wire_line(fields: &Fields, colour: bool) -> Option<String> {
    let text = |value: &Option<String>| value.as_deref().unwrap_or("").to_string();
    match fields.kind.as_deref()? {
        "request" => {
            let (method, url) = (text(&fields.method), text(&fields.url));
            Some(if colour {
                format!("\x1b[1;34m> {method}\x1b[0m {url}")
            } else {
                format!("> {method} {url}")
            })
        }
        "status" => {
            let status = text(&fields.status);
            Some(if colour {
                format!("\x1b[1;31m< {status}\x1b[0m")
            } else {
                format!("< {status}")
            })
        }
        "header" => {
            let (name, value) = (text(&fields.name), text(&fields.value));
            Some(if colour {
                format!("\x1b[1;32m< {name}\x1b[0m: {value}")
            } else {
                format!("< {name}: {value}")
            })
        }
        "end" => Some(String::new()),
        // The sentence is the binary's: 3.x printed it with the binary's name,
        // and the library only reports which header the caller supplied.
        "note" => fields.header.as_ref().map_or_else(
            || fields.message.clone(),
            |header| {
                Some(format!(
                    "info: user-supplied {header} detected; skipping xurl append"
                ))
            },
        ),
        _ => None,
    }
}

/// One legacy key the library read under its current name, as `xr --verbose`
/// prints it: the two spellings, then the JSON type of the value X sent and
/// its length when it has one.
fn vocabulary_line(fields: &Fields) -> Option<String> {
    let legacy = fields.legacy.as_deref()?;
    let normalized = fields.normalized.as_deref()?;
    let mut shape = fields.value_type.clone()?;
    if let Some(len) = &fields.value_len {
        shape.push_str(", length ");
        shape.push_str(len);
    }
    Some(if fields.collision.as_deref() == Some("true") {
        format!("info: X sent both {legacy} and {normalized}; kept {normalized} ({shape})")
    } else {
        format!("info: X sent {legacy}; read as {normalized} ({shape})")
    })
}

/// The fields the library's events carry, collected by name.
#[derive(Default)]
struct Fields {
    kind: Option<String>,
    method: Option<String>,
    url: Option<String>,
    status: Option<String>,
    name: Option<String>,
    value: Option<String>,
    header: Option<String>,
    message: Option<String>,
    legacy: Option<String>,
    normalized: Option<String>,
    value_type: Option<String>,
    value_len: Option<String>,
    collision: Option<String>,
}

impl Fields {
    fn set(&mut self, field: &Field, text: String) {
        let slot = match field.name() {
            "kind" => &mut self.kind,
            "method" => &mut self.method,
            "url" => &mut self.url,
            "status" => &mut self.status,
            "name" => &mut self.name,
            "value" => &mut self.value,
            "header" => &mut self.header,
            "message" => &mut self.message,
            "legacy" => &mut self.legacy,
            "normalized" => &mut self.normalized,
            "value_type" => &mut self.value_type,
            "value_len" => &mut self.value_len,
            "collision" => &mut self.collision,
            _ => return,
        };
        *slot = Some(text);
    }
}

impl Visit for Fields {
    fn record_str(&mut self, field: &Field, value: &str) {
        self.set(field, value.to_string());
    }

    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        self.set(field, format!("{value:?}"));
    }
}

impl Subscriber for Diagnostics {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.target().starts_with("xdk::")
    }

    fn new_span(&self, _attributes: &span::Attributes<'_>) -> span::Id {
        span::Id::from_u64(self.next_span.fetch_add(1, Ordering::Relaxed))
    }

    fn record(&self, _span: &span::Id, _values: &span::Record<'_>) {}

    fn record_follows_from(&self, _span: &span::Id, _follows: &span::Id) {}

    fn event(&self, event: &Event<'_>) {
        if let Some(line) = self.render(event) {
            let stderr = std::io::stderr();
            let mut lock = stderr.lock();
            let _ = writeln!(lock, "{line}");
        }
    }

    fn enter(&self, _span: &span::Id) {}

    fn exit(&self, _span: &span::Id) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::ColorChoice;
    use crate::cli::output::OutputFormat;

    fn diagnostics(verbose: bool, colour: ColorChoice) -> Diagnostics {
        Diagnostics::new(OutputConfig::new(
            OutputFormat::Text,
            false,
            verbose,
            colour,
        ))
    }

    fn rendered(diagnostics: Diagnostics, emit: impl FnOnce()) -> Vec<Option<String>> {
        let lines = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let probe = Probe {
            inner: diagnostics,
            lines: std::sync::Arc::clone(&lines),
        };
        tracing::subscriber::with_default(probe, emit);
        lines.lock().unwrap().clone()
    }

    /// Captures what [`Diagnostics::render`] would print instead of writing it.
    struct Probe {
        inner: Diagnostics,
        lines: std::sync::Arc<std::sync::Mutex<Vec<Option<String>>>>,
    }

    impl Subscriber for Probe {
        fn enabled(&self, metadata: &Metadata<'_>) -> bool {
            self.inner.enabled(metadata)
        }
        fn new_span(&self, attributes: &span::Attributes<'_>) -> span::Id {
            self.inner.new_span(attributes)
        }
        fn record(&self, _: &span::Id, _: &span::Record<'_>) {}
        fn record_follows_from(&self, _: &span::Id, _: &span::Id) {}
        fn event(&self, event: &Event<'_>) {
            self.lines.lock().unwrap().push(self.inner.render(event));
        }
        fn enter(&self, _: &span::Id) {}
        fn exit(&self, _: &span::Id) {}
    }

    #[test]
    fn wire_lines_match_the_verbose_format_in_both_colour_modes() {
        let lines = rendered(diagnostics(true, ColorChoice::Never), || {
            tracing::debug!(target: WIRE_TARGET, kind = "request", method = "GET", url = "http://h/p");
            tracing::debug!(target: WIRE_TARGET, kind = "status", status = "200 OK");
            tracing::debug!(target: WIRE_TARGET, kind = "header", name = "date", value = "now");
            tracing::debug!(target: WIRE_TARGET, kind = "end");
        });
        assert_eq!(
            lines,
            vec![
                Some("> GET http://h/p".to_string()),
                Some("< 200 OK".to_string()),
                Some("< date: now".to_string()),
                Some(String::new()),
            ]
        );

        let lines = rendered(diagnostics(true, ColorChoice::Always), || {
            tracing::debug!(target: WIRE_TARGET, kind = "request", method = "GET", url = "u");
            tracing::debug!(target: WIRE_TARGET, kind = "status", status = "200 OK");
            tracing::debug!(target: WIRE_TARGET, kind = "header", name = "k", value = "v");
        });
        assert_eq!(
            lines,
            vec![
                Some("\x1b[1;34m> GET\x1b[0m u".to_string()),
                Some("\x1b[1;31m< 200 OK\x1b[0m".to_string()),
                Some("\x1b[1;32m< k\x1b[0m: v".to_string()),
            ]
        );
    }

    #[test]
    fn wire_lines_are_silent_without_verbose_and_warnings_always_print() {
        let lines = rendered(diagnostics(false, ColorChoice::Never), || {
            tracing::debug!(target: WIRE_TARGET, kind = "request", method = "GET", url = "u");
            tracing::warn!(target: "xdk::auth", "token stored under unnamed slot");
        });
        assert_eq!(
            lines,
            vec![
                None,
                Some("warning: token stored under unnamed slot".to_string())
            ]
        );
    }

    #[test]
    fn error_events_print_on_every_target_and_never_as_warnings() {
        let lines = rendered(diagnostics(false, ColorChoice::Never), || {
            tracing::error!(target: "xdk::store", "the store could not be saved");
            tracing::error!(target: WIRE_TARGET, "the connection dropped");
        });
        assert_eq!(
            lines,
            vec![
                Some("error: the store could not be saved".to_string()),
                Some("error: the connection dropped".to_string())
            ]
        );
    }

    /// Decodes bodies spelled in X's legacy vocabulary through the library,
    /// so the lines rendered below come from the events it really emits:
    /// a renamed array, a collision on a number, and a renamed string.
    fn decode_legacy_bodies() {
        let post = serde_json::json!({"data": {
            "id": "1",
            "text": "t",
            "edit_history_tweet_ids": [""],
            "public_metrics": {"retweet_count": 1, "repost_count": 2}
        }});
        xdk::api::deserialize_response::<xdk::api::Post>(post).expect("the post decodes");
        let user = serde_json::json!({"data": {
            "id": "1",
            "name": "n",
            "username": "u",
            "pinned_tweet_id": "2101712260468977783"
        }});
        xdk::api::deserialize_response::<xdk::api::User>(user).expect("the user decodes");
    }

    #[test]
    fn vocabulary_lines_match_the_verbose_format_without_colour() {
        for colour in [ColorChoice::Never, ColorChoice::Always] {
            let lines = rendered(diagnostics(true, colour), decode_legacy_bodies);
            assert_eq!(
                lines,
                vec![
                    Some(
                        "info: X sent edit_history_tweet_ids; read as edit_history_post_ids (array, length 1)"
                            .to_string()
                    ),
                    Some(
                        "info: X sent both retweet_count and repost_count; kept repost_count (number)"
                            .to_string()
                    ),
                    Some(
                        "info: X sent pinned_tweet_id; read as pinned_post_id (string, length 19)"
                            .to_string()
                    ),
                ],
                "colour {colour:?}"
            );
        }
    }

    #[test]
    fn vocabulary_lines_print_only_under_verbose_in_text_mode() {
        let silent = [
            ("no --verbose", OutputFormat::Text, false, false),
            ("--quiet --verbose", OutputFormat::Text, true, true),
            ("--output json --verbose", OutputFormat::Json, false, true),
            ("--output jsonl --verbose", OutputFormat::Jsonl, false, true),
        ];
        for (flags, format, quiet, verbose) in silent {
            let out = OutputConfig::new(format, quiet, verbose, ColorChoice::Never);
            let lines = rendered(Diagnostics::new(out), decode_legacy_bodies);
            assert_eq!(lines, vec![None, None, None], "{flags}");
        }
    }

    #[test]
    fn media_status_is_green_and_progress_needs_verbose() {
        let lines = rendered(diagnostics(false, ColorChoice::Always), || {
            tracing::info!(target: MEDIA_TARGET, "Upload complete!");
            tracing::debug!(target: MEDIA_TARGET, "Uploaded 1 of 2 bytes (50.00%)");
        });
        assert_eq!(
            lines,
            vec![Some("\x1b[32mUpload complete!\x1b[0m".to_string()), None]
        );
    }
}
