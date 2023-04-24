# Changelog
All notable changes to this project will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)

## [Unreleased]
### Added
- Add new metrics from tokio-metrics 0.2.

## [0.10.4] - 2023-04-21
### Fixed
- Runtime error in `tracing-subscriber` because of `regex 1.8`:
  - https://github.com/rust-lang/regex/issues/982
  - https://github.com/tokio-rs/tracing/issues/2565

## [0.10.3] - 2023-03-14
### Added
- Reversed proxy middleware ([#170](https://github.com/elefant-dev/fregate-rs/pull/170))

## [0.10.2] - 2023-03-06
### Changed
- Add feature `map-response-body` to tower-http crate. ([#169](https://github.com/elefant-dev/fregate-rs/pull/169))

## [0.10.1] - 2023-02-28
### Changed
- Add feature `util` to tower-http crate.([#168](https://github.com/elefant-dev/fregate-rs/pull/168))

## [0.10.0] - 2023-02-20
### Added
- [`ManagementConfig`](https://github.com/elefant-dev/fregate-rs/blob/main/src/configuration/management.rs). ([#165](https://github.com/elefant-dev/fregate-rs/pull/165))

## [0.9.0] - 2023-02-20
### Added
- `TracingFields::merge()`. ([#161](https://github.com/elefant-dev/fregate-rs/pull/161))

### Removed
- Proxy handler ([#159](https://github.com/elefant-dev/fregate-rs/pull/159))
- `RouterOptionalExt` trait ([#159](https://github.com/elefant-dev/fregate-rs/pull/159))

### Changed
- Restructure crate. Now everything connected to tracing/metrics is in [`observability`](https://github.com/elefant-dev/fregate-rs/tree/main/src/observability) mod.([#159](https://github.com/elefant-dev/fregate-rs/pull/159))
- `TracingFields::insert() --> TracingFields::insert_ref()`. ([#161](https://github.com/elefant-dev/fregate-rs/pull/161))
- `TracingFields::insert()` now takes value. ([#161](https://github.com/elefant-dev/fregate-rs/pull/161))
- `TracingFields` now takes only `&'static str` as a key. ([#161](https://github.com/elefant-dev/fregate-rs/pull/161))
- `LoggerConfig --> ObservabilityConfig`. ([#159](https://github.com/elefant-dev/fregate-rs/pull/159))
- `floor_char_boundary` public now. ([#159](https://github.com/elefant-dev/fregate-rs/pull/159))

## [0.8.1] - 2023-02-08
### Changed
- make AppConfig worker_guard field to be public([#157](https://github.com/elefant-dev/fregate-rs/pull/157))

## [0.8.0] - 2023-02-03
### Added
- Add HeadersExt([#155](https://github.com/elefant-dev/fregate-rs/pull/155))

### Removed
- Unused metrics initialization([#153](https://github.com/elefant-dev/fregate-rs/pull/153))

### Changed
- Use none blocking writer from tracing_appender crate([#151](https://github.com/elefant-dev/fregate-rs/pull/151))
- Make tracing_layer optional in Application([#149](https://github.com/elefant-dev/fregate-rs/pull/149))

## [0.7.0] - 2023-01-16
### Changed
- Limit msg field in log through EvenFormatter([#147](https://github.com/elefant-dev/fregate-rs/pull/147))

## [0.6.3] - 2022-12-16
### Changed
- Fixed tokio_metrics compile time error([#142](https://github.com/elefant-dev/fregate-rs/pull/142))

## [0.6.2] - 2022-12-13
### Added
- http::Request extension to inject span into headers([#138](https://github.com/elefant-dev/fregate-rs/pull/138))

### Changed
- Change access to public for tracing layer([#138](https://github.com/elefant-dev/fregate-rs/pull/138))

### Removed
- Metrics tracking in tracing layer([#140](https://github.com/elefant-dev/fregate-rs/pull/140))

## [0.6.1] - 2022-12-07
### Changed
- Fix metrics initialization([#136](https://github.com/elefant-dev/fregate-rs/pull/136))

## [0.6.0] - 2022-12-01
### Changed
- Update axum to 0.6([#133](https://github.com/elefant-dev/fregate-rs/pull/133))

## [0.5.0] - 2022-11-24
### Added
- Make trace filter level reloadable([#126](https://github.com/elefant-dev/fregate-rs/pull/126))
- Add HashBuilder struct with sugar for easier hash calculation.([#127](https://github.com/elefant-dev/fregate-rs/pull/127))

### Changed
- Print exporter error through tracing([#129](https://github.com/elefant-dev/fregate-rs/pull/129))
- Use w3c context propagation instead of Zipkin([#131](https://github.com/elefant-dev/fregate-rs/pull/131))

## [0.4.7] - 2022-11-16
### Added
- Add metrics callback([#118](https://github.com/elefant-dev/fregate-rs/pull/118))
- Print in log traceId and spanId if event is in span([#119](https://github.com/elefant-dev/fregate-rs/pull/119))

### Changed
- trace_request middleware moved to Application([#122](https://github.com/elefant-dev/fregate-rs/pull/122))

## [0.4.6] - 2022-11-10
### Added
- `Context` Injections for reqwest and tonic([#111](https://github.com/elefant-dev/fregate-rs/pull/111))
- `TracingFields::insert_as_debug()` for types which impl only `Debug` trait ([#107](https://github.com/elefant-dev/fregate-rs/pull/107))
- Some sugar functions to work with `tonic::Code` ([#110](https://github.com/elefant-dev/fregate-rs/pull/110))

### Changed
- Forbid export of events in traces to grafana([#90](https://github.com/elefant-dev/fregate-rs/pull/90))
- Changed return type for `extract_grpc_status_code` fn to be `tonic::Code`([#109](https://github.com/elefant-dev/fregate-rs/pull/109))

## [0.4.5] - 2022-11-07
### Added
-  Optional tokio metrics([#98](https://github.com/elefant-dev/fregate-rs/pull/98))

## [0.4.4] - 2022-11-03
### Added
- `TracingFields::insert_as_string()` for types which do not impl `Valuable` trait ([#95](https://github.com/elefant-dev/fregate-rs/pull/95))

### Changed
- Rename tls flags with "use_" prefix. ([#93](https://github.com/elefant-dev/fregate-rs/pull/93))

## [0.4.3] - 2022-10-28
### Changed
- Make TracingFields to be Send ([#88](https://github.com/elefant-dev/fregate-rs/pull/88))

## [0.4.2] - 2022-10-26
### Changed
- Add timeout for tls handshake ([#83](https://github.com/elefant-dev/fregate-rs/pull/83))
- Tls Handshake in separate task ([#83](https://github.com/elefant-dev/fregate-rs/pull/83))
- Disable default feature in opentelemetry-zipkin ([#86](https://github.com/elefant-dev/fregate-rs/pull/86))

### Added
- Native-tls support ([#82](https://github.com/elefant-dev/fregate-rs/pull/82))
- Rustls support ([#85](https://github.com/elefant-dev/fregate-rs/pull/85))

## [0.4.1] - 2022-10-18
### Changed
- Make AlwaysReadyAndAlive return OK to "/ready" request ([#78](https://github.com/elefant-dev/fregate-rs/pull/78))

## [0.4.0]
### Added
- Bootstrap fn to read config and initialize tracing ([#58](https://github.com/elefant-dev/fregate-rs/pull/58))
- Metrics endpoint ([#70](https://github.com/elefant-dev/fregate-rs/pull/70))
- Custom event formatter for tracing_subscirber::layer() ([#72](https://github.com/elefant-dev/fregate-rs/pull/72))
- TracingFields structure to flatten fields in logs ([#72](https://github.com/elefant-dev/fregate-rs/pull/72))
- Docs ([#62](https://github.com/elefant-dev/fregate-rs/pull/62))

### Changed
- Add Clippy lints ([#59](https://github.com/elefant-dev/fregate-rs/pull/59))
- Review fixes ([#68](https://github.com/elefant-dev/fregate-rs/pull/68))
- AppConfig is no longer singleton ([#58](https://github.com/elefant-dev/fregate-rs/pull/58))
- Tracing layer ([#72](https://github.com/elefant-dev/fregate-rs/pull/72))

## [0.3.1] - 2022-09-15
### Removed
-  `grpc-sys` dependency.

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
