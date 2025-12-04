# Ola Web UI

A web interface for the Ola CLI tool, built with Rocket framework.

## Features

- **Configure LLM Providers**: Set up OpenAI, Anthropic Claude, Google Gemini, or local Ollama
- **Structured Prompting**: Use the Goals/Format/Warnings structure for better AI responses
- **Real-time Responses**: Stream responses from configured LLM providers
- **Beautiful UI**: Clean, modern interface with gradient backgrounds

## Running the Web Server

From the repository root:

```bash
cargo run -p ola-web
```

Or from the `ola-web` directory:

```bash
cargo run
```

The server will start at `http://localhost:8000` by default.

## Configuration

You can configure the server settings in `Rocket.toml`:

- **address**: IP address to bind to (default: 127.0.0.1)
- **port**: Port to listen on (default: 8000)
- **workers**: Number of worker threads (default: 4)

## API Endpoints

### Configuration

- `GET /configure` - Configuration page
- `POST /api/configure` - Save provider configuration

### Prompting

- `GET /` - Main prompting interface
- `POST /api/prompt` - Submit a prompt
- `GET /api/models` - List available models for the configured provider

## Development

The web UI shares core functionality with the CLI through the `ola-core` library, ensuring consistent behavior across both interfaces.

### Project Structure

```
ola-web/
├── src/
│   └── main.rs          # Rocket routes and handlers
├── templates/
│   ├── base.html.tera   # Base template
│   ├── index.html.tera  # Main prompting interface
│   └── configure.html.tera  # Configuration page
├── static/              # Static assets (CSS, JS, images)
├── Cargo.toml           # Dependencies
└── Rocket.toml          # Server configuration
```

## Notes

- The web UI uses the same configuration file as the CLI (`~/.ola/settings.yaml` or `~/.ola/config.json`)
- API keys are stored securely with restricted file permissions
- All prompts use the structured reasoning format for optimal results
