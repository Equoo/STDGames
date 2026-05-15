# Tauri + Vanilla

This template should help get you started developing with Tauri in vanilla HTML, CSS and Javascript.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

Notes:
- setting to enable sending crash report
- having possibility to save files from wineprefix or elsewere
- being able to add Desktop for each games
- default save folder setting
- when installing game Default tmp else ask for where
- game save folder setting
- game launch options


MangoHud/etc
SSHFS - Docker
SteamEmu
SteamOnlineFix
overlay - savescript:
 - thread save and hook save at end
Reaper -> steam mode/online
SteamENV
SteamRuntime
Proton (contain SteamRuntime)
CompatibilityLayer


For all:
- overlay
- MangoHUD/etc
- no download ? SSHFS-Docker

Native:
- isSteam ? steam emu
- SteamRuntime
- Windows ? Proton

SteamOnline:
- SteamOnlineFix
- SteamRuntime
- Windows ? Proton
- Reaper

Steam: // for store: install in tmp - save library and auth info
- SteamRuntime
- WIndows ? Proton
- Reaper

Epic:
- legendary
// LEGENDARY_CONFIG_PATH=~/.config/heroic/legendaryConfig/legendary \
//   legendary launch YOUR_APP_ID \
//   --no-wine \
//   --wrapper "umu-run"

Switch: NO OVERLAY - need install
- Ryujinix
