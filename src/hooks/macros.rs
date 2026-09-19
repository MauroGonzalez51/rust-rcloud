/// Wires a set of hook kinds into the type system in one place.
///
/// For every entry it generates:
/// - a variant on the serde-tagged `HookConfig` enum (`#[serde(tag = "type")]`),
/// - `From<HookConfig> for Box<dyn Hook>` so a stored config becomes a runnable hook,
/// - `HookConfig::modifies_filename`, `exec_type`, `hook_type`, and `Display`,
/// - `Hooks::describe` (per-direction help text) and `Hooks::share_config`.
///
/// # Fields
/// - `config` / `hook`: the `XHookConfig` (serialized) and `XHook` (runtime) types,
///   typically produced by [`crate::define_hook!`].
/// - `enum_type`: the matching [`Hooks`](crate::config::prelude::Hooks) variant.
/// - `modifies_name`: whether the hook changes the output filename
///   (drives [`compute_remote_filename`](crate::cli::commands::sync::utils::compute_remote_filename())).
/// - `share_config`: whether a single interactive setup can produce both the
///   push and pull configs at once.
/// - `display`: closure rendering the config for listings.
/// - `push_desc` / `pull_desc`: help strings shown when selecting the hook.
///
/// # Example
/// ```rust, ignore
/// register_hooks! {
///     Zip {
///         config: ZipHookConfig,
///         hook: ZipHook,
///         enum_type: Hooks::Zip,
///         modifies_name: true,
///         share_config: false,
///         display: |cfg: &ZipHookConfig, f: &mut std::fmt::Formatter| write!(f, "Zip"),
///         push_desc: "Compress before uploading",
///         pull_desc: "Extract after downloading",
///     },
/// }
/// ```
#[macro_export]
macro_rules! register_hooks {
    (
        $(
            $variant:ident {
                config: $config_ty:ty,
                hook: $hook_ty:ty,
                enum_type: $enum_val:path,
                modifies_name: $modifies:expr,
                share_config: $share_config:expr,
                display: $display_fn:expr,
                push_desc: $push_desc:literal,
                pull_desc: $pull_desc:literal,
            }
        ),* $(,)?
    ) => {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[serde(tag = "type", rename_all = "snake_case")]
        pub enum HookConfig {
            $(
                $variant($config_ty),
            )*
        }

        impl HookConfig {
            /// Runs this hook over `ctx`, dispatching to the concrete hook kind.
            ///
            /// Replaces the old `Box<dyn Hook>` path: because `Hook` carries an
            /// associated `Acquired` type it is not object-safe, so the pipeline
            /// matches on the config enum and calls each concrete hook's
            /// monomorphized `process` (acquire + transform).
            pub fn process(
                &self,
                ctx: $crate::hooks::prelude::HookContext,
                cfg: &AppConfig,
            ) -> anyhow::Result<$crate::hooks::prelude::HookContext> {
                match self {
                    $(
                        HookConfig::$variant(cfg_inner) => {
                            <$hook_ty>::from(cfg_inner.clone()).process(ctx, cfg)
                        }
                    )*
                }
            }

            pub fn modifies_filename(&self) -> bool {
                match self {
                    $(
                        HookConfig::$variant(_) => $modifies,
                    )*
                }
            }

            pub fn exec_type(&self) -> &HookExecType {
                match self {
                    $(
                        HookConfig::$variant(cfg) => &cfg.exec,
                    )*
                }
            }

            pub fn hook_type(&self) -> &Hooks {
                match self {
                    $(
                        HookConfig::$variant(_) => &$enum_val,
                    )*
                }
            }
        }

        impl std::fmt::Display for HookConfig {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        HookConfig::$variant(cfg) => $display_fn(cfg, f),
                    )*
                }
            }
        }

        impl Hooks {
            pub fn describe(&self, direction: $crate::config::hook_config::HookExecType) -> &'static str {
                match self {
                    $(
                        $enum_val => match direction {
                            $crate::config::hook_config::HookExecType::Push => $push_desc,
                            $crate::config::hook_config::HookExecType::Pull => $pull_desc,
                        },
                    )*
                }
            }

            pub fn share_config(&self) -> bool {
                match self {
                    $(
                        $enum_val => $share_config,
                    )*
                }
            }
        }
    };
}

/// Generates the paired config and runtime structs for a hook.
///
/// Given `Name { field: Type, ... }` it produces:
/// - `NameConfig`: `Serialize`/`Deserialize` struct with an `exec:
///   HookExecType` field plus the declared fields — this is what lives in the
///   registry JSON.
/// - `Name`: the runtime struct with the same fields.
/// - `From<NameConfig> for Name`: converts the stored config into the runnable
///   hook.
///
/// You still implement [`Hook`](crate::config::prelude::Hook) for `Name` and
/// [`HookBuilderTrait`](crate::hooks::prelude::HookBuilderTrait) for
/// `NameConfig`, then register both with [`crate::register_hooks!`].
///
/// # Example
/// ```rust, ignore
/// define_hook!(ZipHook {
///     level: Option<i64>,
///     exclude: Option<Vec<String>>,
/// });
/// // Expands to ZipHookConfig { exec, level, exclude } and ZipHook { .. }.
/// ```
#[macro_export]
macro_rules! define_hook {
    (
        $hook_name:ident {
            $($field:ident: $field_ty:ty),* $(,)?
        }
    ) => {
        paste::paste! {
            #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
            pub struct [<$hook_name Config>] {
                pub exec: $crate::config::hook_config::HookExecType,
                $(pub $field: $field_ty),*
            }
        }

        #[derive(Debug)]
        pub struct $hook_name {
            pub exec: $crate::config::hook_config::HookExecType,
            $(pub $field: $field_ty),*
        }

        paste::paste! {
            impl From<[<$hook_name Config>]> for $hook_name {
                fn from(config: [<$hook_name Config>]) -> Self {
                    Self {
                        exec: config.exec,
                        $($field: config.$field),*
                    }
                }
            }
        }
    };
}
