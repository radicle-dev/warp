#![allow(missing_docs)]

use http::Method;
use openapiv3::{OpenAPI, PathItem, ReferenceOr};

use crate::filter::{FilterBase, Internal};

use std::{any::TypeId, collections::HashMap, fmt::Debug, iter::IntoIterator};

pub trait DocumentedFilter {
    type Output: IntoIterator<Item=RouteDocumentation>;

    fn document(&self, route: RouteDocumentation) -> Self::Output;
}

pub trait DocumentedReply {
    type Output: IntoIterator<Item=RouteDocumentation>;

    fn document(route: RouteDocumentation) -> Self::Output;
}

#[derive(Clone, Debug, Default)]
pub struct RouteDocumentation {
    pub headers: Vec<DocumentedHeader>,
    pub method: Option<Method>,
    pub parameters: Vec<DocumentedParameter>,
    pub path: String,
    pub queries: Vec<DocumentedQuery>,
    pub responses: HashMap<u16, DocumentedResponse>,
}

#[derive(Clone, Debug)]
pub struct DocumentedHeader {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
}

#[derive(Clone, Debug)]
pub struct DocumentedParameter {
    pub name: String,
    pub description: Option<String>,
    pub parameter_type: DocumentedType,
}

#[derive(Clone, Debug)]
pub struct DocumentedQuery {
    pub name: String,
    pub description: Option<String>,
    pub parameter_type: DocumentedType,
    pub required: bool,
}

#[derive(Clone, Debug)]
pub struct DocumentedResponse {
    pub description: String,
    pub headers: Vec<DocumentedHeader>,
    pub body: Vec<DocumentedResponseBody>,
}

#[derive(Clone, Debug)]
pub struct DocumentedResponseBody {
    pub body: DocumentedType,
    pub mime: Option<String>,
}

#[derive(Clone, Debug)]
pub enum DocumentedType {
    Integer(Option<String>),
    String(Option<String>),
    Object(HashMap<String, DocumentedType>),
}
impl From<TypeId> for DocumentedType {
    fn from(id: TypeId) -> Self {
        // A HashMap initialised with Once might be better.
        match id {
            t if t == TypeId::of::<u8>() => Self::Integer(None),
            t if t == TypeId::of::<u16>() => Self::Integer(None),
            t if t == TypeId::of::<u32>() => Self::Integer(None),
            t if t == TypeId::of::<u64>() => Self::Integer(None),
            t if t == TypeId::of::<u128>() => Self::Integer(None),
            t if t == TypeId::of::<i8>() => Self::Integer(None),
            t if t == TypeId::of::<i16>() => Self::Integer(None),
            t if t == TypeId::of::<i32>() => Self::Integer(None),
            t if t == TypeId::of::<i64>() => Self::Integer(None),
            t if t == TypeId::of::<i128>() => Self::Integer(None),
            t if t == TypeId::of::<String>() => Self::String(None),
            _ => Self::String(None),
        }
    }
}

pub fn document<F: DocumentedFilter>(filter: F) -> Vec<RouteDocumentation> {
    filter.document(RouteDocumentation::default())
        .into_iter()
        .collect()
}

#[derive(Copy, Clone, Debug)]
pub struct ExplicitDocumentation<T, F>
where F: Fn(&mut RouteDocumentation) {
    item: T,
    callback: F,
}
impl<T, F: Fn(&mut RouteDocumentation)> ExplicitDocumentation<T, F> {
    pub fn new(item: T, callback: F) -> Self {
        ExplicitDocumentation{ item, callback }
    }
}
impl<T, F: Fn(&mut RouteDocumentation)> FilterBase for ExplicitDocumentation<T, F>
where T: FilterBase {
    type Extract = T::Extract;
    type Error = T::Error;
    type Future = T::Future;
    
    fn filter(&self, internal: Internal) -> Self::Future {
        self.item.filter(internal)
    }
}
impl<T, F> DocumentedFilter for ExplicitDocumentation<T, F>
where F: Fn(&mut RouteDocumentation) {
    type Output = Vec<RouteDocumentation>;

    fn document(&self, mut item: RouteDocumentation) -> Self::Output {
        let ExplicitDocumentation{ callback, .. } = self;
        (callback)(&mut item);
        vec![item]
    }
}

