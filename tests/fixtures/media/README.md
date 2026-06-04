# Media test fixtures

Files in this directory back the `RELEASES-PREFLIGHT.md` "Real-world smoke" media-upload gate. They are NOT consumed
by `cargo test`; the in-repo test suite mocks the HTTP layer and never uploads real bytes. These exist purely so the
release walker has a deterministic, license-clean artifact to pass to `xr media upload <path> --auth oauth1` during
the live X API smoke.

## `smoke-test.jpg`

800x600 JPEG, ~14 KB, generated locally via ImageMagick (`magick -size 800x600 gradient:steelblue-white
smoke-test.jpg`). License: trivial generative output, MIT-equivalent for project use. Replace freely with a different
sample if X tightens its media-validation contract; the gate cares about the state machine (INIT -> APPEND ->
FINALIZE -> STATUS poll loop), not the bytes.

Note that X's async processing may reject specific synthetic content; the gate passes when the state machine
completes the loop and produces a coherent envelope (success OR `reason: "validation"` with the upstream body
preserved in `message`), not when X accepts the image. See the v1.3.0 smoke notes for a known instance where X
rejected ImageMagick gradient/plasma output during processing while the state machine ran correctly.
