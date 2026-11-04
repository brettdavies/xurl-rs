#!/usr/bin/env python3
"""Drive `xr auth oauth2`'s interactive sign-in offline, one run per mode.

Usage: python3 -B -I oauth2-interactive.py XR_BINARY [--mode MODE]...

Needs Linux, Python 3.10 or later, and `script` from util-linux.

Each mode signs in on its own scratch store, with `xr` under a pty so it takes
its interactive branch, a fake `xdg-open` standing in for the browser, and a
loopback stub standing in for X. Every endpoint `xr` reads points at the stub,
so no request leaves the machine.

Modes, by what the authorize redirect brings to `xr`'s callback listener:

  decoded   the state as the authorize URL gave it
  encoded   the same state with every byte percent-encoded
  refused   another state
  denied    error=access_denied beside the right state
  idle      the decoded redirect, behind a connection that never sends a byte

Prints one tab-separated line per mode: PASS or FAIL, the mode, the detail.

Exit codes: 0 every mode passed, 1 a mode failed, 2 the harness could not run.
"""

import sys

sys.dont_write_bytecode = True

import argparse
import base64
import contextlib
import hashlib
import http.client
import http.server
import json
import os
import random
import shlex
import shutil
import signal
import socket
import stat
import subprocess
import tempfile
import threading
import time
import traceback
import urllib.parse
from collections.abc import Callable
from dataclasses import dataclass, field

APP = "standin"
CODE = "standin-code"
LOGIN = "standin_user"
TOKEN_PATH = "/2/oauth2/token"
USERS_ME_PATH = "/2/users/me"
AUTHORIZE_PATH = "/i/oauth2/authorize"
TOKEN_CALL = f"POST {TOKEN_PATH}"
USERS_ME_CALL = f"GET {USERS_ME_PATH}"

# The redirect port is named before the listener binds it, so it comes from
# below every OS's ephemeral range, where no port-0 listener lands.
PORT_BAND = (20000, 32767)
PORT_PICKS = 5
BIND_FAILURE = "could not bind callback listener"

EXIT_AUTH = 77
SIGN_IN_LIMIT_SECS = 30.0
IDLE_LIMIT_SECS = 5.0
BROWSER_LIMIT_SECS = 20.0
ANSWER_WAIT_SECS = 2.0
# A request that arrives after the accept, split inside the state value, is what
# a slow browser sends. A listener that reads once sees none of it or half of it.
SEND_PAUSE_SECS = 0.3


class StubProvider(http.server.ThreadingHTTPServer):
    """The stand-in for X: authorize redirect, token exchange, and `/2/users/me`."""

    daemon_threads = True

    def __init__(self, mode: str):
        super().__init__(("127.0.0.1", 0), StubHandler)
        self.mode = mode
        self.calls: list[str] = []
        self.code_challenge = ""

    @property
    def origin(self) -> str:
        return f"http://127.0.0.1:{self.server_address[1]}"

    def redirect_params(self, state: str) -> str:
        if self.mode == "denied":
            params = "error=access_denied&error_description=The+user+denied+access"
        else:
            params = f"code={CODE}"
        return f"{params}&state={self.redirect_state(state)}"

    def redirect_state(self, state: str) -> str:
        if self.mode == "refused":
            return "NOT-THE-STATE"
        if self.mode == "encoded":
            # The state's alphabet needs no escaping, so urllib's quote would
            # hand it back unchanged and this mode would repeat `decoded`.
            return "".join(f"%{byte:02X}" for byte in state.encode())
        return state


