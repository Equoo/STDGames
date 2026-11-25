//! Concrete `LayerCatalog` / `RunnerCatalog` implementation.

use std::collections::BTreeMap;
use std::fmt;

use stdg_core::{
    CoreError, Layer, LayerCatalog, LayerId, LayerRef, ResolvedConfig, Runner, RunnerCatalog,
    RunnerId, TargetKind,
};

/// Builds a layer from its `LayerRef` and the game config.
pub type LayerFactory =
    Box<dyn Fn(&LayerRef, &ResolvedConfig) -> Result<Box<dyn Layer>, CoreError> + Send + Sync>;

/// Every runner and layer factory known to this process.
#[derive(Default)]
pub struct Registry {
    layer_factories: BTreeMap<LayerId, LayerFactory>,
    runners: Vec<Box<dyn Runner>>,
}

impl fmt::Debug for Registry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Registry")
            .field("layers", &self.layer_factories.keys().collect::<Vec<_>>())
            .field(
                "runners",
                &self.runners.iter().map(|r| r.id()).collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl Registry {
    /// An empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers `factory` under `id`, replacing any previous one.
    pub fn register_layer(&mut self, id: LayerId, factory: LayerFactory) {
        self.layer_factories.insert(id, factory);
    }

    /// Registers a runner.
    pub fn register_runner(&mut self, runner: Box<dyn Runner>) {
        self.runners.push(runner);
    }
}

impl LayerCatalog for Registry {
    fn resolve_layer(
        &self,
        r: &LayerRef,
        config: &ResolvedConfig,
    ) -> Result<Box<dyn Layer>, CoreError> {
        let factory = self
            .layer_factories
            .get(&r.id)
            .ok_or_else(|| CoreError::UnknownLayer(r.id.clone()))?;
        factory(r, config)
    }

    fn known_layer_ids(&self) -> Vec<LayerId> {
        self.layer_factories.keys().cloned().collect()
    }
}

impl RunnerCatalog for Registry {
    fn find_for_target(&self, target: &TargetKind) -> Option<&dyn Runner> {
        self.runners
            .iter()
            .find(|r| r.accepts(target))
            .map(AsRef::as_ref)
    }

    fn resolve_runner(&self, id: &RunnerId) -> Option<&dyn Runner> {
        self.runners
            .iter()
            .find(|r| r.id() == *id)
            .map(AsRef::as_ref)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use stdg_core::{CommandSpec, GameId, LaunchCtx, PathValue, Slot};

    use super::*;

    #[derive(Debug)]
    struct FakeLayer;
    impl Layer for FakeLayer {
        fn id(&self) -> LayerId {
            LayerId("fake".to_string())
        }
        fn slot(&self) -> Slot {
            Slot::Tooling
        }
    }

    #[derive(Debug)]
    struct FakeRunner;
    impl Runner for FakeRunner {
        fn id(&self) -> RunnerId {
            RunnerId("fake".to_string())
        }
        fn accepts(&self, target: &TargetKind) -> bool {
            matches!(target, TargetKind::NativeLinux)
        }
        fn build(&self, _ctx: &LaunchCtx) -> Result<CommandSpec, CoreError> {
            Ok(CommandSpec::new(PathValue::Host(PathBuf::from(
                "/bin/true",
            ))))
        }
    }

    fn config() -> ResolvedConfig {
        ResolvedConfig {
            game_id: GameId("g".to_string()),
            target: TargetKind::NativeLinux,
            root: PathBuf::from("/g"),
            executable: PathBuf::from("g"),
            args: vec![],
            command_prefix: vec![],
            env: BTreeMap::new(),
        }
    }

    fn registry() -> Registry {
        let mut registry = Registry::new();
        registry.register_layer(
            LayerId("fake".to_string()),
            Box::new(|_, _| Ok(Box::new(FakeLayer))),
        );
        registry.register_runner(Box::new(FakeRunner));
        registry
    }

    #[test]
    fn resolves_a_registered_layer() {
        let layer = registry()
            .resolve_layer(&LayerRef::new("fake"), &config())
            .expect("fake is registered");
        assert_eq!(layer.slot(), Slot::Tooling);
        assert_eq!(
            registry().known_layer_ids(),
            vec![LayerId("fake".to_string())]
        );
    }

    #[test]
    fn unknown_layer_is_an_error() {
        let err = registry()
            .resolve_layer(&LayerRef::new("nope"), &config())
            .map(|_| ())
            .expect_err("nope is not registered");
        assert!(matches!(err, CoreError::UnknownLayer(id) if id.0 == "nope"));
    }

    #[test]
    fn finds_runners_by_target_and_by_id() {
        let registry = registry();
        assert!(registry.find_for_target(&TargetKind::NativeLinux).is_some());
        assert!(registry.find_for_target(&TargetKind::Windows).is_none());
        assert!(
            registry
                .resolve_runner(&RunnerId("fake".to_string()))
                .is_some()
        );
        assert!(
            registry
                .resolve_runner(&RunnerId("other".to_string()))
                .is_none()
        );
    }
}
