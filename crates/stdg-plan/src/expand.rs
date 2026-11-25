//! `${...}` variable expansion in config strings.
//!
//! - `${env:NAME}`: environment variable; a leading `~` means `${env:HOME}`.
//! - `${game.id}`, `${game.root}`, `${game.executable}`, `${mode.id}`, `${runner.id}`.
//! - `$$`: a literal `$`.

use std::path::PathBuf;

use stdg_core::{ConfigError, CoreError, LayerRef, ModeId, ResolvedConfig, RunnerId};

/// Values available to `${...}` references.
pub struct Vars<'a> {
    /// `${game.id}`.
    pub game_id: String,
    /// `${game.root}`; `None` while expanding `root` itself.
    pub game_root: Option<String>,
    /// `${game.executable}`; `None` while expanding `root` or `executable`.
    pub game_executable: Option<String>,
    /// `${mode.id}`.
    pub mode_id: String,
    /// `${runner.id}`.
    pub runner_id: String,
    /// Environment lookup.
    pub env: &'a dyn Fn(&str) -> Option<String>,
}

impl std::fmt::Debug for Vars<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Vars")
            .field("game_id", &self.game_id)
            .field("game_root", &self.game_root)
            .field("game_executable", &self.game_executable)
            .field("mode_id", &self.mode_id)
            .field("runner_id", &self.runner_id)
            .finish_non_exhaustive()
    }
}

impl Vars<'_> {
    fn lookup(&self, name: &str) -> Result<String, String> {
        if let Some(var) = name.strip_prefix("env:") {
            return (self.env)(var)
                .ok_or_else(|| format!("environment variable `{var}` is not set"));
        }
        let value = match name {
            "game.id" => Some(&self.game_id),
            "mode.id" => Some(&self.mode_id),
            "runner.id" => Some(&self.runner_id),
            "game.root" => self.game_root.as_ref(),
            "game.executable" => self.game_executable.as_ref(),
            _ => return Err(format!("unknown variable `${{{name}}}`")),
        };
        value
            .cloned()
            .ok_or_else(|| format!("`${{{name}}}` cannot be used here"))
    }
}

/// Expands every `${...}` reference in `input`.
pub fn expand(input: &str, vars: &Vars<'_>) -> Result<String, String> {
    let mut out = String::with_capacity(input.len());
    let mut rest = input;

    if rest == "~" || rest.starts_with("~/") {
        out.push_str(&vars.lookup("env:HOME")?);
        rest = &rest[1..];
    }

    while let Some(pos) = rest.find('$') {
        out.push_str(&rest[..pos]);
        let after = &rest[pos + 1..];
        if let Some(tail) = after.strip_prefix('$') {
            out.push('$');
            rest = tail;
        } else if let Some(body) = after.strip_prefix('{') {
            let end = body
                .find('}')
                .ok_or_else(|| format!("unclosed `${{` in {input:?}"))?;
            out.push_str(&vars.lookup(&body[..end])?);
            rest = &body[end + 1..];
        } else {
            return Err(format!(
                "lone `$` in {input:?} (write `$$` for a literal `$`)"
            ));
        }
    }
    out.push_str(rest);
    Ok(out)
}

fn expand_at(
    input: &str,
    vars: &Vars<'_>,
    location: impl Into<String>,
) -> Result<String, CoreError> {
    expand(input, vars).map_err(|reason| {
        ConfigError::Expansion {
            location: location.into(),
            reason,
        }
        .into()
    })
}

fn path_str(path: &std::path::Path) -> String {
    path.to_string_lossy().into_owned()
}

/// Expands every string of `config` in place and returns the final variables.
pub(crate) fn expand_config<'a>(
    config: &mut ResolvedConfig,
    mode_id: &ModeId,
    runner_id: &RunnerId,
    env: &'a dyn Fn(&str) -> Option<String>,
) -> Result<Vars<'a>, CoreError> {
    let mut vars = Vars {
        game_id: config.game_id.0.clone(),
        game_root: None,
        game_executable: None,
        mode_id: mode_id.0.clone(),
        runner_id: runner_id.0.clone(),
        env,
    };

    // `root` first, then `executable`: later values may refer to them.
    config.root = PathBuf::from(expand_at(&path_str(&config.root), &vars, "root")?);
    vars.game_root = Some(path_str(&config.root));
    config.executable = PathBuf::from(expand_at(
        &path_str(&config.executable),
        &vars,
        "executable",
    )?);
    vars.game_executable = Some(path_str(&config.executable));

    for arg in &mut config.args {
        *arg = expand_at(arg, &vars, "args")?;
    }
    for item in &mut config.command_prefix {
        *item = expand_at(item, &vars, "command_prefix")?;
    }
    for (key, value) in &mut config.env {
        *value = expand_at(value, &vars, format!("env {key}"))?;
    }
    Ok(vars)
}

