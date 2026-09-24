use std::time::Duration;

use anyhow::Result;
use reqwest::{RequestBuilder, Response, StatusCode};

const ATTEMPTS: u32 = 3;
const MAX_PAUSE: Duration = Duration::from_secs(30);

// Провайдеры отвечают 429, когда упёрлись в лимит за минуту, и сами говорят, сколько ждать.
pub async fn send(request: RequestBuilder) -> Result<Response> {
    for attempt in 0..ATTEMPTS {
        let Some(copy) = request.try_clone() else {
            break;
        };

        let response = copy.send().await?;
        if response.status() != StatusCode::TOO_MANY_REQUESTS {
            return Ok(response);
        }

        tokio::time::sleep(pause(&response, attempt)).await;
    }

    Ok(request.send().await?)
}

fn pause(response: &Response, attempt: u32) -> Duration {
    let asked = response
        .headers()
        .get("retry-after")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<f64>().ok())
        .map(Duration::from_secs_f64);

    // Секунда сверху: провайдер считает свой лимит по своим часам, а не по нашим.
    asked
        .unwrap_or_else(|| Duration::from_secs(2u64 << attempt))
        .saturating_add(Duration::from_secs(1))
        .min(MAX_PAUSE)
}
