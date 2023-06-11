### Web app to register for Cashu lnurl service

https://github.com/thesimplekid/cashu-lnurl

Set env var `LNURL_SERVICE_PUBKEY` at build to with the pubkey of the service
`RELAY` with the nostr relay the service listens on.


### Building
```
cargo install trunk
rustup target add wasm32-unknown-unknown
```

Install tailwind 
https://tailwindcss.com/docs/installation

## Run
```
trunk serve --open
```


