use actix_web::{
    web, App, HttpServer, HttpResponse, middleware, Error,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};
use dotenvy::dotenv;

// ==================== Data Models ====================

#[derive(Serialize, Deserialize, Clone)]
pub struct Station {
    pub code: String,
    pub name: String,
    pub line: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub address: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Schedule {
    pub station_code: String,
    pub station_name: String,
    pub departure_time: String,
    pub schedule_type: String,
    pub direction: String,
}

#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
    pub meta: Option<ApiMeta>,
}

#[derive(Serialize)]
pub struct ApiMeta {
    pub total: usize,
    pub page: u32,
    pub per_page: u32,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub version: String,
    pub database: String,
}

// ==================== AppState ====================

pub struct AppState {
    pub db: PgPool,
    pub api_keys: Arc<Mutex<HashMap<String, ApiKeyInfo>>>,
    pub rate_limits: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
}

#[derive(Clone)]
pub struct ApiKeyInfo {
    pub name: String,
    pub owner: String,
    pub rate_limit_rpm: u32,
    pub is_active: bool,
}

// ==================== Rate Limiting Helper ====================

fn check_rate_limit(
    state: &AppState,
    peer_addr: &str,
    api_key: Option<&str>,
) -> Result<u32, HttpResponse> {
    let rpm_limit = if let Some(key) = api_key {
        let keys = state.api_keys.lock().unwrap();
        keys.get(key).map(|k| k.rate_limit_rpm).unwrap_or(10)
    } else {
        10 // Default 10 RPM for unauthenticated
    };
    
    let now = Instant::now();
    let window = Duration::from_secs(60);
    
    let mut limits = state.rate_limits.lock().unwrap();
    let requests = limits.entry(peer_addr.to_string()).or_insert_with(Vec::new);
    
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

pub fn validate_api_key(api_key: &str, state: &AppState) -> Result<ApiKeyInfo, String> {
    let keys = state.api_keys.lock().unwrap();
    keys.get(api_key)
        .filter(|k| k.is_active)
        .cloned()
        .ok_or_else(|| "Invalid or inactive API key".to_string())
}

// ==================== API Endpoints ====================

#[actix_web::get("/")]
async fn health(state: web::Data<AppState>) -> HttpResponse {
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

#[actix_web::get("/stations")]
async fn get_stations(
    req: actix_web::HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let conn_info = req.connection_info().clone();
    let peer_addr = conn_info.peer_addr().unwrap_or("127.0.0.1").to_string();
    let api_key = req.headers()
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok());
    
    match check_rate_limit(&state, &peer_addr, api_key) {
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

#[actix_web::get("/schedules")]
async fn get_schedules(
    req: actix_web::HttpRequest,
    query: web::Query<HashMap<String, String>>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let conn_info = req.connection_info().clone();
    let peer_addr = conn_info.peer_addr().unwrap_or("127.0.0.1").to_string();
    let api_key = req.headers()
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok());
    
    match check_rate_limit(&state, &peer_addr, api_key) {
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

#[actix_web::get("/routes")]
async fn get_routes(
    req: actix_web::HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let conn_info = req.connection_info().clone();
    let peer_addr = conn_info.peer_addr().unwrap_or("127.0.0.1").to_string();
    let api_key = req.headers()
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok());
    
    match check_rate_limit(&state, &peer_addr, api_key) {
        Ok(remaining) => {
            match sqlx::query_as::<_, (String, String, Option<String>)>(
                "SELECT route_code, name, description FROM routes ORDER BY id"
            )
            .fetch_all(&state.db)
            .await
            {
                Ok(rows) => {
                    let route_data: Vec<(String, String, Option<String>)> = rows;
                    let total = route_data.len();
                    Ok(HttpResponse::Ok()
                        .append_header(("X-RateLimit-Remaining", remaining.to_string()))
                        .json(ApiResponse {
                            success: true,
                            data: Some(route_data),
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

// ==================== API Key Management ====================

pub async fn seed_api_keys(state: &web::Data<AppState>) {
    let mut keys = state.api_keys.lock().unwrap();
    
    // Add default API keys for testing
    let test_key = "inamute_test_key_2024_abc123def456";
    keys.insert(test_key.to_string(), ApiKeyInfo {
        name: "Test Key".to_string(),
        owner: "system".to_string(),
        rate_limit_rpm: 10,
        is_active: true,
    });
    
    // Production key
    let prod_key = "inamute_prod_key_2024_xyz789ghi012";
    keys.insert(prod_key.to_string(), ApiKeyInfo {
        name: "Production Key".to_string(),
        owner: "ilmimris".to_string(),
        rate_limit_rpm: 10,
        is_active: true,
    });
    
    log::info!("API keys seeded successfully");
    log::info!("Test key: {}", test_key);
    log::info!("Prod key: {}", prod_key);
    log::info!("Use header: X-API-Key: <your-key>");
}

// ==================== Main ====================

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://inamute:inamute_password@localhost:5432/inamute".to_string());
    
    log::info!("Connecting to database: {}", database_url);
    
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
    
    let app_state = web::Data::new(AppState {
        db: pool,
        api_keys: Arc::new(Mutex::new(HashMap::new())),
        rate_limits: Arc::new(Mutex::new(HashMap::new())),
    });
    
    // Seed API keys
    seed_api_keys(&app_state).await;
    
    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    
    log::info!("Starting Inamute API server on {}", bind_addr);
    
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(middleware::Logger::default())
            .service(health)
            .service(get_stations)
            .service(get_schedules)
            .service(get_routes)
    })
    .bind(&bind_addr)?
    .run()
    .await
}
