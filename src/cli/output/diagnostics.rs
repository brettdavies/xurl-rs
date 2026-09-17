//! Renders the library's `tracing` events the way `xr` prints them: the
//! `--verbose` wire lines, the media status lines, and warnings.
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
use crate::api::{MEDIA_TARGET, WIRE_TARGET};

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
        "note" => fields.message.clone(),
        _ => None,
    }
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
    message: Option<String>,
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
            "message" => &mut self.message,
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
        metadata.target().starts_with("xurl::")
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
            tracing::warn!(target: "xurl::auth", "token stored under unnamed slot");
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
            tracing::error!(target: "xurl::store", "the store could not be saved");
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
