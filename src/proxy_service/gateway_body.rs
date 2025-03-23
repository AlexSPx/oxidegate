use std::{pin::Pin, task::Poll};

use hyper::{
    body::{Body, Bytes, Incoming},
    Error,
};

pub enum GatewayBody {
    Incomming(Incoming),
    Empty,
}

impl Body for GatewayBody {
    type Data = Bytes;

    type Error = Error;

    fn poll_frame(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Result<hyper::body::Frame<Self::Data>, Self::Error>>> {
        match &mut *self.get_mut() {
            GatewayBody::Incomming(incoming) => Pin::new(incoming).poll_frame(cx),
            GatewayBody::Empty => Poll::Ready(None),
        }
    }
}
