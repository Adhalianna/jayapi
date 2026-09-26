use core::panic;
use std::fmt::Debug;

use darling::{
    util::{Override, SpannedValue},
    FromDeriveInput, FromMeta,
};
use proc_macro2::Span;
use quote::{quote, ToTokens};

extern crate proc_macro;

#[derive(darling::FromDeriveInput, Clone, Debug)]
#[darling(attributes(jayapi), forward_attrs(doc, allow, cfg))]
struct ResourceDerive {
    ident: syn::Ident,
    generics: syn::Generics,
    data: darling::ast::Data<darling::util::Ignored, DeriveField>,
    #[darling(default)]
    resource_type: Option<String>,
}

#[derive(darling::FromField, Clone, Debug)]
#[darling(attributes(jayapi))]
struct DeriveField {
    ident: Option<syn::Ident>,
    ty: syn::Type,
    skip: darling::util::Flag,
    id: Option<SpannedValue<Override<IdFieldAttr>>>,
    lid: Option<SpannedValue<Override<LidFieldAttr>>>,
    attribute: Option<SpannedValue<Override<AttributeFieldAttr>>>,
    relationship: Option<SpannedValue<Override<RelationshipFieldAttr>>>,
}

impl TryFrom<&DeriveField> for DeriveFieldVariant {
    type Error = darling::Error;
    fn try_from(value: &DeriveField) -> Result<Self, Self::Error> {
        if (value.id.is_some() && value.attribute.is_some())
            || (value.id.is_some() && value.relationship.is_some())
            || (value.attribute.is_some() && value.relationship.is_some())
        {
            if let Some(v) = value.id.as_ref() {
                return Err(darling::Error::custom(
                    "the 'id' field cannot be also an 'attribute' nor 'relationship'",
                )
                .with_span(&v.span())
                .at("id"));
            }
            if let Some(v) = value.attribute.as_ref() {
                return Err(darling::Error::custom(
                    "the 'attribute' field cannot be also a 'relationship'",
                )
                .with_span(&v.span())
                .at("attribute"));
            }
        }
        if (value.lid.is_some() && value.attribute.is_some())
            || (value.lid.is_some() && value.relationship.is_some())
        {
            if let Some(v) = value.id.as_ref() {
                return Err(darling::Error::custom(
                    "the 'lid' field cannot be also an 'attribute' nor 'relationship'",
                )
                .with_span(&v.span())
                .at("lid"));
            }
        }
        if value.skip.is_present() {
            return Ok(Self::Skip);
        }
        if let Some(id) = &value.id {
            if let Some(lid) = &value.lid {
                return match (id.as_ref(), lid.as_ref()) {
                    (Override::Inherit, Override::Inherit) => Ok(Self::IdAndLid(
                        SpannedValue::new(IdFieldAttr::default(), id.span()),
                        SpannedValue::new(LidFieldAttr::default(), lid.span()),
                    )),
                    (Override::Inherit, Override::Explicit(lidv)) => Ok(Self::IdAndLid(
                        SpannedValue::new(IdFieldAttr::default(), id.span()),
                        SpannedValue::new(lidv.clone(), lid.span()),
                    )),
                    (Override::Explicit(idv), Override::Inherit) => Ok(Self::IdAndLid(
                        SpannedValue::new(idv.clone(), id.span()),
                        SpannedValue::new(LidFieldAttr::default(), lid.span()),
                    )),
                    (Override::Explicit(idv), Override::Explicit(lidv)) => Ok(Self::IdAndLid(
                        SpannedValue::new(idv.clone(), id.span()),
                        SpannedValue::new(lidv.clone(), lid.span()),
                    )),
                };
            }
            return match id.as_ref() {
                Override::Inherit => Ok(Self::Id(SpannedValue::new(
                    IdFieldAttr::default(),
                    id.span(),
                ))),
                Override::Explicit(v) => Ok(Self::Id(SpannedValue::new(v.clone(), id.span()))),
            };
        }
        if let Some(lid) = &value.lid {
            return match lid.as_ref() {
                Override::Inherit => Ok(Self::Lid(SpannedValue::new(
                    LidFieldAttr::default(),
                    lid.span(),
                ))),
                Override::Explicit(v) => Ok(Self::Lid(SpannedValue::new(v.clone(), lid.span()))),
            };
        }
        if let Some(attr) = &value.attribute {
            return match attr.as_ref() {
                Override::Inherit => Ok(Self::Attribute(SpannedValue::new(
                    AttributeFieldAttr::default(),
                    attr.span(),
                ))),
                Override::Explicit(v) => {
                    Ok(Self::Attribute(SpannedValue::new(v.clone(), attr.span())))
                }
            };
        }
        if let Some(rel) = &value.relationship {
            return match rel.as_ref() {
                Override::Inherit => Ok(Self::Relationship(SpannedValue::new(
                    RelationshipFieldAttr::default(),
                    rel.span(),
                ))),
                Override::Explicit(v) => {
                    Ok(Self::Relationship(SpannedValue::new(v.clone(), rel.span())))
                }
            };
        }
        return Ok(Self::Attribute(SpannedValue::new(
            AttributeFieldAttr {
                name: None,
                serialize_method: None,
                deserialize_method: None,
                default: false.into(),
                omit_none: false.into(),
            },
            Span::call_site(),
        )));
    }
}

enum DeriveFieldVariant {
    Skip,
    Id(SpannedValue<IdFieldAttr>),
    Lid(SpannedValue<LidFieldAttr>),
    IdAndLid(SpannedValue<IdFieldAttr>, SpannedValue<LidFieldAttr>),
    Attribute(SpannedValue<AttributeFieldAttr>),
    Relationship(SpannedValue<RelationshipFieldAttr>),
}

#[derive(darling::FromMeta, Clone, Debug, Default)]
#[darling(default)]
struct IdFieldAttr {
    #[darling(rename = "parse_with")]
    parse_method: Option<syn::Path>,
    #[darling(rename = "to_string_with")]
    to_string_method: Option<syn::Path>,
}

impl IdFieldAttr {
    fn into_data(
        &self,
        field_ident: IndexOrField,
        field_type: &syn::Type,
        resource_type: String,
    ) -> IdData {
        IdData {
            field: field_ident,
            parse_method: self.parse_method.to_owned().unwrap_or_else(|| {
                syn::Path::from_string(
                    &("str::parse::<".to_owned() + &field_type.to_token_stream().to_string() + ">"),
                )
                .unwrap()
            }),
            to_string_method: self.to_string_method.to_owned().unwrap_or_else(|| {
                syn::Path::from_string("::std::string::ToString::to_string").unwrap()
            }),
            resource_type: resource_type.clone(),
            field_type: field_type.clone(),
        }
    }
}

struct IdData {
    field: IndexOrField,
    field_type: syn::Type,
    resource_type: String,
    parse_method: syn::Path,
    to_string_method: syn::Path,
}

