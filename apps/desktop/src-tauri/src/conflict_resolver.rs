use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResolutionStrategy {
    ServerWins,
    ClientWins,
    ManualMerge,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictPayload {
    pub conflict_type: String,
    pub entity_type: String,
    pub entity_id: String,
    pub server_version: serde_json::Value,
    pub client_version: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedConflict {
    pub entity_id: String,
    pub entity_type: String,
    pub strategy_used: ResolutionStrategy,
    pub final_data: serde_json::Value,
    pub resolved_at: String,
}

pub fn resolve_conflict(
    conflict: &ConflictPayload,
    strategy: ResolutionStrategy,
) -> ResolvedConflict {
    let now = chrono::Utc::now().to_rfc3339();

    let final_data = match strategy {
        ResolutionStrategy::ServerWins => conflict.server_version.clone(),
        ResolutionStrategy::ClientWins => conflict.client_version.clone(),
        ResolutionStrategy::ManualMerge => {
            let mut merged = conflict.server_version.clone();
            if let (Some(server_obj), Some(client_obj)) = (
                merged.as_object_mut(),
                conflict.client_version.as_object(),
            ) {
                for (k, v) in client_obj {
                    if !server_obj.contains_key(k) {
                        server_obj.insert(k.clone(), v.clone());
                    }
                }
            }
            merged
        }
    };

    ResolvedConflict {
        entity_id: conflict.entity_id.clone(),
        entity_type: conflict.entity_type.clone(),
        strategy_used: strategy,
        final_data,
        resolved_at: now,
    }
}
