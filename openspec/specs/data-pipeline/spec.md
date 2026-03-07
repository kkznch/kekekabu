## ADDED Requirements

### Requirement: Scan fetches price data from J-Quants V2 API
The system SHALL fetch daily OHLCV price data from J-Quants V2 API for all watchlist stocks when `kktd scan` is executed.

#### Scenario: Successful scan with watchlist stocks
- **WHEN** user runs `kktd scan --days 60` with stocks in the watchlist
- **THEN** system fetches price data for each watchlist stock from J-Quants V2 API, saves to DB, and outputs scan results as JSON to stdout

#### Scenario: Empty watchlist
- **WHEN** user runs `kktd scan` with no stocks in the watchlist
- **THEN** system outputs an empty JSON array `[]` to stdout

#### Scenario: Rate limiting between API calls
- **WHEN** system fetches data for multiple stocks
- **THEN** system SHALL wait at least 1 second between consecutive J-Quants API calls

### Requirement: Scan computes technical indicators
The system SHALL compute technical indicators (SMA, EMA, RSI, MACD, Bollinger Bands, ATR, Volume MA) from fetched price data.

#### Scenario: Full indicator computation
- **WHEN** sufficient price data exists (>= 75 data points)
- **THEN** system computes SMA(5/25/75), EMA(12/26), RSI(14), MACD(12,26,9), Bollinger Bands(20,2), ATR(14), Volume MA(20) and includes them in scan output

#### Scenario: Insufficient data
- **WHEN** fewer data points than required for an indicator
- **THEN** system returns empty results for that indicator without error

### Requirement: Scan detects trading signals
The system SHALL detect trading signals from computed indicators.

#### Scenario: Golden cross detection
- **WHEN** SMA(5) crosses above SMA(25)
- **THEN** system includes "golden_cross_5_25" in signals array

#### Scenario: Dead cross detection
- **WHEN** SMA(5) crosses below SMA(25)
- **THEN** system includes "dead_cross_5_25" in signals array

#### Scenario: Volume spike detection
- **WHEN** latest volume exceeds 2x the Volume MA(20)
- **THEN** system includes "volume_spike" in signals array

### Requirement: Price data is saved to database
The system SHALL persist fetched price data to the SQLite database with idempotent writes.

#### Scenario: Idempotent price save
- **WHEN** same price data is saved twice for the same ticker and date
- **THEN** system does not create duplicates (INSERT OR IGNORE)
