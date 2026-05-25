use actix_web::{
    web, App, HttpServer, HttpResponse, middleware, Error,
    http::StatusCode,
};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};
use dotenvy::dotenv;
use utoipa::{Modify, OpenApi, ToSchema};
use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
use utoipa_swagger_ui::SwaggerUi;

type HmacSha256 = Hmac<Sha256>;
const API_KEY_PREFIX: &str = "inamute_live_";
const MIN_API_KEY_HASH_SECRET_LEN: usize = 32;

// ==================== Data Models ====================

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct Station {
    pub code: String,
    pub name: String,
    pub line: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub address: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct Schedule {
    pub station_code: String,
    pub station_name: String,
    pub departure_time: String,
    pub schedule_type: String,
    pub direction: String,
}

#[derive(Serialize, ToSchema)]
pub struct Route {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
    pub meta: Option<ApiMeta>,
}

#[derive(Serialize, ToSchema)]
pub struct ApiMeta {
    pub total: usize,
    pub page: u32,
    pub per_page: u32,
}

#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub version: String,
    pub database: String,
}

#[derive(ToSchema)]
pub struct EmptyApiResponse {
    pub success: bool,
    pub data: Option<()>,
    pub message: Option<String>,
    pub meta: Option<ApiMeta>,
}

#[derive(ToSchema)]
pub struct HealthApiResponse {
    pub success: bool,
    pub data: Option<HealthResponse>,
    pub message: Option<String>,
    pub meta: Option<ApiMeta>,
}

#[derive(ToSchema)]
pub struct StationsApiResponse {
    pub success: bool,
    pub data: Option<Vec<Station>>,
    pub message: Option<String>,
    pub meta: Option<ApiMeta>,
}

#[derive(ToSchema)]
pub struct RoutesApiResponse {
    pub success: bool,
    pub data: Option<Vec<Route>>,
    pub message: Option<String>,
    pub meta: Option<ApiMeta>,
}

#[derive(ToSchema)]
pub struct SchedulesApiResponse {
    pub success: bool,
    pub data: Option<Vec<Schedule>>,
    pub message: Option<String>,
    pub meta: Option<ApiMeta>,
}

#[derive(OpenApi)]
#[openapi(
    paths(api_health, get_stations, get_routes, get_schedules),
    components(
        schemas(
            ApiMeta,
            EmptyApiResponse,
            HealthResponse,
            HealthApiResponse,
            Station,
            StationsApiResponse,
            Route,
            RoutesApiResponse,
            Schedule,
            SchedulesApiResponse
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "health", description = "Service health and readiness"),
        (name = "stations", description = "Transit station data"),
        (name = "routes", description = "Transit route data"),
        (name = "schedules", description = "Transit schedule queries")
    )
)]
struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "ApiKeyAuth",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("X-API-Key"))),
            );
        }
    }
}

// ==================== AppState ====================

pub struct AppState {
    pub db: PgPool,
    pub api_key_hash_secret: String,
    pub rate_limits: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
}

#[derive(Clone)]
pub struct ApiKeyInfo {
    pub rate_limit_rpm: u32,
    pub key_hash: String,
}

// ==================== Rate Limiting Helper ====================

fn check_rate_limit(
    state: &AppState,
    rate_limit_key: &str,
    rpm_limit: u32,
) -> Result<u32, HttpResponse> {
    let now = Instant::now();
    let window = Duration::from_secs(60);
    
    let mut limits = state.rate_limits.lock().unwrap();
    let requests = limits.entry(rate_limit_key.to_string()).or_insert_with(Vec::new);
    
    // Clean old entries outside the window
    requests.retain(|t| t.elapsed() < window);
    
    if requests.len() >= rpm_limit as usize {
        return Err(HttpResponse::build(StatusCode::TOO_MANY_REQUESTS)
            .append_header(("X-RateLimit-Limit", rpm_limit.to_string()))
            .append_header(("X-RateLimit-Remaining", "0"))
            .json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some(format!("Rate limit exceeded. Maximum {} requests per minute.", rpm_limit)),
                meta: None,
            }));
    }
    
    requests.push(now);
    let remaining = rpm_limit.saturating_sub(requests.len() as u32);
    
    Ok(remaining)
}

