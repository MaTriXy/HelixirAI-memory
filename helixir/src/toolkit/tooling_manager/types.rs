use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::toolkit::mind_toolbox::chunking::ChunkingError;
use crate::toolkit::mind_toolbox::entity::EntityError;
use crate::toolkit::mind_toolbox::ontology::OntologyError;
use crate::toolkit::mind_toolbox::reasoning::ReasoningError;
use crate::toolkit::mind_toolbox::search::SearchError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddMemoryResult {
    pub added: Vec<String>,
    pub updated: Vec<String>,
    pub deleted: Vec<String>,
    pub skipped: usize,
    pub entities_extracted: usize,
    pub reasoning_relations_created: usize,
    pub chunks_created: usize,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMemoryResult {
    pub memory_id: String,
    pub content: String,
    pub score: f64,
    pub method: String,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningChainSearchResult {
    pub chains: Vec<ToolingReasoningChain>,
    pub total_memories: usize,
    pub deepest_chain: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolingReasoningChain {
    pub seed: SearchMemoryResult,
    pub nodes: Vec<ChainNode>,
    pub chain_type: String,
    pub reasoning_trail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainNode {
    pub memory_id: String,
    pub content: String,
    pub relation: String,
    pub depth: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum ToolingError {
    #[error("Embedding failed: {0}")]
    Embedding(String),
    #[error("Extraction failed: {0}")]
    Extraction(String),
    #[error("Chunking failed: {0}")]
    Chunking(#[from] ChunkingError),
    #[error("Entity operation failed: {0}")]
    Entity(#[from] EntityError),
    #[error("Ontology operation failed: {0}")]
    Ontology(#[from] OntologyError),
    #[error("Reasoning operation failed: {0}")]
    Reasoning(#[from] ReasoningError),
    #[error("Memory operation failed: {0}")]
    Memory(String),
    #[error("Search failed: {0}")]
    Search(#[from] SearchError),
    #[error("Database error: {0}")]
    Database(String),
}
