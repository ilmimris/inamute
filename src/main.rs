use actix_web::{web, App, HttpServer, HttpResponse, get};
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub version: String,
}

#[derive(Serialize)]
pub struct Station {
    pub id: String,
    pub name: String,
    pub line: String,
    pub coordinates: Option<(f64, f64)>,
}

#[derive(Serialize)]
pub struct Schedule {
    pub station_id: String,
    pub departure_time: String,
    pub direction: String,
}

#[get("/")]
async fn health() -> HttpResponse {
    HttpResponse::Ok().json(HealthResponse {
        status: "ok".to_string(),
        service: "inamute".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[get("/stations")]
async fn get_stations() -> HttpResponse {
    let stations = vec![
        Station {
            id: "dukuh_atas".to_string(),
            name: "Dukuh Atas".to_string(),
            line: "LRT_Jabodebek".to_string(),
            coordinates: Some((-6.21, 106.82)),
        },
        Station {
            id: "cawang".to_string(),
            name: "Cawang".to_string(),
            line: "LRT_Jabodebek".to_string(),
            coordinates: Some((-6.26, 106.86)),
        },
        Station {
            id: "jati_mulya".to_string(),
            name: "Jati Mulya".to_string(),
            line: "LRT_Jabodebek".to_string(),
            coordinates: Some((-6.29, 106.91)),
        },
    ];
    HttpResponse::Ok().json(stations)
}

#[get("/schedules/{station_id}")]
async fn get_schedules(path: web::Path<String>) -> HttpResponse {
    let station_id = path.into_inner();
    
    let schedules = vec![
        Schedule {
            station_id: station_id.clone(),
            departure_time: "06:00".to_string(),
            direction: "Dukuh Atas".to_string(),
        },
        Schedule {
            station_id: station_id.clone(),
            departure_time: "06:20".to_string(),
            direction: "Dukuh Atas".to_string(),
        },
        Schedule {
            station_id: station_id.clone(),
            departure_time: "06:40".to_string(),
            direction: "Dukuh Atas".to_string(),
        },
    ];
    
    HttpResponse::Ok().json(schedules)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    log::info!("Starting Inamute API server on http://127.0.0.1:8080");
    
    HttpServer::new(|| {
        App::new()
            .service(health)
            .service(get_stations)
            .service(get_schedules)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
