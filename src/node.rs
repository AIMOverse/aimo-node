use std::{path::PathBuf, process, sync::Arc};

use tokio::task::JoinSet;

use crate::{
    config::ServerOptions,
    db::{self, StateDb},
    router::local::LocalRouter,
    server::{self, ServiceContext},
};

enum TaskFinishBehaviour {
    /// Exit the program when the task finishes
    Abort(&'static str),
    // Restart,
}

/// Run the full node service (server + router)
pub async fn run_serve(
    addr: String,
    port: u16,
    _id: Option<PathBuf>,
    state_db_dir: Option<PathBuf>,
) {
    let state_db = Arc::new(
        StateDb::load_or_create(&state_db_dir.unwrap_or(db::default_directory()))
            .expect("Failed to create state db"),
    );
    let server_options = ServerOptions { addr, port };
    let router_instance = Arc::new(LocalRouter::new());

    let mut tasks_js = JoinSet::new();

    // The router task
    let router_cloned = router_instance.clone();
    tasks_js.spawn(async move {
        router_cloned.run().await;

        // Should run forever
        TaskFinishBehaviour::Abort("Router aborted unexpectedly")
    });

    // The server task
    tasks_js.spawn(async move {
        let ctx = ServiceContext::new(router_instance);
        tracing::info!("API server task created.");
        server::serve(&server_options, ctx, state_db).await;

        // This should run forever,
        TaskFinishBehaviour::Abort("API server aborted unexpectedly")
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
