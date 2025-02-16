mod app;
mod prelude;
mod utils;

use crate::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut app = App::init(std::io::stdout().lock()).await?;
    app.process_all().await?;
    app.shutdown().await
}
