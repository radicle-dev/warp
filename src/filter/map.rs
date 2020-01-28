use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::{ready, TryFuture};
use pin_project::pin_project;

use super::{Filter, FilterBase, Func, Internal};
use crate::document::{DocumentedFilter, DocumentedReply, RouteDocumentation};

#[derive(Clone, Copy, Debug)]
pub struct Map<T, F> {
    pub(super) filter: T,
    pub(super) callback: F,
}

impl<T, F> FilterBase for Map<T, F>
where
    T: Filter,
    F: Func<T::Extract> + Clone + Send,
{
    type Extract = (F::Output,);
    type Error = T::Error;
    type Future = MapFuture<T, F>;
    #[inline]
    fn filter(&self, _: Internal) -> Self::Future {
        MapFuture {
            extract: self.filter.filter(Internal),
            callback: self.callback.clone(),
        }
    }
}

impl<T, F, D> DocumentedFilter for Map<T, F>
    where
        T: Filter + DocumentedFilter,
        F: Func<T::Extract, Output=D> + Clone + Send,
        D: DocumentedReply,
{
    type Output = Vec<RouteDocumentation>;

    fn document(&self, item: RouteDocumentation) -> Self::Output {
        let Map{ filter, .. } = self;
        filter.document(item)
            .into_iter()
            .flat_map(|item| D::document(item).into_iter())
            .collect()
    }
}

#[allow(missing_debug_implementations)]
#[pin_project]
pub struct MapFuture<T: Filter, F> {
    #[pin]
    extract: T::Future,
    callback: F,
}

impl<T, F> Future for MapFuture<T, F>
where
    T: Filter,
    F: Func<T::Extract>,
{
    type Output = Result<(F::Output,), T::Error>;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let pin = self.project();
        match ready!(pin.extract.try_poll(cx)) {
            Ok(ex) => {
                let ex = (pin.callback.call(ex),);
                Poll::Ready(Ok(ex))
            }
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}
