# Fantasy Premier League Team Validator

A Rust command-line tool that validates Fantasy Premier League teams against custom league rules by fetching live data from the official FPL API.

## Features

- 🏆 **Price Validation**: Ensures no players cost 10m or more
- 🏟️ **Club Diversity**: Validates maximum one player per club
- 🆙 **Promotion Rule**: Enforces inclusion of players from newly promoted clubs
- 📊 **Live Data**: Fetches current player prices and team compositions from the FPL API
- 🚀 **Fast & Reliable**: Built in Rust for performance and reliability

## Installation

### Prerequisites
- Rust 1.70+ (install from [rustup.rs](https://rustup.rs/))

### Build from Source
```bash
git clone <your-repo-url>
cd fpl-validator
cargo build --release
```

The binary will be available at `target/release/fpl-validator` (or your project name).

## Usage

### Validate Specific Teams
```bash
# Validate multiple teams
cargo run 396409 2239760 2186577 258293 761504 7718758 2242306 8828197

# Validate a single team
cargo run 396409
```

### Using the Binary
```bash
# After building with --release
./target/release/fpl-validator 396409 2239760 258293
```

## How It Works

1. **Fetches FPL data**: Downloads current player prices, names, and club information from the FPL API
2. **Retrieves Team Data**: For each team ID, fetches the current gameweek lineup and captain selection
3. **Validates Rules**: Applies three validation rules:
    - No players costing 10m or more
    - Maximum one player per Premier League club
    - Must include players from newly promoted clubs (Burnley, Sheffield United, Luton Town)
4. **Reports Violations**: Displays colorful error messages for any rule violations

## Sample Output

```
Womp womp! Shane has gone overbudget with Palmer (10.5m) and Haaland (14m)

Oh dear, oh dear! Jake has more than 1 player from Arsenal (Gabriel and Saliba)

Yikes! Harry has not included players from Burnley
```

## Project Structure

```
src/
├── main.rs           # Main entry point and orchestration
├── constants.rs      # Configuration constants
├── models.rs         # Data structures and types
├── api.rs           # HTTP client for FPL API
├── builders.rs      # Data transformation logic
└── validators.rs    # Validation rule implementations
```

## Configuration

The validator is configured with constants in `src/constants.rs`:

- **Newly Promoted Clubs**: Club IDs for teams that must be represented
- **Default Team IDs**: Fallback list when no command-line arguments provided
- **API Endpoints**: FPL API URLs for data fetching

## Dependencies

- **serde**: JSON serialization/deserialization
- **ureq**: Lightweight HTTP client
- **indexmap**: Ordered hash maps for consistent output

## API Data Sources

This tool uses the official Fantasy Premier League API:
- `https://fantasy.premierleague.com/api/bootstrap-static/` - Player and club data
- `https://fantasy.premierleague.com/api/entry/{team_id}/` - Team metadata
- `https://fantasy.premierleague.com/api/entry/{team_id}/event/{gameweek}/picks/` - Team lineups

## Testing

Run the test suite:
```bash
cargo test
```

The project includes comprehensive unit tests for all validation rules and data transformation logic.

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Disclaimer

This tool is not affiliated with the Premier League or Fantasy Premier League. It's a personal project for validating custom league rules.
