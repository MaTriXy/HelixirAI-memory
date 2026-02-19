use serde::Serialize;
use tracing::debug;

use super::ToolingManager;

pub(crate) fn safe_truncate(s: &str, max_chars: usize) -> String {
    s.chars().take(max_chars).collect()
}

impl ToolingManager {
    pub(crate) async fn get_memory_type(&self, memory_id: &str) -> Option<String> {
        #[derive(serde::Deserialize)]
        struct GetMemoryResponse {
            #[serde(default)]
            memory: Option<MemoryFields>,
        }

        #[derive(serde::Deserialize)]
        struct MemoryFields {
            #[serde(default)]
            memory_type: String,
        }

        self.db
            .execute_query::<GetMemoryResponse, _>(
                "getMemory",
                &serde_json::json!({"memory_id": memory_id}),
            )
            .await
            .ok()
            .and_then(|r| r.memory)
            .and_then(|m| if m.memory_type.is_empty() { None } else { Some(m.memory_type) })
    }

    pub(crate) async fn ensure_user_exists(&self, user_id: &str) {
        #[derive(serde::Deserialize)]
        struct UserResponse {
            #[serde(default)]
            user: Option<serde_json::Value>,
        }

        let exists = self.db
            .execute_query::<UserResponse, _>(
                "getUser",
                &serde_json::json!({"user_id": user_id}),
            )
            .await
            .map(|r| r.user.is_some())
            .unwrap_or(false);

        if !exists {
            let _ = self.db
                .execute_query::<serde_json::Value, _>(
                    "addUser",
                    &serde_json::json!({"user_id": user_id, "name": user_id}),
                )
                .await;
            debug!("Created user node: {}", user_id);
        }
    }

    pub(crate) async fn link_memory_to_concept(
        &self,
        memory_id: &str,
        concept_id: &str,
        confidence: i32,
    ) -> Result<(), super::ToolingError> {
        #[derive(serde::Deserialize)]
        struct LinkResponse {
            #[serde(default)]
            link: serde_json::Value,
        }

        self.db
            .execute_query::<LinkResponse, _>(
                "linkMemoryToInstanceOf",
                &serde_json::json!({
                    "memory_id": memory_id,
                    "concept_id": concept_id,
                    "confidence": confidence as i64,
                }),
            )
            .await
            .map_err(|e| super::ToolingError::Database(e.to_string()))?;

        debug!("Linked memory {} to concept {}", memory_id, concept_id);
        Ok(())
    }

    pub(crate) async fn update_memory_internal(
        &self,
        memory_id: &str,
        new_content: &str,
        vector: &[f32],
    ) -> Result<(), super::ToolingError> {
        #[derive(Serialize)]
        struct UpdateInput {
            memory_id: String,
            content: String,
            vector: Vec<f32>,
        }

        self.db
            .execute_query::<(), _>("updateMemory", &UpdateInput {
                memory_id: memory_id.to_string(),
                content: new_content.to_string(),
                vector: vector.to_vec(),
            })
            .await
            .map_err(|e| super::ToolingError::Database(e.to_string()))?;

        debug!("Updated memory: {}", memory_id);
        Ok(())
    }
}
