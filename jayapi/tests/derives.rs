use jayapi::{AsResource, FromResource, JsonSchema};

#[derive(FromResource, AsResource, JsonSchema)]
pub struct Test {
    #[jayapi(id)]
    id: String,
    attribute: String,
    #[jayapi(relationship(to_many))]
    relations: Vec<String>,
    #[jayapi(relationship(resource_type = "test_resource"))]
    relation: String,
}

pub fn main() {}
