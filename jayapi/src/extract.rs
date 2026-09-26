use crate::DataRequest;
use std::marker::PhantomData;

#[cfg(feature = "axum")]
use axum::body::Bytes;

pub trait ResourceTypeList {
    fn types() -> Vec<&'static str>;
}
macro_rules! impl_resource_type_list {
    // Base case: single type (not in a tuple)
    ($single:ident) => {
        impl<$single> ResourceTypeList for $single
        where
            $single: crate::FromLocalResource + crate::ResourceType
        {
            fn types() -> Vec<&'static str> {
                vec![ $single::ty() ]
            }
        }
    };
    // Recursive case: implement for the full tuple, then invoke for one smaller size
    ($head:ident, $($tail:ident),+) => {
        impl<$head, $($tail),+> ResourceTypeList for ($head, $($tail),+)
        where
            $head: crate::FromLocalResource + crate::ResourceType,
            $($tail: crate::FromLocalResource + crate::ResourceType ),+
        {
            fn types() -> Vec<&'static str> {
                vec![ $($tail::ty()),+ ]
            }
        }
        // Recurse with the tail to generate implementations for smaller tuple sizes
        impl_resource_type_list!($($tail),+);
    };
}
impl_resource_type_list!(
    R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12, R13, R14, R15, R16
);

pub struct Any;

pub struct ExtractDataRequest<R, ALLOW = ()> {
    pub data: R,
    included: Option<Vec<crate::LocalResource>>,
    allow_include: PhantomData<ALLOW>,
}

impl<R> ExtractDataRequest<R, Any> {
    pub fn extract_included<R1>(
        &self,
    ) -> Result<Option<Vec<R1>>, <R1 as TryFrom<crate::LocalResource>>::Error>
    where
        R1: TryFrom<crate::LocalResource> + crate::ResourceType,
    {
        let parsed = self.included.as_ref().map(|v| {
            v.iter()
                .filter(|lr| lr.r#type == R1::ty())
                .map(|lr| R1::try_from(lr.clone()))
                .fold(Result::Ok(Vec::new()), |mut acc, r| match &mut acc {
                    Ok(v) => match r {
                        Ok(r1) => {
                            v.push(r1);
                            acc
                        }
                        Err(err) => Err(err),
                    },
                    Err(_) => acc,
                })
        });
        match parsed {
            Some(res) => match res {
                Ok(v) => Ok(Some(v)),
                Err(err) => Err(err),
            },
            None => Result::Ok(Option::None),
        }
    }
}

impl<R, L: ResourceTypeList> ExtractDataRequest<R, L> {
    pub fn extract_included<R1>(
        &self,
    ) -> Result<Option<Vec<R1>>, <R1 as TryFrom<crate::LocalResource>>::Error>
    where
        R1: TryFrom<crate::LocalResource> + crate::ResourceType,
    {
        #[cfg(debug_assertions)]
        {
            L::types().iter().find(|name| **name == R1::ty()).expect("attempted to extract from included a type that was not added to the list of allowed types");
        }

        let parsed = self.included.as_ref().map(|v| {
            v.iter()
                .filter(|lr| lr.r#type == R1::ty())
                .map(|lr| R1::try_from(lr.clone()))
                .fold(Result::Ok(Vec::new()), |mut acc, r| match &mut acc {
                    Ok(v) => match r {
                        Ok(r1) => {
                            v.push(r1);
                            acc
                        }
                        Err(err) => Err(err),
                    },
                    Err(_) => acc,
                })
        });
        match parsed {
            Some(res) => match res {
                Ok(v) => Ok(Some(v)),
                Err(err) => Err(err),
            },
            None => Result::Ok(Option::None),
        }
    }
}

#[cfg(feature = "axum")]
/// # Rejections
/// The extractor will reject requests which main resource (`data`) fails to deserialize and which
/// has unallowed types in its `included` section. It will not attempt to deserialize from resources
/// to target types the `included` section. It will only check the `type` names for those. Furthermore,
/// it will make sure that every [`jayapi::LocalResource`] in the `included` section has `lid` assigned.
impl<
    T: TryFrom<crate::LocalResource, Error = crate::ParsingError>,
    L: ResourceTypeList,
    S: Send + Sync,
> axum::extract::FromRequest<S> for ExtractDataRequest<T, L>
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
            let req: DataRequest = musli::json::from_slice(&bytes).map_err(|e| {
                crate::ParsingError::DeserializationError {
                    source: e.to_string(),
                }
            })?;
            let data = T::try_from(req.data)?;
            if let Some(included) = &req.included {
                for lr in included {
                    if lr.lid.is_none() {
                        return Err(crate::ParsingError::IncludedResourceMissingLid {
                            resource_type: lr.r#type.clone(),
                        });
                    }
                    if L::types().iter().find(|t| **t == lr.r#type).is_none() {
                        return Err(crate::ParsingError::IncludedTypeNotAccepted {
                            obtained: lr.r#type.to_owned(),
                            allowed: L::types().iter().map(|s| String::from(*s)).collect(),
                        });
                    }
                }
            }
            Ok(Self {
                data,
                included: req.included,
                allow_include: PhantomData,
            })
        }
        #[cfg(not(feature = "musli"))]
        {
            let req: DataRequest = serde_json::from_slice(&bytes).map_err(|e| {
                crate::ParsingError::DeserializationError {
                    source: e.to_string(),
                }
            })?;
            let data = T::try_from(req.data)?;
            if let Some(included) = &req.included {
                for lr in included {
                    if lr.lid.is_none() {
                        return Err(crate::ParsingError::IncludedResourceMissingLid {
                            resource_type: lr.r#type.clone(),
                        });
                    }
                    if L::types().iter().find(|t| **t == lr.r#type).is_none() {
                        return Err(crate::ParsingError::IncludedTypeNotAccepted {
                            obtained: lr.r#type.to_owned(),
                            allowed: L::types().iter().map(|s| String::from(*s)).collect(),
                        });
                    }
                }
            }
            Ok(Self {
                data,
                included: req.included,
                allow_include: PhantomData,
            })
        }
    }
}