pub fn to_openapi(routes: Vec<RouteDocumentation>) -> OpenAPI {
    use openapiv3::{Header, IntegerType, MediaType, ObjectType, Operation, Parameter, ParameterData, ParameterSchemaOrContent, PathStyle, Response, Schema, SchemaData, SchemaKind, StatusCode, StringType, Type as OpenApiType};

    let paths = routes.into_iter()
        .map(|route| {
            let RouteDocumentation{
                headers,
                method,
                parameters,
                mut path,
                queries,
                responses
            } = route;
            let mut item = PathItem::default();
            let mut operation = Operation::default();

            fn documented_type_to_openapi(t: DocumentedType) -> Schema {
                match t {
                    DocumentedType::Object(p) => {
                        Schema{
                            schema_data: SchemaData::default(),
                            schema_kind: SchemaKind::Type(OpenApiType::Object(ObjectType{
                                properties: p.into_iter()
                                    .map(|(name, type_)| (name, ReferenceOr::Item(Box::new(documented_type_to_openapi(type_)))))
                                    .collect(),
                                ..ObjectType::default()
                            }))
                        }
                    },
                    DocumentedType::String(s) => {
                        Schema{
                            schema_data: SchemaData{
                                description: s,
                                ..SchemaData::default()
                            },
                            schema_kind: SchemaKind::Type(OpenApiType::String(StringType::default())),
                        }
                    }
                    DocumentedType::Integer(i) => {
                        Schema{
                            schema_data: SchemaData{
                                description: i,
                                ..SchemaData::default()
                            },
                            schema_kind: SchemaKind::Type(OpenApiType::Integer(IntegerType::default())),
                        }
                    }
                }
            }

            operation.parameters.extend(
                parameters.into_iter()
                    .enumerate()
                    .inspect(|(i, param)| path = path.replace(format!("{{{}}}", i).as_str(), format!("{{{}}}", param.name).as_str()))
                    .map(|(_, param)| ReferenceOr::Item(Parameter::Path{style: PathStyle::default(), parameter_data: ParameterData{
                        name: param.name,
                        description: param.description,
                        required: true,
                        deprecated: Some(false),
                        format: ParameterSchemaOrContent::Schema(ReferenceOr::Item(documented_type_to_openapi(param.parameter_type))),
                        example: None,
                        examples: Default::default(),
                    }}))
            );
            operation.parameters.extend(
                headers.into_iter()
                    .map(|header| ReferenceOr::Item(Parameter::Header{style: Default::default(), parameter_data: ParameterData{
                        name: header.name,
                        description: header.description,
                        required: header.required,
                        deprecated: Some(false),
                        format: ParameterSchemaOrContent::Schema(ReferenceOr::Item(Schema{
                            schema_data: SchemaData::default(),
                            schema_kind: SchemaKind::Type(OpenApiType::String(StringType::default())),
                        })),
                        example: None,
                        examples: Default::default(),
                    }}))
            );
            operation.parameters.extend(
                queries.into_iter()
                    .map(|query| ReferenceOr::Item(Parameter::Query{
                        style: Default::default(),
                        allow_reserved: false,
                        allow_empty_value: None,
                        parameter_data: ParameterData{
                            name: query.name,
                            description: query.description,
                            required: query.required,
                            deprecated: Some(false),
                            format: ParameterSchemaOrContent::Schema(ReferenceOr::Item(Schema{
                                schema_data: SchemaData::default(),
                                schema_kind: SchemaKind::Type(OpenApiType::String(StringType::default())),
                            })),
                            example: None,
                            examples: Default::default(),
                        },
                    }))
            );

            let mut responses = responses.into_iter().collect::<Vec<_>>();
            responses.sort_by_key(|(code, _)| *code);
            operation.responses.responses.extend(
                responses.into_iter()
                    .map(|(code, response)| (StatusCode::Code(code), ReferenceOr::Item(Response{
                        description: response.description,
                        headers: response.headers.into_iter().map(|header| (header.name, ReferenceOr::Item(Header{
                            description: header.description,
                            style: Default::default(),
                            required: false,
                            deprecated: None,
                            format: ParameterSchemaOrContent::Schema(ReferenceOr::Item(Schema{
                                schema_kind: SchemaKind::Type(OpenApiType::String(Default::default())),
                                schema_data: SchemaData::default(),
                            })),
                            example: None,
                            examples: Default::default(),
                        }))).collect(),
                        content: response.body.into_iter().map(|body| (body.mime.unwrap_or("*/*".into()), MediaType{
                            example: None,
                            examples: Default::default(),
                            encoding: Default::default(),
                            schema: Some(ReferenceOr::Item(documented_type_to_openapi(body.body)))
                        })).collect(),
                        ..Response::default()
                    })))
            );

            match method.unwrap_or(Method::POST) {
                Method::GET => item.get = Some(operation),
                Method::POST => item.post = Some(operation),
                Method::PUT => item.put = Some(operation),
                Method::DELETE => item.delete = Some(operation),
                Method::HEAD => item.head = Some(operation),
                Method::OPTIONS => item.options = Some(operation),
                Method::PATCH => item.patch = Some(operation),
                Method::TRACE => item.trace = Some(operation),
                _ => unimplemented!(),
            }

            (path, ReferenceOr::Item(item))
        }).collect();
    
    OpenAPI {
        openapi: "3.0.0".into(),
        paths,
        ..OpenAPI::default()
    }
}

impl<S: Into<String>> DocumentedReply for S {
    type Output = Vec<RouteDocumentation>;

    fn document(path: RouteDocumentation) -> Self::Output {
        vec![path]
    }
}
