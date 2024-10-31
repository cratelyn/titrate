use {
    crate::{server::Params, Error},
    bytes::Bytes,
    hyper::body::{Body, Frame},
    pin_project::pin_project,
    std::{
        pin::Pin,
        task::{Context, Poll},
    },
    tap::Pipe,
};

#[pin_project]
pub struct ResponseBody {
    /// a chunk of data in the response body.
    chunk: <Self as Body>::Data,
    /// the number of bytes remaining.
    rem: usize,
}

// === impl ResponseBody ===

impl Body for ResponseBody {
    type Data = Bytes;
    type Error = Error;

    fn poll_frame(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        let this = self.project();
        let chunk = this.chunk;
        let rem = this.rem;

        // return early if we're done.
        if *rem == 0 {
            return Poll::Ready(None);
        }

        // emit the next frame.
        *rem = rem.saturating_sub(chunk.len());
        chunk
            .clone()
            .pipe(Frame::data)
            .pipe(Ok)
            .pipe(Some)
            .pipe(Poll::Ready)
    }
}

impl ResponseBody {
    pub fn new(
        Params {
            body_size,
            frame_size,
            port: _,
        }: &Params,
    ) -> Self {
        let chunk = vec![1; *frame_size].into();
        let rem = *body_size;

        Self { chunk, rem }
    }
}
