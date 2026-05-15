
Notes:
- setting to enable sending crash report
- having possibility to save files from wineprefix or elsewere
- being able to add Desktop for each games
- default save folder setting
- when installing game Default tmp else ask for where
- game save folder setting
- game launch options
- log per games
- adding tool for games
- Splashcreen for launching game


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


















// esync: eventfd-based synchronization (replaces wineserver mutexes)
        ctx.set_env("PROTON_NO_ESYNC", if f.esync { "0" } else { "1" });

        // fsync: futex-based (faster than esync, requires kernel ≥ 5.16)
        ctx.set_env("PROTON_NO_FSYNC", if f.fsync { "0" } else { "1" });

        // FSR: AMD FidelityFX Super Resolution upscaling via Wine
        if f.fsr {
            ctx.set_env("WINE_FULLSCREEN_FSR", "1");
            ctx.set_env("WINE_FULLSCREEN_FSR_STRENGTH", "2");
        }

        // MangoHud: FPS overlay
        if f.mangohud {
            ctx.set_env("MANGOHUD", "1");
            ctx.set_env("MANGOHUD_DLSYM", "1");
        }

        // DXVK async shader compilation (reduces stutters)
        if f.dxvk_async {
            ctx.set_env("DXVK_ASYNC", "1");
        }

        // Disable D3D12 (VKD3D-Proton) — useful for problematic games
        if f.no_d3d12 {
            ctx.set_env("PROTON_NO_D3D12", "1");
        }

        // Proton log output
        if f.log_proton {
            let log_dir = dirs::home_dir().unwrap_or_default().join("proton_logs");
            std::fs::create_dir_all(&log_dir)?;
            ctx.set_env("PROTON_LOG", "1");
            ctx.set_env("PROTON_LOG_DIR", log_dir.to_str().unwrap());
        }

        // Vulkan ICD (GPU selection)
        if let Some(icd) = &f.vulkan_icd {
            ctx.set_env("VK_ICD_FILENAMES", icd);
        }
