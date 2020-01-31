#![allow(missing_docs)]

use http::Method;
use serde_json::Value;

use crate::any;
use crate::filter::{Filter, FilterBase, Internal};

use std::{any::TypeId, collections::{HashMap, HashSet}, convert::Infallible, fmt::Debug, hash::{Hash, Hasher}};

#[derive(Clone, Debug)]
pub struct RouteDocumentation {
    pub bodies: HashSet<DocumentedBody>,
    pub cookies: HashSet<DocumentedCookie>,
    pub description: Option<String>,
    pub headers: HashSet<DocumentedHeader>,
    pub method: Method,
    pub parameters: Vec<DocumentedParameter>,
    pub path: String,
    pub queries: Vec<DocumentedQuery>,
    pub responses: HashSet<DocumentedResponse>,
}
impl Default for RouteDocumentation {
    fn default() -> Self {
        Self {
            bodies: Default::default(),
            cookies: Default::default(),
            description: Default::default(),
            headers: Default::default(),
            method: Method::POST,
            parameters: Default::default(),
            path: Default::default(),
            queries: Default::default(),
            responses: Default::default(),
        }
    }
}
impl RouteDocumentation {
    pub fn body<B: Into<DocumentedBody>>(&mut self, body: B) {
	    self.bodies.insert(body.into());
    }
    pub fn cookie(&mut self, cookie: DocumentedCookie) {
        self.cookies.insert(cookie);
    }
	pub fn description<S: Into<String>>(&mut self, description: S) {
        self.description = Some(description.into());
    }
    pub fn header(&mut self, header: DocumentedHeader) {
        self.headers.insert(header);
    }
    pub fn parameter(&mut self, parameter: DocumentedParameter) {
        self.push_path(format!("{{{}}}", self.parameters.len()));
        self.parameters.push(parameter);
    }
    pub fn push_path<S: AsRef<str>>(&mut self, path: S) {
        self.path.push('/');
        self.path.push_str(path.as_ref());
    }
    pub fn query(&mut self, query: DocumentedQuery) {
        self.queries.push(query);
    }
    pub fn response<R: Into<DocumentedResponse>>(&mut self, response: R) {
        self.responses.insert(response.into());
    }
}

#[derive(Clone, Debug)]
pub struct DocumentedCookie {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
}
pub fn cookie<S: Into<String>>(name: S) -> DocumentedCookie {
    DocumentedCookie { name: name.into(), description: None, required: true }
}
impl DocumentedCookie {
    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }
}
impl Hash for DocumentedCookie {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.name.hash(hasher)
    }
}
impl PartialEq<Self> for DocumentedCookie {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
impl Eq for DocumentedCookie {}

#[derive(Clone, Debug)]
pub struct DocumentedHeader {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
}
pub fn header<S: Into<String>>(name: S) -> DocumentedHeader {
    DocumentedHeader{ name: name.into(), description: None, required: true }
}
impl DocumentedHeader {
    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }
}
impl Hash for DocumentedHeader {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.name.hash(hasher)
    }
}
impl PartialEq<Self> for DocumentedHeader {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
impl Eq for DocumentedHeader {}

#[derive(Clone, Debug)]
pub struct DocumentedParameter {
    pub name: String,
    pub description: Option<String>,
    pub type_: DocumentedType,
    pub required: bool,
}
pub fn parameter<S: Into<String>, T: Into<DocumentedType>>(name: S, type_: T) -> DocumentedParameter {
    DocumentedParameter{ name: name.into(), description: None, type_: type_.into(), required: true }
}
impl DocumentedParameter {
    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }
}

#[derive(Clone, Debug)]
pub struct DocumentedQuery {
    pub name: String,
    pub description: Option<String>,
    pub type_: DocumentedType,
    pub required: bool,
}
pub fn query<S: Into<String>, T: Into<DocumentedType>>(name: S, type_: T) -> DocumentedQuery {
    DocumentedQuery{ name: name.into(), description: None, type_: type_.into(), required: true }
}
impl DocumentedQuery {
    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }
}

