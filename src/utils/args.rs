use crate::prelude::*;

/// A peer to peer CLI tool used to send and receive files using iroh and iroh-blobs.
#[derive(clap::Parser, PartialEq, Eq, Clone, Debug)]
pub struct Args {
    /// Use to send data.
    #[clap(long, action=clap::ArgAction::SetTrue, group = "send/recv")]
    pub send: bool,

    /// Use to receive data.
    #[clap(long, action=clap::ArgAction::SetTrue, group = "send/recv", requires = "ticket")]
    pub receive: bool,

    /// Specify a ticket (only if you're recieving data).
    #[clap(long, value_parser, requires = "receive")]
    pub ticket: Option<String>,

    /// Either an input or an output path based on the `--send` and `--receive` flags.
    #[clap(value_parser)]
    pub path: clio::ClioPath,
}

impl Args {
    fn run_checks(&self) -> anyhow::Result<()> {
        if self.receive && self.ticket.as_ref().is_none_or(|t| t.is_empty()) {
            return Err(anyhow!("A non-empty ticket must be specified."));
        }

        if self.receive && self.path.exists() {
            return Err(anyhow!("The recieving path can't be an existing one."));
        }

        Ok(())
    }

    pub fn new_cli() -> anyhow::Result<Self> {
        let args = Args::parse();

        args.run_checks()?;

        Ok(args)
    }

    #[allow(dead_code, reason = "Potential external usage")]
    pub fn new(
        send: bool,
        receive: bool,
        ticket: Option<String>,
        path: clio::ClioPath,
    ) -> anyhow::Result<Self> {
        let args = Args {
            send,
            receive,
            ticket,
            path,
        };

        args.run_checks()?;

        Ok(args)
    }
}
