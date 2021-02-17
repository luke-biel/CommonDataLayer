use juniper::{graphql_object, FieldResult};
use num_traits::ToPrimitive;
use semver::VersionReq;
use serde_json::value::RawValue;
use utils::message_types::DataRouterInsertMessage;
use uuid::Uuid;

use crate::error::Error;
use crate::schema::context::Context;
use crate::schema::utils::{get_schema, get_view};
use crate::types::data::InputMessage;
use schema_registry::types::SchemaWithDefinitions;

pub struct Mutation;

#[graphql_object(context = Context)]
impl Mutation {
    async fn add_schema(context: &Context, new: NewSchema) -> FieldResult<SchemaWithDefinitions> {
        log::debug!("add schema {:?}", new);

        let conn = context.connect_to_registry().await?;
        let new_id = conn
            .add_schema(schema_registry::types::NewSchema {
                name: new.name,
                r#type: new.schema_type,
                queue: new.topic,
                query_addr: new.query_address,
                definition: new.definition,
            })
            .await?;

        conn.get_schema(new_id).await.into()
    }

    async fn add_schema_definition(
        context: &Context,
        schema_id: Uuid,
        new_version: NewVersion,
    ) -> FieldResult<Definition> {
        log::debug!(
            "add schema definition for {:?} - {:?}",
            schema_id,
            new_version
        );

        let conn = context.connect_to_registry().await?;
        conn.add_schema_definition(
            schema_id,
            schema_registry::types::SchemaDefinition {
                version: new_version.version,
                definition: new_version.definition,
            },
        )
        .await?;

        Ok(Definition {
            definition,
            version,
        })
    }

    async fn update_schema(
        context: &Context,
        id: Uuid,
        update: UpdateSchema,
    ) -> FieldResult<Schema> {
        log::debug!("update schema for {} - {:?}", id, update);

        let mut conn = context.connect_to_registry().await?;

        let UpdateSchema {
            name,
            query_address: address,
            topic,
            schema_type,
        } = update;

        conn.update_schema_metadata(rpc::schema_registry::SchemaMetadataUpdate {
            id: id.to_string(),
            name,
            address,
            topic,
            schema_type: schema_type.and_then(|s| s.to_i32()),
        })
        .await
        .map_err(rpc::error::registry_error)?;
        get_schema(&mut conn, id).await
    }

    async fn insert_message(context: &Context, message: InputMessage) -> FieldResult<bool> {
        log::debug!(
            "inserting single message with ID {} for schema {}",
            message.object_id,
            message.schema_id
        );

        let publisher = context.connect_to_data_router().await?;
        let payload = serde_json::to_vec(&DataRouterInsertMessage {
            object_id: message.object_id,
            schema_id: message.schema_id,
            order_group_id: None,
            data: &RawValue::from_string(message.payload)?,
        })?;

        publisher
            .publish_message(
                &context.config().data_router_topic_or_queue,
                &message.object_id.to_string(),
                payload,
            )
            .await
            .map_err(Error::PublisherError)?;
        Ok(true)
    }

    async fn insert_batch(context: &Context, messages: Vec<InputMessage>) -> FieldResult<bool> {
        log::debug!("inserting batch of {} messages", messages.len());

        let publisher = context.connect_to_data_router().await?;
        let order_group_id = Uuid::new_v4();

        for message in messages {
            let payload = serde_json::to_vec(&DataRouterInsertMessage {
                object_id: message.object_id,
                schema_id: message.schema_id,
                order_group_id: Some(order_group_id),
                data: &RawValue::from_string(message.payload)?,
            })?;

            publisher
                .publish_message(
                    &context.config().data_router_topic_or_queue,
                    &message.object_id.to_string(),
                    payload,
                )
                .await
                .map_err(Error::PublisherError)?;
        }
        Ok(true)
    }
}