impl IdData {
    #[cfg(feature = "json-schema")]
    fn into_id_type_json_schema_part(&self) -> proc_macro2::TokenStream {
        let value_type = &self.field_type;
        let value_type = quote! { #value_type };

        quote! {<#value_type as ::schemars::JsonSchema>::json_schema(generator)}
    }
    fn into_as_resource_tokens_identifier_impl_body(&self) -> proc_macro2::TokenStream {
        let resource_type = &self.resource_type;
        let id_field = &self.field;
        let to_string_method = &self.to_string_method;

        quote! {
            ::jayapi::ResourceIdentifier {
                 r#type: #resource_type.to_owned(),
                 id: #to_string_method(&self.#id_field),
            }
        }
    }
    fn into_from_resource_tokens(&self) -> proc_macro2::TokenStream {
        let parse_method = &self.parse_method;
        let id_field = &self.field;

        quote! {
            #id_field: #parse_method(&resource.id).map_err(|e| ::jayapi::ParsingError::ValueParsingError {
                on_field: "id".to_owned(),
                err: e.to_string(),
            })?,
        }
    }
}

#[derive(darling::FromMeta, Clone, Debug, Default)]
#[darling(default)]
struct LidFieldAttr {
    #[darling(rename = "parse_with")]
    parse_method: Option<syn::Path>,
    #[darling(rename = "to_string_with")]
    to_string_method: Option<syn::Path>,
}

impl LidFieldAttr {
    fn into_data(
        &self,
        field_ident: IndexOrField,
        field_type: &syn::Type,
        resource_type: String,
    ) -> LidData {
        LidData {
            field: field_ident,
            parse_method: self.parse_method.to_owned().unwrap_or_else(|| {
                syn::Path::from_string(
                    &("str::parse::<".to_owned() + &field_type.to_token_stream().to_string() + ">"),
                )
                .unwrap()
            }),
            to_string_method: self.to_string_method.to_owned().unwrap_or_else(|| {
                syn::Path::from_string("::std::string::ToString::to_string").unwrap()
            }),
            resource_type: resource_type.clone(),
            field_type: field_type.clone(),
        }
    }
}

struct LidData {
    field: IndexOrField,
    field_type: syn::Type,
    resource_type: String,
    parse_method: syn::Path,
    to_string_method: syn::Path,
}

impl LidData {
    #[cfg(feature = "json-schema")]
    fn into_id_type_json_schema_part(&self) -> proc_macro2::TokenStream {
        let value_type = &self.field_type;
        let value_type = quote! { #value_type };

        quote! {<#value_type as ::schemars::JsonSchema>::json_schema(generator)}
    }
    fn into_as_local_resource_tokens_identifier_impl_body(&self) -> proc_macro2::TokenStream {
        let resource_type = &self.resource_type;
        let lid_field = &self.field;
        let to_string_method = &self.to_string_method;

        quote! {
            ::std::option::Option::Some(::jayapi::LocalResourceIdentifier {
                 r#type: #resource_type.to_owned(),
                 lid: #to_string_method(&self.#lid_field),
            })
        }
    }
    fn into_from_local_resource_tokens(&self) -> proc_macro2::TokenStream {
        let parse_method = &self.parse_method;
        let lid_field = &self.field;
        let resource_type = &self.resource_type;

        quote! {
            #lid_field: resource.lid.as_ref().map(|lid| #parse_method(lid).map_err(|e| ::jayapi::ParsingError::ValueParsingError {
                on_field: "lid".to_owned(),
                err: e.to_string(),
            })).ok_or_else(|| ::jayapi::ParsingError::LidRequired{ resource_type: #resource_type.to_owned() })??,
        }
    }
}

#[derive(darling::FromMeta, Clone, Debug, Default)]
#[darling(default)]
struct AttributeFieldAttr {
    name: Option<String>,
    #[darling(rename = "serialize_with")]
    serialize_method: Option<syn::Path>,
    #[darling(rename = "deserialize_with")]
    deserialize_method: Option<syn::Path>,
    default: darling::util::Flag,
    omit_none: darling::util::Flag,
}

impl AttributeFieldAttr {
    fn into_data(&self, field_ident: IndexOrField, field_type: &syn::Type) -> AttributeData {
        AttributeData {
            name: self.name.to_owned().unwrap_or_else(|| {
                match &field_ident {
                    IndexOrField::Index(_) => panic!("an attribute field must be either a named field of a struct or have a name assigned using 'name'"),
                    IndexOrField::Field(field) => field.to_string(),
                }
            }),
            field: field_ident,
            serialize_method: self.serialize_method.to_owned().unwrap_or_else(|| syn::Path::from_string("::jayapi::serde_json::value::to_value").unwrap()),
            deserialize_method: self.deserialize_method.to_owned().unwrap_or_else(|| syn::Path::from_string(&("::jayapi::serde_json::value::from_value::<".to_owned()
                            + &field_type.to_token_stream().to_string()
                            + ">")).unwrap()),
            default: self.default.is_present(),
            omit_none: self.omit_none.is_present(),
            field_type: field_type.clone(),
        }
    }
}

struct AttributeData {
    name: String,
    field: IndexOrField,
    field_type: syn::Type,
    serialize_method: syn::Path,
    deserialize_method: syn::Path,
    default: bool,
    omit_none: bool,
}

impl AttributeData {
    #[cfg(feature = "json-schema")]
    fn into_json_schema_part(&self) -> proc_macro2::TokenStream {
        let name = &self.name;
        let value_type = &self.field_type;
        let value_type = quote! { #value_type };

        quote! {
            #name: <#value_type as ::schemars::JsonSchema>::json_schema(generator),
        }
    }
    fn into_as_resource_tokens(&self) -> proc_macro2::TokenStream {
        let field = &self.field;
        let name = &self.name;
        let serialize_method = &self.serialize_method;

        if self.omit_none {
            quote! {
                if self.#field.is_some() {
                    map.insert(
                        #name.to_owned(),
                        #serialize_method(&self.#field).unwrap(),
                    );
                }
            }
        } else {
            quote! {
                map.insert(
                    #name.to_owned(),
                    #serialize_method(&self.#field).unwrap(),
                );
            }
        }
    }
    fn storage_var(&self) -> syn::Ident {
        syn::Ident::new(
            &("__".to_owned() + &self.field.to_token_stream().to_string()),
            proc_macro2::Span::call_site(),
        )
    }
    fn into_from_resource_tokens_storage_var(&self) -> proc_macro2::TokenStream {
        let var_name = &self.storage_var();
        let var_type = &self.field_type;

        if self.default {
            quote! {
                let mut #var_name: #var_type = ::std::default::Default::default();
            }
        } else {
            quote! {
                let mut #var_name: ::std::option::Option<#var_type> = ::std::option::Option::None;
            }
        }
    }
    fn into_from_resource_tokens_match_arm(&self) -> proc_macro2::TokenStream {
        let attr_name = &self.name;
        let var_name = &self.storage_var();
        let deserialize_method = &self.deserialize_method;

        if self.default {
            quote! {
                #attr_name => {
                    #var_name = #deserialize_method(v).map_err(|e| ::jayapi::ParsingError::ValueParsingError {
                        err: e.to_string(),
                        on_field: #attr_name.to_owned(),
                    })?;
                }
            }
        } else {
            quote! {
                #attr_name => {
                    #var_name = Some(#deserialize_method(v).map_err(|e| ::jayapi::ParsingError::ValueParsingError {
                        err: e.to_string(),
                        on_field: #attr_name.to_owned(),
                    })?);
                }
            }
        }
    }
    fn into_from_resource_tokens_var_assign(&self) -> proc_macro2::TokenStream {
        let field = &self.field;
        let var_name = &self.storage_var();
        let attr_name = &self.name;

        if self.default {
            quote! {
                #field: #var_name,
            }
        } else {
            quote! {
                #field: #var_name.ok_or(jayapi::ParsingError::MissingRequiredField {
                    field_name: #attr_name.to_owned(),
                })?,
            }
        }
    }
}

