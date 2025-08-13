use std::path::PathBuf;

use ui_test::{
    Config, color_eyre::eyre::Result, dependencies::DependencyBuilder, run_tests, spanned::Spanned,
};

fn main() -> Result<()> {
    let mut config = Config {
        output_conflict_handling: if std::env::var_os("BLESS").is_some() {
            ui_test::bless_output_files
        } else {
            ui_test::error_on_output_conflict
        },
        ..Config::rustc("tests/fail")
    };

    config.comment_defaults.base().require_annotations = Spanned::dummy(false).into();
    config.comment_defaults.base().set_custom(
        "dependencies",
        DependencyBuilder {
            crate_manifest_path: PathBuf::from("tests/Cargo.toml"),
            ..DependencyBuilder::default()
        },
    );
    run_tests(config)
}
