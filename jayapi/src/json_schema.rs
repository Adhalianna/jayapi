use crate::{AsResource, DataResponse, DataResponseStatus};

const JSON_SCHEMA_RESOURCE_DEF: &'static str = "jayapi_resource";
const JSON_SCHEMA_RESPONSE_DEF: &'static str = "jayapi_data_response";

pub trait JsonSchema: schemars::JsonSchema {}
impl<T> JsonSchema for T where T: schemars::JsonSchema {}

impl schemars::JsonSchema for crate::Resource {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed(JSON_SCHEMA_RESOURCE_DEF)
    }

    fn json_schema(_generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "object",
            "properties": {
                "type": {
                    "type": "string"
                },
                "id": {
                    "type": "string"
                },
                "attributes": {
                    "type": "object"
                },
                "relationships": {
                    "type": "object"
                },
                "links": {
                    "type": "object"
                },
            },
            "required": ["type", "id"],
            "additionalProperties": false,
        })
    }
}

impl<STATUS: DataResponseStatus> schemars::JsonSchema for DataResponse<STATUS, ()> {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed(JSON_SCHEMA_RESPONSE_DEF)
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
                        "type": "object",
                        "properties": {
                            "data": {
                                "oneOf": [
                                    crate::Resource::json_schema(generator),
                                    {
                                        "type": "array",
                                        "items": {
                                            "$ref": crate::Resource::json_schema(generator)
                                        },
                                        "uniqueItems": true,
                                    }
                                ]
                            },
                            "included": {
                                "type": "array",
                                "items": {
                                    "$ref": crate::Resource::json_schema(generator),
                                }
                            },
                        },
                        "required": ["data"],
                    }
        )
    }
}

impl<STATUS: DataResponseStatus, R: schemars::JsonSchema + AsResource> schemars::JsonSchema
    for DataResponse<STATUS, R>
{
    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed(R::ty())
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "object",
            "properties": {
                "data": R::json_schema(generator),
                "included": {
                    "type": "array",
                    "items": crate::Resource::json_schema(generator)
                },
                "links": {
                    "type": "object"
                }
            },
            "required": ["data"]
        })
    }
}

impl<STATUS: DataResponseStatus, R: schemars::JsonSchema + AsResource> schemars::JsonSchema
    for DataResponse<STATUS, Vec<R>>
{
    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed(R::ty())
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "object",
            "properties": {
                "data": {
                    "type": "array",
                    "items": R::json_schema(generator)
                },
                "included": {
                    "type": "array",
                    "items": crate::Resource::json_schema(generator)
                },
                "links": {
                    "type": "object"
                }
            },
            "required": ["data"]
        })
    }
}
