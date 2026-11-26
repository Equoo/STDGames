//! Tests for the save manager.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use stdg_core::{
    BindMode, GameId, LaunchCtx, Layer, ModeId, Plan, ResolvedConfig, RunKind, RunnerId, SessionId,
    SessionInfo, TargetKind, WinePrefix,
};

use crate::{
    GameSaves, Location, ManifestIndex, Os, PathContext, ProfileName, SavesError, SavesLayer,
    resolve,
};

const MANIFEST: &str = r#"
Studio Game:
  files:
    "<winAppData>/Studio/Game":
      tags: [save]
      when:
        - os: windows
    "<xdgData>/studio-game":
      when:
        - os: linux
    "<winDocuments>/Studio/*.cfg":
      when:
        - os: windows
          store: gog
    "<base>/cfg": {}
  steam:
    id: 100
  id:
    steamExtra: [101]
Studio Game Remastered:
  alias: Studio Game
  steam:
    id: 200
No Files:
  steam:
    id: 300
"#;

fn index() -> ManifestIndex {
    ManifestIndex::from_manifest_yaml(MANIFEST).expect("valid manifest")
}

fn rule_paths(index: &ManifestIndex, name: Option<&str>, id: Option<u32>) -> Option<Vec<String>> {
    index
        .rules_for(name, id)
        .map(|(_, rules)| rules.iter().map(|r| r.path.clone()).collect())
}

#[test]
fn index_finds_games_by_name_steam_id_extra_id_and_alias() {
    let index = index();
    assert_eq!(index.len(), 1, "games without files are dropped");
    let by_name = rule_paths(&index, Some("Studio Game"), None);
    assert!(by_name.is_some());
    assert_eq!(rule_paths(&index, None, Some(100)), by_name);
    assert_eq!(rule_paths(&index, None, Some(101)), by_name);
    assert_eq!(rule_paths(&index, None, Some(200)), by_name);
    assert_eq!(
        rule_paths(&index, Some("Studio Game Remastered"), None),
        by_name
    );
    assert_eq!(rule_paths(&index, None, Some(300)), None);
    assert_eq!(rule_paths(&index, None, None), None);
}

#[test]
fn rules_apply_by_os_and_steam_store() {
    let index = index();
    let (_, rules) = index.rules_for(Some("Studio Game"), None).expect("found");
    let applying = |os| -> Vec<&str> {
        rules
            .iter()
            .filter(|r| r.applies_to(os))
            .map(|r| r.path.as_str())
            .collect()
    };
    assert_eq!(
        applying(Os::Windows),
        ["<base>/cfg", "<winAppData>/Studio/Game"]
    );
    assert_eq!(applying(Os::Linux), ["<base>/cfg", "<xdgData>/studio-game"]);
}

#[test]
fn index_round_trips_and_ignores_missing_files() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("sub/index.json");
    assert!(
        ManifestIndex::load(&path)
            .expect("missing is fine")
            .is_none()
    );
    index().save(&path).expect("save");
    let loaded = ManifestIndex::load(&path).expect("load").expect("present");
    assert_eq!(
        rule_paths(&loaded, None, Some(100)),
        rule_paths(&index(), None, Some(100))
    );
}

#[test]
fn a_broken_manifest_is_an_error() {
    assert!(matches!(
        ManifestIndex::from_manifest_yaml("game: [unclosed"),
        Err(SavesError::Manifest(_))
    ));
}

fn wine() -> WinePrefix {
    WinePrefix {
        drive_c: PathBuf::from("/pfx/drive_c"),
        user: "steamuser".to_string(),
    }
}

fn windows_cx() -> PathContext {
    PathContext {
        os: Os::Windows,
        home: Some(PathBuf::from("/home/u")),
        xdg_data: Some(PathBuf::from("/home/u/.local/share")),
        xdg_config: Some(PathBuf::from("/home/u/.config")),
        wine: Some(wine()),
    }
}

