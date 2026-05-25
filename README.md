# Inamute

Indonesian commute schedule API for LRT Jabodebek data. Inamute serves a small homepage plus JSON API endpoints for stations, routes, schedules, and service health.

## Features

- Actix Web HTTP server
- PostgreSQL storage with sqlx migrations
- Seed data for LRT Jabodebek stations, routes, and schedules
- Simple API response envelope with metadata
- In-memory API key lookup and per-minute rate limiting
- Docker and Kubernetes deployment files

## Requirements

- Rust 2021 toolchain
- PostgreSQL
- Docker, optional
- Kubernetes with CloudNativePG, optional

## Configuration

The service reads configuration from environment variables, with `.env` support through `dotenvy`.

| Variable | Default | Description |
| --- | --- | --- |
| `DATABASE_URL` | `postgresql://inamute:inamute_password@localhost:5432/inamute` | PostgreSQL connection string |
| `BIND_ADDR` | `0.0.0.0:8080` | Address and port for the HTTP server |
| `RUST_LOG` | `info` | Rust logging filter |

Example `.env`:

```env
DATABASE_URL=postgresql://inamute:inamute_password@localhost:5432/inamute
BIND_ADDR=127.0.0.1:8080
RUST_LOG=info
```

## Local Development

Start PostgreSQL:

```sh
docker run --name inamute-postgres \
  -e POSTGRES_USER=inamute \
  -e POSTGRES_PASSWORD=inamute_password \
  -e POSTGRES_DB=inamute \
  -p 5432:5432 \
  -d postgres:16
```

Run the API:

```sh
cargo run
```

The application runs migrations automatically on startup. Open `http://localhost:8080` for the homepage or `http://localhost:8080/api` for the JSON health endpoint.

## API

All API responses use this shape:

```json
{
  "success": true,
  "data": {},
  "message": null,
  "meta": null
}
```

### `GET /api`

Returns service health and database connectivity.

```sh
curl http://localhost:8080/api
```

### `GET /api/stations`

Returns all stations.

```sh
curl http://localhost:8080/api/stations
```

### `GET /api/routes`

Returns all routes.

```sh
curl http://localhost:8080/api/routes
```

### `GET /api/schedules`

Returns schedules. Optional query parameters:

- `station`: station code, for example `DA` or `JM`
- `type`: `weekday`, `weekend`, or `holiday`; defaults to `weekday`
- `direction`: destination direction, for example `Dukuh Atas` or `Jati Mulya`

```sh
curl "http://localhost:8080/api/schedules?station=JM&type=weekday&direction=Dukuh%20Atas"
```

## Rate Limiting

Requests are rate limited per peer address. Unauthenticated requests default to 10 requests per minute.

You can pass an API key with the `X-API-Key` header:

```sh
curl -H "X-API-Key: inamute_test_key_2024_abc123def456" \
  http://localhost:8080/api/stations
```

Development keys are seeded in memory on startup:

- `inamute_test_key_2024_abc123def456`
- `inamute_prod_key_2024_xyz789ghi012`

## Docker

Build the multi-stage image:

```sh
docker build -t inamute .
```

Run it:

```sh
docker run --rm \
  -e DATABASE_URL=postgresql://inamute:inamute_password@host.docker.internal:5432/inamute \
  -p 8080:8080 \
  inamute
```

`Dockerfile.deploy` is provided for deployments that already have a local `target/release/inamute` binary and only need to package the runtime image.

## Kubernetes

Kubernetes manifests live in `k8s/`:

- `namespace.yaml`: application namespace
- `cnpg-cluster.yaml`: CloudNativePG PostgreSQL cluster
- `db-credentials.yaml`: database credentials secret
- `deployment.yaml`: API deployment
- `service.yaml`: cluster service
- `ingress.yaml`: Traefik ingress for `inamute.mris.dev`

Apply the manifests with:

```sh
kubectl apply -f k8s/namespace.yaml
kubectl apply -f k8s/db-credentials.yaml
kubectl apply -f k8s/cnpg-cluster.yaml
kubectl apply -f k8s/service.yaml
kubectl apply -f k8s/deployment.yaml
kubectl apply -f k8s/ingress.yaml
```

## Project Structure

```text
.
├── src/main.rs              # Actix Web app and API handlers
├── static/index.html        # Homepage
├── migrations/              # PostgreSQL schema and seed data
├── k8s/                     # Kubernetes deployment manifests
├── Dockerfile               # Multi-stage build image
├── Dockerfile.deploy        # Runtime image for pre-built binaries
├── Cargo.toml
└── LICENSE
```

## License

GPL-3.0. See `LICENSE`.
