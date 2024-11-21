use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use reqwest;
use serde::Deserialize;
use std::env;
use std::time::Instant;

#[derive(Deserialize)]
struct Info {
    id: String,
}

async fn get_videos(info: web::Path<Info>) -> impl Responder {
    let start = Instant::now();
    let id = info.id.replace("UC", "UU");

    let api_key = match env::var("YOUTUBE_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({ "error": "YOUTUBE_API_KEY is not set" }));
        }
    };

    println!("Fetching videos for playlist {}", id);

    let url = format!(
        "https://youtube.googleapis.com/youtube/v3/playlistItems?part=snippet&playlistId={}&key={}&maxResults=5",
        id, api_key
    );

    let response = match reqwest::get(&url).await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(data) => {
                        if let Some(items) = data["items"].as_array() {
                            let videos: Vec<_> = items
                                .iter()
                                .filter_map(|item| {
                                    let video_id = item["snippet"]["resourceId"]["videoId"]
                                        .as_str()
                                        .map(|s| s.to_string());
                                    let title =
                                        item["snippet"]["title"].as_str().map(|s| s.to_string());

                                    match (video_id, title) {
                                        (Some(video_id), Some(title)) => Some(serde_json::json!({
                                            "videoId": video_id,
                                            "title": title
                                        })),
                                        _ => None,
                                    }
                                })
                                .collect();

                            let duration = start.elapsed();
                            HttpResponse::Ok()
                                .insert_header((
                                    "X-Response-Time",
                                    format!("{}ms", duration.as_millis()),
                                ))
                                .json(videos)
                        } else {
                            HttpResponse::NotFound()
                                .json(serde_json::json!({ "error": "No videos found" }))
                        }
                    }
                    Err(_) => HttpResponse::InternalServerError()
                        .json(serde_json::json!({ "error": "Failed to parse response" })),
                }
            } else {
                HttpResponse::InternalServerError()
                    .json(serde_json::json!({ "error": response.status().to_string() }))
            }
        }
        Err(_) => HttpResponse::InternalServerError()
            .json(serde_json::json!({ "error": "Request failed" })),
    };

    response
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    let port = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("Invalid port number");

    println!("Server is running on http://127.0.0.1:{}", port);

    HttpServer::new(|| {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header(),
            )
            .route("/get/{id}", web::get().to(get_videos))
            .route("/get/{id}/", web::get().to(get_videos))
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
