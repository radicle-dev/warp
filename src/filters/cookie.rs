//! Cookie Filters

use futures::future;
use headers::Cookie;

use super::header;
use crate::document::{DocumentedCookie, DocumentedResponse, ExplicitDocumentation};
use crate::filter::{Filter, One};
use crate::reject::Rejection;
use std::convert::Infallible;

/// Creates a `Filter` that requires a cookie by name.
///
/// If found, extracts the value of the cookie, otherwise rejects.
pub fn cookie(name: &'static str) -> impl Filter<Extract = One<String>, Error = Rejection> + Copy {
    let filter = header::header2().and_then(move |cookie: Cookie| {
        let cookie = cookie
            .get(name)
            .map(String::from)
            .ok_or_else(|| crate::reject::missing_cookie(name));
        future::ready(cookie)
    });
    ExplicitDocumentation::new(filter, move |route| {
        route.responses.insert(400, DocumentedResponse {
            description: "Bad Response".into(),
            ..DocumentedResponse::default()
        });
        route.cookies.push(DocumentedCookie {
            name: name.to_string(),
            description: None,
            required: true,
        });
    })
}

/// Creates a `Filter` that looks for an optional cookie by name.
///
/// If found, extracts the value of the cookie, otherwise continues
/// the request, extracting `None`.
pub fn optional(
    name: &'static str,
) -> impl Filter<Extract = One<Option<String>>, Error = Infallible> + Copy {
    let filter = header::optional2()
        .map(move |opt: Option<Cookie>| opt.and_then(|cookie| cookie.get(name).map(String::from)));
    ExplicitDocumentation::new(filter, move |route| {
        route.cookies.push(DocumentedCookie {
            name: name.to_string(),
            description: None,
            required: false,
        });
    })
}
