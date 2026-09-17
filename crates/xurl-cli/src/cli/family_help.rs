//! One shape for a command family's `after_help` examples.
//!
//! A family declares its path, its verbs, and the example arguments each
//! verb takes; every page renders from that declaration, so the command
//! line an example shows is written once per verb rather than once per
//! page. A page that names a verb the family does not declare fails to
//! compile, because pages refer to verbs by constant, not by name.

/// A verb of a family and the two captions its own page renders under.
pub struct Verb {
    /// The word after the family path, such as `add`.
    pub name: &'static str,
    /// The arguments every example passes the verb, or empty.
    pub example_args: &'static str,
    /// Caption of the plain-text example on the verb's page.
    pub text_caption: &'static str,
    /// Caption of the `--output json` example on the verb's page.
    pub json_caption: &'static str,
}

/// One example line: a verb, rendered with or without `--output json`.
pub struct Example {
    verb: &'static Verb,
    json: bool,
}

impl Example {
    /// The verb's command line as typed.
    #[must_use]
    pub const fn text(verb: &'static Verb) -> Self {
        Self { verb, json: false }
    }

    /// The verb's command line with `--output json` appended.
    #[must_use]
    pub const fn json(verb: &'static Verb) -> Self {
        Self { verb, json: true }
    }
}

/// A captioned group of example lines.
pub type Section<'a> = (&'a str, &'a [Example]);

/// A command family: its path under `xr` and the verbs it dispatches.
pub struct Family {
    /// The words between `xr` and the verb, such as `broadcasts moderators`.
    pub path: &'static str,
}

impl Family {
    fn command_line(&self, example: &Example) -> String {
        let mut parts = vec!["xr", self.path, example.verb.name];
        if !example.verb.example_args.is_empty() {
            parts.push(example.verb.example_args);
        }
        if example.json {
            parts.push("--output json");
        }
        parts.join(" ")
    }

    /// Renders captioned sections as an `Examples:` block.
    #[must_use]
    pub fn page(&self, sections: &[Section<'_>]) -> String {
        let mut out = String::from("Examples:\n");
        for (caption, examples) in sections {
            out.push_str("  ");
            out.push_str(caption);
            out.push_str(":\n");
            for example in *examples {
                out.push_str("    ");
                out.push_str(&self.command_line(example));
                out.push('\n');
            }
        }
        out
    }

    /// A verb's own page: its plain-text example, then the same command
    /// as a JSON envelope.
    #[must_use]
    pub fn verb_page(&self, verb: &'static Verb) -> String {
        self.page(&[
            (verb.text_caption, &[Example::text(verb)]),
            (verb.json_caption, &[Example::json(verb)]),
        ])
    }
}

/// The `xr broadcasts` family.
pub mod broadcasts {
    use super::{Example, Family, Section, Verb};

    /// The family every page below belongs to.
    pub const FAMILY: Family = Family {
        path: "broadcasts moderators",
    };

    /// `xr broadcasts moderators list`.
    pub const LIST: Verb = Verb {
        name: "list",
        example_args: "",
        text_caption: "List your broadcast chat moderators (text)",
        json_caption: "As a JSON envelope",
    };

    /// `xr broadcasts moderators add`.
    pub const ADD: Verb = Verb {
        name: "add",
        example_args: "@helper",
        text_caption: "Add a chat moderator (text)",
        json_caption: "Add (JSON envelope)",
    };

    /// `xr broadcasts moderators remove`.
    pub const REMOVE: Verb = Verb {
        name: "remove",
        example_args: "@helper",
        text_caption: "Remove a chat moderator (text)",
        json_caption: "Remove (JSON envelope)",
    };

    /// The page under `xr broadcasts --help`.
    #[must_use]
    pub fn root_page() -> String {
        const SECTIONS: &[Section<'static>] = &[
            (
                "Who moderates your broadcast chats (text)",
                &[Example::text(&LIST)],
            ),
            (
                "Add and remove a moderator, JSON envelope",
                &[Example::json(&ADD), Example::json(&REMOVE)],
            ),
        ];
        FAMILY.page(SECTIONS)
    }

    /// The page under `xr broadcasts moderators --help`.
    #[must_use]
    pub fn moderators_page() -> String {
        const SECTIONS: &[Section<'static>] = &[
            (
                "List your broadcast chat moderators (text)",
                &[Example::text(&LIST)],
            ),
            ("Add a moderator (JSON envelope)", &[Example::json(&ADD)]),
        ];
        FAMILY.page(SECTIONS)
    }
}
