use crate::prelude::*;

pub struct App {
    pub args: Args,
    pub iroh_data: IrohData,
}

impl App {
    pub const APP_NAME: &'static str = env!("CARGO_PKG_NAME");

    pub async fn init() -> anyhow::Result<Self> {
        let (args, iroh_data) = (Args::new_cli()?, IrohData::new().await?);
        Ok(Self { args, iroh_data })
    }

    #[allow(dead_code, reason = "Potential external usage")]
    pub async fn from_args(
        send: bool,
        receive: bool,
        ticket: Option<String>,
        path: clio::ClioPath,
    ) -> anyhow::Result<Self> {
        let (args, iroh_data) = (
            Args::new(send, receive, ticket, path)?,
            IrohData::new().await?,
        );

        Ok(Self { args, iroh_data })
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

    pub async fn process_all(mut self) -> anyhow::Result<Self> {
        self = self.process_recieve().await?;
        self = self.process_send().await?;

        Ok(self)
    }

    pub async fn process_recieve(self) -> anyhow::Result<Self> {
        if !self.args.recieve {
            return Ok(self);
        }

        println!("Starting download...");

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
        .or(Err(anyhow!(
            "{}\n{}",
            "Ticket connection timed out.",
            "Make sure the sender hasn't closed their connection and that you're using the right ticket.",
        )))?;

        let mut file = self.args.path.clone().create()?;
        let mut reader = client.read_at(ticket.hash(), 0, ReadAtLen::All).await?;
        file.write_all(&reader.read_to_bytes().await?)?;

        println!("Download finished!");

        Ok(self)
    }

    pub async fn process_send(mut self) -> anyhow::Result<Self> {
        if !self.args.send {
            return Ok(self);
        }

        let node_id = self.iroh_data.router.endpoint().node_id();
        let blob = self.blob_from_path().await?;

        self.args.ticket =
            Some(BlobTicket::new(node_id.into(), blob.hash, blob.format)?.to_string());

        println!("Opened connection!");
        println!("You can now recieve the data by running the app like so:");
        println!(
            "    {} --recieve --ticket {} {}",
            Self::APP_NAME,
            self.args.ticket.as_deref().unwrap_or(""),
            self.args.path.display()
        );

        tokio::signal::ctrl_c().await?;

        Ok(self)
    }

    pub async fn shutdown(self) -> anyhow::Result<()> {
        self.iroh_data.shutdown().await?;

        // Explicitly drop the other parts of [`Self`].
        drop(self.args);

        Ok(())
    }
}
