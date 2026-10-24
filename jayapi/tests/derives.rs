use jayapi::{
    AsLocalResource, AsResource, FromLocalResource, FromResource, JsonSchema, ResourceType,
};

#[derive(
    ResourceType, FromLocalResource, AsLocalResource, FromResource, AsResource, JsonSchema,
)]
pub struct Test {
    #[jayapi(id, lid)]
    id: u32,
    attribute: String,
    second_attribute: Option<String>,
    #[jayapi(attribute)]
    third_attribute: Vec<u16>,
    #[jayapi(relationship(to_many))]
    relations: Vec<String>,
    #[jayapi(relationship(resource_type = "test_resource"))]
    relation: String,
    #[jayapi(relationship(name = "optional", optional, resource_type = "another_test_resource"))]
    optional_relation: Option<String>,
    #[jayapi(relationship(global_or_local, resource_type = "some_resource"))]
    maybe_local_relation: jayapi::GlobalOrLocalId<String, String>,
    #[jayapi(relationship(global_or_local, to_many, resource_type = "next_resource"))]
    many_local_or_global: Vec<jayapi::GlobalOrLocalId<String, u16>>,
    #[jayapi(relationship(global_or_local, optional, resource_type = "whooosh"))]
    optional_global_or_local: Option<jayapi::GlobalOrLocalId<u16, String>>
}

pub fn main() {
    let mut generator = schemars::SchemaGenerator::default();

    let _schema = dbg!(<Test as schemars::JsonSchema>::json_schema(&mut generator));
}
