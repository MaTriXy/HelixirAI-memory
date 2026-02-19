

use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

use super::providers::base::{LlmProvider, LlmProviderError};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionResult {
    
    pub memories: Vec<ExtractedMemory>,
    
    pub entities: Vec<ExtractedEntity>,
    
    pub relations: Vec<ExtractedRelation>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedMemory {
    
    pub text: String,
    
    pub memory_type: String,
    
    pub certainty: i32,
    
    pub importance: i32,
    
    pub entities: Vec<String>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedEntity {
    
    pub id: String,
    
    pub name: String,
    
    #[serde(rename = "type")]
    pub entity_type: String,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedRelation {
    #[serde(default)]
    pub from_memory_index: Option<usize>,
    #[serde(default)]
    pub to_memory_index: Option<usize>,
    #[serde(default)]
    pub from_memory_content: String,
    #[serde(default)]
    pub to_memory_content: String,
    
    pub relation_type: String,
    
    #[serde(default = "default_strength")]
    pub strength: i32,
    
    #[serde(default = "default_confidence")]
    pub confidence: i32,
    
    #[serde(default)]
    pub explanation: String,
}

fn default_strength() -> i32 { 80 }
fn default_confidence() -> i32 { 80 }


pub struct LlmExtractor<P: LlmProvider> {
    provider: P,
}

impl<P: LlmProvider> LlmExtractor<P> {
    
    #[must_use]
    pub fn new(provider: P) -> Self {
        Self { provider }
    }

    
    pub async fn extract(
        &self,
        text: &str,
        user_id: &str,
        extract_entities: bool,
        extract_relations: bool,
    ) -> Result<ExtractionResult, LlmProviderError> {
        let preview: String = text.chars().take(50).collect();
        info!(
            "Extracting memories from text: {}... (user={})",
            preview,
            user_id
        );

        let system_prompt = self.build_system_prompt(extract_entities, extract_relations);
        let user_prompt = format!("Extract information from this text:\n\n{}", text);

        let (response, _metadata) = self
            .provider
            .generate(&system_prompt, &user_prompt, Some("json_object"))
            .await?;

        
        match serde_json::from_str::<ExtractionResult>(&response) {
            Ok(result) => {
                debug!(
                    "Extracted {} memories, {} entities, {} relations",
                    result.memories.len(),
                    result.entities.len(),
                    result.relations.len()
                );
                Ok(result)
            }
            Err(e) => {
                warn!("Failed to parse extraction result: {}", e);
                
                Ok(ExtractionResult {
                    memories: Vec::new(),
                    entities: Vec::new(),
                    relations: Vec::new(),
                })
            }
        }
    }

    
    fn build_system_prompt(&self, extract_entities: bool, extract_relations: bool) -> String {
        let mut prompt = String::from(
            r#"You are a memory extraction system. Analyze the text and extract structured information.

Output JSON with this structure:
{
  "memories": [
    {
      "text": "atomic fact or preference",
      "memory_type": "fact|preference|goal|opinion|experience",
      "certainty": 80,
      "importance": 50,
      "entities": ["entity_id1", "entity_id2"]
    }
  ]"#,
        );

        if extract_entities {
            prompt.push_str(
                r#",
  "entities": [
    {
      "id": "unique_id",
      "name": "Entity Name",
      "type": "person|organization|location|concept|system"
    }
  ]"#,
            );
        } else {
            prompt.push_str(r#",
  "entities": []"#);
        }

        if extract_relations {
            prompt.push_str(
                r#",
  "relations": [
    {
      "from_memory_index": 0,
      "to_memory_index": 1,
      "relation_type": "IMPLIES|BECAUSE|CONTRADICTS|SUPPORTS",
      "strength": 80,
      "confidence": 80,
      "explanation": "Why this relation exists"
    }
  ]"#,
            );
            prompt.push_str(r#"

Relations use INDICES into the memories array (0-based).
Relation types:
- IMPLIES: memory A logically leads to or suggests memory B
- BECAUSE: memory A is the reason/cause for memory B
- CONTRADICTS: memory A conflicts with memory B
- SUPPORTS: memory A provides evidence for memory B

Always look for causal and logical connections between extracted memories. If the text contains cause-effect, reasoning, or contradictions, you MUST extract them as relations."#);
        } else {
            prompt.push_str(r#",
  "relations": []"#);
        }

        prompt.push_str("\n}\n\nExtract atomic, standalone facts. Each memory should be self-contained.");

        prompt
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extraction_result_serialization() {
        let result = ExtractionResult {
            memories: vec![ExtractedMemory {
                text: "User prefers Rust".to_string(),
                memory_type: "preference".to_string(),
                certainty: 90,
                importance: 70,
                entities: vec!["rust".to_string()],
            }],
            entities: vec![ExtractedEntity {
                id: "rust".to_string(),
                name: "Rust".to_string(),
                entity_type: "concept".to_string(),
            }],
            relations: vec![],
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("preference"));
    }
}