fn linux_cx() -> PathContext {
    PathContext {
        os: Os::Linux,
        wine: None,
        ..windows_cx()
    }
}

fn bind(guest: &str, store_rel: &str) -> Location {
    Location::Bind {
        guest: PathBuf::from(guest),
        store_rel: PathBuf::from(store_rel),
    }
}

#[test]
fn windows_placeholders_land_in_the_wine_prefix() {
    let cx = windows_cx();
    assert_eq!(
        resolve("<winAppData>/Studio/Game", &cx),
        bind(
            "/pfx/drive_c/users/steamuser/AppData/Roaming/Studio/Game",
            "winAppData/Studio/Game"
        )
    );
    assert_eq!(
        resolve("<home>/AppData/LocalLow/semiwork/Repo/saves", &cx),
        bind(
            "/pfx/drive_c/users/steamuser/AppData/LocalLow/semiwork/Repo/saves",
            "home/AppData/LocalLow/semiwork/Repo/saves"
        )
    );
    assert_eq!(
        resolve("<winDocuments>/My Games/<osUserName>/x", &cx),
        bind(
            "/pfx/drive_c/users/steamuser/Documents/My Games/steamuser/x",
            "winDocuments/My Games/steamuser/x"
        )
    );
    assert!(matches!(
        resolve("<xdgData>/game", &cx),
        Location::Skipped(_)
    ));
}

#[test]
fn linux_placeholders_use_the_host_home() {
    let cx = linux_cx();
    assert_eq!(
        resolve("<xdgConfig>/game/settings.ini", &cx),
        bind(
            "/home/u/.config/game/settings.ini",
            "xdgConfig/game/settings.ini"
        )
    );
    assert_eq!(
        resolve("<home>/.game", &cx),
        bind("/home/u/.game", "home/.game")
    );
    assert!(matches!(
        resolve("<winAppData>/Studio", &cx),
        Location::Skipped(_)
    ));
}

#[test]
fn globs_cut_the_path_and_too_broad_paths_are_skipped() {
    let cx = windows_cx();
    assert_eq!(
        resolve("<winAppData>/Studio/*.sav", &cx),
        bind(
            "/pfx/drive_c/users/steamuser/AppData/Roaming/Studio",
            "winAppData/Studio"
        )
    );
    assert!(matches!(
        resolve("<winDocuments>/*.sav", &cx),
        Location::Skipped(_)
    ));
    assert!(matches!(resolve("<home>", &cx), Location::Skipped(_)));
}

#[test]
fn game_dir_cloud_and_unsafe_paths_are_not_bound() {
    let cx = windows_cx();
    assert_eq!(resolve("<base>/cfg", &cx), Location::GameDir);
    assert_eq!(
        resolve("<root>/userdata/<storeUserId>/220/remote", &cx),
        Location::SteamCloud
    );
    assert!(matches!(
        resolve("<winAppData>/../x", &cx),
        Location::Skipped(_)
    ));
    assert!(matches!(resolve("/abs/path", &cx), Location::Skipped(_)));
    assert!(matches!(
        resolve("<storeUserId>/x", &cx),
        Location::Skipped(_)
    ));
}

#[test]
fn profile_names_must_be_plain_file_names() {
    assert!("speedrun-2".parse::<ProfileName>().is_ok());
    for bad in ["", ".hidden", "a/b", "..", "with space"] {
        assert!(bad.parse::<ProfileName>().is_err(), "{bad:?}");
    }
    assert!(GameSaves::new(Path::new("/s"), "../escape").is_err());
}

fn name(s: &str) -> ProfileName {
    s.parse().expect("valid name")
}

