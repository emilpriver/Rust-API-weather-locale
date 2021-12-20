use worker::*;
use serde_json::json;
use serde::{Serialize, Deserialize};

mod utils;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeatherResponse {
    pub lat: f64,
    pub lon: f64,
    pub timezone: String,
    #[serde(rename = "timezone_offset")]
    pub timezone_offset: i64,
    pub current: Current,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Current {
    pub dt: i64,
    pub sunrise: i64,
    pub sunset: i64,
    pub temp: f64,
    #[serde(rename = "feels_like")]
    pub feels_like: f64,
    pub pressure: i64,
    pub humidity: i64,
    #[serde(rename = "dew_point")]
    pub dew_point: f64,
    pub uvi: i64,
    pub clouds: i64,
    pub visibility: i64,
    #[serde(rename = "wind_speed")]
    pub wind_speed: f64,
    #[serde(rename = "wind_deg")]
    pub wind_deg: i64,
    pub weather: Vec<Weather>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Weather {
    pub id: i64,
    pub main: String,
    pub description: String,
    pub icon: String,
}

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or("unknown region".into())
    );
}

#[event(fetch)]
pub async fn main(req: Request, env: Env) -> Result<Response> {
    log_request(&req);

    // Optionally, get more helpful error messages written to the console in the case of a panic.
    utils::set_panic_hook();

    // Optionally, use the Router to handle matching endpoints, use ":name" placeholders, or "*name"
    // catch-alls to match on specific patterns. Alternatively, use `Router::with_data(D)` to
    // provide arbitrary data that will be accessible in each route via the `ctx.data()` method.
    let router = Router::new();

    // Add as many routes as your Worker needs! Each route will get a `Request` for handling HTTP
    // functionality and a `RouteContext` which you can use to  and get route parameters and
    // Environment bindings like KV Stores, Durable Objects, Secrets, and Variables.
    router
        .get_async("/", |req, ctx| async move {
            let client = reqwest::Client::new();
            let cordinates: (f32, f32) = req.cf().coordinates().unwrap_or_default();
            let (lat,lon) = cordinates;

            let request_url = format!("https://api.openweathermap.org/data/2.5/onecall?lat={lat}&lon={lon}&exclude=hourly,daily,minutely&appid={key}",
            key = ctx.var("WEATHER_OPEN_API_KEY")?.to_string(),
            lat = lat,
            lon = lon);
            println!("{}", request_url);

            let data = client
                .get(request_url)
                .header("content-type", "application/json")
                .header("accept", "application/json")
                .send()
                .await
                .unwrap();

            match data.status() {
                reqwest::StatusCode::OK => {
                    match data.json::<WeatherResponse>().await {
                        Ok(parsed) => {
                            return Response::from_json(&json!(parsed))
                        },
                        Err(_) => return Response::error("Bad Request", 400),
                    };
                }
                reqwest::StatusCode::UNAUTHORIZED => return Response::error("Bad Request", 401),
                _ => return Response::error("Bad Request", 400)
            }
        })
        .get("/worker-version", |_, ctx| {
            let version = ctx.var("WORKERS_RS_VERSION")?.to_string();
            Response::ok(version)
        })
        .run(req, env)
        .await
}
