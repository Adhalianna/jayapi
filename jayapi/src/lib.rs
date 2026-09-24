#![cfg_attr(feature = "diagnostics", feature(trace_macros))]

#[cfg(feature = "derive")]
extern crate jayapi_derive;

#[cfg(feature = "derive")]
pub use jayapi_derive::*;

#[cfg(feature = "aide")]
pub mod aide;

#[cfg(feature = "json-schema")]
pub mod json_schema;

use std::marker::PhantomData;

pub mod extract;
pub use serde_json;

#[cfg(all(not(feature = "musli"), not(feature = "serde")))]
std::compile_err!("either 'musli' or 'serde' feature must be enabled");

pub type Error = ErrorResponse;
pub type Data<STATUS, R> = DataResponse<STATUS, R>;

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

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "musli", derive(musli::Encode, musli::Decode))]
pub struct Created {}
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "musli", derive(musli::Encode, musli::Decode))]
pub struct Ok {}

pub trait DataResponseStatus {
    fn status() -> u16;
}

impl DataResponseStatus for Created {
    fn status() -> u16 {
        201
    }
}

impl DataResponseStatus for Ok {
    fn status() -> u16 {
        200
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "musli", derive(musli::Encode, musli::Decode))]
#[allow(private_bounds)]
pub struct DataResponse<STATUS: DataResponseStatus, T = ()> {
    #[cfg_attr(feature = "musli", musli(with = musli::serde))]
    //TODO: waiting for custom musli::Decode on the enum
    pub data: SingleOrCollection,
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    #[cfg_attr(
        feature = "musli",
        musli(skip_encoding_if = Option::is_none, default)
    )]
    pub included: Option<Vec<Resource>>,
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    #[cfg_attr(
        feature = "musli",
        musli(skip_encoding_if = Option::is_none, default)
    )]
    pub links: Option<LinksMap>,
    #[cfg_attr(feature = "musli", musli(skip))]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub status: PhantomData<STATUS>,
    #[cfg_attr(feature = "musli", musli(skip))]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub resource_ty: PhantomData<T>,
}

impl<STATUS: DataResponseStatus, T> DataResponse<STATUS, T> {
    pub fn include(&mut self, resource: impl AsResource) {
        let included: &mut Vec<_> = self.included.get_or_insert_default();
        included.push(resource.into());
    }
    pub fn include_many<R: AsResource, I: IntoIterator<Item = R>>(&mut self, resources: I) {
        let included: &mut Vec<_> = self.included.get_or_insert_default();
        let to_include = resources.into_iter().map(|r| r.into());
        included.extend(to_include);
    }
    pub fn add_links<I: IntoIterator<Item = (String, String)>>(&mut self, links: I) {
        let map = self.links.get_or_insert(LinksMap::new());
        map.extend(links);
    }
}

#[cfg(all(not(feature = "musli"), feature = "axum"))]
impl<STATUS: DataResponseStatus + Send + Sync, T> axum::response::IntoResponse
    for DataResponse<STATUS, T>
{
    fn into_response(self) -> axum::response::Response {
        (
            axum::http::StatusCode::from_u16(STATUS::status()).unwrap(),
            axum::Json(self),
        )
            .into_response()
    }
}

#[cfg(all(feature = "musli", feature = "axum"))]
impl<STATUS: DataResponseStatus + Send + Sync, R> axum::response::IntoResponse
    for DataResponse<STATUS, R>
where
    STATUS: musli::Encode<musli::mode::Text>,
{
    fn into_response(self) -> axum::response::Response {
        (
            axum::StatusCode::from_u16(STATUS::status()).unwrap(),
            [(axum::http::header::CONTENT_TYPE, "application/json")],
            musli::json::to_string(&self).unwrap(),
        )
            .into_response()
    }
}

impl<R: AsResource, STATUS: DataResponseStatus> FromIterator<R> for DataResponse<STATUS, Vec<R>> {
    fn from_iter<T: IntoIterator<Item = R>>(iter: T) -> Self {
        Self {
            data: SingleOrCollection::from_iter(iter),
            status: PhantomData,
            resource_ty: PhantomData,
            included: None,
            links: None,
        }
    }
}

impl<R: AsResource, STATUS: DataResponseStatus> From<R> for DataResponse<STATUS, R> {
    fn from(value: R) -> Self {
        Self {
            data: SingleOrCollection::Single(value.into()),
            status: PhantomData,
            resource_ty: PhantomData,
            included: None,
            links: None,
        }
    }
}

