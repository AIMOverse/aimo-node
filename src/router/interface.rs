use std::pin::Pin;

use anyhow::anyhow;
use tokio::sync::mpsc;

use crate::router::{Request, Response};

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

pub type RequestHandler = Connection<Request, Response>;
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
pub trait Router {
    fn route_request(
        &self,
        request: Request,
    ) -> impl Future<Output = anyhow::Result<mpsc::Receiver<Response>>>;

    fn register_service(
        &self,
        service_id: String,
    ) -> impl Future<Output = anyhow::Result<ResponseHandler>>;
}

pub trait RouterDyn {
    fn route_request(
        &self,
        request: Request,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<mpsc::Receiver<Response>>> + '_>>;

    fn register_service(
        &self,
        service_id: String,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<ResponseHandler>> + '_>>;
}

impl<T: Router> RouterDyn for T {
    fn register_service(
        &self,
        service_id: String,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<ResponseHandler>> + '_>> {
        Box::pin(self.register_service(service_id))
    }

    fn route_request(
        &self,
        request: Request,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<mpsc::Receiver<Response>>> + '_>> {
        Box::pin(self.route_request(request))
    }
}
