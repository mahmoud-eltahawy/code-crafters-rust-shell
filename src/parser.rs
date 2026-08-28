use std::env::home_dir;
use std::path::PathBuf;

use crate::Word;

use super::ShellCommand;
use nom::branch::alt;
use nom::bytes::complete::{tag, take_until, take_while1};
use nom::bytes::is_not;
use nom::character::char;
use nom::character::complete::multispace0;
use nom::combinator::{not, opt};
use nom::sequence::delimited;
use nom::{IResult, Parser};

#[derive(Debug)]
pub enum BuiltinCommand {
    Exit,
    Echo,
    Type,
    Cd,
    Pwd,
    Nothing,
}

impl std::fmt::Display for BuiltinCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            BuiltinCommand::Exit => "exit",
            BuiltinCommand::Echo => "echo",
            BuiltinCommand::Type => "type",
            BuiltinCommand::Cd => "cd",
            BuiltinCommand::Pwd => "pwd",
            BuiltinCommand::Nothing => "",
        };
        write!(f, "{}", name)
    }
}

pub fn parse_command(input: &str) -> IResult<&str, ShellCommand> {
    alt((echo, typep, cd, foreign, exit, pwd, nothing)).parse(input)
}

pub fn parse_builtin_command_name(input: &str) -> IResult<&str, BuiltinCommand> {
    let echo = tag("echo");
    let typep = tag("type");
    let cd = tag("cd");
    let exit = tag("exit");
    let pwd = tag("pwd");
    alt((echo, typep, cd, exit, pwd))
        .parse(input)
        .map(|(rest, command)| {
            let command = match command {
                "echo" => BuiltinCommand::Echo,
                "type" => BuiltinCommand::Type,
                "cd" => BuiltinCommand::Cd,
                "exit" => BuiltinCommand::Exit,
                "pwd" => BuiltinCommand::Pwd,
                _ => BuiltinCommand::Nothing,
            };
            (rest, command)
        })
}

fn nothing(input: &str) -> IResult<&str, ShellCommand> {
    let (rest, _) = multispace0(input)?;
    not(multispace0).parse(rest)?;
    Ok(("", ShellCommand::Nothing))
}

fn exit(input: &str) -> IResult<&str, ShellCommand> {
    tag("exit")(input).map(|(rest, _)| (rest, ShellCommand::Exit))
}

fn pwd(input: &str) -> IResult<&str, ShellCommand> {
    tag("pwd")(input).map(|(rest, _)| (rest, ShellCommand::Pwd))
}

fn char_seq(input: &str) -> IResult<&str, Word> {
    let (rest, word) = take_while1(|x: char| !x.is_whitespace())(input)?;
    Ok((rest, Word::NonQuated(String::from(" ") + word)))
}

fn quated(input: &str) -> IResult<&str, Word> {
    let double = is_not("\"");
    let single = is_not("'");
    let double = delimited(char('"'), double, char('"'));
    let single = delimited(char('\''), single, char('\''));
    let (rest, word) = alt((double, single)).parse(input)?;
    Ok((rest, Word::Quated(word.to_string())))
}

fn echo(input: &str) -> IResult<&str, ShellCommand> {
    let (rest, _) = tag("echo")(input)?;
    let (mut rest, _) = multispace0(rest)?;
    let mut txts = Vec::new();
    let mut parser = (alt((quated, char_seq)), multispace0);
    let mut first_non_quated = true;
    while let Ok((new_rest, (txt, _))) = parser.parse(rest) {
        rest = new_rest;
        let txt = match txt {
            Word::Quated(x) => Word::Quated(x),
            Word::NonQuated(x) => {
                if first_non_quated {
                    let x = x[1..].to_string();
                    first_non_quated = false;
                    Word::NonQuated(x)
                } else {
                    Word::NonQuated(x)
                }
            }
        };
        txts.push(txt);
    }
    Ok((rest, ShellCommand::Echo(txts)))
}

fn typep(input: &str) -> IResult<&str, ShellCommand> {
    let (rest, _) = tag("type")(input)?;
    let (rest, _) = multispace0(rest)?;
    Ok(("", ShellCommand::Type(rest.to_string())))
}

