use std::future::Future;
use std::pin::Pin;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{AssetKind, DocumentAsset, ProvenanceRef};

pub type StoreFuture<'a, T, E> =
    Pin<Box<dyn Future<Output = std::result::Result<T, E>> + Send + 'a>>;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum StoreError {
    #[error("invalid store request: {0}")]
    InvalidRequest(String),
    #[error("store backend error: {0}")]
    Backend(String),
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentAssetFilter {
    pub kind: Option<AssetKind>,
    pub source_id: Option<String>,
    pub limit: Option<usize>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentAssetPage {
    pub assets: Vec<DocumentAsset>,
    pub next_cursor: Option<String>,
}

pub trait DocumentStore {
    type Error;

    fn get_asset<'a>(
        &'a self,
        asset_id: &'a str,
    ) -> StoreFuture<'a, Option<DocumentAsset>, Self::Error>;

    fn list_assets<'a>(
        &'a self,
        filter: DocumentAssetFilter,
    ) -> StoreFuture<'a, DocumentAssetPage, Self::Error>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentSearchHitKind {
    Asset,
    Card,
    Section,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentSearchRequest {
    pub query: String,
    pub kind: Option<DocumentSearchHitKind>,
    pub source_id: Option<String>,
    pub limit: Option<usize>,
}