// ==================== API Key Authentication ====================

pub fn hash_api_key(api_key: &str, secret: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC accepts keys of any size");
    mac.update(api_key.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

fn is_valid_api_key_format(api_key: &str) -> bool {
    api_key.starts_with(API_KEY_PREFIX)
        && api_key.len() >= API_KEY_PREFIX.len() + 24
        && api_key.len() <= 128
        && api_key
            .as_bytes()
            .iter()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'))
}

fn load_api_key_hash_secret() -> std::io::Result<String> {
    let secret = std::env::var("API_KEY_HASH_SECRET").map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "API_KEY_HASH_SECRET must be set and shared by Actix and the Astro account app",
        )
    })?;

    if secret.len() < MIN_API_KEY_HASH_SECRET_LEN {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "API_KEY_HASH_SECRET must be at least {} characters",
                MIN_API_KEY_HASH_SECRET_LEN
            ),
        ));
    }

    Ok(secret)
}

#[cfg(debug_assertions)]
async fn seed_local_test_api_key(pool: &PgPool, api_key_hash_secret: &str) {
    const TEST_USER_ID: &str = "local-test-user";
    const TEST_API_KEY_ID: &str = "00000000-0000-0000-0000-000000000001";
    const TEST_API_KEY: &str = "inamute_live_local_test_key_2024_abc123def456";

    if let Err(e) = sqlx::query(
        r#"INSERT INTO "user" (id, name, email, "emailVerified")
           VALUES ($1, 'Local Test User', 'local-test@inamute.dev', true)
           ON CONFLICT (id) DO NOTHING"#,
    )
    .bind(TEST_USER_ID)
    .execute(pool)
    .await
    {
        log::warn!("Failed to seed local test user: {}", e);
        return;
    }

    if let Err(e) = sqlx::query(
        "INSERT INTO account_api_keys
            (id, user_id, key_hash, key_prefix, name, rate_limit_rpm, is_active)
         VALUES ($1::uuid, $2, $3, $4, 'Local Swagger test key', 60, true)
         ON CONFLICT (key_hash) DO UPDATE
         SET is_active = true,
             rate_limit_rpm = EXCLUDED.rate_limit_rpm,
             expires_at = NULL",
    )
    .bind(TEST_API_KEY_ID)
    .bind(TEST_USER_ID)
    .bind(hash_api_key(TEST_API_KEY, api_key_hash_secret))
    .bind("inamute_live_local")
    .execute(pool)
    .await
    {
        log::warn!("Failed to seed local test API key: {}", e);
        return;
    }

    log::info!(
        "Local test API key seeded because SEED_LOCAL_TEST_API_KEY=true"
    );
}

async fn validate_api_key(api_key: &str, state: &AppState) -> Result<ApiKeyInfo, HttpResponse> {
    if !is_valid_api_key_format(api_key) {
        return Err(unauthorized_response());
    }

    let key_hash = hash_api_key(api_key, &state.api_key_hash_secret);

    match sqlx::query_as::<_, (i32,)>(
        "SELECT rate_limit_rpm
         FROM account_api_keys
         WHERE key_hash = $1
           AND is_active = true
           AND (expires_at IS NULL OR expires_at > NOW())"
    )
    .bind(&key_hash)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some((rate_limit_rpm,))) => {
            if let Err(e) = sqlx::query(
                "UPDATE account_api_keys SET last_used_at = NOW() WHERE key_hash = $1"
            )
            .bind(&key_hash)
            .execute(&state.db)
            .await
            {
                log::warn!("Failed to update API key last_used_at: {}", e);
            }

            Ok(ApiKeyInfo {
                rate_limit_rpm: rate_limit_rpm.max(1) as u32,
                key_hash,
            })
        }
        Ok(None) => Err(unauthorized_response()),
        Err(e) => {
            log::error!("Error validating API key: {}", e);
            Err(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some("Failed to validate API key".to_string()),
                meta: None,
            }))
        }
    }
}

