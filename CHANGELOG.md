# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)

## [Unreleased]

## [0.3.0] - 2022-09-15
### Added
- Add context span propagation from Incoming request ([#52](https://github.com/elefant-dev/fregate-rs/pull/52))
- Add AppConfigBuilder::init_tracing() ([#50](https://github.com/elefant-dev/fregate-rs/pull/50))
- Add Traces export to grafana ([#50](https://github.com/elefant-dev/fregate-rs/pull/50))

### Changed
- Tower::Steer is removed, rely only on axum path matching mechanism([#48](https://github.com/elefant-dev/fregate-rs/pull/48))
- Call for local resolver in proxy middleware if failed to set new Uri([#49](https://github.com/elefant-dev/fregate-rs/pull/49))
- init_tracing() function call moved to AppConfig::build() ([#50](https://github.com/elefant-dev/fregate-rs/pull/50))
- Only 1 AppConfig() is allowed to build ([#50](https://github.com/elefant-dev/fregate-rs/pull/50))
- Add log filter reloader to change log level in runtime ([#50](https://github.com/elefant-dev/fregate-rs/pull/50))
- Use AppConfig::builder() instead of AppConfig::builder_with_private() ([#50](https://github.com/elefant-dev/fregate-rs/pull/50))

### Changed
- AppConfig::builder_with_private() is removed ([#50](https://github.com/elefant-dev/fregate-rs/pull/50))

## [0.2.5] - 2022-09-02
### Added
- Add `Proxy` middleware using http client ([#45](https://github.com/elefant-dev/fregate-rs/pull/45))
- Add public `DeserializeExt` trait ([#46](https://github.com/elefant-dev/fregate-rs/pull/46))
- Add logger settings to `AppConfig` ([#47](https://github.com/elefant-dev/fregate-rs/pull/47))

## [0.2.4] - 2022-08-25
### Added
- Add proxy `Router` using http client ([#37](https://github.com/elefant-dev/fregate-rs/pull/37))
- Add instruction on how to publish ([#31](https://github.com/elefant-dev/fregate-rs/pull/31))

### Changed
- Make `AppConfigBuilder` to take `&str` and `FileFormat` as source ([#39](https://github.com/elefant-dev/fregate-rs/pull/39))

### Removed
- Remove `Application::api_path` ([#38](https://github.com/elefant-dev/fregate-rs/pull/38))

## [0.2.3] - 2022-08-15
### Added
- Add support for "extended" grpc `Content-Type` ([#32](https://github.com/elefant-dev/fregate-rs/pull/32))

### Changed
- Change `Application` to take reference of `AppConfig` as argument  ([#36](https://github.com/elefant-dev/fregate-rs/pull/36))
- Flatten `AppConfig::private` field ([#34](https://github.com/elefant-dev/fregate-rs/pull/34))

## [0.2.2] - 2022-08-12
### Removed
- Remove `[service]` section from `AppConfig` ([#30](https://github.com/elefant-dev/fregate-rs/pull/30))

## [0.2.1] - 2022-08-12
### Changed
- Change health and ready endpoints ([#29](https://github.com/elefant-dev/fregate-rs/pull/29))

## [0.2.0] - 2022-08-11
### Added
- Add `ws` feature from `axum` ([#26](https://github.com/elefant-dev/fregate-rs/pull/26))
- Add private field to `AppConfig` ([#17](https://github.com/elefant-dev/fregate-rs/pull/17))

### Changed
- Change `Application::api_path` to be optional ([#25](https://github.com/elefant-dev/fregate-rs/pull/25))
- Change `Application`to be always ready and alive by default ([#24](https://github.com/elefant-dev/fregate-rs/pull/24))
- Change `Application` to take `AppConfig` as argument  ([#17](https://github.com/elefant-dev/fregate-rs/pull/17))

## [0.1.0] - 2022-08-06
- Initial release.
