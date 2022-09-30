# Usage

"Swiss knife" crate, consists of a lot of useful instruments such as system health diagnostics, logging, proxies etc.

# Questions

### ❓ Dependancies logic

https://github.com/Wandalen/fregate_review/blob/master/examples/grafana/Cargo.toml

not clear what is logic of including dependencies
lots of dependencies come from fregate and public
but some are not public and included by user

```toml
fregate = { path = "../.." }
tracing = "0.1.36"
tokio = { version = "1.0", features = ["full"] }
prost = "0.11.0"
tonic = "0.8.0"
```

why these dependencies are not in fregate?
is it possible to remove dependency `axum`?

# Review

### ✅ No major problem

### ✅ Idiomatic: code is short and concise

### ✅ Disciplinar: Library code is mostly covered in unit tests

Consider measuring test coverage.

### ✅ Architectural: following of deep module / information hiding principles

Read more on [deep module / information hiding] principles](https://medium.com/@nathan.fooo/5-notes-information-hiding-and-leakage-e5bd75dc41f7)

### ✅ Disciplinar: sticking to the best practices

`cargo fmt` - this utility formats all bin and lib files of the current crate using rustfmt. It's used ✅.

`cargo clippy` - checks a package to catch common mistakes and improve your Rust code. Seems it's ignored ✅.
Use (reasonable) preset of wanring/clippy rules  ✅.

[`cargo miri`](https://github.com/rust-lang/miri) - an experimental interpreter for Rust's mid-level intermediate representation (MIR). It can run binaries and test suites of cargo projects and detect certain classes of undefined behavior. Use compile-time conditionals to separate code which `miri` can interpret.❕

[`cargo-audit`](https://github.com/RustSec/rustsec/tree/main/cargo-audit) - audit Cargo files for crates with security vulnerabilities reported to the RustSec Advisory Database ✅.

[`cargo-checkmate`](https://github.com/cargo-checkmate/cargo-checkmate) - ensures your project builds, tests pass, has good format, doesn't have dependencies with known vulnerabilities ✅.

[`cargo-udeps`](https://github.com/est31/cargo-udeps) - ensures there is no unused dependencies in Cargo.toml ✅.

Consider having a Makefiel tools for audit as targets❕.

These tools give free suggestion how to improve quality of code and to avoid common pitfalls.

### ❕ Architectural: modularity is okay

There is no sense to decompose the crate since it looks lightweight (seems to be design requirenment) and there is no advantages in transforming crate into complex system ✅.

Pub usage of axum ✅
Fregate manages the axum versions. The end-user cannot do anything with this. To have a single web server, the user must build his application based on fregate dependencies.

No access to tokio ❕
Crate does not provide public access to tokio, that's why even "hello world" example needs to add this dependency again to run the main function.

Extensive wildcard imports into root ❕
Crate root namespace should be reserved for entities that will be immediately neede by the user.
Currently everything is exported through the crate root.
What is somewhat excessive.
Structs like NoHealth and AlwaysReadyAndAlive should not pollute the root namespace.
https://www.lurklurk.org/effective-rust/wildcard.html

### ❕ Architectural: framework vs toolkit approach ( vendorlock vs agnostic )

Overuse of inversion of control and adapter/facade patterns is obeservable, what is typical for frameworks.

It creates several disadvantages:

1. It decreases maintainability because isolatation from original crates by facade. New features are not necessarily available for user of the framework.
2. It decreases flexibility and extendability of the codebase of users of the crate. Because many parameters are hidden. Interfaces are changed, but new interfaces are not necessarily better than original.
3. It increase time for onboarding. New developers should spend more time to learn custom interfaces of fregate.

There is risk that this crate will become framework. Better I would suggest to keep principle of being agnostics and transparent, providing fundamentals components instead of aggregating eagerly them into higher-order entities.

### ❕ Architectural: risk of utility antipattern

What is responsibility of the crate? List all and evaluate does not it have too much responsibilities?

- Telemetry + logging feels okay.
- Telemetry + logging + creating server feels too much for me.

https://www.yanglinzhao.com/posts/utils-antipattern/

### ❕ Architectural: singletone

It's not possible to create several instances of Application.
Application has global state.
If Application stays, make it module instead of being struct.

```rust
let config = bootstrap::<Empty, _>([]);

Application::new(&config)
    .health_indicator(CustomHealth::default())
    .serve();
```

Eliminate either `bootstrap` or `Application::new`.

### ❕ Security: Potential sensitive information leak through errors

in `src/middleware/proxy.rs:fn handle_result`, Error is directly
passed through to the caller of the API. Errors potentially contain sensitive
information (such as secret keys, URLs, etc), and one should be careful
about passing errors directly to the caller.

### ❕ Performance: async trait.

Async traits use dynamic dispatch under the hood, which has runtime performance cost.
- [Async trait downsides](https://internals.rust-lang.org/t/async-traits-the-less-dynamic-allocations-edition/13048/2)
- [Async trait under the hood](https://smallcultfollowing.com/babysteps/blog/2019/10/26/async-fn-in-traits-are-hard/)

### ❕ Structural: lack of features

Forward control over features of exposed depdendencies and dependencies themselvs.

### ❕ Structural: lack of documentation

Public functions, structs, traits, modules should be descibed well.
Example files should be described.
Most functions/structs should have snipets with examples.
Some example files look more like smoke tests than examples.

### ❕ Strategic: value for open source

The goal: to answer the question whether I should use this crate in my (including commercial) projects and, if so, in what cases it is most useful.

**PROS**
* couple of built-in tools, most of which could be necessary for common web application
* reputation of the company

**CONS**
* fregate doesn't give you any garantees about versions of it's dependencies
* zero flexibility in chosing web frameworks. axum is required
* too framework-like with disadvantages explaned above
