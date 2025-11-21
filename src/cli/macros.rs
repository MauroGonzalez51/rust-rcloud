/// Import handler function only (without LocalArgs)
///
/// # Usage
/// ```rust
/// use_handler!(remote, list);
/// // Expands to: use crate::cli::commands::remote::handlers::list::remote_list;
/// ```
#[macro_export]
macro_rules! use_handler {
    ($module:ident, $handler:ident) => {
        paste::paste! {
            use $crate::cli::commands::$module::handlers::$handler::[<$module _ $handler>];
        }
    };
}

/// Import handler with LocalArgs
///
/// # Usage
/// ```rust
/// use_handler_with_args!(remote, add);
/// // Expands to:
/// // use crate::cli::commands::remote::handlers::add::{remote_add, LocalArgs as RemoteAddArgs};
/// ```
#[macro_export]
macro_rules! use_handler_with_args {
    ($module:ident, $handler:ident) => {
        paste::paste! {
            use $crate::cli::commands::$module::handlers::$handler::{
                [<$module _ $handler>],
                LocalArgs as [<$module:camel $handler:camel Args>]
            };
        }
    };
    ($module:ident, $handler:ident as $alias:ident) => {
        paste::paste! {
            use $crate::cli::commands::$module::handlers::$handler::{
                [<$module _ $handler>] as [<$alias _ $handler>],
                LocalArgs as [<$alias:camel Args>]
            };
        }
    };
}

/// Import multiple handlers with mixed types
///
/// # Usage
/// ```rust
/// use_handlers! {
///     // Handlers without LocalArgs
///     simple: {
///         (remote, list),
///         (path, list),
///     },
///     
///     // Handlers with LocalArgs
///     with_args: {
///         (remote, add),
///         (remote, remove),
///         (remote, update),
///         (remote, ls),
///         (path, add),
///         (path, remove),
///         (sync, path_sync),
///         (sync, all_sync as sync_all), // Alias when needed
///     }
/// }
/// ```
#[macro_export]
macro_rules! use_handlers {
    (
        $(simple: { $(($simple_mod:ident, $simple_handler:ident)),* $(,)? })?
        $(, with_args: { $(($args_mod:ident, $args_handler:ident $(as $alias:ident)?)),* $(,)? })?
        $(,)?
    ) => {
        // Import simple handlers (no LocalArgs)
        $($(
            $crate::use_handler!($simple_mod, $simple_handler);
        )*)?

        // Import handlers with LocalArgs
        $($(
            $crate::use_handler_with_args!($args_mod, $args_handler $(as $alias)?);
        )*)?
    };
}

/// Create CommandContext with LocalArgs inline
///
/// # Usage
/// ```rust
/// // With LocalArgs - direct field names
/// let ctx = command_context!(global, registry, RemoteAddArgs { name, provider });
///
/// // With LocalArgs - field mapping
/// let ctx = command_context!(global, registry, PathRemoveArgs { path_id: id });
///
/// // Without LocalArgs
/// let ctx = command_context!(global, registry);
/// ```
#[macro_export]
macro_rules! command_context {
    // With LocalArgs - support field: value syntax
    ($global:expr, $registry:expr, $args_type:ident { $($field:ident $(: $value:expr)?),* $(,)? }) => {
        CommandContext::from((
            $global,
            $registry,
            $args_type {
                $(
                    $field $(: $value)?
                ),*
            }
        ))
    };

    // Without LocalArgs
    ($global:expr, $registry:expr) => {
        CommandContext::from(($global, $registry))
    };
}
