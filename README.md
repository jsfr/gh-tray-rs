# gh-tray

Cross-platform system tray app monitoring GitHub pull requests.

## Installation

### macOS (Homebrew)

```sh
brew tap jsfr/gh-tray-rs https://github.com/jsfr/gh-tray-rs
brew install --cask jsfr/gh-tray-rs/gh-tray
```

#### First launch on macOS

The `.app` is not code-signed. macOS will block it on first launch with a "cannot verify developer" warning. To bypass:

- Right-click `gh-tray.app` in Finder → Open → Open, **or**
- Run: `xattr -d com.apple.quarantine /Applications/gh-tray.app`

#### Migrating from a prior Formula install

If you installed gh-tray before v0.0.4:

```sh
brew uninstall gh-tray
brew untap  jsfr/gh-tray-rs
brew tap    jsfr/gh-tray-rs https://github.com/jsfr/gh-tray-rs
brew install --cask jsfr/gh-tray-rs/gh-tray
```

### Windows (Scoop)

```powershell
scoop bucket add gh-tray-rs https://github.com/jsfr/gh-tray-rs
scoop install gh-tray-rs/gh-tray
```