#[cfg(feature = "axum")]
impl<T: TryFrom<crate::LocalResource, Error = crate::ParsingError>, S: Send + Sync>
    axum::extract::FromRequest<S> for ExtractDataRequest<T, ()>
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
            let req: DataRequest = musli::json::from_slice(&bytes).map_err(|e| {
                crate::ParsingError::DeserializationError {
                    source: e.to_string(),
                }
            })?;
            let data = T::try_from(req.data)?;
            if req.included.is_some() {
                return Err(crate::ParsingError::NotAcceptingIncluded);
            }
            Ok(Self {
                data,
                included: req.included,
                allow_include: PhantomData,
            })
        }
        #[cfg(not(feature = "musli"))]
        {
            let req: DataRequest = serde_json::from_slice(&bytes).map_err(|e| {
                crate::ParsingError::DeserializationError {
                    source: e.to_string(),
                }
            })?;
            let data = T::try_from(req.data)?;
            if req.included.is_some() {
                return Err(crate::ParsingError::NotAcceptingIncluded);
            }
            Ok(Self {
                data,
                included: req.included,
                allow_include: PhantomData,
            })
        }
    }
}

#[cfg(feature = "axum")]
impl<T: TryFrom<crate::LocalResource, Error = crate::ParsingError>, S: Send + Sync>
    axum::extract::FromRequest<S> for ExtractDataRequest<T, Any>
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
            let req: DataRequest = musli::json::from_slice(&bytes).map_err(|e| {
                crate::ParsingError::DeserializationError {
                    source: e.to_string(),
                }
            })?;
            let data = T::try_from(req.data)?;
            if let Some(included) = &req.included {
                for lr in included {
                    if lr.lid.is_none() {
                        return Err(crate::ParsingError::IncludedResourceMissingLid {
                            resource_type: lr.r#type.clone(),
                        });
                    }
                }
            }
            Ok(Self {
                data,
                included: req.included,
                allow_include: PhantomData,
            })
        }
        #[cfg(not(feature = "musli"))]
        {
            let req: DataRequest = serde_json::from_slice(&bytes).map_err(|e| {
                crate::ParsingError::DeserializationError {
                    source: e.to_string(),
                }
            })?;
            let data = T::try_from(req.data)?;
            if let Some(included) = &req.included {
                for lr in included {
                    if lr.lid.is_none() {
                        return Err(crate::ParsingError::IncludedResourceMissingLid {
                            resource_type: lr.r#type.clone(),
                        });
                    }
                }
            }
            Ok(Self {
                data,
                included: req.included,
                allow_include: PhantomData,
            })
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
