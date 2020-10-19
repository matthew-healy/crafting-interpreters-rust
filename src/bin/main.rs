use std::{
    env,
    io::{self, Write},
};

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();

    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => run_prompt()?,
        2 => run_file(args[1].as_str())?,
        _ => {
            writeln!(&mut stdout, "Usage: rlox [script]")?;
            std::process::exit(64);
        },
    };

    Ok(())
}

fn run_file(path: &str) -> io::Result<()> {
    let contents = std::fs::read_to_string(path)?;
    run(contents);
    Ok(())
}

fn run_prompt() -> io::Result<()> {
    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        write!(stdout, "> ")?;
        stdout.flush()?;

        buffer.clear();

        let num_bytes = stdin.read_line(&mut buffer)?;
        if num_bytes == 0 { break };

        run(buffer.clone());
    }

    Ok(())
}

fn run(_program: String) {

}