#[test]
fn profiles_are_created_copied_listed_and_picked() {
    let store = tempfile::tempdir().expect("tempdir");
    let saves = GameSaves::new(store.path(), "game").expect("valid game");
    assert_eq!(saves.list().expect("list"), Vec::<ProfileName>::new());
    assert_eq!(
        saves.current().expect("current"),
        ProfileName::default_profile()
    );

    saves.create(&name("main"), None).expect("create");
    std::fs::write(saves.profile_dir(&name("main")).join("slot.sav"), "1").expect("write");
    saves
        .create(&name("copy"), Some(&name("main")))
        .expect("copy");
    assert_eq!(
        std::fs::read_to_string(saves.profile_dir(&name("copy")).join("slot.sav")).expect("read"),
        "1"
    );
    assert!(matches!(
        saves.create(&name("main"), None),
        Err(SavesError::ProfileExists(_))
    ));
    assert!(matches!(
        saves.create(&name("x"), Some(&name("missing"))),
        Err(SavesError::NoSuchProfile(_))
    ));

    saves.set_current(&name("copy")).expect("use");
    assert_eq!(saves.current().expect("current"), name("copy"));
    assert!(saves.set_current(&name("missing")).is_err());
    assert_eq!(
        saves.list().expect("list"),
        vec![name("copy"), name("main")]
    );
}

#[test]
fn export_then_import_restores_a_profile() {
    let store = tempfile::tempdir().expect("tempdir");
    let saves = GameSaves::new(store.path(), "game").expect("valid game");
    saves.create(&name("main"), None).expect("create");
    let nested = saves
        .profile_dir(&name("main"))
        .join("paths/winAppData/Studio");
    std::fs::create_dir_all(&nested).expect("mkdir");
    std::fs::write(nested.join("slot.sav"), "42").expect("write");

    let archive = store.path().join("main.tar.gz");
    saves.export(&name("main"), &archive).expect("export");
    saves.import(&name("restored"), &archive).expect("import");
    let restored = saves
        .profile_dir(&name("restored"))
        .join("paths/winAppData/Studio/slot.sav");
    assert_eq!(std::fs::read_to_string(restored).expect("read"), "42");
    assert!(matches!(
        saves.import(&name("restored"), &archive),
        Err(SavesError::ProfileExists(_))
    ));
}

fn ctx(root: &Path, run_kind: RunKind) -> LaunchCtx {
    let config = ResolvedConfig {
        app_id: None,
        game_id: GameId("game".to_string()),
        target: TargetKind::Windows,
        root: root.to_path_buf(),
        executable: PathBuf::from("game.exe"),
        args: vec![],
        command_prefix: vec![],
        env: BTreeMap::new(),
        saves: stdg_core::SavesConfig::default(),
    };
    LaunchCtx {
        plan: Plan {
            game_id: config.game_id.clone(),
            mode_id: ModeId("m".to_string()),
            target: config.target.clone(),
            runner: RunnerId("windows".to_string()),
            slots: BTreeMap::new(),
            config,
        },
        session: SessionInfo {
            id: SessionId("s".to_string()),
            tmp_dir: PathBuf::from("/nonexistent"),
        },
        bindings: Vec::new(),
        run_kind,
        wine_prefix: None,
    }
}

fn layer(store: &Path, profile: &str, rules: &[&str]) -> SavesLayer {
    SavesLayer {
        game_root: PathBuf::from("/games/game"),
        os: Os::Windows,
        saves: GameSaves::new(store, "game").expect("valid game"),
        profile: name(profile),
        rules: rules.iter().map(|r| (*r).to_string()).collect(),
        notice: None,
        cloud: None,
    }
}

