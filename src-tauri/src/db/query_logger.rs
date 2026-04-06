use crate::state::DnsQueryLog;
use tokio::sync::mpsc;

/// Async query logger that buffers DNS queries via mpsc channel
pub struct QueryLogger {
    sender: mpsc::Sender<DnsQueryLog>,
}

impl QueryLogger {
    /// Create a new QueryLogger with the specified buffer size
    /// Returns (logger, receiver) pair
    pub fn new(buffer_size: usize) -> (Self, mpsc::Receiver<DnsQueryLog>) {
        let (sender, receiver) = mpsc::channel(buffer_size);
        (Self { sender }, receiver)
    }

    /// Log a DNS query entry (non-blocking send)
    pub async fn log_query(&self, entry: DnsQueryLog) -> Result<(), Box<dyn std::error::Error>> {
        self.sender.send(entry).await.map_err(|e| e.into())
    }
}

/// Process a batch of query logs from the receiver
/// Collects up to batch_size entries or waits for timeout_ms
pub async fn process_query_log_batch(
    receiver: &mut mpsc::Receiver<DnsQueryLog>,
    batch_size: usize,
    timeout_ms: u64,
) -> Vec<DnsQueryLog> {
    let mut batch = Vec::with_capacity(batch_size);
    let timeout = tokio::time::Duration::from_millis(timeout_ms);

    loop {
        match tokio::time::timeout(timeout, receiver.recv()).await {
            Ok(Some(entry)) => {
                batch.push(entry);
                if batch.len() >= batch_size {
                    return batch;
                }
            }
            Ok(None) => return batch, // Channel closed
            Err(_) => return batch,   // Timeout
        }
    }
}