class StubHandler(http.server.BaseHTTPRequestHandler):
    server: StubProvider

    def log_message(self, format, *args):
        pass

    def reply(self, status: int, document: dict):
        body = json.dumps(document).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self):
        url = urllib.parse.urlsplit(self.path)
        self.server.calls.append(f"GET {url.path}")
        if url.path == AUTHORIZE_PATH:
            query = urllib.parse.parse_qs(url.query)
            self.server.code_challenge = query["code_challenge"][0]
            redirect = query["redirect_uri"][0]
            params = self.server.redirect_params(query["state"][0])
            self.send_response(302)
            self.send_header("Location", f"{redirect}?{params}")
            self.send_header("Content-Length", "0")
            self.end_headers()
        elif url.path == USERS_ME_PATH:
            user = {"id": "1000000001", "name": "Stand In", "username": LOGIN}
            self.reply(200, {"data": user})
        else:
            self.reply(404, {"title": "Not Found"})

    def do_POST(self):
        length = int(self.headers.get("Content-Length", 0))
        form = urllib.parse.parse_qs(self.rfile.read(length).decode())
        path = urllib.parse.urlsplit(self.path).path
        self.server.calls.append(f"POST {path}")
        if path != TOKEN_PATH:
            self.reply(404, {"title": "Not Found"})
        elif not self.grant_is_valid(form):
            self.reply(400, {"error": "invalid_grant"})
        else:
            self.reply(
                200,
                {
                    "token_type": "bearer",
                    "expires_in": 7200,
                    "access_token": "standin-access",
                    "refresh_token": "standin-refresh",
                    "scope": "users.read offline.access",
                },
            )

    def grant_is_valid(self, form: dict) -> bool:
        verifier = form.get("code_verifier", [""])[0].encode()
        digest = base64.urlsafe_b64encode(hashlib.sha256(verifier).digest())
        return (
            form.get("code", [""])[0] == CODE
            and digest.rstrip(b"=").decode() == self.server.code_challenge
        )


def fake_browser(url: str) -> None:
    """What `xdg-open` runs: detach, then visit `url` the way a slow browser can."""
    # `open::that` waits for the opener, so this exits at once as a real one
    # does. The new session keeps the visit alive through the pty's hangup when
    # `xr` exits, so the listener's answer is still read.
    if os.fork():
        os._exit(0)
    os.setsid()
    scratch = os.environ["OAUTH2_GATE_SCRATCH"]
    with open(os.path.join(scratch, "browser.pid"), "w") as pid_file:
        pid_file.write(str(os.getpid()))
    answer = visit(url, os.environ["OAUTH2_GATE_MODE"])
    with open(os.path.join(scratch, "browser.answer"), "w") as answer_file:
        answer_file.write(answer)


def visit(url: str, mode: str) -> str:
    """Requests `url`, follows its redirect, and returns the status line it got."""
    authorize = urllib.parse.urlsplit(url)
    conn = http.client.HTTPConnection(authorize.hostname, authorize.port, timeout=10)
    conn.request("GET", f"{authorize.path}?{authorize.query}")
    location = conn.getresponse().getheader("Location")
    conn.close()
    redirect = urllib.parse.urlsplit(location)
    address = (redirect.hostname, redirect.port)

    idle = None
    if mode == "idle":
        idle = socket.create_connection(address, timeout=BROWSER_LIMIT_SECS)
        time.sleep(SEND_PAUSE_SECS)

    line = f"GET {redirect.path}?{redirect.query} HTTP/1.1\r\n"
    state_at = line.index("state=") + len("state=")
    cut = state_at + (line.index(" HTTP/1.1") - state_at) // 2
    rest = f"Host: {redirect.netloc}\r\nConnection: close\r\n\r\n"
    try:
        with socket.create_connection(address, timeout=BROWSER_LIMIT_SECS) as sock:
            time.sleep(SEND_PAUSE_SECS)
            sock.sendall(line[:cut].encode())
            time.sleep(SEND_PAUSE_SECS)
            sock.sendall((line[cut:] + rest).encode())
            response = b"".join(iter(lambda: sock.recv(4096), b""))
    except OSError as error:
        return f"no answer: {error}"
    finally:
        if idle is not None:
            idle.close()
    return response.decode(errors="replace").split("\r\n")[0] or "closed without a response"


