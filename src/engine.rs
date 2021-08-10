use std::{cell::RefCell, fmt, process::{Child, Command, Stdio}, thread, time::Duration};
use std::io::Write;
use std::io::Read;
use anyhow::Result;

pub struct Engine {
    child: RefCell<Child>
}

impl Engine {
    pub fn new(path: &str) -> Result<Engine> {
        let cmd = Command::new(path)
                            .stdin(Stdio::piped())
                            .stdout(Stdio::piped())
                            .spawn()
                            .expect("Unable to run engine");

        let res = Engine {
            child: RefCell::new(cmd),
        };

        res.read_line()?;
        res.command("uci")?;

        Ok(res)
    }

    pub fn set_position(&self, fen: &str) -> Result<()> {
        self.write_fmt(format_args!("position fen {}\n", fen))
    }
    
    pub fn command(&self, cmd: &str) -> Result<String> {
        self.write_fmt(format_args!("{}\n", cmd.trim()))?;
        thread::sleep(Duration::from_millis(20));
        self.read_output()
    }

    pub fn run(&self, depth: usize) -> Result<String> {
        let cmd = format!("go depth {}", depth);
        self.write_fmt(format_args!("{}\n", cmd.trim()))?;
        self.wait_output_after_go()
    }

    fn read_output(&self) -> Result<String> {
        let mut s: Vec<String> = vec![];

        self.write_fmt(format_args!("isready\n"))?;
        loop {
            let next_line = self.read_line()?;
            match next_line.trim() {
                "readyok" => return Ok(s.join("\n")),
                other     => s.push(other.to_string())
            }
        }
    }

    fn wait_output_after_go(&self) -> Result<String> {
        let mut s: Vec<String> = vec![];

        loop {
            let next_line = self.read_line()?;
            s.push(next_line.to_string());
            if next_line.contains("bestmove") { 
                return Ok(s.join("\n"))
            }
        }
    }

    fn read_line(&self) -> Result<String> {
        let mut s = String::new();
        let mut buf: Vec<u8> = vec![0];

        loop {
            self.child.borrow_mut().stdout.as_mut().unwrap().read(&mut buf)?;
            s.push(buf[0] as char);
            if buf[0] == '\n' as u8 {
                break
            }
        }
        Ok(s)
    }

    fn write_fmt(&self, args: fmt::Arguments) -> Result<()> {
        self.child.borrow_mut().stdin.as_mut().unwrap().write_fmt(args)?;
        Ok(())
    }
}