#[derive(darling::FromMeta, Clone, Debug, Default)]
#[darling(default)]
struct RelationshipFieldAttr {
    name: Option<String>,
    resource_type: Option<String>,
    local: darling::util::Flag,
    to_many: darling::util::Flag,
    optional: darling::util::Flag,
    #[darling(rename = "to_string_with")]
    to_string_method: Option<syn::Path>,
    #[darling(rename = "parse_with")]
    parse_method: Option<syn::Path>,
}

impl RelationshipFieldAttr {
    fn into_data(&self, field_ident: IndexOrField, field_type: &syn::Type) -> RelationshipData {
        let name = self.name.to_owned().unwrap_or_else(|| match &field_ident {
            IndexOrField::Index(_) => {
                panic!("a relationship requires a name")
            }
            IndexOrField::Field(field) => field.to_string(),
        });
        let mut single_item_type = field_type.to_token_stream().to_string();
        if self.to_many.is_present() {
            single_item_type =
                "<".to_owned() + &single_item_type + " as ::std::iter::IntoIterator>::Item";
        }
        if self.optional.is_present() {
            single_item_type = single_item_type
                .replace(" ", "")
                .strip_suffix(">")
                .and_then(|ty| {
                    ty.strip_prefix("::std::option::Option<")
                        .or_else(|| ty.strip_prefix("std::option::Option<"))
                        .or_else(|| ty.strip_prefix("option::Option<"))
                        .or_else(|| ty.strip_prefix("Option<"))
                }).expect("failed to extract the type wrapped in Option for a relationship marked as optional")
                .to_owned();
        }

        if self.optional.is_present() && self.to_many.is_present() {
            panic!("\"to_many\" relationships cannot be marked as optional, only 1-to-1 relations can be wrapped in std::option::Option");
        }

        RelationshipData {
            name: name.clone(),
            field: field_ident,
            resource_type: self.resource_type.clone().unwrap_or(name),
            to_many: self.to_many.is_present(),
            optional: self.optional.is_present(),
            local: self.local.is_present(),
            parse_method: self.parse_method.to_owned().unwrap_or(
                syn::Path::from_string(&("str::parse::<".to_owned() + &single_item_type + ">"))
                    .unwrap(),
            ),
            to_string_method: self
                .to_string_method
                .to_owned()
                .unwrap_or(syn::Path::from_string("::std::string::ToString::to_string").unwrap()),
        }
    }
}

struct RelationshipData {
    name: String,
    field: IndexOrField,
    resource_type: String,
    to_many: bool,
    optional: bool,
    local: bool,
    to_string_method: syn::Path,
    parse_method: syn::Path,
}

