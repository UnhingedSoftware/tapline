mod fetch;
#[cfg(feature = "testing")]
pub mod testing;

pub use fetch::{Fetch, FetchError, Request, Response};

use std::future::Future;
use std::io;
use std::time::{Duration, SystemTime};

pub trait Transport: Send {
    fn send(&mut self, message: &[u8]) -> impl Future<Output = io::Result<()>> + Send;

    fn recv(&mut self) -> impl Future<Output = io::Result<Vec<u8>>> + Send;

    fn close(&mut self) -> impl Future<Output = io::Result<()>> + Send;
}

pub trait Sink: Send + Sync {
    fn write_at(&self, offset: u64, data: &[u8]) -> impl Future<Output = io::Result<()>> + Send;

    fn allocate(&self, len: u64) -> impl Future<Output = io::Result<()>> + Send;

    fn sync(&self) -> impl Future<Output = io::Result<()>> + Send;
}

pub trait Clock: Send + Sync {
    fn now(&self) -> SystemTime;

    fn sleep(&self, duration: Duration) -> impl Future<Output = ()> + Send;
}

#[cfg(all(test, feature = "testing"))]
mod tests {
    use super::*;
    use crate::testing::{MemoryTransport, block_on};

    #[test]
    fn a_memory_backed_transport_satisfies_the_trait() {
        let mut transport = MemoryTransport::new(vec![b"hello".to_vec()]);

        block_on(async {
            let message = transport.recv().await.expect("must receive");
            assert_eq!(message, b"hello");

            transport.send(b"reply").await.expect("must send");
            assert_eq!(transport.sent(), &[b"reply".to_vec()]);

            let error = transport.recv().await.expect_err("must report EOF");
            assert_eq!(error.kind(), io::ErrorKind::UnexpectedEof);
        });
    }
}
