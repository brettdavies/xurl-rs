# Solution: Branch Reset File Inventory Protocol

## Problem

When resetting `dev` to `main` after squash merges, files that exist only on `dev` (blocked from `main` by CI guards like `guard-main-docs.yml`) are silently lost. A naive `git diff main..dev` only shows files that differ in content — it misses files that were already orphaned by prior squash merges when the branches had unrelated histories.

## Root Cause

Squash merges create new commits on `main` with no parent relationship to `dev`. Over time, `dev` and `main` can have completely unrelated histories (no merge base). When `dev` is reset to `main` to re-establish shared history, any files that only existed on `dev` are destroyed.

`git diff main..dev` is insufficient because it compares tree content, not file existence across the full history graph. Files removed in intermediate commits on `dev` before the diff point are invisible.

## Solution: Full Inventory Before Reset

Before any branch reset, run a complete file inventory:

```bash
# BEFORE reset: capture full file list on dev
git ls-tree -r --name-only dev | sort > /tmp/dev-files.txt

# Capture full file list on main
git ls-tree -r --name-only main | sort > /tmp/main-files.txt

# Files on dev but NOT on main — these WILL BE LOST
comm -23 /tmp/dev-files.txt /tmp/main-files.txt
```

Every file in that diff list must be explicitly preserved (saved to /tmp, cherry-picked, or checked out after reset) or explicitly acknowledged as intentionally dropped.

## Recovery

If files are already lost, orphaned commits remain accessible via `git fsck`:

```bash
# Find all unreachable commits
git fsck --unreachable --no-reflogs 2>/dev/null | grep 'unreachable commit' | awk '{print $3}'

# Search each for the missing file
for sha in $(git fsck --unreachable --no-reflogs 2>/dev/null | grep 'unreachable commit' | awk '{print $3}'); do
  git ls-tree -r --name-only "$sha" -- path/to/file 2>/dev/null && echo "  ^ found in $sha"
done
```

These orphaned commits are eventually garbage-collected, so recovery is time-sensitive.

## Prevention

1. Always use `git ls-tree` (not `git diff`) to inventory files before destructive branch operations
2. CI guards that block files from `main` create a split-brain — those files need explicit handling during any branch sync
3. After reset, verify the new branch has all expected files before force-pushing
