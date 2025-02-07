use crate::prelude::*;

pub struct IrohData {
    pub local_pool: LocalPool,
    pub blobstore: Blobs<Store>,
    pub router: Router,
}

impl IrohData {
    /// Make [`IrohData`] based on iroh's n0 DNS discovery service.
    pub async fn new() -> anyhow::Result<Self> {
        let endpoint = Endpoint::builder().discovery_n0().bind().await?;
        let local_pool = LocalPool::default();
        let blobstore = Blobs::memory().build(&endpoint);
        let router = Router::builder(endpoint)
            .accept(iroh_blobs::ALPN, blobstore.clone())
            .spawn()
            .await?;

        Ok(Self {
            local_pool,
            blobstore,
            router,
        })
    }

    pub async fn shutdown(self) -> anyhow::Result<()> {
        self.router.shutdown().await?;
        self.local_pool.shutdown().await;

        Ok(())
    }
}
