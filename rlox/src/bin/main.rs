use rlox::{
    interpreter::Interpreter,
    scanner::Scanner,
    parser::Parser,
};
use std::{
    env,
    io::{self, Write},
};

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();

    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => run_prompt(&mut stdout, &mut stderr)?,
        2 => run_file(args[1].as_str(), &mut stdout, &mut stderr)?,
        _ => {
            writeln!(stdout, "Usage: rlox [script]")?;
            std::process::exit(64);
        },
    };

    Ok(())
}

fn run_file(path: &str, out: &mut io::Stdout, err_out: &mut io::Stderr) -> io::Result<()> {
    let contents = std::fs::read_to_string(path)?;
    Lox::new(out, err_out).run(contents.as_str())
 }

fn run_prompt(out: &mut io::Stdout, err_out: &mut io::Stderr) -> io::Result<()> {
    let mut buffer = String::new();
    let stdin = io::stdin();

    let mut lox = Lox::new(io::stdout(), err_out);

    loop {
        write!(out, "> ")?;
        out.flush()?;

        buffer.clear();

        let num_bytes = stdin.read_line(&mut buffer)?;
        if num_bytes == 0 { break };

        lox.run(buffer.as_str())?;
    }

    Ok(())
}

struct Lox<Out, ErrOut> {
    interpreter: Interpreter<Out>,
    err_out: ErrOut,
}

impl <Out: Write, ErrOut: Write> Lox<Out, ErrOut> {
    fn new(out: Out, err_out: ErrOut) -> Self {
        Self {
            interpreter: Interpreter::new(out),
            err_out,
        }
    }

    fn run(&mut self, source: &str) -> io::Result<()> {
        let scanner = Scanner::new(source);
        let (tokens, errors): (Vec<_>, Vec<_>) = scanner.into_iter().partition(Result::is_ok);

        let errors: Vec<_> = errors.into_iter().map(Result::unwrap_err).collect();
        if !errors.is_empty() {
            for e in errors.iter() {
                writeln!(self.err_out, "{}", e)?;
            }
            std::process::exit(65);
        }

        let tokens: Vec<_> = tokens.into_iter().map(Result::unwrap).collect();
        let mut parser = Parser::new(tokens.into_iter());
        let (statements, errors): (Vec<_>, Vec<_>) = parser.parse().into_iter().partition(Result::is_ok);

        let errors: Vec<_> = errors.into_iter().map(Result::unwrap_err).collect();
        if !errors.is_empty() {
            for e in errors.iter() {
                writeln!(self.err_out, "{}", e)?;
            }
            std::process::exit(65);
        }

        let statements: Vec<_> = statements.into_iter().map(Result::unwrap).collect();
        match self.interpreter.interpret(&statements) {
            Err(e) => {
                writeln!(self.err_out, "{}", e)?;
                std::process::exit(70)
            },
            Ok(()) => Ok(())
        }
    }
}