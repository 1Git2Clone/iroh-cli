use crate::prelude::*;

/// A peer to peer CLI tool used to send and recieve files using iroh and iroh-blobs.
#[derive(clap::Parser, Clone, Debug)]
pub struct Args {
    /// Use to send data.
    #[clap(long, action=clap::ArgAction::SetTrue, group = "send/recv")]
    pub send: bool,

    /// Use to recieve data.
    #[clap(long, action=clap::ArgAction::SetTrue, group = "send/recv", requires = "ticket")]
    pub recieve: bool,

    /// Specify a ticket (only if you're recieving data).
    #[clap(long, value_parser, requires = "recieve")]
    pub ticket: Option<String>,

    /// Either an input or an output path based on the `--send` and `--recieve` flags.
    #[clap(value_parser)]
    pub path: clio::ClioPath,
}

impl Args {
    pub fn new() -> anyhow::Result<Self> {
        let args = Args::parse();

        if args.recieve && args.ticket.as_ref().is_none_or(|t| t.is_empty()) {
            return Err(anyhow!("A non-empty ticket must be specified."));
        }

        if args.recieve && args.path.exists() {
            return Err(anyhow!("The recieving path can't be an existing one."));
        }

        Ok(args)
    }
}
