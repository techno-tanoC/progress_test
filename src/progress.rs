use futures::future::FutureExt;
use std::fmt;
use std::io::SeekFrom;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Poll, Context};
use tokio::io::{AsyncSeek, Result, ErrorKind};
use tokio::prelude::*;
use tokio::sync::Mutex;

use crate::item::Item;

struct ProgressInner<T> {
    size: u64,
    buf: T,
}

impl<T> fmt::Debug for ProgressInner<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProgressInner")
            .field("size", &self.size)
            .finish()
    }
}

pub struct Progress<T> {
    inner: Arc<Mutex<ProgressInner<T>>>,
}

impl<T> std::clone::Clone for Progress<T> {
    fn clone(&self) -> Self {
        Progress { inner: self.inner.clone() }
    }
}

impl<T> Progress<T> {
    pub fn new(buf: T) -> Self {
        let inner = Arc::new(Mutex::new(ProgressInner {
            size: 0,
            buf,
        }));
        Progress { inner }
    }

    pub async fn to_size(&self) -> u64 {
        let pg = self.inner.lock().await;
        pg.size
    }
}

impl<T: AsyncRead + Unpin + Send> AsyncRead for Progress<T> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut [u8]
    ) -> Poll<Result<usize>> {
        match self.inner.lock().boxed().as_mut().poll(cx) {
            Poll::Ready(mut s) => {
                Pin::new(&mut s.buf).poll_read(cx, buf)
            },
            Poll::Pending => {
                Poll::Pending
            }
        }
    }
}

impl<T: AsyncWrite + Unpin + Send> AsyncWrite for Progress<T> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &[u8]
    ) -> Poll<Result<usize>> {
        match self.inner.lock().boxed().as_mut().poll(cx) {
            Poll::Ready(mut s) => {
                let poll = Pin::new(&mut s.buf).poll_write(cx, buf);
                if let Poll::Ready(Ok(n)) = poll {
                    s.size += n as u64;
                }
                poll
            },
            Poll::Pending => {
                Poll::Pending
            }
        }
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut Context
    ) -> Poll<Result<()>> {
        match self.inner.lock().boxed().as_mut().poll(cx) {
            Poll::Ready(mut s) => {
                Pin::new(&mut s.buf).poll_flush(cx)
            },
            Poll::Pending => {
                Poll::Pending
            }
        }
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context
    ) -> Poll<Result<()>> {
        match self.inner.lock().boxed().as_mut().poll(cx) {
            Poll::Ready(mut s) => {
                Pin::new(&mut s.buf).poll_shutdown(cx)
            },
            Poll::Pending => {
                Poll::Pending
            }
        }
    }
}

impl<T: AsyncSeek + Unpin + Send> AsyncSeek for Progress<T> {
    fn start_seek(
        self: Pin<&mut Self>,
        cx: &mut Context,
        position: SeekFrom
    ) -> Poll<Result<()>> {
        match self.inner.lock().boxed().as_mut().poll(cx) {
            Poll::Ready(mut s) => {
                Pin::new(&mut s.buf).start_seek(cx, position)
            },
            Poll::Pending => {
                Poll::Pending
            }
        }
    }

    fn poll_complete(
        self: Pin<&mut Self>,
        cx: &mut Context
    ) -> Poll<Result<u64>> {
        match self.inner.lock().boxed().as_mut().poll(cx) {
            Poll::Ready(mut s) => {
                Pin::new(&mut s.buf).poll_complete(cx)
            },
            Poll::Pending => {
                Poll::Pending
            }
        }
    }
}
