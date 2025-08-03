use std::process;

use tokio::task::JoinSet;

use crate::config::Config;

mod config;
mod server;
mod transport;
mod types;

enum TaskFinishBehaviour {
    /// Exit the program when the task finishes
    Abort(&'static str),
    // Restart,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let Config { api_options } = Config::from_env();

    let mut tasks_js = JoinSet::new();

    // The server task
    tasks_js.spawn(async move {
        tracing::info!("API server task created.");
        server::serve(&api_options).await;

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
