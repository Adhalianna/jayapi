use crate::{DataResponse, status::DataResponseStatus};

const JSON_SCHEMA_RESOURCE_DEF: &'static str = "jayapi_resource";
const JSON_SCHEMA_LOCAL_RESOURCE_DEF: &'static str = "jayapi_local_resource";
const JSON_SCHEMA_RESPONSE_DEF: &'static str = "jayapi_data_response";

pub trait JsonSchema: schemars::JsonSchema {}
impl JsonSchema for crate::Resource {}
impl JsonSchema for crate::LocalResource {}
impl JsonSchema for crate::DataRequest {}
impl<STATUS: DataResponseStatus> JsonSchema for DataResponse<STATUS, ()> {}
impl<STATUS: DataResponseStatus, R: crate::ResourceType + schemars::JsonSchema> JsonSchema
    for DataResponse<STATUS, R>
{
}

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

impl schemars::JsonSchema for crate::LocalResource {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed(JSON_SCHEMA_LOCAL_RESOURCE_DEF)
    }

    fn json_schema(_generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "object",
            "properties": {
                "type": {
                    "type": "string"
                },
                "lid": {
                    "type": "string"
                },
                "attributes": {
                    "type": "object"
                },
                "relationships": {
                    "type": "object"
                },
            },
            "required": ["type"],
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
                                    <crate::Resource as schemars::JsonSchema>::json_schema(generator),
                                    {
                                        "type": "array",
                                        "items": {
                                            "$ref": <crate::Resource as schemars::JsonSchema>::json_schema(generator)
                                        },
                                        "uniqueItems": true,
                                    }
                                ]
                            },
                            "included": {
                                "type": "array",
                                "items": {
                                    "$ref": <crate::Resource as schemars::JsonSchema>::json_schema(generator),
                                }
                            },
                        },
                        "required": ["data"],
                    }
        )
    }
}

impl<STATUS: DataResponseStatus, R: schemars::JsonSchema + crate::ResourceType> schemars::JsonSchema
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
                    "items": <crate::Resource as schemars::JsonSchema>::json_schema(generator)
                },
                "links": {
                    "type": "object"
                }
            },
            "required": ["data"]
        })
    }
}

impl<STATUS: DataResponseStatus, R: schemars::JsonSchema + crate::ResourceType> schemars::JsonSchema
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
                    "items": <crate::Resource as schemars::JsonSchema>::json_schema(generator)
                },
                "links": {
                    "type": "object"
                }
            },
            "required": ["data"]
        })
    }
}

pub trait AlternativeResourceListSchema {
    fn list_json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema;
}
macro_rules! impl_alternative_resource_list_schema {
    // Base case: single type (not in a tuple)
    ($single:ident) => {
        impl<$single> AlternativeResourceListSchema for $single
        where
            $single: JsonSchema,
        {
            fn list_json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
                schemars::json_schema!({
                    "oneOf": [
                        $single::json_schema(generator)
                    ]
                })
            }
        }
    };

    // Recursive case: implement for the full tuple, then recurse for smaller tuples
    ($head:ident, $($tail:ident),+) => {
        impl<$head, $($tail),+> AlternativeResourceListSchema for ($head, $($tail),+)
        where
            $head: JsonSchema,
            $($tail: JsonSchema),+
        {
            fn list_json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
                schemars::json_schema!({
                    "oneOf": [
                        $head::json_schema(generator),
                        $($tail::json_schema(generator)),+
                    ]
                })
            }
        }

        // Recurse with the tail to implement for smaller tuple sizes
        impl_alternative_resource_list_schema!($($tail),+);
    };
}

impl_alternative_resource_list_schema!(
    R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12, R13, R15, R16
);
