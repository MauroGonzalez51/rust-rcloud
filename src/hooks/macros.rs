#[macro_export]
macro_rules! register_hooks {
    (
        $(
            $variant:ident {
                config: $config_ty:ty,
                hook: $hook_ty:ty,
                enum_type: $enum_val:path,
                modifies_name: $modifies:expr,
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

        impl From<HookConfig> for Box<dyn Hook> {
            fn from(val: HookConfig) -> Self {
                match val {
                    $(
                        HookConfig::$variant(cfg) => Box::new(<$hook_ty>::from(cfg)),
                    )*
                }
            }
        }

        impl HookConfig {
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
        }
    };
}

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
