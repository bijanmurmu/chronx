# Contributing to chronx

Thank you for your interest in improving `chronx`! We want to make contributing as easy and welcoming as possible.

## 🛠️ Local Development Setup

1. **Install Rust:** If you don't have Rust installed, get it from [rustup.rs](https://rustup.rs/).
2. **Clone the Repo:** 
   ```bash
   git clone https://github.com/bijanmurmu/chronx.git
   cd chronx
   ```
3. **Test the Daemon Locally:**
   ```bash
   cargo run -- watch
   ```
4. **Compile the Release Binary:**
   ```bash
   cargo build --release
   ```

## 📝 Pull Request Rules (The "Clean PR" Policy)

Like many major open-source projects, we maintain a strictly clean Git history (no `merge` commits, and no messy "wip" commits). 

**Before you submit a Pull Request:**
1. You must squash all your work into **one single commit**.
2. **Pro-tip:** You don't need to know how to use complex `git rebase` commands! Just use `chronx` itself:
   ```bash
   cargo run -- squash -m "feat: added new feature"
   ```
3. Push your branch to GitHub and open the PR!
