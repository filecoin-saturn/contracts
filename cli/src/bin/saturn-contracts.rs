use cli::commands::Cli;
use log::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    colog::init();
    banner();

    match Cli::create() {
        Ok(cli) => match cli.run().await {
            Ok(_) => info!("success"),
            Err(e) => error!("{}", e),
        },
        Err(e) => error!("{}", e),
    }

    Ok(())
}

pub fn banner() {
    info!(
        "{}",
        format!(
            "
            _|_|_|              _|                                    
            _|          _|_|_|  _|_|_|_|  _|    _|  _|  _|_|  _|_|_|    
              _|_|    _|    _|    _|      _|    _|  _|_|      _|    _|  
                  _|  _|    _|    _|      _|    _|  _|        _|    _|  
            _|_|_|      _|_|_|      _|_|    _|_|_|  _|        _|    _|      

        -----------------------------------------------------------
        Saturn smart contracts 🪐.
        -----------------------------------------------------------
        "
        )
    );
}
