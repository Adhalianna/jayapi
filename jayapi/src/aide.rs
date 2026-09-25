use crate::{AsResource, DataResponse, DataResponseStatus};

impl<STATUS: DataResponseStatus, R> aide::OperationOutput for DataResponse<STATUS, R>
where
    DataResponse<STATUS, R>: schemars::JsonSchema,
    R: schemars::JsonSchema + AsResource,
{
    type Inner = Self;

    fn operation_response(
        ctx: &mut aide::generate::GenContext,
        _operation: &mut aide::openapi::Operation,
    ) -> Option<aide::openapi::Response> {
        use schemars::JsonSchema;

        let mut response = aide::openapi::Response::default();
        let mut content = aide::openapi::MediaType::default();

        content.schema = Some(aide::openapi::SchemaObject {
            json_schema: Self::json_schema(&mut ctx.schema),
            external_docs: None,
            example: None,
        });

        response
            .content
            .insert(String::from("application/json"), content);

        Some(response)
    }

    fn inferred_responses(
        ctx: &mut aide::generate::GenContext,
        operation: &mut aide::openapi::Operation,
    ) -> Vec<(Option<u16>, aide::openapi::Response)> {
        vec![(
            Some(STATUS::status()),
            Self::operation_response(ctx, operation).unwrap(),
        )]
    }
}

impl<STATUS: DataResponseStatus, R> aide::OperationOutput for DataResponse<STATUS, Vec<R>>
where
    DataResponse<STATUS, Vec<R>>: schemars::JsonSchema,
    R: schemars::JsonSchema + AsResource,
{
    type Inner = Self;

    fn operation_response(
        ctx: &mut aide::generate::GenContext,
        _operation: &mut aide::openapi::Operation,
    ) -> Option<aide::openapi::Response> {
        use schemars::JsonSchema;

        let mut response = aide::openapi::Response::default();
        let mut content = aide::openapi::MediaType::default();

        content.schema = Some(aide::openapi::SchemaObject {
            json_schema: Self::json_schema(&mut ctx.schema),
            external_docs: None,
            example: None,
        });

        response
            .content
            .insert(String::from("application/json"), content);

        Some(response)
    }

    fn inferred_responses(
        ctx: &mut aide::generate::GenContext,
        operation: &mut aide::openapi::Operation,
    ) -> Vec<(Option<u16>, aide::openapi::Response)> {
        vec![(
            Some(STATUS::status()),
            Self::operation_response(ctx, operation).unwrap(),
        )]
    }
}

#[cfg(feature = "axum")]
impl<R: schemars::JsonSchema> aide::OperationInput for crate::extract::ResourceRequest<R> {
    fn operation_input(
        ctx: &mut aide::generate::GenContext,
        operation: &mut aide::openapi::Operation,
    ) {
        let mut content = aide::openapi::MediaType::default();

        content.schema = Some(aide::openapi::SchemaObject {
            json_schema: schemars::json_schema!({
                "type": "object",
                "properties": {
                    "data": R::json_schema(&mut ctx.schema)
                }
            }),
            external_docs: None,
            example: None,
        });

        operation.request_body = Some(aide::openapi::ReferenceOr::Item({
            let mut body = aide::openapi::RequestBody {
                description: None,
                required: true,
                ..Default::default()
            };
            body.content
                .insert(String::from("application/json"), content);
            body
        }))
    }

    fn inferred_early_responses(
        _ctx: &mut aide::generate::GenContext,
        _operation: &mut aide::openapi::Operation,
    ) -> Vec<(Option<u16>, aide::openapi::Response)> {
        Vec::new()
    }
}
