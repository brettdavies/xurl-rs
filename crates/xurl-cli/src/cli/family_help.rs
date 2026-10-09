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

/// The `xr webhooks` family.
pub mod webhooks {
    use super::{Example, Family, Section, Verb};

    /// The family every page below belongs to.
    pub const FAMILY: Family = Family { path: "webhooks" };

    /// `xr webhooks list`.
    pub const LIST: Verb = Verb {
        name: "list",
        example_args: "",
        text_caption: "List the webhooks registered for the app (text)",
        json_caption: "As a JSON envelope",
    };

    /// `xr webhooks add`.
    pub const ADD: Verb = Verb {
        name: "add",
        example_args: "https://example.com/webhooks/x",
        text_caption: "Register a URL; X sends its CRC check there first (text)",
        json_caption: "Register (JSON envelope)",
    };

    /// `xr webhooks validate`.
    pub const VALIDATE: Verb = Verb {
        name: "validate",
        example_args: "1146654567674912769",
        text_caption: "Ask X to send its CRC check to a webhook again (text)",
        json_caption: "Check again (JSON envelope)",
    };

    /// `xr webhooks remove`.
    pub const REMOVE: Verb = Verb {
        name: "remove",
        example_args: "1146654567674912769 --force --no-interactive",
        text_caption: "Delete a webhook with no prompt, as a script does (text)",
        json_caption: "Delete (JSON envelope)",
    };

    /// `xr webhooks listen`.
    pub const LISTEN: Verb = Verb {
        name: "listen",
        example_args: "--port 8080",
        text_caption: "Answer X's CRC check and print each event as a JSON line (text notices)",
        json_caption: "With a JSON line when the listener is up",
    };

    /// `xr webhooks replay`.
    pub const REPLAY: Verb = Verb {
        name: "replay",
        example_args: "1146654567674912769 --from 202601150000 --to 202601151200",
        text_caption: "Deliver twelve hours of past events again (text)",
        json_caption: "Replay (JSON envelope)",
    };

    /// The page under `xr webhooks --help`.
    #[must_use]
    pub fn root_page() -> String {
        const SECTIONS: &[Section<'static>] = &[
            (
                "Which webhooks the app has registered (text)",
                &[Example::text(&LIST)],
            ),
            (
                "Register a URL, then ask X to check it again, JSON envelope",
                &[Example::json(&ADD), Example::json(&VALIDATE)],
            ),
            (
                "Replay a window of past events (text)",
                &[Example::text(&REPLAY)],
            ),
            (
                "Receive events on a local port you expose at a public HTTPS URL",
                &[Example::text(&LISTEN)],
            ),
        ];
        FAMILY.page(SECTIONS)
    }
}

/// The `xr webhooks subscriptions` family.
pub mod webhook_subscriptions {
    use super::{Example, Family, Section, Verb};

    /// The family every page below belongs to.
    pub const FAMILY: Family = Family {
        path: "webhooks subscriptions",
    };

    /// `xr webhooks subscriptions count`.
    pub const COUNT: Verb = Verb {
        name: "count",
        example_args: "",
        text_caption: "How many subscriptions the app holds and may hold (text)",
        json_caption: "As a JSON envelope",
    };

    /// `xr webhooks subscriptions list`.
    pub const LIST: Verb = Verb {
        name: "list",
        example_args: "1146654567674912769",
        text_caption: "List the accounts a webhook receives activity for (text)",
        json_caption: "As a JSON envelope",
    };

    /// `xr webhooks subscriptions add`.
    pub const ADD: Verb = Verb {
        name: "add",
        example_args: "1146654567674912769",
        text_caption: "Subscribe your account's activity to a webhook (text)",
        json_caption: "Subscribe (JSON envelope)",
    };

    /// `xr webhooks subscriptions check`.
    pub const CHECK: Verb = Verb {
        name: "check",
        example_args: "1146654567674912769",
        text_caption: "Is your account subscribed to this webhook (text)",
        json_caption: "As a JSON envelope",
    };

    /// `xr webhooks subscriptions remove`.
    pub const REMOVE: Verb = Verb {
        name: "remove",
        example_args: "1146654567674912769 2244994945 --force --no-interactive",
        text_caption: "End an account's subscription with no prompt, as a script does (text)",
        json_caption: "Unsubscribe (JSON envelope)",
    };

    /// The page under `xr webhooks subscriptions --help`.
    #[must_use]
    pub fn page() -> String {
        const SECTIONS: &[Section<'static>] = &[
            (
                "Subscribe your account, then confirm it (text)",
                &[Example::text(&ADD), Example::text(&CHECK)],
            ),
            (
                "Who a webhook receives activity for, JSON envelope",
                &[Example::json(&LIST)],
            ),
        ];
        FAMILY.page(SECTIONS)
    }
}

/// The `xr webhooks stream-links` family.
pub mod webhook_stream_links {
    use super::{Example, Family, Section, Verb};

    /// The family every page below belongs to.
    pub const FAMILY: Family = Family {
        path: "webhooks stream-links",
    };

    /// `xr webhooks stream-links list`.
    pub const LIST: Verb = Verb {
        name: "list",
        example_args: "",
        text_caption: "List the webhooks the filtered stream delivers to (text)",
        json_caption: "As a JSON envelope",
    };

    /// `xr webhooks stream-links add`.
    pub const ADD: Verb = Verb {
        name: "add",
        example_args: "1146654567674912769",
        text_caption: "Deliver the filtered stream to a webhook (text)",
        json_caption: "Link (JSON envelope)",
    };

    /// `xr webhooks stream-links remove`.
    pub const REMOVE: Verb = Verb {
        name: "remove",
        example_args: "1146654567674912769 --force --no-interactive",
        text_caption: "Stop delivering to a webhook with no prompt, as a script does (text)",
        json_caption: "Unlink (JSON envelope)",
    };

    /// The page under `xr webhooks stream-links --help`.
    #[must_use]
    pub fn page() -> String {
        const SECTIONS: &[Section<'static>] = &[
            (
                "Deliver the filtered stream to a webhook (text)",
                &[Example::text(&ADD)],
            ),
            (
                "Where the filtered stream delivers, JSON envelope",
                &[Example::json(&LIST)],
            ),
        ];
        FAMILY.page(SECTIONS)
    }
}

/// The `xr media subtitles` family.
pub mod media_subtitles {
    use super::{Example, Family, Section, Verb};

    /// The family every page below belongs to.
    pub const FAMILY: Family = Family {
        path: "media subtitles",
    };

    /// `xr media subtitles add`.
    pub const ADD: Verb = Verb {
        name: "add",
        example_args: "1585341984679469056 1585341984679469057 --language en --name English",
        text_caption: "Add an English track to a video (text)",
        json_caption: "Add (JSON envelope)",
    };

    /// `xr media subtitles remove`.
    pub const REMOVE: Verb = Verb {
        name: "remove",
        example_args: "1585341984679469056 --language en",
        text_caption: "Remove the English track (text)",
        json_caption: "Remove (JSON envelope)",
    };

    /// The page under `xr media subtitles --help`.
    #[must_use]
    pub fn page() -> String {
        const SECTIONS: &[Section<'static>] = &[
            (
                "Add a track from a subtitle file uploaded with --category subtitles (text)",
                &[Example::text(&ADD)],
            ),
            ("Remove a track, JSON envelope", &[Example::json(&REMOVE)]),
        ];
        FAMILY.page(SECTIONS)
    }
}
