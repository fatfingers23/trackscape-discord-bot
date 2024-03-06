use log::info;
use std::env;
use std::future::Future;
use wom_rs::WomClient;

pub fn get_wom_client() -> WomClient {
    let api_key = env::var("WOM_API_KEY").expect("WOM_API_KEY not set!");
    WomClient::new_with_key(api_key)
}

const RATE_LIMIT: i32 = 100;

pub struct ApiLimiter {
    pub one_minute_from_now: chrono::DateTime<chrono::Utc>,
    pub requests_sent: i32,
}

impl ApiLimiter {
    pub fn new() -> Self {
        Self {
            one_minute_from_now: chrono::Utc::now() + chrono::Duration::minutes(1),
            requests_sent: 0,
        }
    }

    pub async fn api_limit_request<Fut, T>(
        &mut self,
        f: impl FnOnce() -> Fut,
        wait_before: Option<std::time::Duration>,
    ) -> anyhow::Result<Vec<T>>
    where
        Fut: Future<Output = anyhow::Result<Vec<T>>>,
    {
        if let Some(wait_before) = wait_before {
            tokio::time::sleep(wait_before).await;
        }

        let time = chrono::Utc::now();
        if time > self.one_minute_from_now {
            self.requests_sent = 0;
            self.one_minute_from_now = chrono::Utc::now() + chrono::Duration::minutes(1);
        }

        if self.requests_sent >= RATE_LIMIT {
            let time_of_rate_limit_reached = chrono::Utc::now();
            let time_until_next_minute = self.one_minute_from_now - time_of_rate_limit_reached;
            info!(
                "Rate limit reached, waiting for next minute. Sleeping for: {:?}",
                time_until_next_minute
            );

            tokio::time::sleep(time_until_next_minute.to_std().unwrap()).await;
            self.requests_sent = 0;
            self.one_minute_from_now = chrono::Utc::now() + chrono::Duration::minutes(1);
        }

        self.requests_sent += 1;
        f().await
    }
}

fn fun_test(value: i32, f: &dyn Fn(i32) -> i32) -> i32 {
    println!("{}", f(value));
    value
}