impl<R: AsResource, STATUS: DataResponseStatus> From<Vec<R>> for DataResponse<STATUS, Vec<R>> {
    fn from(value: Vec<R>) -> Self {
        Self {
            data: SingleOrCollection::from_iter(value),
            status: PhantomData,
            resource_ty: PhantomData,
            included: None,
            links: None,
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "musli", derive(musli::Decode))]
#[cfg_attr(feature = "musli", musli(tag = "type", name_all = "snake_case"))]
pub enum SingleOrCollection {
    #[cfg_attr(feature = "musli", musli(transparent))]
    Collection(Vec<Resource>),
    #[cfg_attr(feature = "musli", musli(transparent))]
    Single(Resource),
}

//TODO: custom musli::Decode
#[cfg(feature = "musli")]
impl<M> musli::Encode<M> for SingleOrCollection
where
    Resource: musli::Encode<M>,
{
    type Encode = Self;

    fn as_encode(&self) -> &Self::Encode {
        self
    }

    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: musli::Encoder<Mode = M>,
    {
        use musli::en::*;

        match self {
            Self::Collection(coll) => {
                let mut seq = encoder.encode_sequence(coll.len())?;
                for elem in coll {
                    seq.push(elem)?;
                }
                seq.finish_sequence()
            }
            Self::Single(item) => encoder.encode(item),
        }
    }
}

impl From<Resource> for SingleOrCollection {
    fn from(val: Resource) -> Self {
        Self::Single(val)
    }
}

impl FromIterator<Resource> for SingleOrCollection {
    fn from_iter<T: IntoIterator<Item = Resource>>(iter: T) -> Self {
        Self::Collection(iter.into_iter().collect())
    }
}

impl<R: AsResource> FromIterator<R> for SingleOrCollection {
    fn from_iter<T: IntoIterator<Item = R>>(iter: T) -> Self {
        Self::Collection(iter.into_iter().map(|r| Resource::from(r)).collect())
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "musli", derive(musli::Encode, musli::Decode))]
pub struct Resource {
    #[cfg_attr(feature = "musli", musli(mode = Text, name = "type"))]
    pub r#type: String,
    #[cfg_attr(feature = "musli", musli(default))]
    #[cfg_attr(feature = "serde", serde(default))]
    pub id: String,
    #[cfg_attr(feature = "musli", musli(skip_encoding_if = Option::is_none, default))]
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub attributes: Option<AttributesMap>,
    #[cfg_attr(feature = "musli", musli(skip_encoding_if = Option::is_none, default))]
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub relationships: Option<RelationshipsMap>,
}

#[derive(Debug, Clone, Eq, PartialOrd, Ord, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "musli", derive(musli::Encode, musli::Decode))]
pub struct ResourceIdentifier {
    #[cfg_attr(feature = "musli", musli(mode = Text, name = "type"))]
    pub r#type: String,
    pub id: String,
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[cfg_attr(feature = "musli", derive(musli::Encode, musli::Decode))]
#[cfg_attr(feature = "musli", musli(transparent))]
pub struct RelationshipsMap(
    #[cfg_attr(feature = "musli", musli(with = musli::serde))]
    std::collections::HashMap<String, Relationship>,
);

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
//#[cfg_attr(feature = "musli", derive(musli::Decode))]
//#[cfg_attr(feature = "musli", musli(tag = "type", name_all = "snake_case"))]
pub enum Relationship {
    Relation1to1 { data: ResourceIdentifier },
    Relation1toM { data: Vec<ResourceIdentifier> },
}

impl RelationshipsMap {
    pub fn new() -> Self {
        Self(std::collections::HashMap::new())
    }
    pub fn with_capacity(capacity: usize) -> Self {
        Self(std::collections::HashMap::with_capacity(capacity))
    }
    pub fn get(&self, key: &str) -> Option<&Relationship> {
        self.0.get(key)
    }
    pub fn remove(&mut self, key: &str) -> Option<Relationship> {
        self.0.remove(key)
    }
    pub fn values(&self) -> std::collections::hash_map::Values<'_, String, Relationship> {
        self.0.values()
    }
    pub fn keys(&self) -> std::collections::hash_map::Keys<'_, String, Relationship> {
        self.0.keys()
    }
    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, String, Relationship> {
        self.0.iter()
    }
    pub fn iter_mut(&mut self) -> std::collections::hash_map::IterMut<'_, String, Relationship> {
        self.0.iter_mut()
    }
    pub fn insert(&mut self, k: String, v: Relationship) -> Option<Relationship> {
        self.0.insert(k, v)
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn clear(&mut self) {
        self.0.clear()
    }
    pub fn contains_key(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }
    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }
}

