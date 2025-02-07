mod app;
mod prelude;
mod utils;

use crate::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    App::init().await?.process_all().await?.shutdown().await
}