#[test]
fn game_dir_is_overlaid_and_nested_save_dirs_bound_once() {
    let store = tempfile::tempdir().expect("tempdir");
    let mut cx = ctx(Path::new("/games/game"), RunKind::DryRun);
    cx.wine_prefix = Some(wine());
    let l = layer(
        store.path(),
        "main",
        &[
            "<winAppData>/Studio/Game/saves",
            "<winAppData>/Studio/Game/*.cfg",
            "<base>/cfg",
        ],
    );
    let needs = l.container_needs(&cx);
    assert_eq!(needs.len(), 2);
    assert_eq!(needs[0].source.effective(), Path::new("/games/game"));
    assert_eq!(
        needs[0].mode,
        BindMode::Overlay {
            upper: store.path().join("game/main/root"),
            work: store.path().join("game/.work/main"),
        }
    );
    assert_eq!(
        needs[1].source.effective(),
        Path::new("/pfx/drive_c/users/steamuser/AppData/Roaming/Studio/Game")
    );
    assert_eq!(
        needs[1].source.host(),
        store.path().join("game/main/paths/winAppData/Studio/Game")
    );
    assert_eq!(needs[1].mode, BindMode::ReadWrite);
}

fn prefix_with_old_save(dir: &Path) -> WinePrefix {
    let prefix = WinePrefix {
        drive_c: dir.join("drive_c"),
        user: "steamuser".to_string(),
    };
    let save_dir = prefix.user_dir().join("AppData/Roaming/Studio");
    std::fs::create_dir_all(&save_dir).expect("mkdir");
    std::fs::write(save_dir.join("old.sav"), "old").expect("write");
    prefix
}

#[test]
fn live_prepare_adopts_existing_saves_into_the_default_profile_only() {
    let store = tempfile::tempdir().expect("tempdir");
    let host = tempfile::tempdir().expect("tempdir");
    let prefix = prefix_with_old_save(host.path());

    for (profile, adopted) in [("default", true), ("fresh", false)] {
        let mut cx = ctx(Path::new("/games/game"), RunKind::Live);
        cx.wine_prefix = Some(prefix.clone());
        layer(store.path(), profile, &["<winAppData>/Studio"])
            .prepare(&mut cx)
            .expect("prepare");
        let stored = store
            .path()
            .join(format!("game/{profile}/paths/winAppData/Studio"));
        assert!(stored.is_dir(), "{profile}: save dir created");
        assert_eq!(stored.join("old.sav").is_file(), adopted, "{profile}");
        assert!(store.path().join(format!("game/{profile}/root")).is_dir());
        assert!(store.path().join(format!("game/.work/{profile}")).is_dir());
    }
}

#[test]
fn dry_run_prepare_touches_nothing() {
    let store = tempfile::tempdir().expect("tempdir");
    let mut cx = ctx(Path::new("/games/game"), RunKind::DryRun);
    cx.wine_prefix = Some(wine());
    layer(store.path(), "default", &["<winAppData>/Studio"])
        .prepare(&mut cx)
        .expect("prepare");
    assert!(!store.path().join("game").exists());
}

#[test]
fn a_profile_is_used_by_one_launch_at_a_time() {
    let store = tempfile::tempdir().expect("tempdir");
    let saves = GameSaves::new(store.path(), "game").expect("valid game");
    saves.create(&name("main"), None).expect("create");

    let lock = saves.lock(&name("main")).expect("first lock");
    assert!(matches!(
        saves.lock(&name("main")),
        Err(SavesError::ProfileInUse(_))
    ));
    assert!(matches!(
        saves.export(&name("main"), &store.path().join("a.tar.gz")),
        Err(SavesError::ProfileInUse(_))
    ));
    assert!(
        saves.lock(&name("other")).is_ok(),
        "other profiles are free"
    );
    drop(lock);
    assert!(saves.lock(&name("main")).is_ok(), "released on drop");
}

#[test]
fn a_live_launch_holds_the_lock_until_its_guard_drops() {
    let store = tempfile::tempdir().expect("tempdir");
    let l = layer(store.path(), "main", &[]);
    let mut cx = ctx(Path::new("/games/game"), RunKind::Live);
    let guard = l.prepare(&mut cx).expect("first launch");
    assert!(l.prepare(&mut cx).is_err(), "second launch refused");
    drop(guard);
    assert!(l.prepare(&mut cx).is_ok());
}