fn path(input: &str) -> IResult<&str, PathBuf> {
    let delslash = tag("~/");
    let del = tag("~");
    let (rest, del) = opt(alt((delslash, del))).parse(input)?;
    let mut path = del.and_then(|_| home_dir()).unwrap_or_default();
    path.push(rest);
    Ok(("", path))
}

fn foreign(input: &str) -> IResult<&str, ShellCommand> {
    let (rest, command) = take_until(" ").parse(input)?;
    Ok((
        "",
        ShellCommand::Foreign {
            command: command.to_string(),
            args: rest.split_whitespace().map(|x| x.to_string()).collect(),
        },
    ))
}
fn cd(input: &str) -> IResult<&str, ShellCommand> {
    let (rest, _) = tag("cd")(input)?;
    let (rest, _) = multispace0(rest)?;
    let (_, path) = path(rest)?;

    Ok(("", ShellCommand::Cd(path)))
}

#[test]
pub fn foreign_test() {
    let (_, command) = foreign("cute hello world").unwrap();

    let ShellCommand::Foreign { command, args } = command else {
        panic!("expected foreign varient");
    };
    assert!(command == "cute");
    assert!(args == ["hello", "world"]);
}

#[test]
pub fn cd_test() {
    let (_, single) = cd("cd ~/magit/dotfiles").unwrap();

    let ShellCommand::Cd(path) = single else {
        panic!("expected cd varient");
    };
    let mut t = home_dir().unwrap();
    t.push("magit/dotfiles");
    assert!(path.display().to_string() == t);
}

#[test]
pub fn path_test() {
    let (_, one) = path("~/magit/dotfiles").unwrap();
    let (_, two) = path("/usr/bin").unwrap();
    let mut h = home_dir().unwrap();
    h.push("magit/dotfiles");
    assert!(one == h);
    assert!(two == "/usr/bin".parse::<PathBuf>().unwrap());
}

#[test]
pub fn echo_test() {
    let (rest, single) = echo("echo 'hello world'").unwrap();
    assert!(rest.is_empty());
    let ShellCommand::Echo(txt) = single else {
        panic!("expectd echo varient");
    };
    assert!(txt.first().unwrap().to_string() == "hello world");
    let (rest, single) = echo("echo 'hello world' ' hello again'").unwrap();
    assert!(rest.is_empty());
    let ShellCommand::Echo(txt) = single else {
        panic!("expectd echo varient");
    };
    let txt = txt.iter().map(|x| x.to_string()).collect::<Vec<_>>();
    assert!(txt == ["hello world", " hello again"]);
    let (rest, single) = echo("echo hello world").unwrap();
    assert!(rest.is_empty());
    let ShellCommand::Echo(txt) = single else {
        panic!("expectd echo varient");
    };
    let txt = txt.iter().map(|x| x.to_string()).collect::<Vec<_>>();
    assert!(dbg!(txt) == ["hello", " world"]);
}

#[test]
pub fn type_test() {
    let (rest, single) = typep("type echo").unwrap();
    dbg!(rest);
    assert!(rest.is_empty());
    let ShellCommand::Type(txt) = single else {
        panic!("expectd type varient");
    };
    assert!(txt == "echo");
}

#[test]
pub fn exit_test() {
    let (rest, command) = exit("exit hello world").unwrap();
    dbg!(rest);
    let is_err = exit("hello exit world").is_err();
    assert!(is_err);
    assert!(matches!(command, ShellCommand::Exit));
}

#[test]
pub fn pwd_test() {
    let (rest, command) = pwd("pwd hello world").unwrap();
    dbg!(rest);
    let is_err = exit("hello pwd world").is_err();
    assert!(is_err);
    assert!(matches!(command, ShellCommand::Pwd));
}

#[test]
pub fn quated_test() {
    let (rest, double) = quated(r#""exit hello" world"#).unwrap();
    dbg!(rest);
    let (rest, single) = quated(r#"'exit hello' world"#).unwrap();
    dbg!(rest);
    let (rest, esc) = quated(r#"'exit "hello"' world"#).unwrap();
    dbg!(rest);
    assert!(double.to_string() == "exit hello");
    assert!(single.to_string() == "exit hello");
    assert!(esc.to_string() == r#"exit "hello""#);
}
