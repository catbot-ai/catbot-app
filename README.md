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

## TODO

- Swap at Jup: https://github.com/jup-ag/jupiter-swap-api-client
- Swap at RAY: [CommandsName::SwapV2](https://github.com/raydium-io/raydium-clmm/blob/master/client/src/main.rs#L1769)
- Get `SOL/JLP` history price: `https://fe-api.jup.ag/api/v1/charts/27G8MtK7VtTcCHkpASjSDdkWWYfoqT6ggEuKidVJidD4?quote_address=So11111111111111111111111111111111111111112&type=1H&time_from=1736917164&time_to=1738119564`
- Get `SOL/JLP` history positions info: [get_all_nft_and_position_by_owner](https://github.com/raydium-io/raydium-clmm/blob/master/client/src/main.rs#L261C4-L261C37)
- Suggest rebalance/open/close.
- Add tooling.
- Call tooling.
- Open `SOL/JLP` position via RAY:[CommandsName::OpenPosition](https://github.com/raydium-io/raydium-clmm/blob/master/client/src/main.rs#L1060)
- Harvest: [harvestLockedPosition](https://github.com/raydium-io/raydium-sdk-V2-demo/blob/daf78a9/src/clmm/harvestLockedPosition.ts)
- Increase Liquidity: [CommandsName::IncreaseLiquidity](https://github.com/raydium-io/raydium-clmm/blob/master/client/src/main.rs#L1247C9-L1247C40)
- Decrease Liquidity: [CommandsName::DecreaseLiquidity](https://github.com/raydium-io/raydium-clmm/blob/master/client/src/main.rs#L1432C9-L1432C40)

## Notes

Bug? after put mac on sleep, after wake up nothing update after that.

```
2025-01-29 00:16:12.222 hello-tauri[62181:229982148] +[IMKClient subclass]: chose IMKClient_Modern
2025-01-29 00:16:12.242 hello-tauri[62181:229982148] +[IMKInputSession subclass]: chose IMKInputSession_Modern
Price fetch error: error decoding response body
2025-01-29 07:47:44.001 hello-tauri[62181:229982148] _TIPropertyValueIsValid called with 16 on nil context!
2025-01-29 07:47:44.001 hello-tauri[62181:229982148] imkxpc_getApplicationProperty:reply: called with incorrect property value 16, bailing.
2025-01-29 07:47:44.001 hello-tauri[62181:229982148] Text input context does not respond to _valueForTIProperty:
2025-01-29 07:47:54.543 hello-tauri[62181:229982148] _TIPropertyValueIsValid called with 16 on nil context!
2025-01-29 07:47:54.543 hello-tauri[62181:229982148] imkxpc_getApplicationProperty:reply: called with incorrect property value 16, bailing.
```
