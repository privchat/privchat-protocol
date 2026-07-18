//! Generic entity invalidation control-plane protocol.

use crate::codec::FlatBufferMessage;
use crate::error::ProtocolError;
use crate::fb;
use flatbuffers::FlatBufferBuilder;

pub const ENTITY_INVALIDATION_SCHEMA_V1: u32 = 1;
pub const ENTITY_INVALIDATION_PUSH_TOPIC_V1: &str = "entity.invalidation.v1";
pub const ENTITY_INVALIDATION_MAX_ITEMS_V1: usize = 128;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum EntityMutationHint {
    #[default]
    Unknown,
    Upsert,
    Delete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityInvalidation {
    pub entity_type: String,
    pub entity_id: Option<String>,
    pub scope: Option<String>,
    pub target_version: u64,
    pub mutation_hint: EntityMutationHint,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityInvalidationBatch {
    pub schema_version: u32,
    pub notification_id: u64,
    pub items: Vec<EntityInvalidation>,
    pub committed_at_ms: i64,
}

impl EntityInvalidationBatch {
    pub fn new_v1(
        notification_id: u64,
        items: Vec<EntityInvalidation>,
        committed_at_ms: i64,
    ) -> Result<Self, ProtocolError> {
        if items.is_empty() || items.len() > ENTITY_INVALIDATION_MAX_ITEMS_V1 {
            return Err(ProtocolError::InvalidValue(format!(
                "entity invalidation item count must be 1..={ENTITY_INVALIDATION_MAX_ITEMS_V1}"
            )));
        }
        if items.iter().any(|item| item.entity_type.trim().is_empty()) {
            return Err(ProtocolError::InvalidValue(
                "entity invalidation entity_type must not be blank".to_string(),
            ));
        }
        Ok(Self {
            schema_version: ENTITY_INVALIDATION_SCHEMA_V1,
            notification_id,
            items,
            committed_at_ms,
        })
    }
}

impl FlatBufferMessage for EntityInvalidationBatch {
    fn encode_fb_into(&self, builder: &mut FlatBufferBuilder<'_>) -> Result<(), ProtocolError> {
        if self.items.is_empty() || self.items.len() > ENTITY_INVALIDATION_MAX_ITEMS_V1 {
            return Err(ProtocolError::InvalidValue(format!(
                "entity invalidation item count must be 1..={ENTITY_INVALIDATION_MAX_ITEMS_V1}"
            )));
        }

        let mut offsets = Vec::with_capacity(self.items.len());
        for item in &self.items {
            if item.entity_type.trim().is_empty() {
                return Err(ProtocolError::InvalidValue(
                    "entity invalidation entity_type must not be blank".to_string(),
                ));
            }
            let entity_type = builder.create_string(&item.entity_type);
            let entity_id = item
                .entity_id
                .as_deref()
                .map(|value| builder.create_string(value));
            let scope = item
                .scope
                .as_deref()
                .map(|value| builder.create_string(value));
            let mutation_hint = match item.mutation_hint {
                EntityMutationHint::Unknown => fb::EntityMutationHint::Unknown,
                EntityMutationHint::Upsert => fb::EntityMutationHint::Upsert,
                EntityMutationHint::Delete => fb::EntityMutationHint::Delete,
            };
            offsets.push(fb::EntityInvalidation::create(
                builder,
                &fb::EntityInvalidationArgs {
                    entity_type: Some(entity_type),
                    entity_id,
                    scope,
                    target_version: item.target_version,
                    mutation_hint,
                },
            ));
        }
        let items = builder.create_vector(&offsets);
        let root = fb::EntityInvalidationBatch::create(
            builder,
            &fb::EntityInvalidationBatchArgs {
                schema_version: self.schema_version,
                notification_id: self.notification_id,
                items: Some(items),
                committed_at_ms: self.committed_at_ms,
            },
        );
        builder.finish(root, None);
        Ok(())
    }

    fn decode_fb(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let view = flatbuffers::root::<fb::EntityInvalidationBatch>(bytes)?;
        let items_view = view.items();
        if items_view.is_empty() || items_view.len() > ENTITY_INVALIDATION_MAX_ITEMS_V1 {
            return Err(ProtocolError::InvalidValue(format!(
                "entity invalidation item count must be 1..={ENTITY_INVALIDATION_MAX_ITEMS_V1}"
            )));
        }
        let mut items = Vec::with_capacity(items_view.len());
        for item in items_view {
            let entity_type = item.entity_type().to_string();
            if entity_type.trim().is_empty() {
                return Err(ProtocolError::InvalidValue(
                    "entity invalidation entity_type must not be blank".to_string(),
                ));
            }
            let mutation_hint = match item.mutation_hint() {
                fb::EntityMutationHint::Unknown => EntityMutationHint::Unknown,
                fb::EntityMutationHint::Upsert => EntityMutationHint::Upsert,
                fb::EntityMutationHint::Delete => EntityMutationHint::Delete,
                _ => EntityMutationHint::Unknown,
            };
            items.push(EntityInvalidation {
                entity_type,
                entity_id: item.entity_id().map(str::to_string),
                scope: item.scope().map(str::to_string),
                target_version: item.target_version(),
                mutation_hint,
            });
        }
        Ok(Self {
            schema_version: view.schema_version(),
            notification_id: view.notification_id(),
            items,
            committed_at_ms: view.committed_at_ms(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_preserves_large_ids_scope_and_tombstone_hint() {
        let batch = EntityInvalidationBatch::new_v1(
            9_007_199_254_740_993,
            vec![EntityInvalidation {
                entity_type: "group_member".to_string(),
                entity_id: Some("9007199254740995".to_string()),
                scope: Some("9007199254740997".to_string()),
                target_version: 9_007_199_254_740_999,
                mutation_hint: EntityMutationHint::Delete,
            }],
            1_780_000_000_123,
        )
        .expect("valid batch");

        let bytes = batch.encode_fb().expect("encode invalidation");
        let decoded = EntityInvalidationBatch::decode_fb(&bytes).expect("decode invalidation");
        assert_eq!(decoded, batch);
    }

    #[test]
    fn rejects_empty_or_oversized_batches() {
        assert!(EntityInvalidationBatch::new_v1(1, vec![], 1).is_err());
        let item = EntityInvalidation {
            entity_type: "friend".to_string(),
            entity_id: None,
            scope: None,
            target_version: 1,
            mutation_hint: EntityMutationHint::Upsert,
        };
        assert!(EntityInvalidationBatch::new_v1(
            1,
            vec![item; ENTITY_INVALIDATION_MAX_ITEMS_V1 + 1],
            1,
        )
        .is_err());
    }
}
