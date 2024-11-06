pub mod node;
pub mod user;

use crate::node::node::Node;
use crate::user::user::User;


use tracing::{debug, info};

fn main() {

    init_tracing();

    let node = Node::new();
    let user = User::new();
    info!("node id: {}", node.id);
    info!("user id: {}", user.id);

}

pub fn init_tracing() {
    use tracing::level_filters::LevelFilter;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::EnvFilter;

    let env = EnvFilter::builder()
        .with_default_directive(LevelFilter::DEBUG.into())
        .with_env_var("RUST_LOG")
        .from_env_lossy();

    let fmt_layer = tracing_subscriber::fmt::layer()
        .compact()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(false)
        .with_target(false);
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(env)
        .init();
}