@dataclass
class Run:
    """What one sign-in left behind."""

    exit_code: int | None = None
    elapsed: float = 0.0
    stderr: str = ""
    answer: str = ""
    calls: list[str] = field(default_factory=list)
    logins: list[str] = field(default_factory=list)

    @property
    def exchange(self) -> list[str]:
        return [call for call in self.calls if call in (TOKEN_CALL, USERS_ME_CALL)]

    def envelope(self) -> dict:
        # This gate also drives the previous release, and through 4.5.2 a warning
        # prints as a text line ahead of the envelope, in every format.
        try:
            document, _ = json.JSONDecoder().raw_decode(self.stderr, self.stderr.index("{"))
        except ValueError:
            return {}
        return document


def child_env(scratch: str, stub: StubProvider, mode: str) -> dict:
    """The whole environment of every process under test, built from nothing."""
    env = {
        "PATH": f"{os.path.join(scratch, 'bin')}:/usr/bin:/bin",
        "HOME": scratch,
        "SHELL": "/bin/sh",
        "TERM": "dumb",
        "XURL_TOKEN_STORE": os.path.join(scratch, "auth.yml"),
        "AUTH_URL": f"{stub.origin}{AUTHORIZE_PATH}",
        "TOKEN_URL": f"{stub.origin}{TOKEN_PATH}",
        "INFO_URL": f"{stub.origin}{USERS_ME_PATH}",
        "API_BASE_URL": stub.origin,
        "OAUTH2_GATE_MODE": mode,
        "OAUTH2_GATE_SCRATCH": scratch,
    }
    if "LD_LIBRARY_PATH" in os.environ:
        env["LD_LIBRARY_PATH"] = os.environ["LD_LIBRARY_PATH"]
    return env


def write_fake_opener(scratch: str) -> None:
    bin_dir = os.path.join(scratch, "bin")
    os.mkdir(bin_dir)
    opener = os.path.join(bin_dir, "xdg-open")
    python, harness = shlex.quote(sys.executable), shlex.quote(os.path.abspath(__file__))
    with open(opener, "w") as opener_file:
        opener_file.write(f'#!/bin/sh\nexec {python} -B -I {harness} --fake-browser "$1"\n')
    os.chmod(opener, os.stat(opener).st_mode | stat.S_IXUSR)


def kill_recorded(path: str) -> None:
    """SIGKILLs the process whose pid `path` records, when it recorded one."""
    with contextlib.suppress(OSError, ValueError):
        with open(path) as pid_file:
            os.kill(int(pid_file.read()), signal.SIGKILL)


def wait_for_browser(scratch: str) -> str:
    """The status line the browser got, read after `xr` has exited."""
    answer_path = os.path.join(scratch, "browser.answer")
    deadline = time.monotonic() + ANSWER_WAIT_SECS
    while time.monotonic() < deadline:
        with contextlib.suppress(OSError):
            with open(answer_path) as answer_file:
                answer = answer_file.read()
                if answer:
                    return answer
        time.sleep(0.02)
    return "no answer recorded"


def sign_in(xr: str, mode: str, spec: "Mode", script: str, scratch: str) -> Run:
    """One registration and one sign-in; the stub lives for exactly this run."""
    write_fake_opener(scratch)
    stub = StubProvider(mode)
    threading.Thread(target=stub.serve_forever, daemon=True).start()
    try:
        return drive(xr, mode, spec, script, scratch, stub)
    finally:
        stub.shutdown()
        stub.server_close()
        kill_recorded(os.path.join(scratch, "browser.pid"))


