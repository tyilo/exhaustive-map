use std::path::PathBuf;

use ui_test::{
    color_eyre::eyre::Result, dependencies::DependencyBuilder, run_tests, spanned::Spanned,
    CommandBuilder, Config,
};

fn main() -> Result<()> {
    let mut config = Config {
        program: CommandBuilder::rustc(),
        output_conflict_handling: if std::env::var_os("BLESS").is_some() {
            ui_test::bless_output_files
        } else {
            ui_test::error_on_output_conflict
        },
        ..Config::rustc("tests/fail")
    };

    let require_annotations = false; // we're not showing errors in a specific line anyway
    config.comment_defaults.base().exit_status = Spanned::dummy(1).into();
    config.comment_defaults.base().require_annotations = Spanned::dummy(require_annotations).into();
    config.comment_defaults.base().set_custom(
        "dependencies",
        DependencyBuilder {
            crate_manifest_path: PathBuf::from("tests/Cargo.toml"),
            ..DependencyBuilder::default()
        },
    );
    run_tests(config)
}