impl RelationshipData {
    #[cfg(feature = "json-schema")]
    fn into_json_schema_part(&self) -> proc_macro2::TokenStream {
        let name = &self.name;
        let resource_type = &self.resource_type;

        if self.to_many {
            quote! {
                    #name: {
                        "type": "object",
                        "properties": {
                            "data": {
                                "type": "array",
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "type": {
                                            "const": #resource_type,
                                        },
                                        "id": {
                                            "type": "string",
                                        },
                                    },
                                    "required": ["type", "id"],
                                    "additionalProperties": false,
                                }
                            }
                        }
                    },
            }
        } else if self.optional {
            quote! {
                    #name: {
                        "type": [ "object", "null" ],
                        "properties": {
                            "data": {
                                    "type": "object",
                                    "properties": {
                                        "type": {
                                            "const": #resource_type,
                                        },
                                        "id": {
                                            "type": "string",
                                        },
                                    },
                                    "required": ["type", "id"],
                                    "additionalProperties": false,
                            }
                        }
                    },
            }
        } else {
            quote! {
                    #name: {
                        "type": "object",
                        "properties": {
                            "data": {
                                    "type": "object",
                                    "properties": {
                                        "type": {
                                            "const": #resource_type,
                                        },
                                        "id": {
                                            "type": "string",
                                        },
                                    },
                                    "required": ["type", "id"],
                                    "additionalProperties": false,
                            }
                        }
                    },
            }
        }
    }
    fn into_as_resource_tokens(&self) -> proc_macro2::TokenStream {
        let name = &self.name;
        let resource_type = &self.resource_type;
        let to_string_method = &self.to_string_method;
        let field = &self.field;

        if self.to_many {
            quote! {
                map.insert(
                    #name.to_owned(),
                    ::jayapi::Relationship::Relation1toM {
                        data: self.#field.iter().map(|item| {
                            ::jayapi::ResourceIdentifier{
                                r#type: #resource_type.to_owned(),
                                id: #to_string_method(item)
                            }
                        }).collect(),
                    }
                );
            }
        } else if self.optional {
            quote! {
                if let ::std::option::Option::Some(id) = &self.#field {
                    map.insert(
                        #name.to_owned(),
                        ::jayapi::Relationship::Relation1to1 {
                            data: ::jayapi::ResourceIdentifier {
                                r#type: #resource_type.to_owned(),
                                id: #to_string_method(id),
                            }
                        }
                    );
                }
            }
        } else {
            quote! {
                map.insert(
                    #name.to_owned(),
                    ::jayapi::Relationship::Relation1to1 {
                        data: ::jayapi::ResourceIdentifier {
                            r#type: #resource_type.to_owned(),
                            id: #to_string_method(&self.#field),
                        }
                    }
                );
            }
        }
    }
    fn into_as_local_resource_tokens(&self) -> proc_macro2::TokenStream {
        let name = &self.name;
        let resource_type = &self.resource_type;
        let to_string_method = &self.to_string_method;
        let field = &self.field;

        if self.local {
            if self.to_many {
                quote! {
                    map.insert(
                        #name.to_owned(),
                        ::jayapi::LocalOrGlobalRelationship::Local(::jayapi::LocalRelationship::Relation1toM {
                            data: self.#field.iter().map(|item| {
                                ::jayapi::LocalResourceIdentifier{
                                    r#type: #resource_type.to_owned(),
                                    lid: #to_string_method(item)
                                }
                            }).collect(),
                        })
                    );
                }
            } else if self.optional {
                quote! {
                    if let ::std::option::Option::Some(id) = &self.#field {
                        map.insert(
                            #name.to_owned(),
                            ::jayapi::LocalOrGlobalRelationship::Local(::jayapi::LocalRelationship::Relation1to1 {
                                data: ::jayapi::LocalResourceIdentifier {
                                    r#type: #resource_type.to_owned(),
                                    lid: #to_string_method(id),
                                }
                            })
                        );
                    }
                }
            } else {
                quote! {
                    map.insert(
                        #name.to_owned(),
                        ::jayapi::LocalOrGlobalRelationship::Local(::jayapi::LocalRelationship::Relation1to1 {
                            data: ::jayapi::LocalResourceIdentifier {
                                r#type: #resource_type.to_owned(),
                                lid: #to_string_method(&self.#field),
                            }
                        })
                    );
                }
            }
        } else {
            if self.to_many {
                quote! {
                    map.insert(
                        #name.to_owned(),
                        ::jayapi::LocalOrGlobalRelationship::Global(::jayapi::Relationship::Relation1toM {
                            data: self.#field.iter().map(|item| {
                                ::jayapi::ResourceIdentifier{
                                    r#type: #resource_type.to_owned(),
                                    id: #to_string_method(item)
                                }
                            }).collect(),
                        })
                    );
                }
            } else if self.optional {
                quote! {
                    if let ::std::option::Option::Some(id) = &self.#field {
                        map.insert(
                            #name.to_owned(),
                            ::jayapi::LocalOrGlobalRelationship::Global(::jayapi::Relationship::Relation1to1 {
                                data: ::jayapi::ResourceIdentifier {
                                    r#type: #resource_type.to_owned(),
                                    id: #to_string_method(id),
                                }
                            })
                        );
                    }
                }
            } else {
                quote! {
                    map.insert(
                        #name.to_owned(),
                        ::jayapi::LocalOrGlobalRelationship::Global(::jayapi::Relationship::Relation1to1 {
                            data: ::jayapi::ResourceIdentifier {
                                r#type: #resource_type.to_owned(),
                                id: #to_string_method(&self.#field),
                            }
                        })
                    );
                }
            }
        }
    }
    fn storage_var(&self) -> syn::Ident {
        syn::Ident::new(
            &("__".to_owned() + &self.field.to_token_stream().to_string()),
            proc_macro2::Span::call_site(),
        )
    }
    fn into_from_resource_tokens_storage_var(&self) -> proc_macro2::TokenStream {
        let var_name = &self.storage_var();
        if self.optional {
            quote! {
                let mut #var_name = ::std::option::Option::None;
            }
        } else {
            quote! {
                let mut #var_name = ::std::default::Default::default();
            }
        }
    }
    fn into_from_resource_tokens_match_arm(&self) -> proc_macro2::TokenStream {
        let relation_name = &self.name;
        let var_name = &self.storage_var();
        let parse_method = &self.parse_method;

        if self.to_many {
            quote! {
                #relation_name => {
                    match v {
                        ::jayapi::Relationship::Relation1toM { data: data } => {
                            #var_name = data.iter().map(|item| #parse_method(&item.id).map_err(|e| ::jayapi::ParsingError::ValueParsingError {
                                err: e.to_string(),
                                on_field: #relation_name.to_owned(),
                            })).collect::<Result<_, ::jayapi::ParsingError>>()?;
                        }
                        _ => {
                            return Err(::jayapi::ParsingError::WrongRelationshipKind {
                                relationship: #relation_name.to_owned(),
                            });
                        }
                    }
                }
            }
        } else if self.optional {
            quote! {
                #relation_name => {
                    match v {
                        ::jayapi::Relationship::Relation1to1 { data: data } => {
                            #var_name = ::std::option::Option::Some(
                                #parse_method(&data.id).map_err(|e| ::jayapi::ParsingError::ValueParsingError {
                                    err: e.to_string(),
                                    on_field: #relation_name.to_owned(),
                                })?
                            );
                        }
                        _ => {
                            return Err(::jayapi::ParsingError::WrongRelationshipKind {
                                relationship: #relation_name.to_owned(),
                            });
                        }
                    }
                }
            }
        } else {
            quote! {
                #relation_name => {
                    match v {
                        ::jayapi::Relationship::Relation1to1 { data: data } => {
                            #var_name = #parse_method(&data.id).map_err(|e| ::jayapi::ParsingError::ValueParsingError {
                                err: e.to_string(),
                                on_field: #relation_name.to_owned(),
                            })?;
                        }
                        _ => {
                            return Err(::jayapi::ParsingError::WrongRelationshipKind {
                                relationship: #relation_name.to_owned(),
                            });
                        }
                    }
                }
            }
        }
    }
    fn into_from_local_resource_tokens_match_arm(&self) -> proc_macro2::TokenStream {
        let relation_name = &self.name;
        let var_name = &self.storage_var();
        let parse_method = &self.parse_method;

        if self.to_many {
            quote! {
                #relation_name => {
                    match v {
                        ::jayapi::LocalOrGlobalRelationship::Local(rel) => match rel {
                            ::jayapi::LocalRelationship::Relation1toM { data: data } => {
                                #var_name = data.iter().map(|item| #parse_method(&item.lid).map_err(|e| ::jayapi::ParsingError::ValueParsingError {
                                    err: e.to_string(),
                                    on_field: #relation_name.to_owned(),
                                })).collect::<Result<_, ::jayapi::ParsingError>>()?;
                            },
                            _ => {
                                return Err(::jayapi::ParsingError::WrongRelationshipKind {
                                    relationship: #relation_name.to_owned(),
                                });
                            }
                        },
                        ::jayapi::LocalOrGlobalRelationship::Global(rel) => match rel {
                            ::jayapi::Relationship::Relation1toM { data: data } => {
                                #var_name = data.iter().map(|item| #parse_method(&item.id).map_err(|e| ::jayapi::ParsingError::ValueParsingError {
                                    err: e.to_string(),
                                    on_field: #relation_name.to_owned(),
                                })).collect::<Result<_, ::jayapi::ParsingError>>()?;
                            },
                            _ => {
                                return Err(::jayapi::ParsingError::WrongRelationshipKind {
                                    relationship: #relation_name.to_owned(),
                                });
                            }
                        }
                    }
                }
            }
        } else if self.optional {
            quote! {
                #relation_name => {
                    match v {
                        ::jayapi::LocalOrGlobalRelationship::Global(rel) => match rel {
                            ::jayapi::Relationship::Relation1to1 { data: data } => {
                                #var_name = ::std::option::Option::Some(
                                    #parse_method(&data.id).map_err(|e| ::jayapi::ParsingError::ValueParsingError {
                                        err: e.to_string(),
                                        on_field: #relation_name.to_owned(),
                                    })?
                                );
                            },
                            _ => {
                                return Err(::jayapi::ParsingError::WrongRelationshipKind {
                                    relationship: #relation_name.to_owned(),
                                });
                            }
                        },
                        ::jayapi::LocalOrGlobalRelationship::Local(rel) => match rel {
                            ::jayapi::LocalRelationship::Relation1to1 { data: data } => {
                                #var_name = ::std::option::Option::Some(
                                    #parse_method(&data.lid).map_err(|e| ::jayapi::ParsingError::ValueParsingError {
                                        err: e.to_string(),
                                        on_field: #relation_name.to_owned(),
                                    })?
                                );
                            }
                            _ => {
                                return Err(::jayapi::ParsingError::WrongRelationshipKind {
                                    relationship: #relation_name.to_owned(),
                                });
                            }
                        }
                    }
                }
            }
        } else {
            quote! {
                #relation_name => {
                    match v {
                        ::jayapi::LocalOrGlobalRelationship::Local(rel) => match rel {
                            ::jayapi::LocalRelationship::Relation1to1 { data: data } => {
                                #var_name = #parse_method(&data.lid).map_err(|e| ::jayapi::ParsingError::ValueParsingError {
                                    err: e.to_string(),
                                    on_field: #relation_name.to_owned(),
                                })?;
                            },
                            _ => {
                                return Err(::jayapi::ParsingError::WrongRelationshipKind {
                                    relationship: #relation_name.to_owned(),
                                });
                            }
                        },
                        ::jayapi::LocalOrGlobalRelationship::Global(rel) => match rel {
                            ::jayapi::Relationship::Relation1to1 { data: data } => {
                                #var_name = #parse_method(&data.id).map_err(|e| ::jayapi::ParsingError::ValueParsingError {
                                    err: e.to_string(),
                                    on_field: #relation_name.to_owned(),
                                })?;
                            },
                            _ => {
                                return Err(::jayapi::ParsingError::WrongRelationshipKind {
                                    relationship: #relation_name.to_owned(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    fn into_from_resource_tokens_var_assign(&self) -> proc_macro2::TokenStream {
        let field = &self.field;
        let var_name = &self.storage_var();

        quote! {
            #field: #var_name,
        }
    }
}

#[derive(Clone)]
enum IndexOrField {
    Index(syn::Index),
    Field(syn::Ident),
}

impl quote::ToTokens for IndexOrField {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Index(idx) => idx.to_tokens(tokens),
            Self::Field(field) => field.to_tokens(tokens),
        }
    }
}

