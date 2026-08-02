# ⏳ chronx

> The invisible, zero-config "Undo" button for your local file system.

`chronx` is a global background daemon written in Rust with a powerful interactive Terminal Dashboard. It silently records everything you do—including file creations, deletions, saves, and copied files across your local machine. This lets you effortlessly rewind time and recover any file, whether you've accidentally broken your code between Git commits or simply lost a file you were working on.

## 🚀 Install

**Via NPM:**
```bash
npm install -g @bijanmurmu/chronx
```

**Via Cargo (Rust):**
```bash
cargo install chronx
```
*(Or download the pre-compiled native `.exe` from the Releases tab).*

## 💻 Usage

Simply run `chronx` with no arguments in your terminal to open the **Interactive Dashboard**! 

From the dashboard, you can:
* **[ RECOVER ]** — Scroll through a timeline of your recent file saves and instantly recover lost work.
* **[ SETUP ]** — Tell the global daemon to start tracking the current folder.
* **[ DAEMON ]** — Run a foreground watcher for the current session.
* **[ GIT SQUASH ]** — Clean up your local git history into a single commit.
* **[ SYSTEM ]** — Install the global daemon to auto-start invisibly when your PC boots.
* **[ DISABLE ]** — Stop and disable the auto-start daemon permanently.

### CLI Commands (Optional)

If you prefer to script things without the UI, you can still use the direct commands:
* `chronx install-daemon` — Registers the daemon to auto-start.
* `chronx uninstall-daemon` — Disables the auto-start daemon and kills any running instances.
* `chronx init` — Tracks the current folder.
* `chronx squash [-m "message"]` — Soft-resets your local git branch and auto-commits.

## 🔗 Links
* [Contributing Guide](CONTRIBUTING.md)
* [Code of Conduct](CODE_OF_CONDUCT.md)
* [Security Policy](SECURITY.md)
* [License](LICENSE)
