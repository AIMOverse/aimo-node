use std::{collections::HashMap, sync::Arc, time::Duration};

use serde_json::Value;
use tokio::time::sleep;

use crate::core::transport::{Request, Response};
use crate::router::{Router, local::LocalRouter};

async fn echo_delay_service(request: Request, delay_ms: u64) -> serde_json::Result<Value> {
    sleep(Duration::from_micros(delay_ms)).await;

    println!(
        "Serving request {}\nbody:\n{}\nafter sleeping for {}ms",
        request.request_id, request.payload, delay_ms
    );
    serde_json::from_str::<Value>(&request.payload)
}

#[tokio::test]
async fn test_local_router() {
    // tracing_subscriber::fmt::init();

    let router = Arc::new(LocalRouter::new());

    let router_clone = router.clone();
    let jh = tokio::spawn(async move {
        router_clone.run().await;
    });

    let router_clone = router.clone();
    let srv_jh = tokio::spawn(async move {
        let mut connection = router_clone
            .register_service("test_service_id".to_string())
            .await
            .unwrap();

        loop {
            let request = connection.recv().await.unwrap();

            assert_eq!(request.service_id, "test_service_id");
            let tx = connection.tx.clone();
            tokio::spawn(async move {
                if let Ok(body) = echo_delay_service(request.clone(), 0).await {
                    tx.send(Response {
                        request_id: request.request_id,
                        status_code: 200,
                        content_type: "json".to_string(),
                        payload: body.to_string(),
                        headers: HashMap::new(),
                        is_stream_chunk: false,
                        stream_done: false,
                    })
                    .await
                    .unwrap();
                    println!("service: Response result sent");
                }
            });
        }
    });

    let router_clone = router.clone();
    tokio::spawn(async move {
        let mut reciever = router_clone
            .route_request(Request {
                sender_id: "sender".to_string(),
                request_id: "request111".to_string(),
                service_id: "test_service_id".to_string(),
                endpoint: None,
                request_type: "test".to_string(),
                payload: "{\"ping\":\"pong\"}".to_string(),
                headers: HashMap::new(),
                payload_encrypted: false,
                signature: None,
                method: "GET".to_string(),
            })
            .await
            // This should not fail
            .unwrap();

        println!("Response received");

        let response = reciever.recv().await.unwrap();
        assert_eq!(response.request_id, "request111");
        assert_eq!(response.payload, "{\"ping\":\"pong\"}");

        // Done. Exit service threads.
        srv_jh.abort();
        jh.abort();
    })
    .await
    .unwrap();
}
