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