#[proc_macro_derive(ResourceType, attributes(jayapi))]
pub fn resource_type_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: syn::DeriveInput = match syn::parse2(input.into()) {
        Ok(input) => input,
        Err(err) => {
            return err.to_compile_error().into();
        }
    };
    let input = match ResourceDerive::from_derive_input(&input) {
        Ok(input) => input,
        Err(err) => {
            return err.write_errors().into();
        }
    };

    let struct_name = &input.ident;
    let generics = &input.generics;

    let resource_type = input
        .resource_type
        .unwrap_or_else(|| input.ident.to_string());

    let expanded_impl = quote::quote! {
        #[automatically_derived]
        impl #generics ::jayapi::ResourceType for #struct_name #generics {
            fn ty() -> &'static str {
                #resource_type
            }
         }
    };

    proc_macro::TokenStream::from(expanded_impl)
}

#[proc_macro_derive(AsResource, attributes(jayapi))]
pub fn as_resource_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: syn::DeriveInput = match syn::parse2(input.into()) {
        Ok(input) => input,
        Err(err) => {
            return err.to_compile_error().into();
        }
    };
    let input = match ResourceDerive::from_derive_input(&input) {
        Ok(input) => input,
        Err(err) => {
            return err.write_errors().into();
        }
    };

    let struct_name = &input.ident;
    let generics = &input.generics;

    let resource_type = input
        .resource_type
        .unwrap_or_else(|| input.ident.to_string());

    let mut resource_id_field: Option<IdData> = None;
    let mut relationships: Vec<RelationshipData> = Vec::new();
    let mut attr_fields: Vec<AttributeData> = Vec::new();

    let struct_fields = input
        .data
        .take_struct()
        .expect("only structs are supported as input");

    for (idx, field) in struct_fields.iter().enumerate() {
        let field_or_idx = field
            .ident
            .clone()
            .map(IndexOrField::Field)
            .unwrap_or(IndexOrField::Index(syn::Index::from(idx)));

        let field_variant: DeriveFieldVariant = match field.try_into() {
            Ok(field) => field,
            Err(err) => {
                return err.write_errors().into();
            }
        };
        match field_variant {
            DeriveFieldVariant::Id(id_attr) | DeriveFieldVariant::IdAndLid(id_attr, _) => {
                resource_id_field =
                    Some(id_attr.into_data(field_or_idx.clone(), &field.ty, resource_type.clone()));
            }
            DeriveFieldVariant::Attribute(attribute_attr) => {
                attr_fields.push(attribute_attr.into_data(field_or_idx.clone(), &field.ty));
            }
            DeriveFieldVariant::Relationship(rel_attr) => {
                relationships.push(rel_attr.into_data(field_or_idx.clone(), &field.ty));
            }
            DeriveFieldVariant::Lid(_) => {
                continue;
            }
            DeriveFieldVariant::Skip => {
                continue;
            }
        }
    }

    let mut expanded_attrs = Vec::with_capacity(attr_fields.len());
    for attr in attr_fields {
        expanded_attrs.push(attr.into_as_resource_tokens());
    }

    let mut expanded_rels = Vec::with_capacity(relationships.len());
    for rel in relationships {
        expanded_rels.push(rel.into_as_resource_tokens());
    }

    let expanded_resource_identifier_impl_body = resource_id_field
        .expect("a single field must be marked as an id field")
        .into_as_resource_tokens_identifier_impl_body();

    let expanded_impls = quote::quote! {
        #[automatically_derived]
        impl #generics ::jayapi::AsResource for #struct_name #generics {
            fn resource_identifier(&self) -> ::jayapi::ResourceIdentifier {
                #expanded_resource_identifier_impl_body
            }
            fn attributes(&self) -> ::std::option::Option<::jayapi::AttributesMap> {
                let mut map = ::jayapi::AttributesMap::new();
                #(#expanded_attrs)*
                if map.len() > 0 {
                    ::std::option::Option::Some(map)
                } else {
                    ::std::option::Option::None
                }
            }
            fn relationships(&self) -> ::std::option::Option<::jayapi::RelationshipsMap> {
                let mut map = ::jayapi::RelationshipsMap::new();
                #(#expanded_rels)*
                if map.len() > 0 {
                    ::std::option::Option::Some(map)
                } else {
                    ::std::option::Option::None
                }
            }
         }
    };

    proc_macro::TokenStream::from(expanded_impls)
}

