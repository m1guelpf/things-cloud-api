# things-cloud-api

> An unofficial API client for the Things Cloud API. Work in progress.

[![crates.io](https://img.shields.io/crates/v/things-cloud.svg)](https://crates.io/crates/things-cloud)
[![download count badge](https://img.shields.io/crates/d/things-cloud.svg)](https://crates.io/crates/things-cloud)
[![docs.rs](https://img.shields.io/badge/docs-latest-blue.svg)](https://docs.rs/things-cloud)

## Usage

```rust
let account = things_api::Account::login(email, password).await?;
let tasks = account.history().tasks;
```

Refer to the [documentation on docs.rs](https://docs.rs/things-cloud) for detailed usage instructions.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