#[derive(Clone, Debug, Default, Eq)]
pub struct DocumentedResponse {
    pub body: HashSet<DocumentedBody>,
    pub description: String,
    pub headers: HashSet<DocumentedHeader>,
    pub status: u16,
}
impl DocumentedResponse {
    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = description.into();
        self
    }
    pub fn body(mut self, body: DocumentedBody) -> Self {
        self.body.insert(body);
        self
    }
    pub fn header(mut self, header: DocumentedHeader) -> Self {
        self.headers.insert(header);
        self
    }
    pub fn status(mut self, status: u16) -> Self {
	    self.status = status;
        self
    }
}
impl Hash for DocumentedResponse {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.status.hash(hasher)
    }
}
impl PartialEq for DocumentedResponse {
    fn eq(&self, other: &Self) -> bool {
        self.status == other.status
    }
}
impl Documentable for DocumentedResponse {
    fn document(&self, route: &mut RouteDocumentation) {
        route.response(self.clone())
    }
}

#[derive(Clone, Debug)]
pub struct DocumentedBody {
    pub body: DocumentedType,
    pub mime: Option<String>,
}
impl Default for DocumentedBody {
    fn default() -> Self {
        Self {
            body: object(HashMap::default()),
            mime: None,
        }
    }
}
impl DocumentedBody {
    pub fn body<T: Into<DocumentedType>>(mut self, type_: T) -> Self {
        self.body = type_.into();
        self
    }
    pub fn mime<S: Into<String>>(mut self, mime: S) -> Self {
        self.mime = Some(mime.into());
        self
    }
}
impl Documentable for DocumentedBody {
    fn document(&self, route: &mut RouteDocumentation) {
        route.body(self.clone())
    }
}
impl Hash for DocumentedBody {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.mime.hash(hasher)
    }
}
impl PartialEq for DocumentedBody {
    fn eq(&self, other: &Self) -> bool {
        self.mime == other.mime
    }
}
impl Eq for DocumentedBody {}

pub fn boolean() -> DocumentedType {
    DocumentedType::Primitive{ ty: InternalDocumentedType::Boolean, description: None, required: true, example: None }
}
pub fn float() -> DocumentedType {
    DocumentedType::Primitive{ ty: InternalDocumentedType::Float, description: None, required: true, example: None }
}
pub fn integer() -> DocumentedType {
    DocumentedType::Primitive{ ty: InternalDocumentedType::Integer, description: None, required: true, example: None }
}
pub fn string() -> DocumentedType {
    DocumentedType::Primitive{ ty: InternalDocumentedType::String, description: None, required: true, example: None }
}
pub fn object(fields: HashMap<String, DocumentedType>) -> DocumentedType {
    DocumentedType::Object{ properties: fields, description: None, example: None }
}
pub fn array<T: Into<Box<DocumentedType>>>(ty: T) -> DocumentedType {
    DocumentedType::Array{ ty: ty.into(), description: None, example: None }
}

#[derive(Clone, Debug)]
pub enum DocumentedType {
    Array{ ty: Box<DocumentedType>, description: Option<String>, example: Option<Value> },
    Object{ properties: HashMap<String, DocumentedType>, description: Option<String>, example: Option<Value> },
    Primitive{ ty: InternalDocumentedType, description: Option<String>, required: bool, example: Option<Value> },
}
impl DocumentedType {
    pub fn example(mut self, value: Value) -> Self {
        match &mut self {
            Self::Array{ example, .. } => example.replace(value),
            Self::Object{ example, .. } => example.replace(value),
            Self::Primitive{ example, .. } => example.replace(value),
        };
        self
    }
    pub fn description<S: Into<String>>(mut self, description_: S) -> Self {
	    match &mut self {
            Self::Array{ description, .. } => description.replace(description_.into()),
            Self::Object{ description, .. } => description.replace(description_.into()),
            Self::Primitive{ description, .. } => description.replace(description_.into()),
        };
        self
    }
}
impl From<HashMap<String, DocumentedType>> for DocumentedType {
    fn from(map: HashMap<String, DocumentedType>) -> Self {
        object(map)
    }
}

#[derive(Clone, Debug)]
pub enum InternalDocumentedType {
    Boolean,
    Float,
    Integer,
    String,
}

impl From<TypeId> for DocumentedType {
    fn from(id: TypeId) -> Self {
        // A HashMap initialised with Once might be better.
        match id {
            t if t == TypeId::of::<u8>() => integer(),
            t if t == TypeId::of::<u16>() => integer(),
            t if t == TypeId::of::<u32>() => integer(),
            t if t == TypeId::of::<u64>() => integer(),
            t if t == TypeId::of::<u128>() => integer(),
            t if t == TypeId::of::<usize>() => integer(),
            t if t == TypeId::of::<i8>() => integer(),
            t if t == TypeId::of::<i16>() => integer(),
            t if t == TypeId::of::<i32>() => integer(),
            t if t == TypeId::of::<i64>() => integer(),
            t if t == TypeId::of::<i128>() => integer(),
            t if t == TypeId::of::<isize>() => integer(),
            t if t == TypeId::of::<String>() => string(),
            t if t == TypeId::of::<&str>() => string(),
            _ => object(HashMap::default()),
        }
    }
}

