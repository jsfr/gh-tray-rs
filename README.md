# gh-tray

macOS menu bar app monitoring GitHub pull requests.

## Usage

Click a pull request in the menu to open it in the browser. Hold ⇧ and
click to copy its link to the clipboard instead.

## Installation

```sh
brew tap jsfr/gh-tray-rs https://github.com/jsfr/gh-tray-rs
brew install --cask jsfr/gh-tray-rs/gh-tray
```

### First launch

The `.app` is not code-signed. macOS will block it on first launch with a "cannot verify developer" warning. To bypass:

- Right-click `gh-tray.app` in Finder → Open → Open, **or**
- Run: `xattr -d com.apple.quarantine /Applications/gh-tray.app`

### Migrating from a prior Formula install

If you installed gh-tray before v0.0.4:

```sh
brew uninstall gh-tray
brew untap  jsfr/gh-tray-rs
brew tap    jsfr/gh-tray-rs https://github.com/jsfr/gh-tray-rs
brew install --cask jsfr/gh-tray-rs/gh-tray
```
