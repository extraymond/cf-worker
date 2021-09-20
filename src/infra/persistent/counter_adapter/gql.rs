use graphql_client::GraphQLQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/infra/persistent/counter_adapter/schemas/schema.graphql",
    query_path = "src/infra/persistent/counter_adapter/schemas/get_counter.graphql",
    response_derives = "Debug,Serialize,Deserialize"
)]
pub struct GetCounter;