#[proc_macro_derive(AsLocalResource, attributes(jayapi))]
pub fn as_local_resource_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: syn::DeriveInput = match syn::parse2(input.into()) {
        Ok(input) => input,
        Err(err) => {
            return err.to_compile_error().into();
        }
    };
    let input = match ResourceDerive::from_derive_input(&input) {
        Ok(input) => input,
        Err(err) => {
            return err.write_errors().into();
        }
    };

    let struct_name = &input.ident;
    let generics = &input.generics;

    let resource_type = input
        .resource_type
        .unwrap_or_else(|| input.ident.to_string());

    let mut resource_lid_field: Option<LidData> = None;
    let mut relationships: Vec<RelationshipData> = Vec::new();
    let mut attr_fields: Vec<AttributeData> = Vec::new();

    let struct_fields = input
        .data
        .take_struct()
        .expect("only structs are supported as input");

    for (idx, field) in struct_fields.iter().enumerate() {
        let field_or_idx = field
            .ident
            .clone()
            .map(IndexOrField::Field)
            .unwrap_or(IndexOrField::Index(syn::Index::from(idx)));

        let field_variant: DeriveFieldVariant = match field.try_into() {
            Ok(field) => field,
            Err(err) => {
                return err.write_errors().into();
            }
        };
        match field_variant {
            DeriveFieldVariant::Lid(lid_attr) | DeriveFieldVariant::IdAndLid(_, lid_attr) => {
                resource_lid_field = Some(lid_attr.into_data(
                    field_or_idx.clone(),
                    &field.ty,
                    resource_type.clone(),
                ));
            }
            DeriveFieldVariant::Attribute(attribute_attr) => {
                attr_fields.push(attribute_attr.into_data(field_or_idx.clone(), &field.ty));
            }
            DeriveFieldVariant::Relationship(rel_attr) => {
                relationships.push(rel_attr.into_data(field_or_idx.clone(), &field.ty));
            }
            DeriveFieldVariant::Id(_) => {
                continue;
            }
            DeriveFieldVariant::Skip => {
                continue;
            }
        }
    }

    let mut expanded_attrs = Vec::with_capacity(attr_fields.len());
    for attr in attr_fields {
        expanded_attrs.push(attr.into_as_resource_tokens());
    }

    let mut expanded_rels = Vec::with_capacity(relationships.len());
    for rel in relationships {
        expanded_rels.push(rel.into_as_local_resource_tokens());
    }

    let expanded_resource_identifier_impl_body = resource_lid_field
        .map(|lid_data| lid_data.into_as_local_resource_tokens_identifier_impl_body())
        .unwrap_or_else(|| {
            quote! {
                None
            }
        });

    let expanded_impls = quote::quote! {
        #[automatically_derived]
        impl #generics ::jayapi::AsLocalResource for #struct_name #generics {
            fn local_resource_identifier(&self) -> ::std::option::Option<::jayapi::LocalResourceIdentifier> {
                #expanded_resource_identifier_impl_body
            }
            fn attributes(&self) -> ::std::option::Option<::jayapi::AttributesMap> {
                let mut map = ::jayapi::AttributesMap::new();
                #(#expanded_attrs)*
                if map.len() > 0 {
                    ::std::option::Option::Some(map)
                } else {
                    ::std::option::Option::None
                }
            }
            fn relationships(&self) -> ::std::option::Option<::jayapi::LocalRelationshipsMap> {
                let mut map = ::jayapi::LocalRelationshipsMap::new();
                #(#expanded_rels)*
                if map.len() > 0 {
                    ::std::option::Option::Some(map)
                } else {
                    ::std::option::Option::None
                }
            }
         }
    };
    proc_macro::TokenStream::from(expanded_impls)
}

#[proc_macro_derive(FromResource, attributes(jayapi))]
pub fn from_resource_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: syn::DeriveInput = match syn::parse2(input.into()) {
        Ok(input) => input,
        Err(err) => {
            return err.to_compile_error().into();
        }
    };
    let input = match ResourceDerive::from_derive_input(&input) {
        Ok(input) => input,
        Err(err) => {
            return err.write_errors().into();
        }
    };

    let struct_name = &input.ident;
    let generics = &input.generics;

    let resource_type = input
        .resource_type
        .unwrap_or_else(|| input.ident.to_string());

    let mut resource_id_field: Option<IdData> = None;
    let mut relationships: Vec<RelationshipData> = Vec::new();
    let mut attr_fields: Vec<AttributeData> = Vec::new();

    let struct_fields = input
        .data
        .take_struct()
        .expect("only structs are supported as input");

    for (idx, field) in struct_fields.iter().enumerate() {
        let field_or_idx = field
            .ident
            .clone()
            .map(IndexOrField::Field)
            .unwrap_or(IndexOrField::Index(syn::Index::from(idx)));

        let field_variant: DeriveFieldVariant = match field.try_into() {
            Ok(field) => field,
            Err(err) => {
                return err.write_errors().into();
            }
        };
        match field_variant {
            DeriveFieldVariant::Id(id_attr) | DeriveFieldVariant::IdAndLid(id_attr, _) => {
                resource_id_field =
                    Some(id_attr.into_data(field_or_idx.clone(), &field.ty, resource_type.clone()));
            }
            DeriveFieldVariant::Attribute(attribute_attr) => {
                attr_fields.push(attribute_attr.into_data(field_or_idx.clone(), &field.ty));
            }
            DeriveFieldVariant::Relationship(rel_attr) => {
                relationships.push(rel_attr.into_data(field_or_idx.clone(), &field.ty))
            }
            DeriveFieldVariant::Lid(_) => {
                continue;
            }
            DeriveFieldVariant::Skip => {
                continue;
            }
        }
    }

    let resource_id_field_tokens = if let Some(id_field) = resource_id_field {
        id_field.into_from_resource_tokens()
    } else {
        quote! { /* no id field specified */ }
    };

    let mut attrs_tmp_variables = Vec::with_capacity(attr_fields.len());
    let mut attrs_parsing_match_arms = Vec::with_capacity(attr_fields.len());
    let mut attrs_assign_to_field = Vec::with_capacity(attr_fields.len());

    for attr in attr_fields {
        attrs_tmp_variables.push(attr.into_from_resource_tokens_storage_var());
        attrs_parsing_match_arms.push(attr.into_from_resource_tokens_match_arm());
        attrs_assign_to_field.push(attr.into_from_resource_tokens_var_assign());
    }

    let mut rels_tmp_variables = Vec::with_capacity(relationships.len());
    let mut rels_parsing_match_arms = Vec::with_capacity(relationships.len());
    let mut rels_assign_to_field = Vec::with_capacity(relationships.len());

    for rel in relationships {
        rels_tmp_variables.push(rel.into_from_resource_tokens_storage_var());
        rels_parsing_match_arms.push(rel.into_from_resource_tokens_match_arm());
        rels_assign_to_field.push(rel.into_from_resource_tokens_var_assign());
    }

    let expanded_impls = quote::quote! {
        #[automatically_derived]
        impl #generics std::convert::TryFrom<jayapi::Resource> for #struct_name #generics {
            type Error = jayapi::ParsingError;
            fn try_from(resource: jayapi::Resource) -> Result<Self, Self::Error> {
                #(#attrs_tmp_variables)*
                #(#rels_tmp_variables)*
                if let Some(attrs) = resource.attributes {
                    for (k,v) in attrs {
                        match k.as_str() {
                            #(#attrs_parsing_match_arms)*
                            f => {
                                return Err(::jayapi::ParsingError::UnknownField { field: f.to_string() });
                            }
                       }
                    }
                }
                if let Some(rels) = resource.relationships {
                    for (k,v) in rels.iter() {
                        match k.as_str() {
                            #(#rels_parsing_match_arms)*
                            f => {
                                return Err(::jayapi::ParsingError::UnknownRelationship { relationship: f.to_owned() });
                            }
                        }
                    }
                }
                Ok(#struct_name {
                    #resource_id_field_tokens
                    #(#attrs_assign_to_field)*
                    #(#rels_assign_to_field)*
                })
            }
        }
    };

    proc_macro::TokenStream::from(expanded_impls)
}