def drive(xr: str, mode: str, spec: "Mode", script: str, scratch: str, stub: StubProvider) -> Run:
    env = child_env(scratch, stub, mode)
    secret = os.path.join(scratch, "client-secret")
    with open(secret, "w") as secret_file:
        secret_file.write("standin-secret")
    port = random.randint(*PORT_BAND)
    # `localhost`, as the built-in redirect URI names it: the listener binds both
    # loopback addresses, the way a sign-in with no stored redirect does.
    added = run_quietly(
        [
            xr,
            "auth",
            "apps",
            "add",
            APP,
            "--client-id",
            "standin-client",
            "--client-secret-file",
            secret,
            "--redirect-uri",
            f"http://localhost:{port}/callback",
        ],
        env,
    )
    if added.returncode != 0:
        return Run(exit_code=added.returncode, stderr=f"auth apps add: {added.stderr.strip()}")

    stderr_path = os.path.join(scratch, "stderr")
    transcript_path = os.path.join(scratch, "pty.log")
    xr_pid_path = os.path.join(scratch, "xr.pid")
    command = [shlex.quote(xr)]
    if spec.output != "text":
        command += ["--output", spec.output]
    command += ["auth", "oauth2", "--app", APP]
    inner = (
        f"echo $$ >{shlex.quote(xr_pid_path)}; "
        f"exec {' '.join(command)} 2>{shlex.quote(stderr_path)}"
    )
    run = Run()
    started = time.monotonic()
    # stdin stays open until `xr` exits, because util-linux `script` passes end
    # of input through to the pty, and a person at a terminal sends none.
    with open(transcript_path, "wb") as transcript, subprocess.Popen(
        [script, "-qec", inner, "/dev/null"],
        env=env,
        stdin=subprocess.PIPE,
        stdout=transcript,
        stderr=subprocess.STDOUT,
    ) as pty:
        try:
            run.exit_code = pty.wait(timeout=spec.limit_secs)
        except subprocess.TimeoutExpired:
            kill_recorded(xr_pid_path)
            pty.kill()
            pty.wait()
    run.elapsed = time.monotonic() - started
    run.calls = list(stub.calls)
    # The redirect exists once `xr` starts; without it, `script` is what failed.
    run.stderr = read_text(stderr_path if os.path.exists(stderr_path) else transcript_path)
    if run.exit_code is not None:
        run.answer = wait_for_browser(scratch)
    run.logins = status_logins(xr, env)
    return run


def run_quietly(argv: list[str], env: dict) -> subprocess.CompletedProcess:
    return subprocess.run(
        argv,
        env=env,
        stdin=subprocess.DEVNULL,
        capture_output=True,
        text=True,
        timeout=SIGN_IN_LIMIT_SECS,
    )


def read_text(path: str) -> str:
    with open(path, errors="replace") as text_file:
        return text_file.read().strip()


def status_logins(xr: str, env: dict) -> list[str]:
    """The logins `xr --output json auth status` lists for the stand-in app."""
    status = run_quietly([xr, "--output", "json", "auth", "status"], env)
    try:
        apps = json.loads(status.stdout)["apps"]
    except (ValueError, KeyError, TypeError):
        return []
    return [login for app in apps if app.get("name") == APP for login in app.get("oauth2_users", [])]


def expect_exit(run: Run, code: int) -> list[str]:
    if run.exit_code == code:
        return []
    got = f"exit {run.exit_code}" if run.exit_code is not None else f"still running after {run.elapsed:.1f} s"
    stderr = f"; stderr: {run.stderr}" if run.stderr else ""
    return [f"expected exit {code}, got {got}{stderr}"]


def check_signed_in(run: Run) -> list[str]:
    faults = expect_exit(run, 0)
    if run.exchange != [TOKEN_CALL, USERS_ME_CALL]:
        faults.append(f"expected {TOKEN_CALL} then {USERS_ME_CALL}, the stub saw {run.calls}")
    if LOGIN not in run.logins:
        faults.append(f"auth status lists {run.logins}, not {LOGIN}")
    return faults


def check_no_exchange(run: Run) -> list[str]:
    if run.exchange:
        return [f"expected no token exchange, the stub saw {run.exchange}"]
    return []


