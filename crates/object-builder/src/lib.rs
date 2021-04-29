use crate::args::Args;
use anyhow::Context;
use async_trait::async_trait;
use bb8::{Pool, PooledConnection};
use futures::Stream;
use rpc::common::RowDefinition as RpcRowDefinition;
use rpc::materializer_general::{MaterializedView, Options};
use rpc::object_builder::{object_builder_server::ObjectBuilder, Empty, View};
use rpc::schema_registry::{schema_registry_client::SchemaRegistryClient, types::SchemaType};
use serde::Serialize;
use serde_json::Value;
use std::{collections::HashMap, convert::TryInto, pin::Pin};
use tonic::transport::Channel;
use utils::communication::{consumer::ConsumerHandler, message::CommunicationMessage};
use utils::{
    metrics::{self, counter},
    types::materialization,
};
use uuid::Uuid;

pub mod args;

#[derive(Clone)]
pub struct ObjectBuilderImpl {
    pool: Pool<SchemaRegistryConnectionManager>,
}

#[derive(Clone)]
pub struct SchemaRegistryConnectionManager {
    pub address: String,
}

pub type SchemaRegistryPool = Pool<SchemaRegistryConnectionManager>;
pub type SchemaRegistryConn = SchemaRegistryClient<Channel>;

#[async_trait::async_trait]
impl bb8::ManageConnection for SchemaRegistryConnectionManager {
    type Connection = SchemaRegistryConn;
    type Error = rpc::error::ClientError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        tracing::debug!("Connecting to registry");

        rpc::schema_registry::connect(self.address.clone()).await
    }

    async fn is_valid(&self, conn: &mut PooledConnection<'_, Self>) -> Result<(), Self::Error> {
        conn.ping(rpc::schema_registry::Empty {})
            .await
            .map_err(rpc::error::schema_registry_error)?;

        Ok(())
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        false
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "snake_case")]
struct Output {
    view_id: Uuid,
    options: Value,
    rows: Vec<RowDefinition>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "snake_case")]
struct RowDefinition {
    object_id: Uuid,
    fields: HashMap<String, Value>,
}

impl ObjectBuilderImpl {
    pub async fn new(args: &Args) -> anyhow::Result<Self> {
        let pool = Pool::builder()
            .build(SchemaRegistryConnectionManager {
                address: args.schema_registry_addr.clone(),
            })
            .await
            .unwrap();

        Ok(Self { pool })
    }
}

impl TryInto<RpcRowDefinition> for RowDefinition {
    type Error = serde_json::Error;

    fn try_into(self) -> Result<RpcRowDefinition, Self::Error> {
        let fields = self
            .fields
            .into_iter()
            .map(|(key, value)| Ok((key, serde_json::to_string(&value)?)))
            .collect::<serde_json::Result<_>>()?;
        Ok(RpcRowDefinition {
            object_id: self.object_id.to_string(),
            fields,
        })
    }
}

impl TryInto<MaterializedView> for Output {
    type Error = serde_json::Error;

    fn try_into(self) -> Result<MaterializedView, Self::Error> {
        let rows = self
            .rows
            .into_iter()
            .map(|row| row.try_into())
            .collect::<serde_json::Result<_>>()?;

        Ok(MaterializedView {
            view_id: self.view_id.to_string(),
            options: Options {
                options: serde_json::to_string(&self.options)?,
            },
            rows,
        })
    }
}

#[async_trait]
impl ConsumerHandler for ObjectBuilderImpl {
    #[tracing::instrument(skip(self, msg))]
    async fn handle<'a>(&'a mut self, msg: &'a dyn CommunicationMessage) -> anyhow::Result<()> {
        let payload = msg.payload()?;
        tracing::debug!(?payload, "Handle MQ message");
        counter!("cdl.object-builder.build-object.mq", 1);
        let request: materialization::Request = serde_json::from_str(&payload)?;
        let view_id = request.view_id;

        let view = self.get_view(&view_id);
        let output = self.build_output(request);

        let (view, output) = futures::try_join!(view, output)?;

        let rpc_output: MaterializedView = output.try_into()?;

        rpc::materializer_general::connect(view.materializer_address)
            .await?
            .upsert_view(rpc_output)
            .await?;

        Ok(())
    }
}

#[tonic::async_trait]
impl ObjectBuilder for ObjectBuilderImpl {
    type MaterializeStream = Pin<
        Box<dyn Stream<Item = Result<RpcRowDefinition, tonic::Status>> + Send + Sync + 'static>,
    >;

