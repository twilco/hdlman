#![feature(array_methods)]

#[macro_use]
extern crate clap;

#[macro_use]
extern crate indoc;

mod cli;
mod hardware;
mod new_command;

use crate::cli::{
    setup_and_get_cli_args, NEW_COMMAND_NAME, PROJECT_NAME_ARG_NAME, TARGETS_HELP_COMMAND_NAME,
};
use crate::hardware::{SUPPORTED_DEV_BOARDS, SUPPORTED_TARGETS};
use crate::new_command::run_new_command;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let arg_matches = setup_and_get_cli_args(
        SUPPORTED_TARGETS.as_slice(),
        SUPPORTED_DEV_BOARDS.as_slice(),
    );

    if let Some(new_command_matches) = arg_matches.subcommand_matches(NEW_COMMAND_NAME) {
        let project_name = cli::project_name_arg(new_command_matches).unwrap_or_else(|| {
            panic!("`{}` arg should be a required", PROJECT_NAME_ARG_NAME)
        });
        let target =
            cli::target_arg(new_command_matches).expect("`target` arg should be a required");
        let dev_board = cli::dev_board_arg(new_command_matches);
        run_new_command(project_name, target, dev_board)?
    }

    if arg_matches
        .subcommand_matches(TARGETS_HELP_COMMAND_NAME)
        .is_some()
    {
        colour::blue_ln!("\n`hdlman` supports the following targets:\n");
        SUPPORTED_TARGETS.iter().for_each(|target| {
            colour::blue_ln!("name: {}", target.name);
            colour::blue_ln!("{}", target.description);
        });
    }

    Ok(())
}
