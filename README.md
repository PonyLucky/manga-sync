# Manga Manager API

A self-hosted REST API written in Rust to manage manga reading progress across devices and sources, for a single user.

### Key Features

- **Security**: Bearer token authentication with SHA-256 hashing. Automatic key generation and rotation (after 365 days) with warnings after 90 days.
- **Persistence**: SQLite database with automatic migrations on startup.
- **Manga Management**: Track manga, sources, and reading history.
- **Website Management**: Manage supported manga websites/domains.
- **Settings**: Simple key-value store for user preferences.
- **Dockerized**: Ready for deployment using Docker with volume support for data persistence.

### Technical Stack

- **Language**: Rust
- **Web Framework**: Axum
- **Runtime**: Tokio
- **Database**: SQLite (via sqlx)
- **Containerization**: Docker

### Security & Authentication

Every request must include the following header:
```
Authorization: Bearer <token>
```

#### API Key Management
On first startup, the API generates a random key (24–64 characters) if `secret/key.pub` does not exist.
- The **plaintext key** is printed **once** to the console.
- Only the **SHA-256 hash** is stored in `secret/key.pub`.
- The key is automatically rotated if it's older than 365 days.
- A warning is logged if the key is older than 90 days.

### Installation & Usage

#### Using Makefile (Recommended for development)

The project includes a `Makefile` to simplify common tasks:
- `make run`: Run the application locally.
- `make test`: Run all tests.
- `make docker-build`: Build the Docker image.
- `make docker-run`: Run the container with a persistent volume.
- `make openapi-update`: Reminds you to keep the OpenAPI spec updated.

#### Using Docker

1. Create a `secret` directory to store your database and API key:
   ```bash
   mkdir secret
   ```
2. Build and run the container:
   ```bash
   make docker-build && make docker-run
   ```
   OR:
   ```bash
   docker build -t manga-sync .
   docker run --name manga-sync -p 7783:7783 -v $(pwd)/secret:/usr/local/bin/secret manga-sync
   ```
3. Watch the console output on the first run to get your generated API key.

#### Local Development

1. Ensure you have Rust and Cargo installed.
2. Run the application:
   ```bash
   cargo run
   ```
3. The API will be available at `http://localhost:7783`.

### API Reference

All responses follow this standard format:
```json
{
  "status": "success" | "error",
  "message": "string",
  "data": any | null
}
```

#### Manga
- `GET /manga`: List paginated manga.
- `GET /manga/:id`: Get detailed manga info.
- `POST /manga`: Create a new manga.
- `PATCH /manga/:id`: Update manga details or progress.
- `DELETE /manga/:id`: Delete a manga (and its sources/history).
- `GET /manga/:id/source`: Get all sources for a manga.
- `DELETE /manga/:id/source/:domain`: Delete a specific source for a manga.
- `GET /manga/:id/history`: Get reading history for a manga.

#### Website
- `GET /website`: List all registered websites.
- `GET /website/:domain`: Check if a website exists.
- `POST /website/:domain`: Register a new website.

#### Settings
- `GET /setting`: Retrieve all settings.
- `POST /setting/:key`: Create or update a setting.

### Persistence

The API uses SQLite for storage. The database file `manga.db` and the authentication hash `key.pub` are stored in the `secret/` directory. Database migrations are applied automatically on startup.

## License

This project is licensed under the GNU AGPLv3 - see the [LICENSE](LICENSE) file for details.