fn unauthorized_response() -> HttpResponse {
    HttpResponse::Unauthorized().json(ApiResponse::<()> {
        success: false,
        data: None,
        message: Some("Invalid API key".to_string()),
        meta: None,
    })
}

async fn rate_limit_request(
    req: &actix_web::HttpRequest,
    state: &AppState,
) -> Result<u32, HttpResponse> {
    let conn_info = req.connection_info().clone();
    let peer_addr = conn_info.peer_addr().unwrap_or("127.0.0.1").to_string();
    let api_key = req.headers()
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok())
        .filter(|v| !v.trim().is_empty());

    let (rate_limit_key, rpm_limit) = if let Some(key) = api_key {
        let key_info = validate_api_key(key, state).await?;
        (format!("key:{}", key_info.key_hash), key_info.rate_limit_rpm)
    } else {
        (format!("ip:{}", peer_addr), 10)
    };

    check_rate_limit(state, &rate_limit_key, rpm_limit)
}

// ==================== Homepage ====================

#[actix_web::get("/")]
async fn homepage() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../static/index.html"))
}

#[actix_web::get("/docs")]
async fn docs_redirect() -> HttpResponse {
    HttpResponse::Found()
        .append_header(("Location", "/docs/"))
        .finish()
}

// ==================== API Endpoints ====================

#[utoipa::path(
    get,
    path = "/api",
    tag = "health",
    responses(
        (status = 200, description = "Service health and database connectivity", body = HealthApiResponse)
    )
)]
#[actix_web::get("/api")]
async fn api_health(state: web::Data<AppState>) -> HttpResponse {
    let db_status = sqlx::query("SELECT 1")
        .fetch_one(&state.db)
        .await
        .is_ok();
    
    HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some(HealthResponse {
            status: "ok".to_string(),
            service: "inamute".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            database: if db_status { "connected".to_string() } else { "disconnected".to_string() },
        }),
        message: None,
        meta: None,
    })
}

#[utoipa::path(
    get,
    path = "/api/stations",
    tag = "stations",
    security(
        (),
        ("ApiKeyAuth" = [])
    ),
    responses(
        (status = 200, description = "All stations", body = StationsApiResponse),
        (status = 401, description = "Invalid, inactive, or expired API key", body = EmptyApiResponse),
        (status = 429, description = "Rate limit exceeded", body = EmptyApiResponse),
        (status = 500, description = "Failed to fetch stations", body = StationsApiResponse)
    )
)]
#[actix_web::get("/api/stations")]
async fn get_stations(
    req: actix_web::HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    match rate_limit_request(&req, &state).await {
        Ok(remaining) => {
            match sqlx::query_as::<_, (String, String, String, Option<String>, Option<String>, Option<String>)>(
                "SELECT station_code, name, line, latitude::text, longitude::text, address FROM stations ORDER BY id"
            )
            .fetch_all(&state.db)
            .await
            {
                Ok(rows) => {
                    let stations: Vec<Station> = rows.iter().map(|r| {
                        Station {
                            code: r.0.clone(),
                            name: r.1.clone(),
                            line: r.2.clone(),
                            latitude: r.3.as_ref().and_then(|s| s.parse::<f64>().ok()),
                            longitude: r.4.as_ref().and_then(|s| s.parse::<f64>().ok()),
                            address: r.5.clone(),
                        }
                    }).collect();
                    
                    Ok(HttpResponse::Ok()
                        .append_header(("X-RateLimit-Remaining", remaining.to_string()))
                        .json(ApiResponse {
                            success: true,
                            data: Some(stations.clone()),
                            message: None,
                            meta: Some(ApiMeta {
                                total: stations.len(),
                                page: 1,
                                per_page: stations.len() as u32,
                            }),
                        }))
                }
                Err(e) => {
                    log::error!("Error fetching stations: {}", e);
                    Ok(HttpResponse::InternalServerError().json(ApiResponse::<Vec<Station>> {
                        success: false,
                        data: None,
                        message: Some("Failed to fetch stations".to_string()),
                        meta: None,
                    }))
                }
            }
        }
        Err(response) => Ok(response),
    }
}

