use crate::node::pool::NodePool;
use tokio::time::{self, Duration};

pub async fn node_health_check_task(node: NodePool) {
    loop {
        let health = node.get().await.map(drop).inspect_err(|_| {
            // error is already logged by the node pool
        });

        // Set delay based on health status
        let delay = Duration::from_secs(if health.is_ok() { 10 } else { 2 });

        time::sleep(delay).await;
    }
}
