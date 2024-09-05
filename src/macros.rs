#[macro_export]
macro_rules! not_implemented {
    () => {{
        eprintln!("Not implemented yet!");
        std::process::exit(1);
    }};
}