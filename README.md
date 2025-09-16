# Ben

A Discord bot written in Rust using poise-rs.

![Rust](https://img.shields.io/badge/Rust-🦀-orange)
![License](https://img.shields.io/github/license/k1llbyte/ben)


## Features

- Slash commands (via Discord interactions API)
- Prefix commands (using prefix `!`)
- Logging
- Error handling
- Virtual money
- Finance Simulation
- Gambling


## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable)
- A Discord bot token ([create one here](https://discord.com/developers/applications))

### Setup

1. **Clone the repo**

```bash
git clone https://github.com/k1llbyte/ben.git
cd ben
```

2. **Configure config file**

Create a `config.toml` file in the root directory:

> [!WARNING]
> Do not commit `config.toml` if it contains authentication tokens. Make sure it’s excluded from commits (e.g., via `.gitignore`).

```toml
discord_token = "..."
cmc_api_key = "..."
```

You can find your application ID in the [Discord Developer Portal](https://discord.com/developers/applications).

3. **Build and run the bot**

```bash
cargo run -- -c config.toml
```

## Example Commands

- `/help` – Help command.
- `/bank` - Displays current money balance. If bank account does not exist, creates one.
- `/give <user> <amount>` - Give money to another user.
- `/leaderboard` - Bank leaderboard. Who's the wealthiest.
- `/price <symbol>` - Displays the current price for a specific coin.
- `/portfolio` - Displays list of owned coins amount and profit.
- `/buy <symbol> <amount>` - Buy crypto currency in euros, if successful prints amount of coins bought.
- `/sell <symbol> <amount>` - Sell crypto currency in euros, if successful prints amount of coins bought.
- `/sellall <symbol>` - Sell crypto currency in euros, if successful prints amount of coins bought.
- `/coin <choice> <amount>` - Bet on heads or tails.
- `/daily` - Claim daily reward.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.