pub fn describe<F: Filter>(filter: F) -> Vec<RouteDocumentation> {
    let mut routes = filter.describe(RouteDocumentation::default());
    routes.iter_mut()
        .filter(|route| route.path.is_empty())
        .for_each(|route| route.path.push('/'));
    routes
}

pub fn explicit<F, D>(filter: F, describe: D) -> ExplicitDocumentation<F, D>
where
    F: Filter,
    D: Fn(&mut RouteDocumentation),
{
    ExplicitDocumentation{ filter, describe }
}

#[derive(Copy, Clone, Debug)]
pub struct ExplicitDocumentation<F, D> {
    filter: F,
    describe: D,
}
impl<F, D: Fn(&mut RouteDocumentation)> FilterBase for ExplicitDocumentation<F, D>
where F: FilterBase {
    type Extract = F::Extract;
    type Error = F::Error;
    type Future = F::Future;

    fn filter(&self, internal: Internal) -> Self::Future {
        self.filter.filter(internal)
    }

    fn describe(&self, mut route: RouteDocumentation) -> Vec<RouteDocumentation> {
        (self.describe)(&mut route);
        vec![route]
    }
}

pub trait Documentable {
    fn document(&self, _: &mut RouteDocumentation);
}
impl<F> Documentable for F where F: Fn(&mut RouteDocumentation) + Clone {
    fn document(&self, route: &mut RouteDocumentation) {
        (self)(route)
    }
}

pub fn document<D: Documentable + Clone>(describe: D) -> impl Filter<Extract = (), Error = Infallible> + Clone {
    explicit(any(), move |route| describe.document(route))
}

/// Sets the description of the route.
pub fn description<S: Into<String>>(description: S) -> impl Fn(&mut RouteDocumentation) + Clone {
	let description = description.into();
    move |route| route.description(description.clone())
}

/// Adds a response to the route documentation.
pub fn response<B: Into<Option<DocumentedBody>>>(status: u16, body: B) -> DocumentedResponse {
	let response = DocumentedResponse::default().status(status);
    match body.into() {
        Some(b) => response.body(b),
        None => response,
    }
}

pub fn body<T: Into<DocumentedType>>(type_: T) -> DocumentedBody {
    DocumentedBody::default().body(type_)
}

