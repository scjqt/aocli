mod action;
mod network;
mod run;

use aocli::{
    error::{AocError, Arg, Context, ErrorDisplayer, Result, ToErr},
    file::{CurrentDirectory, PathInfo},
    ROOT,
};

use std::env;

fn main() {
    cli().display_err();
}

fn cli() -> Result<()> {
    use Command::*;
    use CurrentDirectory::*;

    let args: Vec<_> = env::args().collect();
    let args: Vec<_> = args.iter().map(|arg| arg.trim()).collect();
    if args.len() <= 1 {
        return "must provide a command".err();
    }
    let command = Command::from_arg(args[1])?;
    let args = &args[2..];
    let (root, current) = &CurrentDirectory::get()?;
    match (command, current) {
        (Init, _) => {
            if !args.is_empty() {
                return AocError::ExtraArg(args[0].into()).err().usage("init");
            }
            action::init(root)
        }
        (New, Root) => {
            const USAGE: &str = "new <YEAR> <DAY>";
            assert_args(args, &[Arg::Year, Arg::Day]).usage(USAGE)?;
            let year = &year_from_arg(args[0]).usage(USAGE)?;
            let day = &day_from_arg(args[1]).usage(USAGE)?;
            action::new_day(&root.join(year).join(day), year, day)
        }
        (New, Year { year }) => {
            const USAGE: &str = "new <DAY>";
            assert_args(args, &[Arg::Day]).usage(USAGE)?;
            let day = &day_from_arg(args[0]).usage(USAGE)?;
            action::new_day(&root.join(year).join(day), year, day)
        }
        (New, Day { .. }) => Err(AocError::CommandDir("new".into()).into()),
        (Add, Root) => {
            const USAGE: &str = "add <YEAR> <DAY> <INPUT>";
            assert_args(args, &[Arg::Year, Arg::Day, Arg::Input]).usage(USAGE)?;
            let year = &year_from_arg(args[0]).usage(USAGE)?;
            let day = &day_from_arg(args[1]).usage(USAGE)?;
            let path = &root.join(year).join(day);
            path.assert_day_dir()?;
            let input = args[2];
            if ["1", "2"].contains(&input) {
                return AocError::InvalidArg(Arg::Input, input.into())
                    .err()
                    .usage(USAGE);
            }
            action::add_input(path, input)
        }
        (Add, Year { year }) => {
            const USAGE: &str = "add <DAY> <INPUT>";
            assert_args(args, &[Arg::Day, Arg::Input]).usage(USAGE)?;
            let day = &day_from_arg(args[0]).usage(USAGE)?;
            let path = &root.join(year).join(day);
            path.assert_day_dir()?;
            let input = args[1];
            if ["1", "2"].contains(&input) {
                return AocError::InvalidArg(Arg::Input, input.into())
                    .err()
                    .usage(USAGE);
            }
            action::add_input(path, input)
        }
        (Add, Day { year, day }) => {
            const USAGE: &str = "add <INPUT>";
            assert_args(args, &[Arg::Input]).usage(USAGE)?;
            let input = args[0];
            if ["1", "2"].contains(&input) {
                return AocError::InvalidArg(Arg::Input, input.into())
                    .err()
                    .usage(USAGE);
            }
            action::add_input(&root.join(year).join(day), input)
        }
        (Get, Root) => {
            const USAGE: &str = "get <YEAR> <DAY>";
            assert_args(args, &[Arg::Year, Arg::Day]).usage(USAGE)?;
            let year = &year_from_arg(args[0]).usage(USAGE)?;
            let day = &day_from_arg(args[1]).usage(USAGE)?;
            let path = &root.join(year).join(day);
            path.assert_day_dir()?;
            action::get(path, year, day)
        }
        (Get, Year { year }) => {
            const USAGE: &str = "get <DAY>";
            assert_args(args, &[Arg::Day]).usage(USAGE)?;
            let day = &day_from_arg(args[0]).usage(USAGE)?;
            let path = &root.join(year).join(day);
            path.assert_day_dir()?;
            action::get(path, year, day)
        }
        (Get, Day { year, day }) => {
            assert_args(args, &[]).usage("get")?;
            let path = &root.join(year).join(day);
            action::get(path, year, day)
        }
        (Run, Root) => {
            const USAGE_1: &str = "run <YEAR>";
            const USAGE_2: &str = "run <YEAR> <DAY> [INPUT] [PART]";
            match args.len() {
                0 => AocError::MissingArg(Arg::Year)
                    .err()
                    .usage(USAGE_1)
                    .usage(USAGE_2),
                1 => {
                    let year = &year_from_arg(args[0]).usage(USAGE_1).usage(USAGE_2)?;
                    let path = &root.join(year);
                    path.assert_year_dir()?;
                    action::run_year(path, year)
                }
                _ => {
                    let year = &year_from_arg(args[0]).usage(USAGE_1).usage(USAGE_2)?;
                    let day = &day_from_arg(args[1]).usage(USAGE_1).usage(USAGE_2)?;
                    let path = &root.join(year).join(day);
                    path.assert_day_dir()?;
                    let (input, parts) = input_parts(&args[2..]).usage(USAGE_1).usage(USAGE_2)?;
                    action::run_day(path, year, day, input, parts, false)
                }
            }
        }
        (Run, Year { year }) => {
            if args.is_empty() {
                action::run_year(&root.join(year), year)
            } else {
                const USAGE_1: &str = "run";
                const USAGE_2: &str = "run <DAY> [INPUT] [PART]";
                let day = &day_from_arg(args[0]).usage(USAGE_1).usage(USAGE_2)?;
                let path = &root.join(year).join(day);
                path.assert_day_dir()?;
                let (input, parts) = input_parts(&args[1..]).usage(USAGE_1).usage(USAGE_2)?;
                action::run_day(path, year, day, input, parts, false)
            }
        }
        (Run, Day { year, day }) => {
            let (input, parts) = input_parts(args).usage("run [INPUT] [PART]")?;
            let path = &root.join(year).join(day);
            action::run_day(path, year, day, input, parts, false)
        }
        (Debug, Root) => {
            const USAGE: &str = "debug <YEAR> <DAY> [INPUT] [PART]";
            assert_first_args(args, &[Arg::Year, Arg::Day]).usage(USAGE)?;
            let year = &year_from_arg(args[0]).usage(USAGE)?;
            let day = &day_from_arg(args[1]).usage(USAGE)?;
            let path = &root.join(year).join(day);
            path.assert_day_dir()?;
            let (input, parts) = input_parts(&args[2..]).usage(USAGE)?;
            action::run_day(path, year, day, input, parts, true)
        }
        (Debug, Year { year }) => {
            const USAGE: &str = "debug <DAY> [INPUT] [PART]";
            assert_first_args(args, &[Arg::Day]).usage(USAGE)?;
            let day = &day_from_arg(args[0]).usage(USAGE)?;
            let path = &root.join(year).join(day);
            path.assert_day_dir()?;
            let (input, parts) = input_parts(&args[1..]).usage(USAGE)?;
            action::run_day(path, year, day, input, parts, true)
        }
        (Debug, Day { year, day }) => {
            let (input, parts) = input_parts(args).usage("debug [INPUT] [PART]")?;
            let path = &root.join(year).join(day);
            action::run_day(path, year, day, input, parts, true)
        }
        (Test, Root) => {
            const USAGE_1: &str = "test <YEAR>";
            const USAGE_2: &str = "test <YEAR> <DAY> [PART]";
            match args.len() {
                0 => AocError::MissingArg(Arg::Year)
                    .err()
                    .usage(USAGE_1)
                    .usage(USAGE_2),
                1 => {
                    let year = &year_from_arg(args[0]).usage(USAGE_1).usage(USAGE_2)?;
                    let path = &root.join(year);
                    path.assert_year_dir()?;
                    action::test_year(path, year)
                }
                _ => {
                    let year = &year_from_arg(args[0]).usage(USAGE_1).usage(USAGE_2)?;
                    let day = &day_from_arg(args[1]).usage(USAGE_1).usage(USAGE_2)?;
                    let path = &root.join(year).join(day);
                    path.assert_day_dir()?;
                    let parts = Parts::from_args(&args[2..]).usage(USAGE_1).usage(USAGE_2)?;
                    action::test_day(path, year, day, parts)
                }
            }
        }
        (Test, Year { year }) => {
            if args.is_empty() {
                action::test_year(&root.join(year), year)
            } else {
                const USAGE_1: &str = "test";
                const USAGE_2: &str = "test <DAY> [PART]";
                let day = &day_from_arg(args[0]).usage(USAGE_1).usage(USAGE_2)?;
                let path = &root.join(year).join(day);
                path.assert_day_dir()?;
                let parts = Parts::from_args(&args[1..]).usage(USAGE_1).usage(USAGE_2)?;
                action::test_day(path, year, day, parts)
            }
        }
        (Test, Day { year, day }) => {
            let parts = Parts::from_args(args).usage("test [PART]")?;
            let path = &root.join(year).join(day);
            action::test_day(path, year, day, parts)
        }
        (Submit, Root) => {
            const USAGE: &str = "submit <YEAR> <DAY> [ANSWER]";
            assert_first_args(args, &[Arg::Year, Arg::Day]).usage(USAGE)?;
            let year = &year_from_arg(args[0]).usage(USAGE)?;
            let day = &day_from_arg(args[1]).usage(USAGE)?;
            let path = &root.join(year).join(day);
            path.assert_day_dir()?;
            let answer = answer_from_args(&args[2..]).usage(USAGE)?;
            action::submit(path, year, day, answer)
        }
        (Submit, Year { year }) => {
            const USAGE: &str = "submit <DAY> [ANSWER]";
            assert_first_args(args, &[Arg::Day]).usage(USAGE)?;
            let day = &day_from_arg(args[0]).usage(USAGE)?;
            let path = &root.join(year).join(day);
            path.assert_day_dir()?;
            let answer = answer_from_args(&args[1..]).usage(USAGE)?;
            action::submit(path, year, day, answer)
        }
        (Submit, Day { year, day }) => {
            let answer = answer_from_args(args).usage("submit [ANSWER]")?;
            let path = &root.join(year).join(day);
            action::submit(path, year, day, answer)
        }
        (Open, Root | Unknown) => {
            const USAGE_1: &str = "open <YEAR>";
            const USAGE_2: &str = "open <YEAR> <DAY>";
            assert_first_args(args, &[Arg::Year])
                .usage(USAGE_1)
                .usage(USAGE_2)?;
            let year = &year_from_arg(args[0]).usage(USAGE_1).usage(USAGE_2)?;
            if args.len() == 1 {
                action::open_year(year)
            } else {
                assert_args(&args[1..], &[Arg::Day])?;
                let day = &day_from_arg(args[1]).usage(USAGE_1).usage(USAGE_2)?;
                action::open_day(year, day)
            }
        }
        (Open, Year { year }) => {
            if args.is_empty() {
                action::open_year(year)
            } else {
                assert_args(args, &[Arg::Day])?;
                let day = &day_from_arg(args[0]).usage("open").usage("open <DAY>")?;
                action::open_day(year, day)
            }
        }
        (Open, Day { year, day }) => action::open_day(year, day),
        (Help, _) => {
            assert_args(args, &[])?;
            action::help()
        }
        (_, Unknown) => format!("unknown directory - failed to find file `{ROOT}`").err(),
        (Session, _) => action::session(root),
    }
}

