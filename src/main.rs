pub mod api;
pub mod config;
pub mod litestream;
pub mod media_backup;
pub mod server;
pub mod test;
pub mod utils;

use server::PihkaServer;
use test::TestRunner;

fn main() {
    // TODO: print commit ID to logs if build directory was clean
    let config = config::get_config().unwrap();

    let runtime = tokio::runtime::Runtime::new().unwrap();

    if let Some(test_mode_config) = config.test_mode() {
        runtime.block_on(async { TestRunner::new(config, test_mode_config).run().await })
    } else {
        runtime.block_on(async { PihkaServer::new(config).run().await })
    }
}