#[cfg(feature = "openapi")]
pub fn to_openapi<I: IntoIterator<Item=RouteDocumentation>>(routes: I) -> openapiv3::OpenAPI {
    use indexmap::IndexMap;
    use openapiv3::{ArrayType, Header, IntegerType, MediaType, NumberType, ObjectType, Operation, OpenAPI,
        Parameter, ParameterData, ParameterSchemaOrContent, PathItem, PathStyle, ReferenceOr, RequestBody, Response,
        Schema, SchemaData, SchemaKind, StatusCode, StringType, Type as OpenApiType};

    let mut paths: IndexMap<String, PathItem> = IndexMap::default();
//	let mut routes = routes.into_iter().collect::<Vec<_>>();
//    routes.sort_by_cached_key(|route| route.path.clone()); // Expensive Process
    routes.into_iter()
        .for_each(|route| {
            let RouteDocumentation{
                bodies,
                cookies,
                description,
                headers,
                method,
                parameters,
                mut path,
                queries,
                responses
            } = route;
            let mut operation = Operation::default();

            fn documented_type_to_openapi(t: DocumentedType) -> Schema {
                match t {
                    DocumentedType::Array{ ty, description, example } => {
                        Schema {
                            schema_data: SchemaData {
                                description,
                                example,
                                ..SchemaData::default()
                            },
                            schema_kind: SchemaKind::Type(OpenApiType::Array(ArrayType{
                                items: ReferenceOr::Item(Box::new(documented_type_to_openapi(*ty))),
                                min_items: None,
                                max_items: None,
                                unique_items: false,
                            }))
                        }
                    }
                    DocumentedType::Object{ properties, description, example } => {
                        Schema {
                            schema_data: SchemaData {
                                description,
                                example,
                                ..SchemaData::default()
                            },
                            schema_kind: SchemaKind::Type(OpenApiType::Object(ObjectType{
                                properties: properties.into_iter()
                                    .map(|(name, type_)| (name, ReferenceOr::Item(Box::new(documented_type_to_openapi(type_)))))
                                    .collect(),
                                ..ObjectType::default()
                            }))
                        }
                    }
                    DocumentedType::Primitive{ty, description, example, required} => {
                        Schema {
                            schema_data: SchemaData{
                                description,
                                nullable: !required,
                                example,
                                ..SchemaData::default()
                            },
                            schema_kind: SchemaKind::Type(match ty {
                                InternalDocumentedType::Boolean => OpenApiType::Boolean{},
                                InternalDocumentedType::Float => OpenApiType::Number(NumberType::default()),
                                InternalDocumentedType::Integer => OpenApiType::Integer(IntegerType::default()),
                                InternalDocumentedType::String => OpenApiType::String(StringType::default()),
                            }),
                        }
                    }
                }
            }

	        // The summary should only be about one line, so we'll take the first one.
            if let Some(description) = &description {
                operation.summary = description.lines().next().map(|d| d.into())
            }
            operation.description = description;
            operation.request_body = Some(ReferenceOr::Item(RequestBody{
                required: !bodies.is_empty(),
                content: bodies.into_iter()
                    .map(|body| (
                        body.mime.unwrap_or("*/*".into()),
                        MediaType {
                            schema: Some(ReferenceOr::Item(documented_type_to_openapi(body.body))),
                            ..MediaType::default()
                        }
                    ))
                    .collect(),
	            ..RequestBody::default()
            }));
            operation.parameters.extend(
                parameters.into_iter()
                    .enumerate()
                    .inspect(|(i, param)| path = path.replace(format!("{{{}}}", i).as_str(), format!("{{{}}}", param.name).as_str()))
                    .map(|(_, param)| ReferenceOr::Item(Parameter::Path{style: PathStyle::default(), parameter_data: ParameterData{
                        name: param.name,
                        description: param.description,
                        required: param.required,
                        deprecated: Some(false),
                        format: ParameterSchemaOrContent::Schema(ReferenceOr::Item(documented_type_to_openapi(param.type_))),
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
            operation.parameters.extend(
                cookies.into_iter()
                    .map(|cookie| ReferenceOr::Item(Parameter::Cookie{
                        style: Default::default(),
                        parameter_data: ParameterData {
                            name: cookie.name,
                            description: cookie.description,
                            required: cookie.required,
                            deprecated: Some(false),
                            format: ParameterSchemaOrContent::Schema(ReferenceOr::Item(Schema{
                                schema_data: SchemaData::default(),
                                schema_kind: SchemaKind::Type(OpenApiType::String(StringType::default())),
                            })),
                            example: None,
                            examples: Default::default(),
                        }
                    }))
            );

            let mut responses = responses.into_iter().collect::<Vec<_>>();
            responses.sort_by_key(|response| response.status);
            operation.responses.responses.extend(
                responses.into_iter()
                    .map(|response| (StatusCode::Code(response.status), ReferenceOr::Item(Response{
                        description: response.description,
                        headers: response.headers.into_iter().map(|header| (header.name, ReferenceOr::Item(Header{
                            description: header.description,
                            style: Default::default(),
                            required: header.required,
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

            let item = paths.entry(path).or_insert_with(PathItem::default);
            match method {
                Method::GET => item.get = item.get.take().or(Some(operation)),
                Method::POST => item.post = item.post.take().or(Some(operation)),
                Method::PUT => item.put = item.put.take().or(Some(operation)),
                Method::DELETE => item.delete = item.delete.take().or(Some(operation)),
                Method::HEAD => item.head = item.head.take().or(Some(operation)),
                Method::OPTIONS => item.options = item.options.take().or(Some(operation)),
                Method::PATCH => item.patch = item.patch.take().or(Some(operation)),
                Method::TRACE => item.trace = item.trace.take().or(Some(operation)),
                _ => unimplemented!(),
            }
        });

    let paths = paths.into_iter()
        .map(|(path, item)| (path, ReferenceOr::Item(item)))
        .collect();

    OpenAPI {
        openapi: "3.0.0".into(),
        paths,
        ..OpenAPI::default()
    }
}
