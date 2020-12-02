#![feature(array_methods)]

#[macro_use]
extern crate clap;

#[macro_use]
extern crate indoc;

mod cli;
mod config;
mod hardware;
mod new_command;

use crate::cli::{
    setup_and_get_cli_args, DEV_BOARDS_HELP_COMMAND_NAME, NEW_COMMAND_NAME, PROJECT_NAME_ARG_NAME,
    TARGETS_HELP_COMMAND_NAME,
};
use crate::config::{config_file_path, get_persisted_config};
use crate::hardware::{DevBoard, Target, SUPPORTED_DEV_BOARDS, SUPPORTED_TARGETS};
use crate::new_command::run_new_command;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let (default_target, default_dev_board) = match get_persisted_config() {
        Some(config) => {
            log_config_fields(config.default_target, config.default_dev_board);
            (config.default_target, config.default_dev_board)
        }
        None => (None, None),
    };
    let arg_matches = setup_and_get_cli_args(
        SUPPORTED_TARGETS.as_slice(),
        SUPPORTED_DEV_BOARDS.as_slice(),
        default_target.is_none(),
    );

    if let Some(new_command_matches) = arg_matches.subcommand_matches(NEW_COMMAND_NAME) {
        let project_name = cli::project_name_arg(new_command_matches)
            .unwrap_or_else(|| panic!("`{}` arg should be a required", PROJECT_NAME_ARG_NAME));
        let target = cli::target_arg(new_command_matches)
            .or(default_target)
            .expect("`target` arg should be a required");
        let dev_board = cli::dev_board_arg(new_command_matches).or(default_dev_board);
        run_new_command(project_name.clone(), target, dev_board)?;
        let dev_board_str = match dev_board {
            Some(dev_board) => format!(" and dev-board '{}'", dev_board.as_ref()),
            None => "".to_owned(),
        };
        colour::green!("Created ");
        println!(
            "new HDL project '{}' with target '{}'{}",
            project_name,
            target.as_ref(),
            &dev_board_str
        );
        return Ok(());
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

    if arg_matches
        .subcommand_matches(DEV_BOARDS_HELP_COMMAND_NAME)
        .is_some()
    {
        colour::blue_ln!("\n`hdlman` supports the following dev-boards:\n");
        SUPPORTED_DEV_BOARDS.iter().for_each(|target| {
            colour::blue_ln!("name: {}", target.name);
            colour::blue_ln!("{}", target.description);
        });
    }

    Ok(())
}

fn log_config_fields(target: Option<Target>, dev_board: Option<DevBoard>) {
    let config_file_suffix = format!(
        "from config file '{}'",
        config_file_path().map_or("unknown (please file a bug)".to_owned(), |path| {
            path.to_str().unwrap_or("invalid-utf-8-path").to_owned()
        })
    );
    match (target, dev_board) {
        (Some(target), Some(dev_board)) => {
            colour::blue_ln!(
                "found default target '{}' and default dev-board '{}' {}",
                target.as_ref(),
                dev_board.as_ref(),
                config_file_suffix
            )
        }
        (Some(target), None) => {
            colour::blue_ln!(
                "found default target '{}' {}",
                target.as_ref(),
                config_file_suffix
            )
        }
        (None, Some(dev_board)) => {
            colour::blue_ln!(
                "found default dev-board '{}' {}",
                dev_board.as_ref(),
                config_file_suffix
            )
        }
        (None, None) => {}
    }
}
