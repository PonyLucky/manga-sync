# Manga Manager API

A self-hosted REST API written in Rust to manage manga reading progress across devices and sources, for a single user.

Dockerhub: <https://hub.docker.com/repository/docker/ponylucky/manga-sync>

### Key Features

- **HTTPS**: TLS encryption with auto-generated self-signed certificates for secure connections.
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
- **Caching**: Moka
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

### HTTPS / TLS

The API uses HTTPS for secure connections. By default, if no certificate is found, a self-signed certificate is automatically generated on first startup.

**Certificate files location:**
- `secret/ssl/cert.pem`: TLS certificate
- `secret/ssl/key.pem`: TLS private key

#### Using Let's Encrypt (Recommended for production)

For production use, you should use a trusted certificate from Let's Encrypt instead of self-signed certificates:

1. Install certbot on your host machine:
   ```bash
   # Debian/Ubuntu
   sudo apt install certbot
   # Fedora/RHEL
   sudo dnf install certbot
   # Arch
   sudo pacman -S certbot
   ```

2. Obtain a certificate (replace `your-domain.com` with your actual domain):
   ```bash
   sudo certbot certonly --standalone -d your-domain.com
   ```

3. Copy the certificates to your secret directory:
   ```bash
   sudo cp /etc/letsencrypt/live/your-domain.com/fullchain.pem secret/ssl/cert.pem
   sudo cp /etc/letsencrypt/live/your-domain.com/privkey.pem secret/ssl/key.pem
   sudo chown $USER:$USER secret/ssl/*.pem
   ```

4. Set up automatic renewal (certificates expire every 90 days):
   ```bash
   # Add a cron job or systemd timer to renew and copy certificates
   sudo certbot renew --deploy-hook "cp /etc/letsencrypt/live/your-domain.com/fullchain.pem /path/to/secret/ssl/cert.pem && cp /etc/letsencrypt/live/your-domain.com/privkey.pem /path/to/secret/ssl/key.pem && docker restart manga-sync"
   ```

**Note:** Certbot requires port 80 to be accessible from the internet for domain verification. If you're behind NAT or a firewall, you may need to use the DNS challenge instead (`--preferred-challenges dns`).

#### Using self-signed certificates (Development only)

Self-signed certificates are generated automatically if `secret/ssl/cert.pem` does not exist. Clients need to accept the certificate warning:
- **curl**: Use the `-k` flag: `curl -k https://your-server:7783/manga`
- **Browser**: Visit the API URL directly and accept the certificate warning, or disable OCSP stapling in `about:config` (Firefox).

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

##### Pushing to Docker Hub

1. login if not already `docker login`.
2. Run `make docker-build && make docker-push`.

#### Local Development

1. Ensure you have Rust and Cargo installed.
2. Generate TLS certificates (if not already present):
   ```bash
   mkdir -p secret/ssl
   openssl req -x509 -newkey rsa:4096 -keyout secret/ssl/key.pem \
       -out secret/ssl/cert.pem -days 365 -nodes \
       -subj "/CN=manga-sync/O=Local/C=US"
   ```
3. Run the application:
   ```bash
   cargo run
   ```
4. The API will be available at `https://localhost:7783` (you'll need to accept the self-signed certificate).

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
- `POST /manga/:id/source`: Add a new source to a manga.
- `DELETE /manga/:id/source/:domain`: Delete a specific source for a manga.
- `GET /manga/:id/history`: Get reading history for a manga.
- `POST /manga/refresh-unread`: Refresh all unread manga.

#### Source
- `GET /source`: List all sources.

#### Website
- `GET /website`: List all registered websites.
- `GET /website/:domain`: Check if a website exists.
- `POST /website/:domain`: Register a new website.
- `DELETE /website/:domain`: Delete a website.

#### Settings
- `GET /setting`: Retrieve all settings.
- `PATCH /setting/:key`: Update a setting.

#### Key
- `GET /key`: Get API key age information.
- `POST /key`: Refresh the API key.

### Persistence

The API uses SQLite for storage. The following files are stored in the `secret/` directory:
- `manga.db`: SQLite database
- `key.pub`: SHA-256 hash of the authentication key
- `ssl/cert.pem`: TLS certificate
- `ssl/key.pem`: TLS private key

Database migrations are applied automatically on startup.

## License

This project is licensed under the GNU AGPLv3 - see the [LICENSE](LICENSE) file for details.