#[utoipa::path(
    get,
    path = "/api/schedules",
    tag = "schedules",
    params(
        ("station" = Option<String>, Query, description = "Station code, for example DA or JM"),
        ("type" = Option<String>, Query, description = "Schedule type. Defaults to weekday", example = "weekday"),
        ("direction" = Option<String>, Query, description = "Destination direction, for example Dukuh Atas or Jati Mulya")
    ),
    security(
        (),
        ("ApiKeyAuth" = [])
    ),
    responses(
        (status = 200, description = "Schedules matching the query", body = SchedulesApiResponse),
        (status = 401, description = "Invalid, inactive, or expired API key", body = EmptyApiResponse),
        (status = 429, description = "Rate limit exceeded", body = EmptyApiResponse),
        (status = 500, description = "Failed to fetch schedules", body = SchedulesApiResponse)
    )
)]
#[actix_web::get("/api/schedules")]
async fn get_schedules(
    req: actix_web::HttpRequest,
    query: web::Query<HashMap<String, String>>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    match rate_limit_request(&req, &state).await {
        Ok(remaining) => {
            let station_code = query.get("station").map(|s| s.as_str());
            let schedule_type = query.get("type").map(|s| s.as_str()).unwrap_or("weekday");
            let direction = query.get("direction").map(|s| s.as_str());
            
            // Build dynamic query
            let mut sql = String::from(
                "SELECT st.station_code, st.name as station_name, 
                 TO_CHAR(s.departure_time, 'HH24:MI') as departure_time,
                 s.schedule_type, s.direction
                 FROM schedules s
                 JOIN stations st ON s.station_id = st.id
                 WHERE s.schedule_type = $1"
            );
            
            let mut param_count = 1;
            let mut bindings: Vec<String> = vec![schedule_type.to_string()];
            
            if let Some(code) = station_code {
                param_count += 1;
                sql.push_str(&format!(" AND st.station_code = ${}", param_count));
                bindings.push(code.to_string());
            }
            
            if let Some(dir) = direction {
                param_count += 1;
                sql.push_str(&format!(" AND s.direction = ${}", param_count));
                bindings.push(dir.to_string());
            }
            
            sql.push_str(" ORDER BY s.departure_time");
            
            // Execute query with dynamic parameters
            let mut query_builder = sqlx::query_as::<_, (String, String, String, String, Option<String>)>(&sql);
            for binding in bindings.iter() {
                query_builder = query_builder.bind(binding);
            }
            
            match query_builder.fetch_all(&state.db).await {
                Ok(rows) => {
                    let schedules: Vec<Schedule> = rows.iter().map(|r| {
                        Schedule {
                            station_code: r.0.clone(),
                            station_name: r.1.clone(),
                            departure_time: r.2.clone(),
                            schedule_type: r.3.clone(),
                            direction: r.4.clone().unwrap_or_default(),
                        }
                    }).collect();
                    
                    Ok(HttpResponse::Ok()
                        .append_header(("X-RateLimit-Remaining", remaining.to_string()))
                        .json(ApiResponse {
                            success: true,
                            data: Some(schedules.clone()),
                            message: None,
                            meta: Some(ApiMeta {
                                total: schedules.len(),
                                page: 1,
                                per_page: schedules.len() as u32,
                            }),
                        }))
                }
                Err(e) => {
                    log::error!("Error fetching schedules: {}", e);
                    Ok(HttpResponse::InternalServerError().json(ApiResponse::<Vec<Schedule>> {
                        success: false,
                        data: None,
                        message: Some("Failed to fetch schedules".to_string()),
                        meta: None,
                    }))
                }
            }
        }
        Err(response) => Ok(response),
    }
}

