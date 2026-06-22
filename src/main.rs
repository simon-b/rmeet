use chrono::{Datelike, Local, TimeZone, Timelike, Utc, Weekday};
use chrono_tz::Tz;
use clap::Parser;
use colored::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

const COLUMN_WIDTH: usize = 16;
const AIRPORTS_JSON: &str = include_str!("airports.json");

// Custom error type for better error handling
#[derive(Debug)]
enum AppError {
    NoAirportFound(String),
    TimezoneError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::NoAirportFound(code) => write!(f, "No airport found for: {}", code),
            AppError::TimezoneError(e) => write!(f, "Timezone error: {}", e),
        }
    }
}

impl Error for AppError {}

// Airport data embedded from airports.json (IATA code -> timezone + city)
#[derive(Debug, Clone, Deserialize)]
struct AirportData {
    tz: String,
    city: String,
}

/// Program to generate timezone comparison tables for meeting scheduling
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// List of airport IATA codes to compare timezones
    #[arg(num_args = 1..)]
    airports: Vec<String>,

    /// Number of hours to show in the table
    #[arg(short = 'n', long, default_value = "24")]
    hours: usize,

    /// Starting hour offset from current time
    #[arg(short = 's', long, default_value = "0")]
    start_offset: isize,
}

fn load_airport_data() -> HashMap<String, AirportData> {
    serde_json::from_str(AIRPORTS_JSON).expect("bundled airports.json is malformed")
}

fn parse_timezone(tz_str: &str) -> Result<Tz, AppError> {
    tz_str
        .parse::<Tz>()
        .map_err(|_| AppError::TimezoneError(format!("Invalid timezone: {}", tz_str)))
}

fn get_time_color(hour: u32, weekday: Weekday) -> colored::Color {
    let is_weekend = weekday == Weekday::Sat || weekday == Weekday::Sun;

    // Red: 11pm - 5am (23:00 - 05:00)
    if !(5..23).contains(&hour) {
        return colored::Color::Red;
    }

    // Yellow conditions
    if is_weekend {
        // Yellow on weekends: 5am-11pm (05:00 - 23:00)
        if (5..23).contains(&hour) {
            return colored::Color::Yellow;
        }
    } else {
        // Yellow on weekdays: 7pm-11pm (19:00 - 23:00) or 5am-9am (05:00 - 09:00)
        if (19..23).contains(&hour) || (5..9).contains(&hour) {
            return colored::Color::Yellow;
        }
    }

    // Default: no color (normal/green for good times)
    colored::Color::Green
}

// Truncates to at most `max_chars` characters, respecting UTF-8 char boundaries
fn truncate_chars(s: &str, max_chars: usize) -> String {
    s.chars().take(max_chars).collect()
}

fn format_colored_time<Tz2>(dt: &chrono::DateTime<Tz2>) -> (String, String)
where
    Tz2: TimeZone,
    Tz2::Offset: fmt::Display,
{
    let time_str = dt.format("%A %H:%M").to_string();
    let display_str = if time_str.len() > COLUMN_WIDTH {
        format!("{}...", truncate_chars(&time_str, 13))
    } else {
        time_str
    };

    let color = get_time_color(dt.hour(), dt.date_naive().weekday());
    let colored_str = match color {
        colored::Color::Red => display_str.red().to_string(),
        colored::Color::Yellow => display_str.yellow().to_string(),
        colored::Color::Green => display_str.green().to_string(),
        _ => display_str.clone(),
    };

    (display_str, colored_str)
}

