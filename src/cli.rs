use clap::{AppSettings, Arg, ArgMatches, SubCommand};
use std::str::FromStr;
use crate::hardware::{DevBoard, Target, SupportedEntity};

pub const NEW_COMMAND_NAME: &str = "new";
pub const TARGETS_HELP_COMMAND_NAME: &str = "targets-help";
pub const DEV_BOARDS_HELP_COMMAND_NAME: &str = "dev-boards-help";

pub fn setup_and_get_cli_args<'a>(supported_targets: &[SupportedEntity<'a>], supported_dev_boards: &[SupportedEntity<'a>]) -> ArgMatches<'a> {
    app_from_crate!()
        .subcommand(
            SubCommand::with_name(NEW_COMMAND_NAME)
                .about("Create new HDL project")
                .arg(
                    Arg::with_name("name")
                        .short("n")
                        .long("name")
                        .help("The name of the project to create.  This will be the name of the directory created for the project, and the name of the topfile.")
                        .takes_value(true)
                        .required(true)
                )
                .arg(
                    Arg::with_name("target")
                        .short("t")
                        .long("target")
                        .help("The type of FPGA you will be programming.")
                        .takes_value(true)
                        .possible_values(
                            supported_targets.iter()
                                .map(|target| {
                                target.name
                            })
                                .collect::<Vec<_>>().as_slice()
                        )
                        .required(true)
                )
                .arg(
                    Arg::with_name("dev-board")
                        .short("db")
                        .long("dev-board")
                        .help("The dev-board you targeting (if you have one -- this is not required).")
                        .takes_value(true)
                        .possible_values(
                            supported_dev_boards.iter()
                                .map(|board| {
                                    board.name
                                })
                                .collect::<Vec<_>>().as_slice()
                        )
                )
        )
        .subcommand(
            SubCommand::with_name(TARGETS_HELP_COMMAND_NAME)
                .about("List detailed information about the targets `hdlman` supports.  A target is the FPGA that you will be programming.")
        )
        .subcommand(
            SubCommand::with_name(DEV_BOARDS_HELP_COMMAND_NAME)
                .about("List detailed information about the targets `hdlman` supports.  A dev-board is board that hosts an FPGA plus other nice-to-haves.")
        )
        .setting(AppSettings::ArgRequiredElseHelp)
        .get_matches()
}

pub fn dev_board_arg(arg_matches: &ArgMatches) -> Option<DevBoard> {
    try_get_arg::<DevBoard>(arg_matches, "dev-board")
}

pub fn name_arg(arg_matches: &ArgMatches) -> Option<String> {
    try_get_arg::<String>(arg_matches, "name")
}

pub fn target_arg(arg_matches: &ArgMatches) -> Option<Target> {
    try_get_arg::<Target>(arg_matches, "target")
}

fn try_get_arg<'a, T: FromStr>(arg_matches: &ArgMatches, arg_name: &'a str) -> Option<T> {
    arg_matches
        .value_of(arg_name)
        .map(|arg_str| arg_str.parse::<T>().ok())
        .unwrap_or(None)
}
