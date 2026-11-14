mod inline;
mod promote;

use rustc_session::Session;
use std::env;
use std::sync::LazyLock;

/// Highest cost (instruction count) a function may have to be inlined
static INLINE_COST: LazyLock<u32> = LazyLock::new(|| match env::var("TPDE_INLINE_COST") {
    Ok(val) => val
        .parse::<u32>()
        .expect("TPDE_INLINE_COST has to be a number"),
    Err(_) => 50,
});

/// The optimizations that run on a module, enabled through `-Cllvm-args`:
/// `-Cllvm-args=inline=yes` and `-Cllvm-args=promote=yes`
#[derive(Debug, Copy, Clone)]
pub struct OptConfig {
    /// Highest cost a function is inlined at, `None` only inlines `#[inline(always)]` functions
    pub inline_cost: Option<u32>,
    /// Promote allocas into slots, the builder has to collect the access infos for it
    pub promote: bool,
}

impl OptConfig {
    pub fn new(sess: &Session) -> Self {
        let mut config = Self {
            inline_cost: None,
            promote: false,
        };
        for arg in &sess.opts.cg.llvm_args {
            let Some((key, value)) = arg.split_once('=') else {
                sess.dcx().fatal(format!(
                    "invalid backend option `{arg}`, expected `key=value`"
                ));
            };
            let enabled = match value {
                "yes" | "y" | "on" | "true" => true,
                "no" | "n" | "off" | "false" => false,
                _ => sess.dcx().fatal(format!(
                    "invalid value `{value}` for backend option `{key}`, expected yes/no"
                )),
            };
            match key {
                "inline" => config.inline_cost = enabled.then(|| *INLINE_COST),
                "promote" => config.promote = enabled,
                _ => sess.dcx().fatal(format!("unknown backend option `{key}`")),
            }
        }
        config
    }
}
