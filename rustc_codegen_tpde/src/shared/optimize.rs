mod inline;
mod promote;

use rustc_session::config::OptLevel;
use std::env;
use std::sync::LazyLock;

/// Highest cost (instruction count) a function may have to be inlined
static INLINE_COST: LazyLock<u32> = LazyLock::new(|| match env::var("TPDE_INLINE_COST") {
    Ok(val) => val
        .parse::<u32>()
        .expect("TPDE_INLINE_COST has to be a number"),
    Err(_) => 50,
});

/// The optimizations that run on a module, selected by the optimization level
#[derive(Debug, Copy, Clone)]
pub struct OptConfig {
    /// Highest cost a function is inlined at, `None` only inlines `#[inline(always)]` functions
    pub inline_cost: Option<u32>,
    /// Promote allocas into slots, the builder has to collect the access infos for it
    pub promote: bool,
}

impl OptConfig {
    pub fn new(level: OptLevel) -> Self {
        match level {
            OptLevel::No => Self {
                inline_cost: None,
                promote: false,
            },
            OptLevel::Less => Self {
                inline_cost: Some(*INLINE_COST),
                promote: false,
            },
            OptLevel::More | OptLevel::Aggressive | OptLevel::Size | OptLevel::SizeMin => Self {
                inline_cost: Some(*INLINE_COST),
                promote: true,
            },
        }
    }
}
