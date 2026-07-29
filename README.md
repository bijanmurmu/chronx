# ⏳ chronx

> The invisible, zero-config "Undo" button for your local file system.

`chronx` is a global background daemon written in Rust. It silently records everything you do—including file creations, deletions, saves, and copied files across your local machine. This lets you effortlessly rewind time and recover any file, whether you've accidentally broken your code between Git commits or simply lost a file you were working on.

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

## 💻 Commands

* `chronx install-daemon` — Registers the daemon to auto-start invisibly when your PC boots.
* `chronx init` — Tells the global daemon to start tracking the current folder.
* `chronx log` — Displays a timestamped timeline of your recent file saves.
* `chronx squash [-m "message"]` — Soft-resets your local git branch. Pass `-m` to instantly auto-commit the squashed code!

## 🔗 Links
* [Contributing Guide](CONTRIBUTING.md)
* [Code of Conduct](CODE_OF_CONDUCT.md)
* [Security Policy](SECURITY.md)
* [License](LICENSE)
