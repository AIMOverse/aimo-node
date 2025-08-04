use std::{process, sync::Arc};

use tokio::task::JoinSet;

use crate::{config::Config, router::local::LocalRouter};

mod config;
mod router;
mod server;
mod types;

enum TaskFinishBehaviour {
    /// Exit the program when the task finishes
    Abort(&'static str),
    // Restart,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let Config { server_options } = Config::from_env();

    let router_instance = Arc::new(LocalRouter::new());

    let mut tasks_js = JoinSet::new();

    // The server task
    tasks_js.spawn(async move {
        tracing::info!("API server task created.");
        server::serve(&server_options).await;

        // This should run forever,
        TaskFinishBehaviour::Abort("API server aborted unexpectedly")
    });

    // The router task
    tasks_js.spawn(async move {
        router_instance.run().await;

        // Should run forever
        TaskFinishBehaviour::Abort("Router aborted unexpectedly")
    });

    while let Ok(finish_behaviour) = tasks_js
        .join_next()
        .await
        // We guarantee the JoinSet is not empty
        .unwrap()
    {
        match finish_behaviour {
            // Abort the process
            TaskFinishBehaviour::Abort(reason) => {
                tracing::error!("Process aborted: {reason}");
                process::exit(1);
            }
        }
    }
}
