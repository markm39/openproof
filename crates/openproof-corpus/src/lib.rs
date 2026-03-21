mod ingest;
mod manifest;
mod packages;
mod search;

pub use ingest::*;
pub use manifest::*;
pub use packages::*;
pub use search::*;

use openproof_cloud::CloudCorpusClient;
use openproof_protocol::{
    CloudCorpusAuthContext, CloudCorpusSearchHit, CloudCorpusUploadItem, IngestLibrarySeedResult,
    ShareMode,
};
use openproof_store::AppStore;
use std::path::PathBuf;

/// Orchestrates corpus operations across the store, cloud, and lean toolchain.
pub struct CorpusManager {
    pub cloud_client: CloudCorpusClient,
    store: AppStore,
    lean_project_dir: PathBuf,
}

impl CorpusManager {
    pub fn new(
        store: AppStore,
        cloud_client: CloudCorpusClient,
        lean_project_dir: PathBuf,
    ) -> Self {
        Self {
            cloud_client,
            store,
            lean_project_dir,
        }
    }

    pub fn remote_url(&self) -> Option<String> {
        self.cloud_client.base_url()
    }

    pub fn describe_remote(&self) -> String {
        self.cloud_client.describe()
    }

    pub fn remote_cache_key(
        &self,
        share_mode: ShareMode,
        auth: Option<&CloudCorpusAuthContext>,
    ) -> Option<String> {
        self.cloud_client.cache_key(share_mode, auth)
    }

    /// Run the full lake-based library seed ingestion pipeline.
    pub async fn ingest_library_seed(
        &self,
        force: bool,
    ) -> anyhow::Result<IngestLibrarySeedResult> {
        ingest::run_library_seed_ingestion(&self.store, &self.lean_project_dir, force).await
    }

    /// Ingest only if the environment fingerprint has changed.
    pub async fn ensure_library_seed_ingested(&self) -> anyhow::Result<IngestLibrarySeedResult> {
        self.ingest_library_seed(false).await
    }

    /// Search the shared corpus (local cache + remote with fallback).
    pub async fn search_shared_corpus(
        &self,
        query: &str,
        limit: usize,
        share_mode: ShareMode,
        auth: Option<&CloudCorpusAuthContext>,
        include_community_overlay: bool,
    ) -> anyhow::Result<Vec<CloudCorpusSearchHit>> {
        search::search_shared_corpus(
            &self.store,
            &self.cloud_client,
            query,
            limit,
            share_mode,
            auth,
            include_community_overlay,
        )
        .await
    }

    /// Drain pending sync queue jobs to the remote corpus.
    pub async fn drain_sync_queue(
        &self,
        share_mode: ShareMode,
        sync_enabled: bool,
        auth: Option<&CloudCorpusAuthContext>,
    ) -> anyhow::Result<DrainSyncResult> {
        search::drain_sync_queue(
            &self.store,
            &self.cloud_client,
            share_mode,
            sync_enabled,
            auth,
        )
        .await
    }
}

#[derive(Debug, Clone, Default)]
pub struct DrainSyncResult {
    pub sent: usize,
    pub failed: usize,
    pub skipped: bool,
}