    #[tracing::instrument(skip(self))]
    async fn materialize(
        &self,
        request: tonic::Request<View>,
    ) -> Result<tonic::Response<Self::MaterializeStream>, tonic::Status> {
        let view: View = request.into_inner();

        let request: materialization::Request = view
            .try_into()
            .map_err(|_| tonic::Status::invalid_argument("view"))?;

        let output = self
            .build_output(request) // TODO use another method because we dont need MaterializerOptions here.
            .await
            .map_err(|err| tonic::Status::internal(format!("{}", err)))?;

        let iter = output.rows.into_iter().map(|row| {
            row.try_into().map_err(|err| {
                tracing::error!("Could not serialize materialized row: {:?}", err);
                tonic::Status::internal("Could not serialize materialized row")
            })
        });

        // TODO: This is just an example how could we leverage streaming API.
        // In theory we could go further and make our `rpc::query_service::query_multiple` an streaming method.
        // Then you could start processing views without collecting all objects.
        //
        // Of course its not a trivial task, especially when there are joins. The solution would have to handle waiting
        // for the all connected objects before sending the row.
        // Still an iterator could be a little bit benefitial.
        let stream = Box::pin(futures::stream::iter(iter));

        Ok(tonic::Response::new(stream))
    }

    #[tracing::instrument(skip(self))]
    async fn heartbeat(
        &self,
        _request: tonic::Request<Empty>,
    ) -> Result<tonic::Response<Empty>, tonic::Status> {
        //empty
        Ok(tonic::Response::new(Empty {}))
    }
}

impl ObjectBuilderImpl {
    //TODO: We should no longer use one method to handle both RPC request and message queues, because the responsibilites splitted.
    #[tracing::instrument(skip(self))]
    async fn build_output(&self, request: materialization::Request) -> anyhow::Result<Output> {
        tracing::debug!(?request, "Handling");

        let materialization::Request { view_id, schemas } = request;

        let view = self.get_view(&view_id).await?;
        tracing::debug!(?view, "View");

        // TODO: Handle more than one schema
        // TODO: Handle empty filter for seeding view (maybe in another method)
        let (schema_id, schema) = schemas.into_iter().next().unwrap();
        let objects = self.get_objects(schema_id, schema).await?;
        tracing::debug!(?objects, "Objects");

        let fields_defs: HashMap<String, materialization::FieldDefinition> = view
            .fields
            .into_iter()
            .map(|(key, field)| Ok((key, serde_json::from_str(&field)?)))
            .collect::<anyhow::Result<HashMap<_, _>>>()?;

        let rows = objects
            .into_iter()
            .map(|(object_id, object)| Self::build_row_def(object_id, object, &fields_defs))
            .collect::<anyhow::Result<_>>()?;

        let options = serde_json::from_str(&view.materializer_options)?;

        let output = Output {
            view_id,
            options,
            rows,
        };

        tracing::debug!(?output, "Output");

        Ok(output)
    }

    #[tracing::instrument]
    fn build_row_def(
        object_id: Uuid,
        object: Value,
        fields_defs: &HashMap<String, materialization::FieldDefinition>,
    ) -> anyhow::Result<RowDefinition> {
        use materialization::FieldDefinition::*;

        let object = object
            .as_object()
            .with_context(|| format!("Expected object ({}) to be a JSON object", object_id))?;

        let fields = fields_defs
            .iter()
            .map(|(field_def_key, field_def)| {
                Ok((
                    field_def_key.into(),
                    match field_def {
                        FieldName(field_name) => {
                            let value = object.get(field_name).with_context(|| {
                                format!(
                                    "Object ({}) does not have a field named `{}`",
                                    object_id, field_name
                                )
                            })?;
                            value.clone()
                        }
                    },
                ))
            })
            .collect::<anyhow::Result<_>>()?;
        Ok(RowDefinition { object_id, fields })
    }

    #[tracing::instrument(skip(self))]
    async fn get_objects(
        &self,
        schema_id: Uuid,
        schema: materialization::Schema,
    ) -> anyhow::Result<HashMap<Uuid, Value>> {
        let schema_meta = self.get_schema_metadata(schema_id).await?;

        let query_address = schema_meta.query_address.clone();
        let schema_type = schema_meta.schema_type().into();

        match schema_type {
            SchemaType::DocumentStorage => {
                let values = rpc::query_service::query_multiple(
                    schema
                        .object_ids
                        .into_iter()
                        .map(|id| id.to_string())
                        .collect(),
                    query_address,
                )
                .await?;

                values
                    .into_iter()
                    .map(|(object_id, value)| {
                        let id: Uuid = object_id.parse()?;
                        Ok((id, serde_json::from_slice(&value)?))
                    })
                    .collect()
            }

            SchemaType::Timeseries => {
                // TODO:
                anyhow::bail!("Timeseries storage is not supported yet")
            }
        }
    }

    #[tracing::instrument(skip(self))]
    async fn get_view(&self, view_id: &Uuid) -> anyhow::Result<rpc::schema_registry::View> {
        let view = self
            .pool
            .get()
            .await?
            .get_view(rpc::schema_registry::Id {
                id: view_id.to_string(),
            })
            .await?
            .into_inner();

        Ok(view)
    }

    #[tracing::instrument(skip(self))]
    async fn get_schema_metadata(
        &self,
        schema_id: Uuid,
    ) -> anyhow::Result<rpc::schema_registry::SchemaMetadata> {
        let schema = self
            .pool
            .get()
            .await?
            .get_schema_metadata(rpc::schema_registry::Id {
                id: schema_id.to_string(),
            })
            .await?
            .into_inner();
        Ok(schema)
    }
}
