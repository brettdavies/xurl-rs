//! Secret arguments: a plain flag's value, or its `--<name>-file` twin read
//! from a file or from stdin, so a secret never has to appear in argv.

use std::io::{IsTerminal, Read, Write};

use crate::cli::failure::Failure;
use crate::cli::output::OutputConfig;
use xdk::error::{EXIT_USAGE_ERROR, Error};

/// The path that names stdin.
const STDIN: &str = "-";

/// One secret as clap parsed it. clap keeps the two sources exclusive.
pub(super) struct SecretArg {
    /// The file twin's flag, as errors name it.
    pub(super) file_flag: &'static str,
    /// The plain flag's value.
    pub(super) value: Option<String>,
    /// The file twin's path; `-` names stdin.
    pub(super) file: Option<String>,
}

/// Why a secret could not be resolved.
#[derive(Debug)]
pub(super) enum SecretError {
    /// The invocation is wrong, and changing its arguments fixes it.
    Usage(String),
    /// A named source could not be read.
    Io(Error),
}

impl SecretError {
    /// Reports the failure for `command`: a usage mistake as the
    /// `invalid-args` envelope, a read failure as the error the runner renders.
    pub(super) fn report(
        self,
        command: &str,
        out: &OutputConfig,
        stderr: &mut dyn Write,
    ) -> Failure {
        match self {
            Self::Usage(message) => {
                out.print_error_envelope(
                    stderr,
                    "invalid-args",
                    EXIT_USAGE_ERROR,
                    &format!("{message}\n\nTry '{command} --help'."),
                );
                Failure::Emitted {
                    exit_code: EXIT_USAGE_ERROR,
                }
            }
            Self::Io(error) => Failure::Error(error),
        }
    }
}

/// [`resolve`] against the process's stdin.
pub(super) fn resolve_from_process<const N: usize>(
    args: [SecretArg; N],
) -> Result<[Option<String>; N], SecretError> {
    let stdin = std::io::stdin();
    let is_terminal = stdin.is_terminal();
    resolve(args, &mut stdin.lock(), is_terminal)
}

/// Resolves every secret of one invocation, in the order given.
///
/// Stdin carries one value, so two `-` paths are refused before anything is
/// read, and so is a `-` path when stdin is a terminal: reading there would
/// echo the secret as it is typed and wait for an end-of-file a caller rarely
/// knows to send.
pub(super) fn resolve<const N: usize>(
    args: [SecretArg; N],
    stdin: &mut dyn Read,
    stdin_is_terminal: bool,
) -> Result<[Option<String>; N], SecretError> {
    let from_stdin: Vec<&str> = args
        .iter()
        .filter(|arg| arg.file.as_deref() == Some(STDIN))
        .map(|arg| arg.file_flag)
        .collect();
    match from_stdin.as_slice() {
        [] => {}
        [flag] if stdin_is_terminal => {
            return Err(SecretError::Usage(format!(
                "'{flag} -' reads the secret from stdin, and stdin is a terminal. \
                 Pipe the secret in, or pass a file path."
            )));
        }
        [_] => {}
        several => {
            let flags = several
                .iter()
                .map(|flag| format!("'{flag} -'"))
                .collect::<Vec<_>>()
                .join(" and ");
            return Err(SecretError::Usage(format!(
                "{flags} each read stdin, which carries one value. \
                 Pass a file path to all but one of them."
            )));
        }
    }

    let mut failure = None;
    let resolved = args.map(|arg| {
        if failure.is_some() {
            return None;
        }
        read(arg, stdin).unwrap_or_else(|error| {
            failure = Some(error);
            None
        })
    });
    match failure {
        Some(error) => Err(SecretError::Io(error)),
        None => Ok(resolved),
    }
}

/// The secret `arg` names, with a file's one trailing line ending removed.
fn read(arg: SecretArg, stdin: &mut dyn Read) -> Result<Option<String>, Error> {
    let Some(path) = arg.file else {
        return Ok(arg.value);
    };
    let flag = arg.file_flag;
    let contents = if path == STDIN {
        let mut piped = String::new();
        stdin
            .read_to_string(&mut piped)
            .map_err(|e| Error::Io(format!("cannot read {flag} from stdin: {e}")))?;
        piped
    } else {
        std::fs::read_to_string(&path)
            .map_err(|e| Error::Io(format!("cannot read {flag} {path}: {e}")))?
    };
    Ok(Some(without_line_ending(contents)))
}

/// Drops one trailing `\n` or `\r\n`, the line ending an editor or `echo`
/// leaves after the secret.
fn without_line_ending(mut contents: String) -> String {
    if contents.ends_with('\n') {
        contents.pop();
        if contents.ends_with('\r') {
            contents.pop();
        }
    }
    contents
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A stdin the resolver must not touch.
    struct Unread;

    impl Read for Unread {
        fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
            panic!("stdin was read");
        }
    }

    fn from_stdin(file_flag: &'static str) -> SecretArg {
        SecretArg {
            file_flag,
            value: None,
            file: Some(STDIN.to_string()),
        }
    }

    #[test]
    fn stdin_on_a_terminal_is_refused_without_reading() {
        let error = resolve([from_stdin("--client-secret-file")], &mut Unread, true)
            .expect_err("a terminal is refused");
        let SecretError::Usage(message) = error else {
            panic!("expected a usage error, got {error:?}");
        };
        assert!(message.contains("--client-secret-file"), "{message}");
        assert!(message.contains("Pipe the secret in"), "{message}");
        assert!(message.contains("pass a file path"), "{message}");
    }

    #[test]
    fn a_second_stdin_secret_is_refused_without_reading() {
        let error = resolve(
            [
                from_stdin("--consumer-secret-file"),
                from_stdin("--token-secret-file"),
            ],
            &mut Unread,
            false,
        )
        .expect_err("stdin carries one value");
        let SecretError::Usage(message) = error else {
            panic!("expected a usage error, got {error:?}");
        };
        assert!(message.contains("--consumer-secret-file"), "{message}");
        assert!(message.contains("--token-secret-file"), "{message}");
    }

    #[test]
    fn a_piped_secret_loses_its_line_ending_and_a_plain_value_passes_through() {
        let mut piped: &[u8] = b"PIPED\r\n";
        let [from_pipe, plain] = resolve(
            [
                from_stdin("--token-secret-file"),
                SecretArg {
                    file_flag: "--consumer-secret-file",
                    value: Some("PLAIN".to_string()),
                    file: None,
                },
            ],
            &mut piped,
            false,
        )
        .expect("both resolve");
        assert_eq!(from_pipe.as_deref(), Some("PIPED"));
        assert_eq!(plain.as_deref(), Some("PLAIN"));
    }
}