fn assert_args(args: &[&str], params: &[Arg]) -> Result<()> {
    match args.len().cmp(&params.len()) {
        std::cmp::Ordering::Less => AocError::MissingArg(params[args.len()]).err(),
        std::cmp::Ordering::Equal => Ok(()),
        std::cmp::Ordering::Greater => AocError::ExtraArg(args[params.len()].into()).err(),
    }
}

fn assert_first_args(args: &[&str], params: &[Arg]) -> Result<()> {
    if args.len() < params.len() {
        AocError::MissingArg(params[args.len()]).err()
    } else {
        Ok(())
    }
}

fn input_parts<'a>(args: &[&'a str]) -> Result<(&'a str, Parts)> {
    Ok(match args.len() {
        0 => ("actual", Parts::Default),
        1 => match args[0] {
            "1" => ("actual", Parts::Part("1".into())),
            "2" => ("actual", Parts::Part("2".into())),
            input => (input, Parts::Default),
        },
        2 => {
            if ["1", "2"].contains(&args[0]) {
                return AocError::InvalidArg(Arg::Input, args[0].into()).err();
            }
            let parts = match args[1] {
                "1" => Parts::Part("1".into()),
                "2" => Parts::Part("2".into()),
                _ => {
                    return AocError::Part
                        .err()
                        .context(AocError::InvalidArg(Arg::Part, args[1].into()));
                }
            };
            (args[0], parts)
        }
        _ => return AocError::ExtraArg(args[2].into()).err(),
    })
}

