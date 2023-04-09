use std::env;
use std::fs::File;
use std::path::Path;
use std::process::Command;
use std::{fs, process};

pub enum Action {
    Save,
    List,
    Show,
    Run,
    Delete,
}

pub struct Saver {
    pub db_dir: String,
    pub name: String,
    pub cmd: String,
    pub cmd_args: Vec<String>,
    pub action: Action,
}

impl Saver {
    pub fn build(mut args: impl Iterator<Item = String>) -> Saver {
        args.next();
        let empty_str = &String::new();

        let show_help_and_exit = || {
            Self::show_help();
            process::exit(1);
        };

        let validate_arg = |arg| {
            if arg == empty_str {
                Self::show_help();
                process::exit(1);
            }
        };

        let arg1 = match args.next() {
            Some(v) => v,
            None => return show_help_and_exit(),
        };

        let action = match arg1.as_ref() {
            "s" => Action::Save,
            "l" => Action::List,
            "g" => Action::Show,
            "r" => Action::Run,
            "d" => Action::Delete,
            _ => {
                return show_help_and_exit();
            }
        };

        let mut name = String::new();
        let mut cmd = String::new();
        let mut cmd_args: Vec<String> = Vec::new();

        let mut db_dir = Self::get_default_db_dir();
        let mut next_is_dir = false;

        match action {
            Action::Save => {
                let arg2 = args.next().unwrap_or_else(String::new);
                let arg3 = args.next().unwrap_or_else(String::new);
                validate_arg(&arg2);
                validate_arg(&arg3);
                name.push_str(&arg2);
                cmd.push_str(&arg3);
            }
            Action::List => (),
            Action::Show => {
                let arg2 = args.next().unwrap_or_else(String::new);
                validate_arg(&arg2);
                name.push_str(&arg2);
            }
            Action::Run => {
                let arg2 = args.next().unwrap_or_else(String::new);
                validate_arg(&arg2);
                name.push_str(&arg2);
            }
            Action::Delete => {
                let arg2 = args.next().unwrap_or_else(String::new);
                validate_arg(&arg2);
                name.push_str(&arg2);
            }
        }

        for arg in args {
            if arg.trim() == "--saver-db" {
                next_is_dir = true;
                continue;
            }
            if next_is_dir {
                db_dir = arg;
                next_is_dir = false;
                continue;
            }
            cmd_args.push(arg);
        }

        Saver {
            action,
            name,
            cmd,
            cmd_args,
            db_dir,
        }
    }

    fn get_default_db_dir() -> String {
        let home_dir = match env::var_os("HOME") {
            Some(path) => path,
            None => {
                eprintln!("Failed to get home directory");
                process::exit(1);
            }
        };

        let mut path = home_dir.into_string().unwrap_or("/tmp".to_string());
        path.push_str("/.saver/db");

        path
    }

    pub fn show_help() {
        println!("Usage: saver <action> <cmd?> <args?>");
        println!("Actions:\n");
        println!("s <name>");
        println!("used to save command under <name>");
        println!("Example: saver s curl-example curl https://example.com");
        println!("\nl");
        println!("used to list saved commands");
        println!("Example: saver l");
        println!("\ng <name>");
        println!("used to get saved command");
        println!("Example: saver g curl-example");
        println!("\nr <name>");
        println!("used to execute saved command");
        println!("Example: saver r curl-example");
        println!("\nd <name>");
        println!("used to remove saved command");
        println!("Example: saver d curl-example");
        println!("\n\nArgs:");
        println!("--saver-db - database folder path (path should be accessible by the program)");
        println!("Example: saver list --saver-db /tmp/saver/db");
    }

    pub fn run(&self) -> Result<(), String> {
        self.create_db_dir()?;
        match self.action {
            Action::Save => {
                self.save_cmd()?;
                self.run_cmd(&self.cmd, &self.cmd_args)
            }
            Action::List => self.list()?,
            Action::Run => {
                let mut args: Vec<String> = self
                    .get_cmd()
                    .split(" ")
                    .map(|v| v.trim().to_string())
                    .collect();
                let cmd = args.remove(0);
                self.run_cmd(&cmd, &args);
            }
            Action::Show => {
                let cmd = self.get_cmd();
                println!("{}", cmd);
            }
            Action::Delete => self.delete_cmd(),
        }
        Ok(())
    }

    fn create_db_dir(&self) -> Result<(), String> {
        let exists = Path::new(&self.db_dir).is_dir();
        if exists {
            return Ok(());
        }

        match fs::create_dir_all(&self.db_dir) {
            Ok(_) => (),
            Err(e) => return Err(e.to_string()),
        }

        Ok(())
    }

    fn list(&self) -> Result<(), String> {
        let files = fs::read_dir(&self.db_dir).unwrap();

        for (idx, path) in files.enumerate() {
            println!(
                "{}) {}",
                idx + 1,
                path.unwrap().file_name().into_string().unwrap()
            )
        }

        Ok(())
    }

    fn run_cmd(&self, cmd: &String, args: &Vec<String>) {
        println!("");
        let mut p = Command::new(cmd);
        p.args(args);

        match p.status() {
            Err(e) => {
                eprintln!("{}", e);
                process::exit(1);
            }
            Ok(val) => {
                let code = val.code().unwrap_or(0);
                process::exit(code);
            }
        }
    }

    fn delete_cmd(&self) {
        let path = Path::new(&self.db_dir).join(&self.name);
        match fs::remove_file(path) {
            Ok(_) => (),
            Err(_) => {
                println!(
                    "Command {} not found in database {}",
                    &self.name, &self.db_dir
                );
                process::exit(1);
            }
        };
    }

    fn get_cmd(&self) -> String {
        let path = Path::new(&self.db_dir).join(&self.name);
        let content = match fs::read_to_string(path) {
            Ok(val) => val,
            Err(_) => {
                println!(
                    "Command {} not found in database {}",
                    &self.name, &self.db_dir
                );
                process::exit(1);
            }
        };

        content
    }

    fn save_cmd(&self) -> Result<(), String> {
        let path = Path::new(&self.db_dir).join(&self.name);
        let mut cmd_str = String::from(&self.cmd);
        let error_msg = "Failed to save cmd".to_string();
        cmd_str.push(' ');
        cmd_str.push_str(&self.cmd_args.join(" "));

        match File::create(&path) {
            Err(_) => return Err(error_msg),
            Ok(_) => match fs::write(&path, &cmd_str) {
                Ok(_) => (),
                Err(_) => return Err(error_msg),
            },
        }

        Ok(())
    }
}
