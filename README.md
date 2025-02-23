## Broken

This is an example from official but it's stuck at build page forever.

```
"build": {
    "beforeDevCommand": "dx serve --port 1420",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "dx build --release",
    "frontendDist": "../dist"
  },
```

## Workaround

Delete `beforeDevCommand` and run `dx serve --port 1420` ourself

```
"build": {
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "dx build --release",
    "frontendDist": "../dist"
  },
```

## Dev

```
dx serve --port 1420
cargo tauri dev
```

## Settings

```
/Users/katopz/Library/Application Support/com.catbot.app/settings.yaml
```

## Release

```
./release.sh v0.2.0
```

## Features

- Display current token or pair price on MacOS tray with minimal resources used.
- Link to JUP portfolio.

## DONE

- [Settings] Create settings with `wallet_address`.
- [Settings] Load settings with `wallet_address`.
- [Settings] Load settings with `recent_token_or_pair`.

## TODO

- [INDICATOR] Calculate `MACD`, `BB`, `RSI`.
- [SUGGESTION] The price `JLP/SOL` will be stable at 1% range for 3 days ahead, estimated 2 `SOL` profit, consider open the pool.
- [SUGGESTION] The price `JLP ⟢ SOL` will move 1.5% to the right, consider rebalance the pool to the right.
- [SUGGESTION] The price `SOL +10%` will be up more than 10%, consider withdraw and long.
- [SUGGESTION] The price `JLP` will be down, consider shot for 1 day, estimated 2 `SOL` profit.
- [SUGGESTION] The price `JLP` will be up, consider long for 1 day, estimated 2 `SOL` profit.
- [SUGGESTION] The price `JLP/SOL` is at the bottom, consider DCA 3 times for the next 12 hours.
- [PRICE] Better use quoted price?
- [MENU] Can switch price e.g. `JLP/SOL`, `SOL/JLP`.
- [MENU] Add indicator ↗︎↗︎↗︎, ↑↓↘︎↴, ⥂⥄+−⦧⦦⟡⟢⟣⫠⫠⫟.
- [FORECAST] Add 1 day forecast from historical data.
- [HOLD] Holder will get top most perf model. stake `JLP`, system get 57% yield.
- [AGENT] Build agent chat interface with example prompt.
- [SUGGESTION] Build prompt with price,ta.
- [SUGGESTION] Rust call `VertexAI` via `CloudRun`.
- [SUGGESTION] RAG + Prompt + Thinking
- [SUGGESTION] Notify rebalance/open/close.
- [SUGGESTION] Notify harvested.
- [SUGGESTION] PNL Graph.
- [TOOLS] Swap at RAY: [CommandsName::SwapV2](https://github.com/raydium-io/raydium-clmm/blob/master/client/src/main.rs#L1769)
- [TOOLS] Add tooling via [MCP](https://github.com/Derek-X-Wang/mcp-rust-sdk).
- [TOOLS] Open `SOL/JLP` position via RAY:[CommandsName::OpenPosition](https://github.com/raydium-io/raydium-clmm/blob/master/client/src/main.rs#L1060)
- [TOOLS] Harvest: [harvestLockedPosition](https://github.com/raydium-io/raydium-sdk-V2-demo/blob/daf78a9/src/clmm/harvestLockedPosition.ts)
- [TOOLS] Increase Liquidity: [CommandsName::IncreaseLiquidity](https://github.com/raydium-io/raydium-clmm/blob/master/client/src/main.rs#L1247C9-L1247C40)
- [TOOLS] Decrease Liquidity: [CommandsName::DecreaseLiquidity](https://github.com/raydium-io/raydium-clmm/blob/master/client/src/main.rs#L1432C9-L1432C40)
- [PREDICTION] Fine-tuning model.
- [PREDICTION] Use fine-tuned model.
- [PREDICTION] Able to use custom `API`.
- [PREDICTION] Able to select strategy with fee tier.
- [TASKS] Display `Schedule Tasks` with time interval window.
- [TASKS] Call `CloudFlare` API for parse task.
- [PRICE] `API` rotation and fallback.
- [PRICE] Can set alert at price on CloudFlare schedule.
- [PRICE] Can set alert at price on local.
- [OUT] Multiply `JLP` at Kamino.
- [OUT] Multiply `hSOL` at Kamino.
- Swap at Jup: https://github.com/jup-ag/jupiter-swap-api-client
- Get `SOL/JLP` history price: `https://fe-api.jup.ag/api/v1/charts/27G8MtK7VtTcCHkpASjSDdkWWYfoqT6ggEuKidVJidD4?quote_address=So11111111111111111111111111111111111111112&type=1H&time_from=1736917164&time_to=1738119564`
- [OUT] Long Jup perf when `SOL` down.
- [OUT] Short Jup perf when `SOL` up.
- [OUT] Set harvested target(+jup fee) and Token out.
- Simulation mode.
- Get `SOL/JLP` history positions info: [get_all_nft_and_position_by_owner](https://github.com/raydium-io/raydium-clmm/blob/master/client/src/main.rs#L261C4-L261C37)
- Get Technical Analysis: [https://github.com/00x4/m4rs]
- Plot Graph with [plotters](https://github.com/plotters-rs/plotters)
- Support more [verified token](https://api.jup.ag/tokens/v1/tagged/verified)