impl FromIterator<(String, Relationship)> for RelationshipsMap {
    fn from_iter<T: IntoIterator<Item = (String, Relationship)>>(iter: T) -> Self {
        Self(std::collections::HashMap::from_iter(iter))
    }
}

impl std::ops::Index<&str> for RelationshipsMap {
    type Output = Relationship;
    fn index(&self, index: &str) -> &Self::Output {
        self.0.index(index)
    }
}

//TODO: custom musli::Decode
#[cfg(feature = "musli")]
impl<'s, M> musli::Encode<M> for Relationship
where
    Resource: musli::Encode<M>,
    ResourceIdentifier: musli::Encode<M>,
{
    type Encode = Self;

    #[inline]
    fn as_encode(&self) -> &Self::Encode {
        self
    }

    #[inline]
    fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
    where
        E: musli::Encoder<Mode = M>,
    {
        use musli::en::*;

        match self {
            Self::Relation1toM { data } => {
                let mut strct = encoder.encode_map(1)?;
                let mut entry = strct.encode_entry()?;
                let key = entry.encode_key()?;
                key.encode_string("data")?;
                let val = entry.encode_value()?;
                let mut seq = val.encode_sequence(data.len())?;
                for elem in data {
                    seq.push(elem)?;
                }
                seq.finish_sequence()?;
                entry.finish_entry()?;
                strct.finish_map()
            }
            Self::Relation1to1 { data } => {
                let mut strct = encoder.encode_map(1)?;
                let mut entry = strct.encode_entry()?;
                let key = entry.encode_key()?;
                key.encode_string("data")?;
                let val = entry.encode_value()?;
                val.encode(data)
            }
        }
    }
}

pub trait AsResource {
    fn ty() -> &'static str;
    fn resource_identifier(&self) -> ResourceIdentifier;
    fn attributes(&self) -> Option<AttributesMap> {
        None
    }
    fn relationships(&self) -> Option<RelationshipsMap> {
        None
    }
}

pub trait FromResource: TryFrom<Resource> {}
impl<T> FromResource for T where T: TryFrom<Resource> {}

#[derive(Debug, Clone)]
#[non_exhaustive]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "musli", derive(musli::Encode, musli::Decode))]
pub enum ParsingError {
    UnrecognizedFormat { source: String },
    MissingRequiredField { field_name: String },
    MismatchedTypes { expected: String, got: String },
    ValueParsingError { on_field: String, err: String },
    UnknownField { field: String },
    UnknownRelationship { relationship: String },
    WrongRelationshipKind { relationship: String }, //TODO: add received and accepted arity/kind
    DeserializationError { source: String },
}

impl std::fmt::Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingRequiredField { field_name } => {
                write!(f, "required field \"{}\" is missing", field_name)
            }
            Self::MismatchedTypes { expected, got } => {
                write!(
                    f,
                    "expected a resource of type \"{}\" but got \"{}\"",
                    expected, got
                )
            }
            Self::UnrecognizedFormat { source } => {
                write!(f, "failed to parse provided format with: {}", source)
            }
            Self::ValueParsingError { on_field, err } => {
                write!(
                    f,
                    "failed to parse a value of field \"{}\" with error: {}",
                    on_field, err
                )
            }
            Self::UnknownField { field } => {
                write!(f, "provided attribute \"{field}\" is not accepted")
            }
            Self::DeserializationError { source } => {
                write!(
                    f,
                    "failed to deserialize from JSON with error: \"{source}\""
                )
            }
            Self::UnknownRelationship { relationship } => {
                write!(
                    f,
                    "cannot recognize a relationship \"{relationship}\" for this resource type"
                )
            }
            Self::WrongRelationshipKind { relationship } => {
                //TODO: improve
                write!(f, "relationship \"{relationship}\" has wrong arity")
            }
        }
    }
}

impl std::error::Error for ParsingError {}

