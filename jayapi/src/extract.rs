#[cfg(feature = "axum")]
use axum::body::Bytes;

#[cfg(feature = "axum")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "musli", derive(musli::Encode, musli::Decode))]
struct ResourceRequestBody {
    data: crate::Resource,
}

pub struct ResourceRequest<T>(pub T);

#[cfg(feature = "axum")]
impl<T: TryFrom<crate::Resource, Error = crate::ParsingError>, S: Send + Sync>
    axum::extract::FromRequest<S> for ResourceRequest<T>
{
    type Rejection = crate::ParsingError;

    async fn from_request(req: axum::extract::Request, state: &S) -> Result<Self, Self::Rejection> {
        let bytes = Bytes::from_request(req, state).await.map_err(|e| {
            crate::ParsingError::UnrecognizedFormat {
                source: e.to_string(),
            }
        })?;
        #[cfg(feature = "musli")]
        {
            let req: ResourceRequestBody = musli::json::from_slice(&bytes).map_err(|e| {
                crate::ParsingError::DeserializationError {
                    source: e.to_string(),
                }
            })?;
            let inner = T::try_from(req.data)?;
            Ok(Self(inner))
        }
        #[cfg(not(feature = "musli"))]
        {
            let req: ResourceRequestBody = serde_json::from_slice(&bytes).map_err(|e| {
                crate::ParsingError::DeserializationError {
                    source: e.to_string(),
                }
            })?;
            let inner = T::try_from(req.data)?;
            Ok(Self(inner))
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct NotIncluded;

impl std::fmt::Display for NotIncluded {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "resource not included")
    }
}

impl std::error::Error for NotIncluded {}
