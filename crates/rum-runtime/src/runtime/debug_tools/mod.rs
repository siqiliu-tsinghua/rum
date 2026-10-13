use std::time::{Duration, Instant, SystemTime};

use super::*;

mod basic;
mod evaluation_data;
mod message_switch;
mod monitor;
mod properties;
mod reap;
mod reap_state;
pub(in crate::runtime) mod resources;
mod shared;
pub(in crate::runtime) mod test_report;
mod timing;

mod trace;

use self::properties::association_property_by_name;
use self::shared::{
    debug_bool_expr, elapsed_duration_expr, render_echo_fragment, runtime_message_emit_expr,
};
