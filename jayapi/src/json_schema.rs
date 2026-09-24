use crate::{AsResource, DataResponse, DataResponseStatus};

const JSON_SCHEMA_RESOURCE_DEF: &'static str = "jayapi_resource";
const JSON_SCHEMA_RESPONSE_DEF: &'static str = "jayapi_data_response";

pub trait JsonSchema: schemars::JsonSchema {}
impl<T> JsonSchema for T where T: schemars::JsonSchema {}

impl<STATUS: DataResponseStatus, R: schemars::JsonSchema + AsResource> schemars::JsonSchema
    for DataResponse<STATUS, R>
{
    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed(R::ty())
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        util::attach_resource_def(generator);

        let def_path =
            generator.settings().definitions_path.clone() + "/" + JSON_SCHEMA_RESOURCE_DEF;

        schemars::json_schema!({
            "type": "object",
            "properties": {
                "data": R::json_schema(generator),
                "included": {
                    "type": "array",
                    "items": {
                        "$ref": def_path,
                    }
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
        util::attach_resource_def(generator);

        let def_path =
            generator.settings().definitions_path.clone() + "/" + JSON_SCHEMA_RESOURCE_DEF;

        schemars::json_schema!({
            "type": "object",
            "properties": {
                "data": {
                    "type": "array",
                    "items": R::json_schema(generator)
                },
                "included": {
                    "type": "array",
                    "items": {
                        "$ref": def_path,
                    }
                },
                "links": {
                    "type": "object"
                }
            },
            "required": ["data"]
        })
    }
}

impl<STATUS: DataResponseStatus> schemars::JsonSchema for DataResponse<STATUS, ()> {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed(JSON_SCHEMA_RESPONSE_DEF)
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        util::attach_resource_def(generator);

        let def_path =
            generator.settings().definitions_path.clone() + "/" + JSON_SCHEMA_RESOURCE_DEF;

        schemars::json_schema!({
            "type": "object",
            "properties": {
                "data": {
                    "oneOf": [
                        {
                            "$ref": def_path
                        },
                        {
                            "type": "array",
                            "items": {
                                "$ref": def_path
                            }
                        }
                    ]
                },
                "included": {
                    "type": "array",
                    "items": {
                        "$ref": def_path,
                    }
                },
                "links": {
                    "type": "object"
                }
            },
            "required": ["data"],
        })
    }
}

pub mod util {
    pub fn attach_resource_def(generator: &mut schemars::SchemaGenerator) {
        let defs = generator.definitions_mut();
        match defs.entry(super::JSON_SCHEMA_RESOURCE_DEF) {
            serde_json::map::Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(
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
                    .to_value()
                    .to_owned(),
                );
            }
            serde_json::map::Entry::Occupied(_) => {}
        };
    }

    pub fn attach_response_def(generator: &mut schemars::SchemaGenerator) {
        let def_path =
            generator.settings().definitions_path.clone() + "/" + super::JSON_SCHEMA_RESOURCE_DEF;
        let defs = generator.definitions_mut();
        match defs.entry(super::JSON_SCHEMA_RESPONSE_DEF) {
            serde_json::map::Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(
                    schemars::json_schema!({
                        "type": "object",
                        "properties": {
                            "data": {
                                "oneOf": [
                                    {
                                        "$ref": def_path
                                    },
                                    {
                                        "type": "array",
                                        "items": {
                                            "$ref": def_path
                                        },
                                        "uniqueItems": true,
                                    }
                                ]
                            },
                            "included": {
                                "type": "array",
                                "items": {
                                    "$ref": def_path,
                                }
                            },
                        },
                        "required": ["data"],
                    })
                    .to_value()
                    .to_owned(),
                );
            }
            serde_json::map::Entry::Occupied(_) => {}
        };
    }

    pub fn attach_defs(generator: &mut schemars::SchemaGenerator) {
        attach_resource_def(generator);
        attach_response_def(generator);
    }
}
