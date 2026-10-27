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

    fn push_sections(&self, out: &mut String, sections: &[Section<'_>]) {
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
    }

    /// Renders captioned sections as an `Examples:` block.
    #[must_use]
    pub fn page(&self, sections: &[Section<'_>]) -> String {
        page_of(&[(self, sections)])
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

/// Renders sections drawn from several families as one `Examples:` block,
/// for a page above more than one family.
#[must_use]
pub fn page_of(families: &[(&Family, &[Section<'_>])]) -> String {
    let mut out = String::from("Examples:\n");
    for (family, sections) in families {
        family.push_sections(&mut out, sections);
    }
    out
}

/// The `xr bookmark-folders` family.
pub mod bookmark_folders {
    use super::{Example, Family, Section, Verb};

    /// The family every page below belongs to.
    pub const FAMILY: Family = Family {
        path: "bookmark-folders",
    };

    /// `xr bookmark-folders list`.
    pub const LIST: Verb = Verb {
        name: "list",
        example_args: "",
        text_caption: "List the folders you sort bookmarks into (text)",
        json_caption: "As a JSON envelope",
    };

    /// `xr bookmark-folders get`.
    pub const GET: Verb = Verb {
        name: "get",
        example_args: "1146654567674912769",
        text_caption: "List the posts in a folder, by id (text)",
        json_caption: "As a JSON envelope",
    };

    /// `xr bookmark-folders add`.
    pub const ADD: Verb = Verb {
        name: "add",
        example_args: "\"Reading list\"",
        text_caption: "Create a folder (text)",
        json_caption: "Create (JSON envelope)",
    };

    /// The page under `xr bookmark-folders --help`.
    #[must_use]
    pub fn page() -> String {
        const SECTIONS: &[Section<'static>] = &[
            ("Your bookmark folders (text)", &[Example::text(&LIST)]),
            (
                "Create one, then list the posts in one, JSON envelope",
                &[Example::json(&ADD), Example::json(&GET)],
            ),
        ];
        FAMILY.page(SECTIONS)
    }
}

/// The `xr conversations` family.
pub mod conversations {
    use super::{Example, Family, Section, Verb};

    /// The family every page below belongs to.
    pub const FAMILY: Family = Family {
        path: "conversations",
    };

    /// `xr conversations start`.
    pub const START: Verb = Verb {
        name: "start",
        example_args: "@alice @bob --text \"Plans for Friday?\"",
        text_caption: "Start a group with two people and send the first message (text)",
        json_caption: "Start (JSON envelope)",
    };

    /// `xr conversations send`.
    pub const SEND: Verb = Verb {
        name: "send",
        example_args: "1580705921830768643 \"Running late\"",
        text_caption: "Send into a conversation, by id (text)",
        json_caption: "Send (JSON envelope)",
    };

    /// `xr conversations history`.
    pub const HISTORY: Verb = Verb {
        name: "history",
        example_args: "1580705921830768643 -n 50",
        text_caption: "The messages in a conversation, by id (text)",
        json_caption: "As a JSON envelope",
    };

    /// `xr conversations with`.
    pub const WITH: Verb = Verb {
        name: "with",
        example_args: "@alice",
        text_caption: "The messages between you and one person (text)",
        json_caption: "As a JSON envelope",
    };

    /// The page under `xr conversations --help`.
    #[must_use]
    pub fn page() -> String {
        const SECTIONS: &[Section<'static>] = &[
            (
                "Start a group, then send into it (text)",
                &[Example::text(&START), Example::text(&SEND)],
            ),
            (
                "Read a conversation, JSON envelope",
                &[Example::json(&HISTORY), Example::json(&WITH)],
            ),
        ];
        FAMILY.page(SECTIONS)
    }
}

/// The `xr dm-events` family.
pub mod dm_events {
    use super::{Example, Family, Section, Verb};

    /// The family every page below belongs to.
    pub const FAMILY: Family = Family { path: "dm-events" };

    /// `xr dm-events get`.
    pub const GET: Verb = Verb {
        name: "get",
        example_args: "1580705921830768647",
        text_caption: "Read one direct message, by event id (text)",
        json_caption: "As a JSON envelope",
    };

    /// `xr dm-events remove`.
    pub const REMOVE: Verb = Verb {
        name: "remove",
        example_args: "1580705921830768647 --force --no-interactive",
        text_caption: "Delete one of your direct messages without a prompt (text)",
        json_caption: "Delete (JSON envelope)",
    };

    /// The page under `xr dm-events --help`.
    #[must_use]
    pub fn page() -> String {
        const SECTIONS: &[Section<'static>] = &[
            ("Read one direct message (text)", &[Example::text(&GET)]),
            (
                "Delete one without a prompt, JSON envelope",
                &[Example::json(&REMOVE)],
            ),
        ];
        FAMILY.page(SECTIONS)
    }
}

/// The `xr lists` family.
pub mod lists {
    use super::{Example, Family, Section, Verb, list_members, page_of};

    /// The family every page below belongs to.
    pub const FAMILY: Family = Family { path: "lists" };

    /// `xr lists list`.
    pub const LIST: Verb = Verb {
        name: "list",
        example_args: "",
        text_caption: "Lists you own (text)",
        json_caption: "As a JSON envelope",
    };

    /// `xr lists followed`.
    pub const FOLLOWED: Verb = Verb {
        name: "followed",
        example_args: "",
        text_caption: "Lists you follow (text)",
        json_caption: "As a JSON envelope",
    };

    /// `xr lists memberships`.
    pub const MEMBERSHIPS: Verb = Verb {
        name: "memberships",
        example_args: "",
        text_caption: "Lists you are a member of (text)",
        json_caption: "As a JSON envelope",
    };

    /// `xr lists pinned`.
    pub const PINNED: Verb = Verb {
        name: "pinned",
        example_args: "",
        text_caption: "Lists you have pinned (text)",
        json_caption: "As a JSON envelope",
    };

    /// `xr lists get`.
    pub const GET: Verb = Verb {
        name: "get",
        example_args: "1146654567674912769",
        text_caption: "Read one list, by id (text)",
        json_caption: "As a JSON envelope",
    };

    /// `xr lists posts`.
    pub const POSTS: Verb = Verb {
        name: "posts",
        example_args: "1146654567674912769 -n 50",
        text_caption: "Posts from a list's members (text)",
        json_caption: "As a JSON envelope",
    };

    /// `xr lists followers`.
    pub const FOLLOWERS: Verb = Verb {
        name: "followers",
        example_args: "1146654567674912769",
        text_caption: "Who follows a list (text)",
        json_caption: "As a JSON envelope",
    };

    /// `xr lists add`.
    pub const ADD: Verb = Verb {
        name: "add",
        example_args: "Rustaceans --description \"People who write Rust\" --private",
        text_caption: "Create a private list (text)",
        json_caption: "As a JSON envelope",
    };

    /// `xr lists update`.
    pub const UPDATE: Verb = Verb {
        name: "update",
        example_args: "1146654567674912769 --name \"Rust people\" --public",
        text_caption: "Rename a list and make it public (text)",
        json_caption: "As a JSON envelope",
    };

    /// `xr lists remove`.
    pub const REMOVE: Verb = Verb {
        name: "remove",
        example_args: "1146654567674912769 --force --no-interactive",
        text_caption: "Delete a list without a prompt (text)",
        json_caption: "As a JSON envelope",
    };

    /// `xr lists follow`.
    pub const FOLLOW: Verb = Verb {
        name: "follow",
        example_args: "1146654567674912769",
        text_caption: "Follow a list (text)",
        json_caption: "As a JSON envelope",
    };

    /// `xr lists unfollow`.
    pub const UNFOLLOW: Verb = Verb {
        name: "unfollow",
        example_args: "1146654567674912769",
        text_caption: "Stop following a list (text)",
        json_caption: "As a JSON envelope",
    };

    /// `xr lists pin`.
    pub const PIN: Verb = Verb {
        name: "pin",
        example_args: "1146654567674912769",
        text_caption: "Pin a list (text)",
        json_caption: "As a JSON envelope",
    };

    /// `xr lists unpin`.
    pub const UNPIN: Verb = Verb {
        name: "unpin",
        example_args: "1146654567674912769",
        text_caption: "Unpin a list (text)",
        json_caption: "As a JSON envelope",
    };

    /// The page under `xr lists --help`.
    #[must_use]
    pub fn page() -> String {
        const LISTS: &[Section<'static>] = &[
            (
                "Your lists, and the posts in one (text)",
                &[Example::text(&LIST), Example::text(&POSTS)],
            ),
            (
                "Create a list, then follow and pin another, JSON envelope",
                &[
                    Example::json(&ADD),
                    Example::json(&FOLLOW),
                    Example::json(&PIN),
                ],
            ),
        ];
        const MEMBERS: &[Section<'static>] = &[(
            "Add an account to a list, JSON envelope",
            &[Example::json(&list_members::ADD)],
        )];
        page_of(&[(&FAMILY, LISTS), (&list_members::FAMILY, MEMBERS)])
    }
}

/// The `xr lists members` family.
pub mod list_members {
    use super::{Example, Family, Section, Verb};

    /// The family every page below belongs to.
    pub const FAMILY: Family = Family {
        path: "lists members",
    };

    /// `xr lists members list`.
    pub const LIST: Verb = Verb {
        name: "list",
        example_args: "1146654567674912769",
        text_caption: "The accounts on a list (text)",
        json_caption: "As a JSON envelope",
    };

    /// `xr lists members add`.
    pub const ADD: Verb = Verb {
        name: "add",
        example_args: "1146654567674912769 @XDevelopers",
        text_caption: "Add an account to a list you own (text)",
        json_caption: "As a JSON envelope",
    };

    /// `xr lists members remove`.
    pub const REMOVE: Verb = Verb {
        name: "remove",
        example_args: "1146654567674912769 @XDevelopers",
        text_caption: "Take an account off a list (text)",
        json_caption: "As a JSON envelope",
    };

    /// The page under `xr lists members --help`.
    #[must_use]
    pub fn page() -> String {
        const SECTIONS: &[Section<'static>] = &[
            ("The accounts on a list (text)", &[Example::text(&LIST)]),
            (
                "Add and take off an account, JSON envelope",
                &[Example::json(&ADD), Example::json(&REMOVE)],
            ),
        ];
        FAMILY.page(SECTIONS)
    }
}

/// The `xr communities` family.
pub mod communities {
    use super::{Example, Family, Section, Verb};

    /// The family every page below belongs to.
    pub const FAMILY: Family = Family {
        path: "communities",
    };

    /// `xr communities search`.
    pub const SEARCH: Verb = Verb {
        name: "search",
        example_args: "rust",
        text_caption: "Communities about a topic (text)",
        json_caption: "As a JSON envelope",
    };

    /// `xr communities get`.
    pub const GET: Verb = Verb {
        name: "get",
        example_args: "1146654567674912769",
        text_caption: "Read one community, by id (text)",
        json_caption: "As a JSON envelope",
    };

    /// The page under `xr communities --help`.
    #[must_use]
    pub fn page() -> String {
        const SECTIONS: &[Section<'static>] = &[
            ("Search communities (text)", &[Example::text(&SEARCH)]),
            ("Read one community, JSON envelope", &[Example::json(&GET)]),
        ];
        FAMILY.page(SECTIONS)
    }
}

/// The `xr news` family.
pub mod news {
    use super::{Example, Family, Section, Verb};

    /// The family every page below belongs to.
    pub const FAMILY: Family = Family { path: "news" };

    /// `xr news search`.
    pub const SEARCH: Verb = Verb {
        name: "search",
        example_args: "\"open source\" --max-age-hours 24",
        text_caption: "News stories from the last day about a topic (text)",
        json_caption: "As a JSON envelope",
    };

    /// `xr news get`.
    pub const GET: Verb = Verb {
        name: "get",
        example_args: "1850000000000000000",
        text_caption: "Read one story, by id (text)",
        json_caption: "As a JSON envelope",
    };

    /// The page under `xr news --help`.
    #[must_use]
    pub fn page() -> String {
        const SECTIONS: &[Section<'static>] = &[
            ("Search news stories (text)", &[Example::text(&SEARCH)]),
            ("Read one story, JSON envelope", &[Example::json(&GET)]),
        ];
        FAMILY.page(SECTIONS)
    }
}

/// The `xr notes` family.
pub mod notes {
    use super::{Example, Family, Section, Verb};

    /// The family every page below belongs to.
    pub const FAMILY: Family = Family { path: "notes" };

    /// `xr notes write`.
    pub const WRITE: Verb = Verb {
        name: "write",
        example_args: "1585341984679469056 --text \"The photo is from 2019: https://example.com/source\" --classification misinformed_or_potentially_misleading --tag outdated_information --trustworthy-sources --test",
        text_caption: "Write a note in test mode (text)",
        json_caption: "Write (JSON envelope)",
    };

    /// `xr notes evaluate`.
    pub const EVALUATE: Verb = Verb {
        name: "evaluate",
        example_args: "1585341984679469056 --text \"The photo is from 2019: https://example.com/source\"",
        text_caption: "Score a draft before writing it (text)",
        json_caption: "Score (JSON envelope)",
    };

    /// `xr notes eligible`.
    pub const ELIGIBLE: Verb = Verb {
        name: "eligible",
        example_args: "-n 20 --test",
        text_caption: "Posts you could write a note on, in test mode (text)",
        json_caption: "As a JSON envelope",
    };

    /// `xr notes written`.
    pub const WRITTEN: Verb = Verb {
        name: "written",
        example_args: "--test",
        text_caption: "The notes you wrote in test mode (text)",
        json_caption: "As a JSON envelope",
    };

    /// `xr notes remove`.
    pub const REMOVE: Verb = Verb {
        name: "remove",
        example_args: "1850000000000000002 --force --no-interactive",
        text_caption: "Delete a note without a prompt (text)",
        json_caption: "Delete (JSON envelope)",
    };

    /// The page under `xr notes --help`.
    #[must_use]
    pub fn page() -> String {
        const SECTIONS: &[Section<'static>] = &[
            (
                "Find a post, score a draft, then write it in test mode (text)",
                &[
                    Example::text(&ELIGIBLE),
                    Example::text(&EVALUATE),
                    Example::text(&WRITE),
                ],
            ),
            (
                "The notes you wrote, JSON envelope",
                &[Example::json(&WRITTEN)],
            ),
        ];
        FAMILY.page(SECTIONS)
    }
}

/// The `xr trends` family.
pub mod trends {
    use super::{Example, Family, Section, Verb};

    /// The family every page below belongs to.
    pub const FAMILY: Family = Family { path: "trends" };

    /// `xr trends place`.
    pub const PLACE: Verb = Verb {
        name: "place",
        example_args: "1 --max 10",
        text_caption: "The top ten trends worldwide, WOEID 1 (text)",
        json_caption: "As a JSON envelope",
    };

    /// `xr trends personal`.
    pub const PERSONAL: Verb = Verb {
        name: "personal",
        example_args: "",
        text_caption: "The trends X picks for you (text)",
        json_caption: "As a JSON envelope",
    };

    /// The page under `xr trends --help`.
    #[must_use]
    pub fn page() -> String {
        const SECTIONS: &[Section<'static>] = &[
            (
                "Trends for a place, by its WOEID (text)",
                &[Example::text(&PLACE)],
            ),
            (
                "Your personalized trends, JSON envelope",
                &[Example::json(&PERSONAL)],
            ),
        ];
        FAMILY.page(SECTIONS)
    }
}

/// The verbs at the top of `xr broadcasts`.
pub mod broadcasts_own {
    use super::{Family, Verb};

    /// The family every verb below belongs to.
    pub const FAMILY: Family = Family { path: "broadcasts" };

    /// `xr broadcasts list`.
    pub const LIST: Verb = Verb {
        name: "list",
        example_args: "",
        text_caption: "Your broadcasts (text)",
        json_caption: "As a JSON envelope",
    };

    /// `xr broadcasts get`.
    pub const GET: Verb = Verb {
        name: "get",
        example_args: "1YqKDqWqdPLGV",
        text_caption: "Read one broadcast, by id (text)",
        json_caption: "As a JSON envelope",
    };
}

/// The `xr broadcasts scheduled` family.
pub mod broadcasts_scheduled {
    use super::{Example, Family, Section, Verb};

    /// The family every page below belongs to.
    pub const FAMILY: Family = Family {
        path: "broadcasts scheduled",
    };

    /// `xr broadcasts scheduled list`.
    pub const LIST: Verb = Verb {
        name: "list",
        example_args: "",
        text_caption: "Your scheduled broadcasts (text)",
        json_caption: "As a JSON envelope",
    };

    /// `xr broadcasts scheduled get`.
    pub const GET: Verb = Verb {
        name: "get",
        example_args: "1850000000000000001",
        text_caption: "Read one scheduled broadcast, by id (text)",
        json_caption: "As a JSON envelope",
    };

    /// `xr broadcasts scheduled add`.
    pub const ADD: Verb = Verb {
        name: "add",
        example_args: "--source 1x2y3z4w5v6u --start 2026-11-02T16:00:00Z --title \"Launch day\"",
        text_caption: "Schedule a broadcast from a stream source (text)",
        json_caption: "Schedule (JSON envelope)",
    };

    /// `xr broadcasts scheduled update`.
    pub const UPDATE: Verb = Verb {
        name: "update",
        example_args: "1850000000000000001 --start 2026-11-02T17:00:00Z --end 2026-11-02T18:00:00Z",
        text_caption: "Move a scheduled broadcast; X takes both times (text)",
        json_caption: "Move (JSON envelope)",
    };

    /// `xr broadcasts scheduled remove`.
    pub const REMOVE: Verb = Verb {
        name: "remove",
        example_args: "1850000000000000001 --force --no-interactive",
        text_caption: "Cancel a scheduled broadcast without a prompt (text)",
        json_caption: "Cancel (JSON envelope)",
    };

    /// `xr broadcasts scheduled live`.
    pub const LIVE: Verb = Verb {
        name: "live",
        example_args: "1850000000000000001 --force --no-interactive",
        text_caption: "Start a scheduled broadcast now, without a prompt (text)",
        json_caption: "Go live (JSON envelope)",
    };

    /// The page under `xr broadcasts scheduled --help`.
    #[must_use]
    pub fn page() -> String {
        const SECTIONS: &[Section<'static>] = &[
            ("Your scheduled broadcasts (text)", &[Example::text(&LIST)]),
            (
                "Schedule one, then start it early, JSON envelope",
                &[Example::json(&ADD), Example::json(&LIVE)],
            ),
        ];
        FAMILY.page(SECTIONS)
    }
}

/// The `xr broadcasts chat` family.
pub mod broadcast_chat {
    use super::{Example, Family, Section, Verb};

    /// The family every page below belongs to.
    pub const FAMILY: Family = Family {
        path: "broadcasts chat",
    };

    /// `xr broadcasts chat history`.
    pub const HISTORY: Verb = Verb {
        name: "history",
        example_args: "1YqKDqWqdPLGV -n 50",
        text_caption: "A broadcast's chat (text)",
        json_caption: "As a JSON envelope",
    };

    /// `xr broadcasts chat send`.
    pub const SEND: Verb = Verb {
        name: "send",
        example_args: "1YqKDqWqdPLGV \"Welcome!\"",
        text_caption: "Send into a broadcast's chat (text)",
        json_caption: "Send (JSON envelope)",
    };

    /// `xr broadcasts chat remove`.
    pub const REMOVE: Verb = Verb {
        name: "remove",
        example_args: "1YqKDqWqdPLGV 1700000000001 --force --no-interactive",
        text_caption: "Remove a chat message without a prompt (text)",
        json_caption: "Remove (JSON envelope)",
    };

    /// `xr broadcasts chat mute`.
    pub const MUTE: Verb = Verb {
        name: "mute",
        example_args: "1YqKDqWqdPLGV @troll --until 2026-11-02T17:00:00Z",
        text_caption: "Mute someone in the chat for an hour (text)",
        json_caption: "Mute (JSON envelope)",
    };

    /// `xr broadcasts chat unmute`.
    pub const UNMUTE: Verb = Verb {
        name: "unmute",
        example_args: "1YqKDqWqdPLGV @troll",
        text_caption: "Unmute someone in the chat (text)",
        json_caption: "Unmute (JSON envelope)",
    };

    /// The page under `xr broadcasts chat --help`.
    #[must_use]
    pub fn page() -> String {
        const SECTIONS: &[Section<'static>] = &[
            ("A broadcast's chat (text)", &[Example::text(&HISTORY)]),
            (
                "Send, then mute someone, JSON envelope",
                &[Example::json(&SEND), Example::json(&MUTE)],
            ),
        ];
        FAMILY.page(SECTIONS)
    }
}

/// The `xr broadcasts` family.
pub mod broadcasts {
    use super::{
        Example, Family, Section, Verb, broadcast_chat, broadcast_sources, broadcasts_own,
        broadcasts_scheduled, page_of,
    };

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
        const MODERATORS: &[Section<'static>] = &[
            (
                "Who moderates your broadcast chats (text)",
                &[Example::text(&LIST)],
            ),
            (
                "Add and remove a moderator, JSON envelope",
                &[Example::json(&ADD), Example::json(&REMOVE)],
            ),
        ];
        const SOURCES: &[Section<'static>] = &[
            (
                "The ingest points you stream to (text)",
                &[Example::text(&broadcast_sources::LIST)],
            ),
            (
                "Create one, JSON envelope",
                &[Example::json(&broadcast_sources::ADD)],
            ),
        ];
        const YOURS: &[Section<'static>] = &[(
            "Your broadcasts (text)",
            &[Example::text(&broadcasts_own::LIST)],
        )];
        const SCHEDULED: &[Section<'static>] = &[(
            "Schedule one, JSON envelope",
            &[Example::json(&broadcasts_scheduled::ADD)],
        )];
        const CHAT: &[Section<'static>] = &[(
            "Send into a broadcast's chat, JSON envelope",
            &[Example::json(&broadcast_chat::SEND)],
        )];
        page_of(&[
            (&broadcasts_own::FAMILY, YOURS),
            (&broadcasts_scheduled::FAMILY, SCHEDULED),
            (&FAMILY, MODERATORS),
            (&broadcast_sources::FAMILY, SOURCES),
            (&broadcast_chat::FAMILY, CHAT),
        ])
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

/// The `xr broadcasts sources` family.
pub mod broadcast_sources {
    use super::{Example, Family, Section, Verb};

    /// The family every page below belongs to.
    pub const FAMILY: Family = Family {
        path: "broadcasts sources",
    };

    /// `xr broadcasts sources list`.
    pub const LIST: Verb = Verb {
        name: "list",
        example_args: "",
        text_caption: "List your stream sources (text)",
        json_caption: "As a JSON envelope",
    };

    /// `xr broadcasts sources get`.
    pub const GET: Verb = Verb {
        name: "get",
        example_args: "1x2y3z4w5v6u",
        text_caption: "Show a source and whether it is receiving video (text)",
        json_caption: "As a JSON envelope",
    };

    /// `xr broadcasts sources add`.
    pub const ADD: Verb = Verb {
        name: "add",
        example_args: "\"Studio A\" --region us-east-1",
        text_caption: "Create a source; stream to its rtmps_url with its id as the key (text)",
        json_caption: "Create (JSON envelope)",
    };

    /// `xr broadcasts sources rename`.
    pub const RENAME: Verb = Verb {
        name: "rename",
        example_args: "1x2y3z4w5v6u \"Studio B\"",
        text_caption: "Rename a source (text)",
        json_caption: "Rename (JSON envelope)",
    };

    /// `xr broadcasts sources remove`.
    pub const REMOVE: Verb = Verb {
        name: "remove",
        example_args: "1x2y3z4w5v6u --force --no-interactive",
        text_caption: "Delete a source with no prompt, as a script does (text)",
        json_caption: "Delete (JSON envelope)",
    };

    /// The page under `xr broadcasts sources --help`.
    #[must_use]
    pub fn page() -> String {
        const SECTIONS: &[Section<'static>] = &[
            ("Your stream sources (text)", &[Example::text(&LIST)]),
            (
                "Create one, then check it is receiving video, JSON envelope",
                &[Example::json(&ADD), Example::json(&GET)],
            ),
        ];
        FAMILY.page(SECTIONS)
    }
}

/// The `xr calls` family.
pub mod calls {
    use super::{Example, Family, Section, Verb, page_of, scheduled_calls};

    /// The family every page below belongs to.
    pub const FAMILY: Family = Family { path: "calls" };

    /// `xr calls get`.
    pub const GET: Verb = Verb {
        name: "get",
        example_args: "8a6f1c2e",
        text_caption: "Who hosts a call, who is in it, and whether you may join (text)",
        json_caption: "As a JSON envelope",
    };

    /// The page under `xr calls --help`.
    #[must_use]
    pub fn root_page() -> String {
        const CALLS: &[Section<'static>] = &[("Look up a call (text)", &[Example::text(&GET)])];
        const SCHEDULED: &[Section<'static>] = &[
            (
                "The calls on your calendar this week (text)",
                &[Example::text(&scheduled_calls::LIST)],
            ),
            (
                "Schedule one, JSON envelope",
                &[Example::json(&scheduled_calls::ADD)],
            ),
        ];
        page_of(&[(&FAMILY, CALLS), (&scheduled_calls::FAMILY, SCHEDULED)])
    }
}

/// The `xr calls scheduled` family.
pub mod scheduled_calls {
    use super::{Example, Family, Section, Verb};

    /// The family every page below belongs to.
    pub const FAMILY: Family = Family {
        path: "calls scheduled",
    };

    /// `xr calls scheduled list`.
    pub const LIST: Verb = Verb {
        name: "list",
        example_args: "--from 2026-11-02T00:00:00Z --to 2026-11-09T00:00:00Z",
        text_caption: "The calls on your calendar in a week (text)",
        json_caption: "As a JSON envelope",
    };

    /// `xr calls scheduled get`.
    pub const GET: Verb = Verb {
        name: "get",
        example_args: "1849302018273645",
        text_caption: "A call with its invitees and their answers (text)",
        json_caption: "As a JSON envelope",
    };

    /// `xr calls scheduled add`.
    pub const ADD: Verb = Verb {
        name: "add",
        example_args: "--start 2026-11-02T16:00:00Z --title Standup --duration 15 --invite 2244994945",
        text_caption: "Schedule a 15-minute call and invite someone (text)",
        json_caption: "Schedule (JSON envelope)",
    };

    /// `xr calls scheduled update`.
    pub const UPDATE: Verb = Verb {
        name: "update",
        example_args: "1849302018273645 --start 2026-11-02T17:00:00Z",
        text_caption: "Move a call an hour later (text)",
        json_caption: "Move (JSON envelope)",
    };

    /// `xr calls scheduled cancel`.
    pub const CANCEL: Verb = Verb {
        name: "cancel",
        example_args: "1849302018273645 --force --no-interactive",
        text_caption: "Cancel a call with no prompt, as a script does (text)",
        json_caption: "Cancel (JSON envelope)",
    };

    /// `xr calls scheduled rsvp`.
    pub const RSVP: Verb = Verb {
        name: "rsvp",
        example_args: "1849302018273645 accepted",
        text_caption: "Accept an invitation (text)",
        json_caption: "Accept (JSON envelope)",
    };

    /// The page under `xr calls scheduled --help`.
    #[must_use]
    pub fn page() -> String {
        const SECTIONS: &[Section<'static>] = &[
            (
                "The calls on your calendar in a week (text)",
                &[Example::text(&LIST)],
            ),
            (
                "Schedule a call, then move it, JSON envelope",
                &[Example::json(&ADD), Example::json(&UPDATE)],
            ),
            ("Answer an invitation (text)", &[Example::text(&RSVP)]),
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
