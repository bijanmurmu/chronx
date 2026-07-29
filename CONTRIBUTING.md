# Contributing to chronx

Thank you for your interest in improving `chronx`! We want to make contributing as easy and welcoming as possible.

## 🛠️ Local Development Setup

1. **Install Rust:** If you don't have Rust installed, get it from [rustup.rs](https://rustup.rs/).
2. **Clone the Repo:** 
   ```bash
   git clone https://github.com/bijanmurmu/chronx.git
   cd chronx
   ```
3. **Test the Daemon and Recovery Locally:**
   - Initialize tracking in the current directory:
     ```bash
     cargo run -- init
     ```
   - Start the file watcher in the foreground:
     ```bash
     cargo run -- watch
     ```
   - In a separate terminal, create and modify some test files in the directory.
   - Test the interactive recovery menu:
     ```bash
     cargo run -- log
     ```
     Use your arrow keys to select a previous snapshot and press `Enter` to restore your test file.
4. **Compile the Release Binary:**
   ```bash
   cargo build --release
   ```

### 🐛 Troubleshooting: Access is Denied (os error 5)

When compiling or running the tool locally, you might encounter an `Access is denied (os error 5)` error. This happens because Cargo cannot overwrite the `.exe` file if `chronx` is currently running in the background.

**How to kill the running process:**
1. Check if you left a terminal window open that is running `cargo run -- watch` and press `Ctrl+C` to stop it.
2. If it is running invisibly in the background, forcefully kill it using your terminal:
   - **Windows (PowerShell):** `Stop-Process -Name "chronx" -Force`
   - **MacOS / Linux:** `pkill chronx`

## 📝 Pull Request Rules (The "Clean PR" Policy)

Like many major open-source projects, we maintain a strictly clean Git history (no `merge` commits, and no messy "wip" commits). 

**Before you submit a Pull Request:**
1. You must squash all your work into **one single commit**.
2. **Pro-tip:** You don't need to know how to use complex `git rebase` commands! Just use `chronx` itself:
   ```bash
   cargo run -- squash -m "feat: added new feature"
   ```
3. Push your branch to GitHub and open the PR!
