#[macro_export]
macro_rules! unpack_command {
    (
        $args:expr,
        $command_pattern:pat => { $($field:ident),+ $(,)? }
    ) => {
        if let $command_pattern = &$args.command {
            ($($field.clone()),+)
        } else {
            panic!("Invalid command type")
        }
    };
}