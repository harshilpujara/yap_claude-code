# Contributing

Thanks for helping. Bug reports and small, focused pull requests are welcome.

## Run in development

Requires Rust, Node.js 22 and the Visual Studio C++ Build Tools (Windows).

```
npm install
npm --prefix web install
npm run tauri dev
```

## Run the tests

```
cd src-tauri && cargo test          # Rust unit tests
cd web && npx tsc --noEmit          # TypeScript type check
```

`cargo test live_cleanup -- --ignored --nocapture` runs real dictations through your saved API key; it is optional and uses your provider quota.

## Open an issue

Use the [issue templates](https://github.com/harshilpujara/yap_claude-code/issues/new/choose). For a bug, include your Windows version, provider, language setting and what happened. **Never paste an API key**, and remove anything private from logs.
