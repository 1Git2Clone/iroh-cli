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
    fn run_checks(&self) -> anyhow::Result<()> {
        if self.recieve && self.ticket.as_ref().is_none_or(|t| t.is_empty()) {
            return Err(anyhow!("A non-empty ticket must be specified."));
        }

        if self.recieve && self.path.exists() {
            return Err(anyhow!("The recieving path can't be an existing one."));
        }

        Ok(())
    }

    pub fn new_cli() -> anyhow::Result<Self> {
        let args = Args::parse();

        args.run_checks()?;

        Ok(args)
    }

    pub fn new(
        send: bool,
        recieve: bool,
        ticket: Option<String>,
        path: clio::ClioPath,
    ) -> anyhow::Result<Self> {
        let args = Args {
            send,
            recieve,
            ticket,
            path,
        };

        args.run_checks()?;

        Ok(args)
    }
}
