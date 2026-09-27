#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "musli", derive(musli::Encode))]
pub struct ErrorResponse {
    pub errors: Vec<ErrorObject>,
    #[cfg_attr(feature = "musli", musli(skip))]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub status: u16,
}

#[cfg(feature = "axum")]
impl axum::response::IntoResponse for ErrorResponse {
    fn into_response(self) -> axum::response::Response {
        #[cfg(feature = "musli")]
        let res = (
            axum::http::StatusCode::from_u16(self.status).unwrap(),
            [(axum::http::header::CONTENT_TYPE, "application/json")],
            musli::json::to_string(&self).unwrap(),
        )
            .into_response();
        #[cfg(not(feature = "musli"))]
        let res = (
            axum::http::StatusCode::from_u16(self.status).unwrap(),
            axum::Json(self),
        )
            .into_response();

        res
    }
}

impl From<ErrorObject> for ErrorResponse {
    fn from(val: ErrorObject) -> Self {
        Self {
            status: val.status,
            errors: vec![val],
        }
    }
}

#[cfg(feature = "anyhow")]
impl From<anyhow::Error> for ErrorResponse {
    fn from(val: anyhow::Error) -> Self {
        ErrorObject::from(val).into()
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "musli", derive(musli::Encode, musli::Decode))]
pub struct ErrorObject {
    pub status: u16,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    #[cfg_attr(feature = "musli", musli(default, skip_encoding_if = Option::is_none))]
    pub code: Option<String>,
    pub title: String,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    #[cfg_attr(feature = "musli", musli(default, skip_encoding_if = Option::is_none))]
    pub description: Option<String>,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    #[cfg_attr(feature = "musli", musli(default, skip_encoding_if = Option::is_none))]
    pub source: Option<ErrorSource>,
}

impl ErrorObject {
    pub fn with_status(&mut self, status: u16) -> &mut Self {
        self.status = status;
        self
    }
    pub fn with_code(&mut self, code: String) -> &mut Self {
        self.code = Some(code);
        self
    }
    pub fn with_description(&mut self, description: String) -> &mut Self {
        self.description = Some(description);
        self
    }
    pub fn with_source(&mut self, source: ErrorSource) -> &mut Self {
        self.source = Some(source);
        self
    }
}

#[cfg(feature = "axum")]
impl axum::response::IntoResponse for ErrorObject {
    fn into_response(self) -> axum::response::Response {
        ErrorResponse {
            status: self.status,
            errors: vec![self],
        }
        .into_response()
    }
}

impl Default for ErrorObject {
    // ErrorObject defaults to a very unhelpful internal server error.
    fn default() -> Self {
        Self {
            status: 500,
            code: None,
            title: String::from("internal server error"),
            description: None,
            source: None,
        }
    }
}

#[cfg(feature = "anyhow")]
impl From<anyhow::Error> for ErrorObject {
    fn from(val: anyhow::Error) -> Self {
        Self {
            title: val.to_string(),
            ..Default::default()
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "musli", derive(musli::Encode, musli::Decode))]
pub enum ErrorSource {
    Header { header: String },
    Parameter { parameter: String },
    Pointer { pointer: String },
}
