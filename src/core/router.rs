use anyhow::anyhow;
use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::core::transport::{Request, Response};

/// RequestTransport is owned by connection sender
pub struct Connection<TSend, TRecv> {
    pub tx: mpsc::Sender<TSend>,
    pub rx: mpsc::Receiver<TRecv>,
}

impl<TSend, TRecv> Connection<TSend, TRecv> {
    pub async fn recv(&mut self) -> anyhow::Result<TRecv> {
        self.rx.recv().await.ok_or(anyhow!("Connection closed"))
    }
}

pub fn make_connection<TSend, TRecv>(
    send_buffer: usize,
    recv_buffer: usize,
) -> (Connection<TSend, TRecv>, Connection<TRecv, TSend>) {
    let (send_tx, send_rx) = mpsc::channel(send_buffer);
    let (recv_tx, recv_rx) = mpsc::channel(recv_buffer);

    (
        Connection {
            tx: send_tx,
            rx: recv_rx,
        },
        Connection {
            tx: recv_tx,
            rx: send_rx,
        },
    )
}

pub type ResponseHandler = Connection<Response, Request>;

/// Request to response transport abstraction
///
/// ```
/// // > Clients
/// // send request ------------------> Received response
/// // --------------------------------------
/// // > Connection to Router
/// // tx                               rx
/// //  |                                |
/// // rx                               tx
/// // --------------------------------------
/// // > Router: We're here
/// // Find responder tx by id         Find requester by id
/// // --------------------------------------
/// // > Connection to service providers
/// // tx                              rx
/// //  |                               |
/// // rx                              tx
/// // --------------------------------------
/// // > Service providers
/// // responder -----> process -----> Response
/// ```
#[async_trait]
pub trait Router {
    async fn route_request(&self, request: Request) -> anyhow::Result<mpsc::Receiver<Response>>;

    async fn register_service(&self, service_id: String) -> anyhow::Result<ResponseHandler>;

    async fn drop_service(&self, service_id: String) -> anyhow::Result<()>;
}
