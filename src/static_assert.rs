/// # Example:
///
/// ```rust
/// use fregate::static_assert;
///
/// #[repr(i32)]
/// enum Enum {
///     One = 1,
///     Two,
///     Three,
/// }
///
/// static_assert!(Enum::One as i32 == 1);
/// ```
#[macro_export]
macro_rules! static_assert {
    ($cond:expr) => {
        const _: () = assert!($cond);
    };
}

/// # Example:
///
/// ```rust
/// use fregate::static_trait_assert;
///
/// #[repr(i32)]
/// enum Enum {
///     One,
///     Two,
///     Three,
/// }
///
/// impl From<Enum> for i32 {
///     fn from(value: Enum) -> Self {
///         value as i32
///     }
/// }
///
/// static_trait_assert!(Enum, Into<i32>);
/// ```
#[macro_export]
macro_rules! static_trait_assert {
    ($t:ty, $traits:path) => {
        const fn type_trait_check()
        where
            $t: $traits,
        {
        }

        const _: () = type_trait_check();
    };
}