#[cfg(feature = "axum")]
impl axum::response::IntoResponse for ParsingError {
    fn into_response(self) -> axum::response::Response {
        let res = ErrorResponse {
            status: axum::http::StatusCode::BAD_REQUEST.into(),
            errors: vec![match &self {
                Self::ValueParsingError {
                    on_field: _,
                    err: _,
                } => ErrorObject {
                    status: 400,
                    description: Some(self.to_string()),
                    title: "failed to parse a value".to_owned(),
                    ..Default::default()
                },
                Self::MissingRequiredField { field_name: _ } => ErrorObject {
                    status: 400,
                    title: "missing a required field".to_owned(),
                    description: Some(self.to_string()),
                    ..Default::default()
                },
                Self::UnrecognizedFormat { source: _ } => ErrorObject {
                    status: 400,
                    title: "unrecognized format".to_owned(),
                    description: Some(self.to_string()),
                    ..Default::default()
                },
                Self::MismatchedTypes {
                    expected: _,
                    got: _,
                } => ErrorObject {
                    status: 400,
                    title: "mismatched types".to_owned(),
                    description: Some(self.to_string()),
                    ..Default::default()
                },
                Self::UnknownField { field: _ } => ErrorObject {
                    status: 400,
                    title: "unknown field".to_owned(),
                    description: Some(self.to_string()),
                    ..Default::default()
                },
                Self::DeserializationError { source: _ } => ErrorObject {
                    status: 400,
                    title: "failed to deserialize".to_owned(),
                    description: Some(self.to_string()),
                    ..Default::default()
                },
                Self::UnknownRelationship { relationship: _ } => ErrorObject {
                    status: 400,
                    title: "unknown relationship".to_owned(),
                    description: Some(self.to_string()),
                    ..Default::default()
                },
                Self::WrongRelationshipKind { relationship: _ } => ErrorObject {
                    status: 400,
                    title: "wrong relationship kind".to_owned(),
                    description: Some(self.to_string()),
                    ..Default::default()
                },
            }],
        };
        #[cfg(feature = "musli")]
        let res = (
            axum::http::StatusCode::BAD_REQUEST,
            [(axum::http::header::CONTENT_TYPE, "application/json")],
            musli::json::to_string(&res).unwrap(),
        )
            .into_response();
        #[cfg(not(feature = "musli"))]
        let res = (axum::http::StatusCode::BAD_REQUEST, axum::Json(res)).into_response();

        res
    }
}

impl<T> From<T> for Resource
where
    T: AsResource,
{
    fn from(val: T) -> Self {
        let iden = val.resource_identifier();
        let attr = val.attributes();
        Self {
            r#type: iden.r#type.to_string(),
            id: iden.id,
            attributes: attr,
            relationships: None,
        }
    }
}