fn generate_timezone_table(
    airports_data: &HashMap<String, AirportData>,
    airport_codes: &[String],
    hours: usize,
    start_offset: isize,
) -> Result<(), AppError> {
    // Validate all airport codes and get their data
    let mut airport_info = Vec::new();
    for code in airport_codes {
        let upper_code = code.to_uppercase();
        if let Some(data) = airports_data.get(&upper_code) {
            let timezone = parse_timezone(&data.tz)?;
            airport_info.push((upper_code.clone(), data.city.clone(), timezone));
        } else {
            return Err(AppError::NoAirportFound(code.clone()));
        }
    }

    // Get current local time
    let now = Local::now();
    let start_time = now + chrono::Duration::hours(start_offset as i64);

    // Create table header
    print!("┏");
    print!("{}", "━".repeat(COLUMN_WIDTH));
    for _ in &airport_info {
        print!("┳");
        print!("{}", "━".repeat(COLUMN_WIDTH));
    }
    println!("┓");

    // Print header row
    print!("┃{:>width$}", "Current", width = COLUMN_WIDTH);
    for (code, city, _) in &airport_info {
        let header = format!("{} ({})", code, city);
        print!(
            "┃{:>width$}",
            if header.len() > COLUMN_WIDTH {
                format!("{}...", truncate_chars(&header, 13))
            } else {
                header
            },
            width = COLUMN_WIDTH
        );
    }
    println!("┃");

    // Print separator
    print!("┡");
    print!("{}", "━".repeat(COLUMN_WIDTH));
    for _ in &airport_info {
        print!("╇");
        print!("{}", "━".repeat(COLUMN_WIDTH));
    }
    println!("┩");

    // Generate time rows
    for i in 0..hours {
        let current_time = start_time + chrono::Duration::hours(i as i64);

        // Format current time with color (use system local time)
        let local_time_for_display = current_time.with_timezone(&Local);
        let (current_plain, current_colored) = format_colored_time(&local_time_for_display);

        // Calculate padding for colored text
        let current_padding = COLUMN_WIDTH - current_plain.len();
        print!("│{}{}", " ".repeat(current_padding), current_colored);

        // Convert to each timezone with color
        for (_, _, tz) in &airport_info {
            let utc_time = current_time.with_timezone(&Utc);
            let local_time = tz.from_utc_datetime(&utc_time.naive_utc());
            let (time_plain, time_colored) = format_colored_time(&local_time);
            let time_padding = COLUMN_WIDTH - time_plain.len();
            print!("│{}{}", " ".repeat(time_padding), time_colored);
        }
        println!("│");
    }

    // Print bottom border
    print!("└");
    print!("{}", "─".repeat(COLUMN_WIDTH));
    for _ in &airport_info {
        print!("┴");
        print!("{}", "─".repeat(COLUMN_WIDTH));
    }
    println!("┘");

    // Print color legend
    println!("\nColor Legend:");
    println!("  {} - Poor meeting times (11pm-5am)", "Red".red());
    println!(
        "  {} - Suboptimal times (early morning/late evening)",
        "Yellow".yellow()
    );
    println!("  {} - Good meeting times", "Green".green());

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    if args.airports.is_empty() {
        eprintln!("Error: Please provide at least one airport code");
        eprintln!("Example: rmeet LAX CDG AKL");
        std::process::exit(1);
    }

    if args.hours == 0 || args.hours > 168 {
        // Max 1 week
        eprintln!("Error: Hours must be between 1 and 168 (1 week)");
        std::process::exit(1);
    }

    if args.start_offset.unsigned_abs() > 8760 {
        // Max 1 year
        eprintln!("Error: Start offset must be between -8760 and 8760 hours (1 year)");
        std::process::exit(1);
    }

    let airport_data = load_airport_data();

    println!("\nGenerating timezone comparison table...\n");

    // Generate and display the timezone table
    match generate_timezone_table(&airport_data, &args.airports, args.hours, args.start_offset) {
        Ok(()) => {
            println!("\nTable shows times for the next {} hours", args.hours);
            if args.start_offset != 0 {
                println!("Starting {} hours from current time", args.start_offset);
            }
        }
        Err(e) => {
            eprintln!("Error generating table: {}", e);

            match e {
                AppError::NoAirportFound(code) => {
                    eprintln!("Suggestions:");
                    eprintln!("  - Use valid 3-letter IATA airport codes (e.g., LAX, JFK, LHR)");
                    eprintln!("  - Check spelling of airport code: {}", code);
                }
                AppError::TimezoneError(_) => {
                    eprintln!("Suggestions:");
                    eprintln!("  - The bundled airport data may contain invalid timezone information");
                }
            }

            std::process::exit(1);
        }
    }

    Ok(())
}
