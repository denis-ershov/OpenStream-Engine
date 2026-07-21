//! Singleflight: N клиентов на один URL → один in-flight compute.

use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;

use parking_lot::Mutex;
use thiserror::Error;
use tokio::sync::broadcast;

#[derive(Debug, Error, Clone)]
#[error("{0}")]
pub struct CoalesceError(pub String);

type WaitTx = broadcast::Sender<Result<Arc<[u8]>, CoalesceError>>;

/// Coalescer по строковому ключу (обычно cache identity / URL).
pub struct Singleflight {
    inflight: Mutex<HashMap<String, WaitTx>>,
}

impl Default for Singleflight {
    fn default() -> Self {
        Self::new()
    }
}

impl Singleflight {
    pub fn new() -> Self {
        Self {
            inflight: Mutex::new(HashMap::new()),
        }
    }

    /// Если ключ уже в полёте — ждём результат лидера; иначе выполняем `work`.
    pub async fn run<F, Fut>(
        &self,
        key: impl Into<String>,
        work: F,
    ) -> Result<Arc<[u8]>, CoalesceError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<Vec<u8>, CoalesceError>>,
    {
        let key = key.into();

        // Фаза 1: либо подписка, либо регистрация лидера (без await под lock).
        enum Role {
            Leader(WaitTx),
            Follower(broadcast::Receiver<Result<Arc<[u8]>, CoalesceError>>),
        }

        let role = {
            let mut map = self.inflight.lock();
            if let Some(tx) = map.get(&key) {
                Role::Follower(tx.subscribe())
            } else {
                let (tx, _rx) = broadcast::channel(16);
                map.insert(key.clone(), tx.clone());
                Role::Leader(tx)
            }
        };

        match role {
            Role::Follower(mut rx) => match rx.recv().await {
                Ok(r) => r,
                Err(broadcast::error::RecvError::Closed) => {
                    Err(CoalesceError("coalesce channel closed".into()))
                }
                Err(broadcast::error::RecvError::Lagged(_)) => {
                    Err(CoalesceError("coalesce lagged".into()))
                }
            },
            Role::Leader(tx) => {
                let result = match work().await {
                    Ok(v) => Ok(Arc::<[u8]>::from(v.into_boxed_slice())),
                    Err(e) => Err(e),
                };
                {
                    let mut map = self.inflight.lock();
                    map.remove(&key);
                }
                let _ = tx.send(result.clone());
                result
            }
        }
    }

    pub fn inflight_len(&self) -> usize {
        self.inflight.lock().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    #[tokio::test]
    async fn coalesces_concurrent_work() {
        let sf = Arc::new(Singleflight::new());
        let calls = Arc::new(AtomicUsize::new(0));

        let mut handles = Vec::new();
        for _ in 0..8 {
            let sf = sf.clone();
            let calls = calls.clone();
            handles.push(tokio::spawn(async move {
                sf.run("https://cdn/x.m3u8", || {
                    let calls = calls.clone();
                    async move {
                        calls.fetch_add(1, Ordering::SeqCst);
                        tokio::time::sleep(Duration::from_millis(30)).await;
                        Ok(b"body".to_vec())
                    }
                })
                .await
            }));
        }

        let mut bodies = Vec::new();
        for h in handles {
            bodies.push(h.await.unwrap().unwrap());
        }
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(bodies.iter().all(|b| b.as_ref() == b"body"));
    }
}
