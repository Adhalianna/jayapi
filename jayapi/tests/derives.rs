use jayapi::{AsResource, FromResource, JsonSchema};

#[derive(FromResource, AsResource, JsonSchema)]
pub struct Test {
    #[jayapi(id)]
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
}

pub fn main() {
    let mut gen = schemars::SchemaGenerator::default();

    let _schema = dbg!(<Test as schemars::JsonSchema>::json_schema(&mut gen));
}
