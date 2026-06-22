# rmeet

Timezone comparison table generator for international meeting scheduling.

## Install

Grab the [latest release]( https://github.com/simon-b/rmeet/releases/latest), or on Mac/Linux:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/simon-b/rmeet/releases/latest/download/rmeet-installer.sh | sh
```

## Usage

```bash
# Compare timezones for 24 hours
rmeet LAX CDG AKL

# Show 4 hours starting now
rmeet LAX CDG -n 4

# Plan meeting 3 hours from now, show next 8 hours
rmeet JFK LHR NRT -n 8 -s 3
```

## Options

- `-n, --hours <N>` - Number of hours to display (default: 24)
- `-s, --start-offset <N>` - Start N hours from current time (default: 0)

## Color Coding

- **Red**: Poor meeting times (11pm-5am)
- **Yellow**: Suboptimal times (early morning/late evening)
- **Green**: Good meeting times

## Features

- System timezone for "Current" column
- Airport timezone data bundled at compile time
- Unicode table formatting
- Error handling with helpful suggestions

## Data

Airport timezone/city data in `src/airports.json` is trimmed from
[mwgg/Airports](https://github.com/mwgg/Airports) (MIT License, see
`THIRD_PARTY_NOTICES.md`).