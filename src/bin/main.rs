use lox_rs::{
    printer,
    scanner::Scanner,
    parser::Parser,
};
use std::{
    env,
    io::{self, Write},
};

fn main() {
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();

    let args: Vec<String> = env::args().collect();
    let result = match args.len() {
        1 => run_prompt(),
        2 => run_file(args[1].as_str()),
        _ => {
            writeln!(stdout, "Usage: rlox [script]").expect("Something went wrong");
            std::process::exit(64);
        },
    };

    match result {
        Err(e) => {
            writeln!(stderr, "{}", e).expect("Something went wrong");
            std::process::exit(65);
        },
        Ok(()) => return,
    }
}

fn run_file(path: &str) -> io::Result<()> {
    let contents = std::fs::read_to_string(path)?;
    run(contents.as_str())
 }

fn run_prompt() -> io::Result<()> {
    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();

    loop {
        write!(stdout, "> ")?;
        stdout.flush()?;

        buffer.clear();

        let num_bytes = stdin.read_line(&mut buffer)?;
        if num_bytes == 0 { break };

        if let Err(e) = run(buffer.as_str()) {
            writeln!(stderr, "{}", e)?;
        }
    }

    Ok(())
}

fn run(source: &str) -> io::Result<()> {
    let scanner = Scanner::new(source);
    let tokens = scanner.into_iter().filter_map(|e| e.ok() );
    let mut parser = Parser::new(tokens);
    let parsed = parser.parse()?;

    println!("{}", printer::print(&parsed));

    Ok(())
}