impl<STATUS: DataResponseStatus, R> From<Resource> for DataResponse<STATUS, R> {
    fn from(val: Resource) -> Self {
        DataResponse::<STATUS, R> {
            data: SingleOrCollection::Single(val),
            included: None,
            resource_ty: PhantomData,
            status: PhantomData,
            links: None,
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[cfg_attr(feature = "musli", derive(musli::Encode, musli::Decode))]
#[cfg_attr(feature = "musli", musli(transparent))]
pub struct AttributesMap(
    #[cfg_attr(feature = "musli", musli(with = musli::serde))]
    serde_json::Map<String, serde_json::Value>,
);

impl AttributesMap {
    pub fn new() -> Self {
        Self(serde_json::Map::<String, serde_json::Value>::new())
    }
    pub fn with_capacity(capacity: usize) -> Self {
        Self(serde_json::Map::<String, serde_json::Value>::with_capacity(
            capacity,
        ))
    }
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.0.get(key)
    }
    pub fn insert(&mut self, key: String, value: serde_json::Value) -> Option<serde_json::Value> {
        self.0.insert(key, value)
    }
    pub fn remove(&mut self, key: &str) -> Option<serde_json::Value> {
        self.0.remove(key)
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn keys(&'_ self) -> serde_json::map::Keys<'_> {
        self.0.keys()
    }
    pub fn values(&'_ self) -> serde_json::map::Values<'_> {
        self.0.values()
    }
    pub fn iter(&'_ self) -> serde_json::map::Iter<'_> {
        self.0.iter()
    }
    pub fn iter_mut(&'_ mut self) -> serde_json::map::IterMut<'_> {
        self.0.iter_mut()
    }
}

impl From<std::collections::HashMap<String, serde_json::Value>> for AttributesMap {
    fn from(val: std::collections::HashMap<String, serde_json::Value>) -> Self {
        let mut res = serde_json::Map::with_capacity(val.len());
        for (k, v) in val {
            res.insert(k, v);
        }
        Self(res)
    }
}

impl IntoIterator for AttributesMap {
    type Item = (String, serde_json::Value);
    type IntoIter = <serde_json::Map<String, serde_json::Value> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl From<serde_json::Map<String, serde_json::Value>> for AttributesMap {
    fn from(val: serde_json::Map<String, serde_json::Value>) -> Self {
        Self(val)
    }
}

#[cfg(not(feature = "serde"))]
impl FromIterator<(String, serde_json::Value)> for AttributesMap {
    fn from_iter<T: IntoIterator<Item = (String, serde_json::Value)>>(iter: T) -> Self {
        Self(serde_json::Map::<String, serde_json::Value>::from_iter(
            iter,
        ))
    }
}

#[cfg(not(feature = "serde"))]
impl FromIterator<(String, serde_json::Value)> for AttributesMap {
    fn from_iter<T: IntoIterator<Item = (String, serde_json::Value)>>(iter: T) -> Self {
        Self(serde_json::Map::<String, serde_json::Value>::from_iter(
            iter,
        ))
    }
}

#[cfg(not(feature = "serde"))]
impl FromIterator<(&'static str, serde_json::Value)> for AttributesMap {
    fn from_iter<T: IntoIterator<Item = (&'static str, serde_json::Value)>>(iter: T) -> Self {
        Self(serde_json::Map::<String, serde_json::Value>::from_iter(
            iter.into_iter().map(|(k, v)| (String::from(k), v)),
        ))
    }
}

#[cfg(feature = "serde")]
impl<S: serde::Serialize + for<'a> serde::Deserialize<'a>> FromIterator<(String, S)>
    for AttributesMap
{
    fn from_iter<T: IntoIterator<Item = (String, S)>>(iter: T) -> Self {
        Self(serde_json::Map::<String, serde_json::Value>::from_iter(
            iter.into_iter()
                .map(|(k, v)| (k, serde_json::value::to_value(v).unwrap())),
        ))
    }
}

#[cfg(feature = "serde")]
impl<S: serde::Serialize + for<'a> serde::Deserialize<'a>> FromIterator<(&'static str, S)>
    for AttributesMap
{
    fn from_iter<T: IntoIterator<Item = (&'static str, S)>>(iter: T) -> Self {
        Self(serde_json::Map::<String, serde_json::Value>::from_iter(
            iter.into_iter()
                .map(|(k, v)| (String::from(k), serde_json::value::to_value(v).unwrap())),
        ))
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[cfg_attr(feature = "musli", derive(musli::Encode, musli::Decode))]
#[cfg_attr(feature = "musli", musli(transparent))]
pub struct LinksMap(
    #[cfg_attr(feature = "musli", musli(with = musli::serde))]
    std::collections::HashMap<String, String>,
);

impl LinksMap {
    pub fn new() -> Self {
        Self(std::collections::HashMap::<String, String>::new())
    }
    pub fn with_capacity(capacity: usize) -> Self {
        Self(std::collections::HashMap::<String, String>::with_capacity(
            capacity,
        ))
    }
    pub fn get(&self, key: &str) -> Option<&String> {
        self.0.get(key)
    }
    pub fn insert(&mut self, key: String, value: String) -> Option<String> {
        self.0.insert(key, value)
    }
    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.0.remove(key)
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn keys(&'_ self) -> std::collections::hash_map::Keys<'_, String, String> {
        self.0.keys()
    }
    pub fn values(&'_ self) -> std::collections::hash_map::Values<'_, String, String> {
        self.0.values()
    }
    pub fn iter(&'_ self) -> std::collections::hash_map::Iter<'_, String, String> {
        self.0.iter()
    }
    pub fn iter_mut(&'_ mut self) -> std::collections::hash_map::IterMut<'_, String, String> {
        self.0.iter_mut()
    }
}

impl Extend<(String, String)> for LinksMap {
    fn extend<T: IntoIterator<Item = (String, String)>>(&mut self, iter: T) {
        self.0.extend(iter);
    }
}

impl From<std::collections::HashMap<String, String>> for LinksMap {
    fn from(val: std::collections::HashMap<String, String>) -> Self {
        Self(val)
    }
}

impl IntoIterator for LinksMap {
    type Item = (String, String);
    type IntoIter = <std::collections::HashMap<String, String> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<(String, String)> for LinksMap {
    fn from_iter<T: IntoIterator<Item = (String, String)>>(iter: T) -> Self {
        Self(std::collections::HashMap::<String, String>::from_iter(iter))
    }
}

#[cfg(all(test, feature = "derive"))]
mod test_derives {
    #[test]
    fn trybuild_tests() {
        let t = trybuild::TestCases::new();
        t.pass("tests/derives.rs");
    }
}
