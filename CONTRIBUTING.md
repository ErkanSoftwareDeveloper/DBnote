# Contributing

Contributions are welcome.

## Development

```bash
npm install
npm run tauri dev
```

## Checks

Run these before opening a pull request:

```bash
npm run build
cd src-tauri
cargo fmt --check
cargo test
```

Keep changes focused, readable, and covered by tests when they touch storage, migrations, or graph behavior.
