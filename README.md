# ro-soku🕯️

<div align="center">
<div>⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛  </div>
<div>⬛⬛⬛⬛⬛⬛⬛🟩⬛⬛⬛⬛⬛⬛  </div>
<div>⬛⬛🟥⬛⬛🟩⬛🟩⬛⬛⬛⬛⬛⬛  </div>
<div>⬛⬛🟥🟥🟥🟩⬛🟩⬛⬛⬛⬛⬛⬛  </div>
<div>⬛🟩🟥🟥🟥🟩⬛🟩⬛⬛⬛⬛⬛⬛  </div>
<div>⬛🟩🟥🟥🟥🟩⬛🟩🟥🟩⬛⬛⬛⬛  </div>
<div>⬛🟩🟥🟥🟥🟩🟥🟩🟥🟩🟩🟩⬛⬛  </div>
<div>⬛⬛⬛🟥🟥🟩🟥🟩🟥🟩🟩🟩🟩⬛  </div>
<div>⬛⬛⬛⬛🟥⬛🟥⬛🟥🟩🟩⬛⬛⬛  </div>
<div>⬛⬛⬛⬛🟥⬛🟥⬛🟥🟩🟩⬛⬛⬛  </div>
<div>⬛⬛⬛⬛🟥⬛🟥⬛🟥🟩🟩⬛⬛⬛  </div>
<div>⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛  </div>
</div>

 ͏

<img alt="GitHub release (latest by date)" src="https://img.shields.io/github/v/release/mirumirumi/ro-soku"> <img alt="GitHub Workflow Status" src="https://img.shields.io/github/actions/workflow/status/mirumirumi/ro-soku/release.yaml"> <img alt="GitHub Workflow Status" src="https://img.shields.io/github/actions/workflow/status/mirumirumi/ro-soku/unit-test.yaml?label=unit%20test"> <img alt="GitHub Workflow Status" src="https://img.shields.io/github/actions/workflow/status/mirumirumi/ro-soku/cron-test.yaml?label=integration%20tests"> <img alt="GitHub" src="https://img.shields.io/github/license/mirumirumi/ro-soku">

## About

ro-soku (meaning "candle"🕯️ in Japanese) allows you to easily retrieve OHLCV (Kline) data from various cryptocurrency exchanges.

Features:

- No config, No set up, No API key
- Retrieve data exceeding the `limit` in a single execution if needed
- `ro-soku guide` to interactively build commands to retrieve the data you want
- Flexible OHLCV data processing and output formats

### Supported Exchanges

| Exchange | Support Status | Notes                        |
| -------- | :------------: | ---------------------------- |
| Binance  |       ✅      | |
| bitbank  |       ✅      | |
| BitMEX   |       ✅      | |
| Bybit    |       ✅      | Inverse type is not supported. |
| Kraken   |       ❌      | API has some strange bugs;<br />implementation was done but release was abandoned.<br />(see: https://bit.ly/3NNVZOD) |
| OKX      |       ✅      | |

## Install

### Mac

```bash
brew tap mirumirumi/ro-soku
brew install ro-soku
```

### Linux

```bash
wget https://raw.githubusercontent.com/mirumirumi/ro-soku/main/scripts/install.sh -P /tmp/
sh /tmp/install.sh
```

### Windows

```bash
Invoke-WebRequest -Uri "https://raw.githubusercontent.com/mirumirumi/ro-soku/main/scripts/install.ps1" -OutFile "$env:TEMP\install.ps1"
Set-ExecutionPolicy Bypass -Scope Process -Force; & "$env:TEMP\install.ps1"
```

## Basic Usage

```bash
ro-soku \
    --exchange binance \
    --type spot \
    --symbol BTC/USDT \
    --interval 1sec \
    --term-start 2023-05-10T13:25:33Z \
    --term-end 1683725270000
```

Or, when you always want to obtain the most recent data, such as "past 30 minutes":

```bash
ro-soku \
    --exchange binance \
    --type perpetual \
    --symbol BTC/USDT \
    --interval 1min \
    --past \
    --range 30min
```

Output for csv file to analysis close prices:

```bash
ro-soku \
    --past \
    --range 1hour \
    --pick t,c \
    --format csv \
    > data.csv
```

Guide to build a command:

```bash
ro-soku guide
```

> **Note**
> It can also be executed without arguments (`ro-soku`), but this is not very useful since all options are executed in their default state.

## Options

Refer to `ro-soku --help` for all options, available values, and other details.

### `--term-start` and `--term-end`:

```bash
# All of the following are valid

--term-start 2023-05-08T13:25:33+00:00
--term-start 2023-05-08T13:25:33+09:00
--term-start 2023-05-08T13:25:33Z
--term-start 1683725270000  # Unixtime (milliseconds)
```

### `--pick`:

```bash
# default
--pick t,o,h,l,c,v

# outputs:
[1614984720000, 49225.0, 49254.0, 49225.0, 49240.0, 912082.0981]

# only high and low
--pick t,h,l

# outputs:
[1614984720000, 49254.0, 49225.0]

# allow duplicate selection
--pick v,v,v,t

# outputs:
[912082.0981, 912082.0981, 912082.0981, 1614984720000]
```

### `--format`:

#### raw

```raw
[1614984720000, 49225.0, 49254.0, 49225.0, 49240.0, 912082.0981]
[1614984780000, 49240.0, 49240.0, 49219.0, 49222.0, 427743.0204]
[1614984840000, 49222.0, 49229.0, 49221.0, 49229.0, 97785.225]
...
```

#### CSV

```csv
1614984720000,49225.0,49254.0,49225.0,49240.0,912082.0981
1614984780000,49240.0,49240.0,49219.0,49222.0,427743.0204
1614984840000,49222.0,49229.0,49221.0,49229.0,97785.225
...
```

#### TSV

```tsv
1614984720000	49225.0	49254.0	49225.0	49240.0	912082.0981
1614984780000	49240.0	49240.0	49219.0	49222.0	427743.0204
1614984840000	49222.0	49229.0	49221.0	49229.0	97785.225
...
```

#### JSON

```jsonc
[
    {
        "unixtime": 1614984720000,
        "open": 49225.0,
        "high": 49254.0,
        "low": 49225.0,
        "close": 49240.0,
        "volume": 912082.0981
    },
    {
        "unixtime": 1614984780000,
        "open": 49240.0,
        "high": 49240.0,
        "low": 49219.0,
        "close": 49222.0,
        "volume": 427743.0204
    },
    {
        "unixtime": 1614984840000,
        "open": 49222.0,
        "high": 49229.0,
        "low": 49221.0,
        "close": 49229.0,
        "volume": 97785.225
    }
    // ...
]
```

## LICENSE

MIT