/// Expands every parameter of `layer_ref` in place.
pub(crate) fn expand_layer_ref(layer_ref: &mut LayerRef, vars: &Vars<'_>) -> Result<(), CoreError> {
    for (key, value) in &mut layer_ref.params {
        *value = expand_at(value, vars, format!("layer {}, param {key}", layer_ref.id))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use stdg_core::{GameId, TargetKind};

    use super::*;

    fn fake_env(name: &str) -> Option<String> {
        match name {
            "HOME" => Some("/home/u".to_string()),
            "GAMES" => Some("/srv/games".to_string()),
            _ => None,
        }
    }

    fn vars() -> Vars<'static> {
        Vars {
            game_id: "repo".to_string(),
            game_root: Some("/srv/games/repo".to_string()),
            game_executable: Some("REPO.exe".to_string()),
            mode_id: "steam".to_string(),
            runner_id: "windows".to_string(),
            env: &fake_env,
        }
    }

    #[test]
    fn expands_env_and_config_variables() {
        assert_eq!(
            expand("${env:HOME}/pfx/${game.id}-${mode.id}", &vars()).as_deref(),
            Ok("/home/u/pfx/repo-steam")
        );
        assert_eq!(
            expand("${game.root}/${game.executable} on ${runner.id}", &vars()).as_deref(),
            Ok("/srv/games/repo/REPO.exe on windows")
        );
    }

    #[test]
    fn leading_tilde_is_home() {
        assert_eq!(expand("~/x", &vars()).as_deref(), Ok("/home/u/x"));
        assert_eq!(expand("~", &vars()).as_deref(), Ok("/home/u"));
        assert_eq!(expand("a~b", &vars()).as_deref(), Ok("a~b"));
    }

    #[test]
    fn plain_strings_and_escapes_pass_through() {
        assert_eq!(expand("no vars", &vars()).as_deref(), Ok("no vars"));
        assert_eq!(expand("cost $$5", &vars()).as_deref(), Ok("cost $5"));
        assert_eq!(expand("{rom}", &vars()).as_deref(), Ok("{rom}"));
    }

    #[test]
    fn bad_references_are_errors() {
        assert!(expand("${env:NOPE}", &vars()).is_err());
        assert!(expand("${game.nope}", &vars()).is_err());
        assert!(expand("${game.root", &vars()).is_err());
        assert!(expand("a $ b", &vars()).is_err());
    }

    #[test]
    fn root_cannot_refer_to_itself() {
        let mut v = vars();
        v.game_root = None;
        let err = expand("${game.root}/x", &v).expect_err("not available yet");
        assert!(err.contains("cannot be used here"));
    }

    #[test]
    fn expand_config_resolves_root_before_dependent_fields() {
        let mut config = ResolvedConfig {
            game_id: GameId("repo".to_string()),
            target: TargetKind::Windows,
            root: PathBuf::from("${env:GAMES}/${game.id}"),
            executable: PathBuf::from("REPO.exe"),
            args: vec!["--data=${game.root}/data".to_string()],
            command_prefix: vec!["~/bin/wrap".to_string()],
            env: BTreeMap::from([("SAVE".to_string(), "${game.root}/save".to_string())]),
        };
        let vars = expand_config(
            &mut config,
            &ModeId("steam".to_string()),
            &RunnerId("windows".to_string()),
            &fake_env,
        )
        .expect("all variables resolve");
        assert_eq!(config.root, PathBuf::from("/srv/games/repo"));
        assert_eq!(config.args, vec!["--data=/srv/games/repo/data"]);
        assert_eq!(config.command_prefix, vec!["/home/u/bin/wrap"]);
        assert_eq!(config.env["SAVE"], "/srv/games/repo/save");

        let mut r = LayerRef::new("proton").with_param("prefix_path", "${game.root}/pfx");
        expand_layer_ref(&mut r, &vars).expect("expands");
        assert_eq!(r.param("prefix_path"), Some("/srv/games/repo/pfx"));
    }

    #[test]
    fn errors_name_where_the_bad_value_is() {
        let mut r = LayerRef::new("proton").with_param("prefix_path", "${oops}");
        let err = expand_layer_ref(&mut r, &vars()).expect_err("unknown variable");
        let msg = format!("{}", std::error::Error::source(&err).expect("chained"));
        assert!(msg.contains("layer proton, param prefix_path"), "{msg}");
    }
}
