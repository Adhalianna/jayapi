use jayapi::{AsResource, FromResource, JsonSchema};

#[derive(FromResource, AsResource, JsonSchema)]
pub struct Test {
    #[jayapi(id)]
    id: String,
    attribute: String,
    second_attribute: Option<String>,
    #[jayapi(attribute)]
    third_attribute: Vec<u16>,
    #[jayapi(relationship(to_many))]
    relations: Vec<String>,
    #[jayapi(relationship(resource_type = "test_resource"))]
    relation: String,
}

pub fn main() {
    let mut gen = schemars::SchemaGenerator::default();

    dbg!(<Test as schemars::JsonSchema>::json_schema(&mut gen));
}
