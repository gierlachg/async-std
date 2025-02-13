use std::future::Future;
use std::pin::Pin;
use std::default::Default;
use pin_project_lite::pin_project;

use crate::stream::Stream;
use crate::task::{Context, Poll};

pin_project! {
    #[derive(Debug)]
    #[allow(missing_debug_implementations)]
    #[cfg(all(feature = "default", feature = "unstable"))]
    #[cfg_attr(feature = "docs", doc(cfg(unstable)))]
    pub struct PartitionFuture<S, F, B> {
        #[pin]
        stream: S,
        f: F,
        res: Option<(B, B)>,
    }
}

impl<S, F, B: Default> PartitionFuture<S, F, B> {
    pub(super) fn new(stream: S, f: F) -> Self {
        Self {
            stream,
            f,
            res: Some((B::default(), B::default())),
        }
    }
}

impl<S, F, B> Future for PartitionFuture<S, F, B>
where
    S: Stream + Sized,
    F: FnMut(&S::Item) -> bool,
    B: Default + Extend<S::Item>,
{
    type Output = (B, B);

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        loop {
            let next = futures_core::ready!(this.stream.as_mut().poll_next(cx));

            match next {
                Some(v) => {
                    let res = this.res.as_mut().unwrap();
                    match (this.f)(&v) {
                        true => res.0.extend(Some(v)),
                        false => res.1.extend(Some(v)),
                    };
                }
                None => return Poll::Ready(this.res.take().unwrap()),
            }
        }
    }
}
