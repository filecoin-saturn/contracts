use cli::commands::Cli;
use cli::utils::banner;
use eyre::Result;
use log::error;

#[tokio::main]
async fn main() -> Result<()> {
    colog::init();
    banner();

    match Cli::create() {
        Ok(cli) => cli.run().await,
        Err(e) => error!("{}", e),
    }

    Ok(())
}
