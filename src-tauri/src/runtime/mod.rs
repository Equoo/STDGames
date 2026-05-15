
pub mod stages;
pub mod context;
pub mod runner;

use std::collections::HashMap;

pub use context::PipelineContext;
use fs_extra::error::Result;
pub use runner::Pipeline;


struct Overlay {
    reads: Vec<String>,
    write: String,
}

struct RuntimeBuilder {
    args: Vec<String>,
    envs: HashMap<String, String>,
    overlays: Vec<Overlay>,
    stages: Vec<String>,
    loop_hook: Fn<Result<()>>,
    end_hook: Fn<Result<()>>,
}

impl RuntimeBuilder {

}