fn answer_from_args<'a>(args: &[&'a str]) -> Result<Option<&'a str>> {
    match args.len() {
        0 => Ok(None),
        1 => Ok(Some(args[0])),
        _ => Err(AocError::ExtraArg(args[1].into()).into()),
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Parts {
    Part(String),
    Default,
}

impl Parts {
    fn from_args(args: &[&str]) -> Result<Self> {
        match args.len() {
            0 => Ok(Self::Default),
            1 => match args[0] {
                "1" => Ok(Self::Part("1".into())),
                "2" => Ok(Self::Part("2".into())),
                _ => Err(AocError::Part).context(AocError::InvalidArg(Arg::Part, args[0].into())),
            },
            _ => Err(AocError::ExtraArg(args[1].into()).into()),
        }
    }
}

enum Command {
    Add,
    Debug,
    Get,
    Help,
    Init,
    New,
    Open,
    Run,
    Session,
    Submit,
    Test,
}

impl Command {
    fn from_arg(arg: &str) -> Result<Self> {
        match arg {
            "add" => Ok(Self::Add),
            "debug" => Ok(Self::Debug),
            "get" => Ok(Self::Get),
            "help" => Ok(Self::Help),
            "init" => Ok(Self::Init),
            "new" => Ok(Self::New),
            "open" => Ok(Self::Open),
            "run" => Ok(Self::Run),
            "session" => Ok(Self::Session),
            "submit" => Ok(Self::Submit),
            "test" => Ok(Self::Test),
            _ => format!("invalid command `{arg}`").err(),
        }
    }
}

fn year_from_arg(arg: &str) -> Result<String> {
    let mut num = arg
        .parse::<u16>()
        .map_err(|_| AocError::Integer)
        .context(AocError::InvalidArg(Arg::Year, arg.into()))?;
    if num < 1000 {
        num += 2000;
    }
    (num >= 2015)
        .then(|| num.to_string())
        .ok_or(AocError::YearRange)
        .context(AocError::InvalidArg(Arg::Year, arg.into()))
}

fn day_from_arg(arg: &str) -> Result<String> {
    let num = arg
        .parse::<u16>()
        .map_err(|_| AocError::Integer)
        .context(AocError::InvalidArg(Arg::Day, arg.into()))?;
    ((1..=25).contains(&num))
        .then(|| format!("{num:02}"))
        .ok_or(AocError::DayRange)
        .context(AocError::InvalidArg(Arg::Day, arg.into()))
}
