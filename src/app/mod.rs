use std::any::{Any, TypeId};

use crate::prelude::*;

#[derive(Debug)]
pub struct App<Out: Write + Any> {
    pub args: Args,
    pub iroh_data: IrohData,
    pub output_stream: Out,
}

impl<Out: Write + Any> App<Out> {
    pub const APP_NAME: &'static str = env!("CARGO_PKG_NAME");

    /// Check if the output stream is any of the standard terminal ones:
    /// - [`std::io::StdoutLock`]
    /// - [`std::io::Stdout`]
    /// - [`std::io::StdinLock`]
    /// - [`std::io::Stdin`]
    /// - [`std::io::StderrLock`]
    /// - [`std::io::Stderr`]
    fn is_output_tty(&self) -> bool {
        self.output_stream.type_id() == TypeId::of::<std::io::StdoutLock>()
            || self.output_stream.type_id() == TypeId::of::<std::io::Stdout>()
            || self.output_stream.type_id() == TypeId::of::<std::io::StdinLock>()
            || self.output_stream.type_id() == TypeId::of::<std::io::Stdin>()
            || self.output_stream.type_id() == TypeId::of::<std::io::StderrLock>()
            || self.output_stream.type_id() == TypeId::of::<std::io::Stderr>()
    }

    pub async fn init(output_stream: Out) -> anyhow::Result<Self> {
        let (args, iroh_data) = (Args::new_cli()?, IrohData::new().await?);
        Ok(Self {
            args,
            iroh_data,
            output_stream,
        })
    }

    #[allow(dead_code, reason = "Potential external usage")]
    pub async fn from_args(args: Args, output_stream: Out) -> anyhow::Result<Self> {
        let iroh_data = IrohData::new().await?;

        Ok(Self {
            args,
            iroh_data,
            output_stream,
        })
    }

    pub async fn blob_from_path(&self) -> anyhow::Result<AddOutcome> {
        self.iroh_data
            .blobstore
            .client()
            .add_from_path(
                self.args.path.canonicalize()?,
                true,
                SetTagOption::Auto,
                WrapOption::NoWrap,
            )
            .await?
            .finish()
            .await
    }

    pub async fn process_all(&mut self) -> anyhow::Result<()> {
        self.process_recieve().await?;
        self.process_send().await?;

        Ok(())
    }

    pub async fn process_recieve(&mut self) -> anyhow::Result<()> {
        if !self.args.recieve {
            return Ok(());
        }

        writeln!(self.output_stream, "Strating download...")?;

        let client = self.iroh_data.blobstore.client();

        let ticket = Arc::new(BlobTicket::from_str(
            self.args.ticket.as_deref().unwrap_or(""),
        )?);

        timeout(Duration::from_secs(5), {
            let tic = Arc::clone(&ticket);

            async move {
                client
                    .download(tic.hash(), tic.node_addr().clone())
                    .await
                    .unwrap()
                    .finish()
                    .await
                    .unwrap()
            }
        })
        .await
        .or({
            let err = format!(
                "{}\n{} {}",
                "Ticket connection timed out.",
                "Make sure the sender hasn't closed their connection and that you're using",
                "the right ticket.",
            );
            if !self.is_output_tty() {
                writeln!(self.output_stream, "{}", err)?;
            }
            Err(anyhow!(err))
        })?;

        let mut file = self.args.path.clone().create()?;
        let mut reader = client.read_at(ticket.hash(), 0, ReadAtLen::All).await?;
        file.write_all(&reader.read_to_bytes().await?)?;

        writeln!(self.output_stream, "Download finished!")?;

        Ok(())
    }

    pub async fn process_send(&mut self) -> anyhow::Result<()> {
        if !self.args.send {
            return Ok(());
        }

        let node_id = self.iroh_data.router.endpoint().node_id();
        let blob = self.blob_from_path().await?;

        self.args.ticket =
            Some(BlobTicket::new(node_id.into(), blob.hash, blob.format)?.to_string());

        if !self.is_output_tty() {
            writeln!(
                self.output_stream,
                "{}",
                self.args.ticket.as_deref().unwrap_or("")
            )?;

            return Ok(());
        }

        writeln!(self.output_stream, "Opened connection!")?;
        writeln!(
            self.output_stream,
            "You can now recieve the data by running the app like so:"
        )?;
        writeln!(
            self.output_stream,
            "    {} --recieve --ticket {} {}",
            Self::APP_NAME,
            self.args.ticket.as_deref().unwrap_or(""),
            self.args.path.display()
        )?;

        tokio::signal::ctrl_c().await?;

        Ok(())
    }

    pub async fn shutdown(self) -> anyhow::Result<()> {
        self.iroh_data.shutdown().await?;

        // Explicitly drop the other parts of [`Self`].
        drop(self.args);

        Ok(())
    }
}