def check_refused(run: Run) -> list[str]:
    faults = expect_exit(run, EXIT_AUTH) + check_no_exchange(run)
    envelope = run.envelope()
    reason = envelope.get("reason")
    if reason != "auth-required":
        faults.append(f"expected reason auth-required, got {reason!r}; stderr: {run.stderr}")
    action = (envelope.get("next_step") or {}).get("action")
    if action != "sign-in":
        faults.append(f"expected next_step.action sign-in, got {action!r}")
    return faults


def check_denied(run: Run) -> list[str]:
    faults = expect_exit(run, EXIT_AUTH) + check_no_exchange(run)
    message = run.envelope().get("message") or ""
    if "access_denied" not in message:
        faults.append(f"expected the message to name access_denied; stderr: {run.stderr}")
    return faults


def check_idle(run: Run) -> list[str]:
    faults = check_signed_in(run)
    if run.elapsed >= IDLE_LIMIT_SECS:
        faults.append(f"expected to finish in under {IDLE_LIMIT_SECS:.0f} s, took {run.elapsed:.1f} s")
    return faults


@dataclass(frozen=True)
class Mode:
    output: str
    answer: str
    check: Callable[[Run], list[str]]
    passed: str
    limit_secs: float = SIGN_IN_LIMIT_SECS


SIGNED_IN = f"exit 0, the stub saw {TOKEN_CALL} then {USERS_ME_CALL}, auth status lists {LOGIN}"
MODES = {
    "decoded": Mode("text", "200", check_signed_in, SIGNED_IN),
    "encoded": Mode("text", "200", check_signed_in, SIGNED_IN),
    "refused": Mode(
        "json",
        "400",
        check_refused,
        f"exit {EXIT_AUTH}, reason auth-required, next_step.action sign-in, no token exchange",
    ),
    "denied": Mode(
        "json",
        "400",
        check_denied,
        f"exit {EXIT_AUTH}, the message names access_denied, no token exchange",
    ),
    "idle": Mode("text", "200", check_idle, SIGNED_IN, IDLE_LIMIT_SECS),
}


def run_mode(xr: str, mode: str, script: str) -> tuple[bool, str]:
    spec = MODES[mode]
    for _ in range(PORT_PICKS):
        with tempfile.TemporaryDirectory(prefix="xr-oauth2-gate-") as scratch:
            run = sign_in(xr, mode, spec, script, scratch)
        if BIND_FAILURE not in run.stderr:
            break
    faults = spec.check(run)
    if run.exit_code is not None and spec.answer not in run.answer.split()[1:2]:
        faults.append(f"expected the listener to answer {spec.answer}, the browser got: {run.answer}")
    if faults:
        # One line per mode: an error envelope in a detail spans several.
        return False, " ".join("; ".join(faults).split())
    return True, f"{spec.passed}, {run.elapsed:.1f} s"


def main() -> int:
    if sys.argv[1:2] == ["--fake-browser"]:
        fake_browser(sys.argv[2])
        return 0

    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("xr", help="the xr binary to drive")
    parser.add_argument("--mode", action="append", choices=list(MODES), help="run only this mode (repeatable)")
    args = parser.parse_args()

    xr = os.path.abspath(args.xr)
    script = shutil.which("script")
    problem = None
    if not sys.platform.startswith("linux"):
        problem = f"needs Linux, this is {sys.platform}"
    elif script is None:
        problem = "needs script(1) from util-linux on PATH"
    elif not os.access(xr, os.X_OK):
        problem = f"{xr} is not an executable"
    if problem:
        print(f"oauth2-interactive: {problem}", file=sys.stderr)
        return 2

    all_passed = True
    for mode in args.mode or MODES:
        passed, detail = run_mode(xr, mode, script)
        all_passed = all_passed and passed
        print(f"{'PASS' if passed else 'FAIL'}\t{mode}\t{detail}", flush=True)
    return 0 if all_passed else 1


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception:
        # Python's own exit for a crash is 1, which here means a mode failed.
        traceback.print_exc()
        sys.exit(2)