impl DocumentSearchRequest {
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            kind: None,
            source_id: None,
            limit: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentSearchHit {
    pub hit_id: String,
    pub kind: DocumentSearchHitKind,
    pub asset_id: String,
    pub section_id: Option<String>,
    pub title: Option<String>,
    pub snippet: Option<String>,
    pub score: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentSearchResult {
    pub hits: Vec<DocumentSearchHit>,
    pub total: Option<usize>,
}

pub trait DocumentSearchStore {
    type Error;

    fn search_documents<'a>(
        &'a self,
        request: DocumentSearchRequest,
    ) -> StoreFuture<'a, DocumentSearchResult, Self::Error>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryWriteRequest {
    pub library: String,
    pub memory_type: String,
    pub title: String,
    pub body: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub links: Vec<String>,
    #[serde(default)]
    pub source_refs: Vec<String>,
    #[serde(default)]
    pub provenance_refs: Vec<ProvenanceRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryWriteResult {
    pub memory_id: String,
    pub content_hash: Option<String>,
    pub provenance_event_id: Option<String>,
}

pub trait MemoryWriteStore {
    type Error;

    fn write_memory<'a>(
        &'a self,
        request: MemoryWriteRequest,
    ) -> StoreFuture<'a, MemoryWriteResult, Self::Error>;
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeConfig {
    pub profile_name: Option<String>,
    pub schema_version: Option<String>,
    #[serde(default)]
    pub capabilities: Vec<String>,
}

pub trait RuntimeConfigProvider {
    type Error;

    fn runtime_config<'a>(&'a self) -> StoreFuture<'a, RuntimeConfig, Self::Error>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextAssemblyRequest {
    pub query: String,
    pub source_id: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextItem {
    pub item_id: String,
    pub kind: DocumentSearchHitKind,
    pub asset_id: String,
    pub section_id: Option<String>,
    pub title: Option<String>,
    pub snippet: Option<String>,
    pub content_hash: Option<String>,
    pub score: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextAssembly {
    pub query: String,
    pub items: Vec<ContextItem>,
}

pub trait ContextAssembler {
    type Error;

    fn assemble_context<'a>(
        &'a self,
        request: ContextAssemblyRequest,
    ) -> StoreFuture<'a, ContextAssembly, Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::task::{Context, Poll, Wake, Waker};

    struct NoopWaker;

    impl Wake for NoopWaker {
        fn wake(self: Arc<Self>) {}
    }

    fn block_ready<T, E>(mut future: StoreFuture<'_, T, E>) -> std::result::Result<T, E> {
        let waker = Waker::from(Arc::new(NoopWaker));
        let mut context = Context::from_waker(&waker);
        match future.as_mut().poll(&mut context) {
            Poll::Ready(value) => value,
            Poll::Pending => panic!("test future unexpectedly pending"),
        }
    }

    #[derive(Clone)]
    struct InMemoryStore {
        asset: DocumentAsset,
    }

    impl DocumentStore for InMemoryStore {
        type Error = StoreError;

        fn get_asset<'a>(
            &'a self,
            asset_id: &'a str,
        ) -> StoreFuture<'a, Option<DocumentAsset>, Self::Error> {
            Box::pin(
                async move { Ok((self.asset.asset_id == asset_id).then(|| self.asset.clone())) },
            )
        }

        fn list_assets<'a>(
            &'a self,
            _filter: DocumentAssetFilter,
        ) -> StoreFuture<'a, DocumentAssetPage, Self::Error> {
            Box::pin(async move {
                Ok(DocumentAssetPage {
                    assets: vec![self.asset.clone()],
                    next_cursor: None,
                })
            })
        }
    }

    impl DocumentSearchStore for InMemoryStore {
        type Error = StoreError;

        fn search_documents<'a>(
            &'a self,
            request: DocumentSearchRequest,
        ) -> StoreFuture<'a, DocumentSearchResult, Self::Error> {
            Box::pin(async move {
                Ok(DocumentSearchResult {
                    hits: vec![DocumentSearchHit {
                        hit_id: format!("hit:{}", request.query),
                        kind: DocumentSearchHitKind::Asset,
                        asset_id: self.asset.asset_id.clone(),
                        section_id: None,
                        title: self.asset.source_label.clone(),
                        snippet: None,
                        score: Some(1.0),
                    }],
                    total: Some(1),
                })
            })
        }
    }

    impl MemoryWriteStore for InMemoryStore {
        type Error = StoreError;

        fn write_memory<'a>(
            &'a self,
            request: MemoryWriteRequest,
        ) -> StoreFuture<'a, MemoryWriteResult, Self::Error> {
            Box::pin(async move {
                Ok(MemoryWriteResult {
                    memory_id: format!("memory:{}", request.title),
                    content_hash: Some(self.asset.content_hash.clone()),
                    provenance_event_id: None,
                })
            })
        }
    }

    impl RuntimeConfigProvider for InMemoryStore {
        type Error = StoreError;

        fn runtime_config<'a>(&'a self) -> StoreFuture<'a, RuntimeConfig, Self::Error> {
            Box::pin(async move {
                Ok(RuntimeConfig {
                    profile_name: Some("example".into()),
                    schema_version: Some("1".into()),
                    capabilities: vec!["document_search".into(), "memory_write".into()],
                })
            })
        }
    }

    impl ContextAssembler for InMemoryStore {
        type Error = StoreError;

        fn assemble_context<'a>(
            &'a self,
            request: ContextAssemblyRequest,
        ) -> StoreFuture<'a, ContextAssembly, Self::Error> {
            Box::pin(async move {
                Ok(ContextAssembly {
                    query: request.query,
                    items: vec![ContextItem {
                        item_id: "ctx:asset_example_001".into(),
                        kind: DocumentSearchHitKind::Asset,
                        asset_id: self.asset.asset_id.clone(),
                        section_id: None,
                        title: self.asset.source_label.clone(),
                        snippet: None,
                        content_hash: Some(self.asset.content_hash.clone()),
                        score: Some(1.0),
                    }],
                })
            })
        }
    }

    #[test]
    fn document_store_trait_supports_async_backends_without_runtime_dependency() {
        let store = InMemoryStore {
            asset: DocumentAsset {
                asset_id: "asset_example_001".into(),
                kind: AssetKind::Markdown,
                source_label: Some("Policy Memo A".into()),
                content_hash:
                    "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
            },
        };

        let asset = block_ready(store.get_asset("asset_example_001"))
            .expect("store read")
            .expect("asset exists");
        assert_eq!(asset.source_label.as_deref(), Some("Policy Memo A"));

        let page =
            block_ready(store.list_assets(DocumentAssetFilter::default())).expect("list assets");
        assert_eq!(page.assets.len(), 1);

        let result =
            block_ready(store.search_documents(DocumentSearchRequest::new("Project Alpha")))
                .expect("search documents");
        assert_eq!(result.hits[0].kind, DocumentSearchHitKind::Asset);

        let written = block_ready(store.write_memory(MemoryWriteRequest {
            library: "tech".into(),
            memory_type: "procedure".into(),
            title: "example".into(),
            body: "[Pattern] synthetic public memory".into(),
            tags: vec!["public".into()],
            links: Vec::new(),
            source_refs: vec!["source:example".into()],
            provenance_refs: Vec::new(),
        }))
        .expect("write memory");
        assert_eq!(written.memory_id, "memory:example");

        let config = block_ready(store.runtime_config()).expect("runtime config");
        assert!(config.capabilities.contains(&"memory_write".into()));

        let context = block_ready(store.assemble_context(ContextAssemblyRequest {
            query: "Project Alpha".into(),
            source_id: None,
            limit: Some(3),
        }))
        .expect("assemble context");
        assert_eq!(context.items[0].asset_id, "asset_example_001");
    }
}