#[proc_macro_derive(FromLocalResource, attributes(jayapi))]
pub fn from_local_resource_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: syn::DeriveInput = match syn::parse2(input.into()) {
        Ok(input) => input,
        Err(err) => {
            return err.to_compile_error().into();
        }
    };
    let input = match ResourceDerive::from_derive_input(&input) {
        Ok(input) => input,
        Err(err) => {
            return err.write_errors().into();
        }
    };

    let struct_name = &input.ident;
    let generics = &input.generics;

    let resource_type = input
        .resource_type
        .unwrap_or_else(|| input.ident.to_string());

    let mut resource_lid_field: Option<LidData> = None;
    let mut relationships: Vec<RelationshipData> = Vec::new();
    let mut attr_fields: Vec<AttributeData> = Vec::new();

    let struct_fields = input
        .data
        .take_struct()
        .expect("only structs are supported as input");

    for (idx, field) in struct_fields.iter().enumerate() {
        let field_or_idx = field
            .ident
            .clone()
            .map(IndexOrField::Field)
            .unwrap_or(IndexOrField::Index(syn::Index::from(idx)));

        let field_variant: DeriveFieldVariant = match field.try_into() {
            Ok(field) => field,
            Err(err) => {
                return err.write_errors().into();
            }
        };
        match field_variant {
            DeriveFieldVariant::Lid(lid_attr) | DeriveFieldVariant::IdAndLid(_, lid_attr) => {
                resource_lid_field = Some(lid_attr.into_data(
                    field_or_idx.clone(),
                    &field.ty,
                    resource_type.clone(),
                ));
            }
            DeriveFieldVariant::Attribute(attribute_attr) => {
                attr_fields.push(attribute_attr.into_data(field_or_idx.clone(), &field.ty));
            }
            DeriveFieldVariant::Relationship(rel_attr) => {
                relationships.push(rel_attr.into_data(field_or_idx.clone(), &field.ty))
            }
            DeriveFieldVariant::Id(_) => {
                continue;
            }
            DeriveFieldVariant::Skip => {
                continue;
            }
        }
    }

    let resource_id_field_tokens = if let Some(id_field) = resource_lid_field {
        id_field.into_from_local_resource_tokens()
    } else {
        quote! { /* no id field specified */ }
    };

    let mut attrs_tmp_variables = Vec::with_capacity(attr_fields.len());
    let mut attrs_parsing_match_arms = Vec::with_capacity(attr_fields.len());
    let mut attrs_assign_to_field = Vec::with_capacity(attr_fields.len());

    for attr in attr_fields {
        attrs_tmp_variables.push(attr.into_from_resource_tokens_storage_var());
        attrs_parsing_match_arms.push(attr.into_from_resource_tokens_match_arm());
        attrs_assign_to_field.push(attr.into_from_resource_tokens_var_assign());
    }

    let mut rels_tmp_variables = Vec::with_capacity(relationships.len());
    let mut rels_parsing_match_arms = Vec::with_capacity(relationships.len());
    let mut rels_assign_to_field = Vec::with_capacity(relationships.len());

    for rel in relationships {
        rels_tmp_variables.push(rel.into_from_resource_tokens_storage_var());
        rels_parsing_match_arms.push(rel.into_from_local_resource_tokens_match_arm());
        rels_assign_to_field.push(rel.into_from_resource_tokens_var_assign());
    }

    let expanded_impls = quote::quote! {
        #[automatically_derived]
        impl #generics std::convert::TryFrom<jayapi::LocalResource> for #struct_name #generics {
            type Error = ::jayapi::ParsingError;
            fn try_from(resource: ::jayapi::LocalResource) -> Result<Self, Self::Error> {
                #(#attrs_tmp_variables)*
                #(#rels_tmp_variables)*
                if let Some(attrs) = resource.attributes {
                    for (k,v) in attrs {
                        match k.as_str() {
                            #(#attrs_parsing_match_arms)*
                            f => {
                                return Err(::jayapi::ParsingError::UnknownField { field: f.to_string() });
                            }
                       }
                    }
                }
                if let Some(rels) = resource.relationships {
                    for (k,v) in rels.iter() {
                        match k.as_str() {
                            #(#rels_parsing_match_arms)*
                            f => {
                                return Err(::jayapi::ParsingError::UnknownRelationship { relationship: f.to_owned() });
                            }
                        }
                    }
                }
                Ok(#struct_name {
                    #resource_id_field_tokens
                    #(#attrs_assign_to_field)*
                    #(#rels_assign_to_field)*
                })
            }
        }
    };

    proc_macro::TokenStream::from(expanded_impls)
}