#[utoipa::path(
    get,
    path = "/api/routes",
    tag = "routes",
    security(
        (),
        ("ApiKeyAuth" = [])
    ),
    responses(
        (status = 200, description = "All routes", body = RoutesApiResponse),
        (status = 401, description = "Invalid, inactive, or expired API key", body = EmptyApiResponse),
        (status = 429, description = "Rate limit exceeded", body = EmptyApiResponse),
        (status = 500, description = "Failed to fetch routes", body = EmptyApiResponse)
    )
)]
#[actix_web::get("/api/routes")]
async fn get_routes(
    req: actix_web::HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    match rate_limit_request(&req, &state).await {
        Ok(remaining) => {
            match sqlx::query_as::<_, (String, String, Option<String>)>(
                "SELECT route_code, name, description FROM routes ORDER BY id"
            )
            .fetch_all(&state.db)
            .await
            {
                Ok(rows) => {
                    let routes: Vec<Route> = rows.iter().map(|r| Route {
                        code: r.0.clone(),
                        name: r.1.clone(),
                        description: r.2.clone(),
                    }).collect();
                    let total = routes.len();
                    Ok(HttpResponse::Ok()
                        .append_header(("X-RateLimit-Remaining", remaining.to_string()))
                        .json(ApiResponse {
                            success: true,
                            data: Some(routes),
                            message: None,
                            meta: Some(ApiMeta {
                                total,
                                page: 1,
                                per_page: total as u32,
                            }),
                        }))
                }
                Err(e) => {
                    log::error!("Error fetching routes: {}", e);
                    Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                        success: false,
                        data: None,
                        message: Some("Failed to fetch routes".to_string()),
                        meta: None,
                    }))
                }
            }
        }
        Err(response) => Ok(response),
    }
}

// ==================== Main ====================

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://inamute:inamute_password@localhost:5432/inamute".to_string());
    
    log::info!("Connecting to database");
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool");
    
    // Run migrations
    log::info!("Running migrations...");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let api_key_hash_secret = load_api_key_hash_secret()?;

    #[cfg(debug_assertions)]
    if std::env::var("SEED_LOCAL_TEST_API_KEY").as_deref() == Ok("true") {
        seed_local_test_api_key(&pool, &api_key_hash_secret).await;
    }

    let app_state = web::Data::new(AppState {
        db: pool,
        api_key_hash_secret,
        rate_limits: Arc::new(Mutex::new(HashMap::new())),
    });
    
    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    
    log::info!("Starting Inamute API server on {}", bind_addr);
    
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(middleware::Logger::default())
            .service(homepage)
            .service(docs_redirect)
            .service(api_health)
            .service(get_stations)
            .service(get_schedules)
            .service(get_routes)
            .service(
                SwaggerUi::new("/docs/{_:.*}")
                    .url("/api-docs/openapi.json", ApiDoc::openapi())
            )
    })
    .bind(&bind_addr)?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_api_key_uses_sha256_hex() {
        assert_eq!(
            hash_api_key(
                "inamute_live_test",
                "test-secret-that-is-long-enough-for-hmac"
            ),
            "f9ad2f3f60ff8758446eba7916a63b2693330259e895af8e82b0effa72d0b8b1"
        );
    }

    #[test]
    fn api_key_format_rejects_non_live_or_unsafe_values() {
        assert!(is_valid_api_key_format("inamute_live_abcdefghijklmnopqrstuvwxyz"));
        assert!(!is_valid_api_key_format("inamute_test_key_2024_abc123def456"));
        assert!(!is_valid_api_key_format("inamute_live_short"));
        assert!(!is_valid_api_key_format("inamute_live_has space in it"));
    }

    #[actix_rt::test]
    async fn rate_limit_allows_until_limit_then_rejects() {
        let state = AppState {
            db: PgPoolOptions::new().connect_lazy("postgresql://example").unwrap(),
            api_key_hash_secret: "test-secret-that-is-long-enough-for-hmac".to_string(),
            rate_limits: Arc::new(Mutex::new(HashMap::new())),
        };

        assert_eq!(check_rate_limit(&state, "ip:127.0.0.1", 2).unwrap(), 1);
        assert_eq!(check_rate_limit(&state, "ip:127.0.0.1", 2).unwrap(), 0);
        assert!(check_rate_limit(&state, "ip:127.0.0.1", 2).is_err());
    }
}
