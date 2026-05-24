use crate::cdn::CdnClient;
use crate::cloudflare::CloudflareClient;
use lighty_events::{AppEvent, EventSink};
use std::sync::Arc;
use tokio::runtime::Handle;

// Listens for CdnPurgeRequested / CloudflarePurgeRequested and dispatches the
// HTTP work to the surrounding tokio runtime. The bus stays synchronous; the
// network calls run in background tasks.
pub struct CdnEventSink {
    cdn: Option<Arc<CdnClient>>,
    cloudflare: Option<Arc<CloudflareClient>>,
    runtime: Handle,
}

impl CdnEventSink {
    pub fn new(cdn: Option<Arc<CdnClient>>, cloudflare: Option<Arc<CloudflareClient>>) -> Self {
        Self {
            cdn,
            cloudflare,
            runtime: Handle::current(),
        }
    }
}

impl EventSink for CdnEventSink {
    fn handle(&self, event: &AppEvent) {
        match event {
            AppEvent::CdnPurgeRequested { server, urls } => {
                let Some(cdn) = self.cdn.clone() else {
                    return;
                };
                let server = server.clone();
                let urls = urls.clone();
                self.runtime.spawn(async move {
                    if let Err(error) = cdn.purge_files(urls).await {
                        tracing::warn!("CDN purge failed for {}: {}", server, error);
                    }
                });
            }
            AppEvent::CloudflarePurgeRequested { server } => {
                let Some(cloudflare) = self.cloudflare.clone() else {
                    return;
                };
                let server = server.clone();
                self.runtime.spawn(async move {
                    if let Err(error) = cloudflare.purge_cache(&server).await {
                        tracing::warn!("Cloudflare purge failed for {}: {}", server, error);
                    }
                });
            }
            _ => {}
        }
    }
}