#[cfg(feature = "json-schema")]
#[proc_macro_derive(JsonSchema, attributes(jayapi))]
pub fn json_schema_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: syn::DeriveInput = match syn::parse2(input.into()) {
        Ok(input) => input,
        Err(err) => {
            return err.to_compile_error().into();
        }
    };
    let input = match ResourceDerive::from_derive_input(&input) {
        Ok(input) => input,
        Err(err) => {
            return err.write_errors().into();
        }
    };

    let struct_name = &input.ident;
    let generics = &input.generics;

    let resource_type = input
        .resource_type
        .unwrap_or_else(|| input.ident.to_string());

    let mut relationships: Vec<RelationshipData> = Vec::new();
    let mut attr_fields: Vec<AttributeData> = Vec::new();
    let mut id_field: Option<IdData> = None;
    let mut lid_field: Option<LidData> = None;

    let struct_fields = input
        .data
        .take_struct()
        .expect("only structs are supported as input");

    for (idx, field) in struct_fields.iter().enumerate() {
        let field_or_idx = field
            .ident
            .clone()
            .map(IndexOrField::Field)
            .unwrap_or(IndexOrField::Index(syn::Index::from(idx)));

        let field_variant: DeriveFieldVariant = match field.try_into() {
            Ok(field) => field,
            Err(err) => {
                return err.write_errors().into();
            }
        };
        match field_variant {
            DeriveFieldVariant::Id(id_attr) => {
                id_field =
                    Some(id_attr.into_data(field_or_idx.clone(), &field.ty, resource_type.clone()));
            }
            DeriveFieldVariant::Lid(lid_attr) => {
                lid_field = Some(lid_attr.into_data(
                    field_or_idx.clone(),
                    &field.ty,
                    resource_type.clone(),
                ));
            }
            DeriveFieldVariant::IdAndLid(id_attr, lid_attr) => {
                id_field =
                    Some(id_attr.into_data(field_or_idx.clone(), &field.ty, resource_type.clone()));
                lid_field = Some(lid_attr.into_data(
                    field_or_idx.clone(),
                    &field.ty,
                    resource_type.clone(),
                ));
            }
            DeriveFieldVariant::Attribute(attribute_attr) => {
                attr_fields.push(attribute_attr.into_data(field_or_idx.clone(), &field.ty));
            }
            DeriveFieldVariant::Relationship(rel_attr) => {
                relationships.push(rel_attr.into_data(field_or_idx.clone(), &field.ty))
            }
            DeriveFieldVariant::Skip => {
                continue;
            }
        }
    }

    let attributes_schema_parts = attr_fields
        .into_iter()
        .map(|a| a.into_json_schema_part())
        .collect::<Vec<_>>();
    let relationships_schema_parts = relationships
        .into_iter()
        .map(|a| a.into_json_schema_part())
        .collect::<Vec<_>>();

    let resource_id_schema_section = match (id_field, lid_field) {
        (None, None) => quote! {},
        (None, Some(lid_data)) => {
            let lid_schema_part = lid_data.into_id_type_json_schema_part();
            quote! {
                "lid": #lid_schema_part,
            }
        }
        (Some(id_data), None) => {
            let id_schema_part = id_data.into_id_type_json_schema_part();
            quote! {
                "id": #id_schema_part,
            }
        }
        (Some(id_data), Some(lid_data)) => {
            // NOTE: this is somewhat nonsensical but there is no reason to stop users from documenting their API like that
            let id_schema_part = id_data.into_id_type_json_schema_part();
            let lid_schema_part = lid_data.into_id_type_json_schema_part();
            quote! {
                "id": #id_schema_part,
                "lid": #lid_schema_part,
            }
        }
    };

    let expanded_impls = quote::quote! {
        #[automatically_derived]
        impl #generics ::schemars::JsonSchema for #struct_name #generics {
            fn schema_name() -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(#resource_type)
            }
            fn json_schema(generator: &mut ::schemars::SchemaGenerator) -> ::schemars::Schema {
                ::schemars::json_schema!({
                    "type": "object",
                    "properties": {
                        #resource_id_schema_section
                        "type": {
                            "type": "string",
                            "const": #resource_type
                        },
                        "attributes": {
                            "type": "object",
                            "properties": {
                                #(#attributes_schema_parts)*
                            }
                        },
                        "relationships": {
                            "type": "object",
                            "properties": {
                                #(#relationships_schema_parts)*
                            }
                        }
                    },
                    "required": ["type"]
                })
            }
        }
        #[automatically_derived]
        impl #generics ::jayapi::json_schema::JsonSchema for #struct_name #generics {}
    };

    proc_macro::TokenStream::from(expanded_impls)
}

#[cfg(test)]
mod test {
    use darling::FromDeriveInput;
    use quote::{quote, ToTokens};

    use crate::{DeriveFieldVariant, ResourceDerive};

    #[test]
    fn minimal_from_resource() {
        let input = quote! {
            #[derive(FromResource)]
            pub struct Test{}
        };

        let parsed: syn::DeriveInput = syn::parse2(input.clone()).unwrap();
        let parsed = ResourceDerive::from_derive_input(&parsed).unwrap();

        let struct_name = parsed.ident.to_token_stream().to_string();
        assert_eq!(&struct_name, "Test");

        println!(
            r#"
            INPUT:
            
            {}

            PARSED AS:

            {:?}

        "#,
            input, parsed
        );
    }
    #[test]
    fn renamed_from_resource() {
        let input = quote! {
            #[derive(jayapi::FromResource)]
            #[jayapi(resource_type = "test")]
            pub struct Test{}
        };

        let parsed: syn::DeriveInput = syn::parse2(input.clone()).unwrap();
        let parsed = ResourceDerive::from_derive_input(&parsed).unwrap();

        let name_literal = parsed.resource_type.as_ref().unwrap();
        assert_eq!(&name_literal.into_token_stream().to_string(), "\"test\"");

        println!(
            r#"
            INPUT:
            
            {}

            PARSED AS:

            {:?}
            
        "#,
            input, parsed
        );
    }
    #[test]
    fn minimal_as_resource() {
        let input = quote! {
            #[derive(jayapi::AsResource)]
            pub struct Test{
                #[jayapi(id)]
                id: String,
            }
        };

        let parsed: syn::DeriveInput = syn::parse2(input.clone()).unwrap();
        let parsed = ResourceDerive::from_derive_input(&parsed).unwrap();
        let fields = parsed.data.clone().take_struct().unwrap();

        fields.iter().last().unwrap().id.clone().unwrap();

        println!(
            r#"
            INPUT:
            
            {}

            PARSED AS:

            {:?}
            
        "#,
            input, parsed
        );
    }
    #[test]
    fn as_resource_with_explicit_short_attribute() {
        let input = quote! {
            #[derive(jayapi::AsResource)]
            pub struct Test{
                #[jayapi(id)]
                id: String,
                #[jayapi(attribute)]
                attr: String,
            }
        };

        let parsed: syn::DeriveInput = syn::parse2(input.clone()).unwrap();
        let parsed = ResourceDerive::from_derive_input(&parsed).unwrap();
        let fields = parsed.data.as_ref().take_struct().unwrap();

        fields.iter().last().unwrap().attribute.clone().unwrap();

        println!(
            r#"
            INPUT:
            
            {}

            PARSED AS:

            {:?}
            
        "#,
            input, parsed
        );
    }

    #[test]
    fn as_resource_with_implicit_attribute() {
        let input = quote! {
            #[derive(jayapi::AsResource)]
            pub struct Test{
                #[jayapi(id)]
                id: String,
                attr: String,
            }
        };

        let parsed: syn::DeriveInput = syn::parse2(input).unwrap();
        let parsed = ResourceDerive::from_derive_input(&parsed).unwrap();
        let fields = parsed.data.take_struct().unwrap();

        let field_variant = fields
            .iter()
            .last()
            .map(DeriveFieldVariant::try_from)
            .unwrap()
            .unwrap();

        match field_variant {
            DeriveFieldVariant::Attribute(_) => {
                return;
            }
            _ => {
                panic!("should be an attribute");
            }
        }
    }
}
