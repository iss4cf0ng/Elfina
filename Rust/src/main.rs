use std::{fs::OpenOptions, string};

mod entropy;
mod hexdump;
mod disasm;
mod elf_arch;
mod elf_loader;

const BANNER: &str = r#"
  _        (`-')            <-. (`-')_  _  (`-') (`-')  _ 
 (_)    <-.(OO )      .->      \( OO) ) \-.(OO ) ( OO).-/ 
 ,-(`-'),------,)(`-')----. ,--./ ,--/  _.'    \(,------. 
 | ( OO)|   /`. '( OO).-.  '|   \ |  | (_...--'' |  .---' 
 |  |  )|  |_.' |( _) | |  ||  . '|  |)|  |_.' |(|  '--.  
(|  |_/ |  .   .' \|  |)|  ||  |\    | |  .___.' |  .--'  
 |  |'->|  |\  \   '  '-'  '|  | \   | |  |      |  `---. 
 `--'   `--' '--'   `-----' `--'  `--' `--'      `------' 
"#;

const DESCRIPTION: &str = "Author: iss4cf0ng/ISSAC\nGitHub: https://github.com/iss4cf0ng/Elfina";

const USAGE: &str = "Example:
\tIronPE.exe --x86 <FilePath>
\tIronPE.exe --x64 <FilePath>
\tIronPE.exe --coffee
";

const COFFEE: &str = r#"
    (  )   (   )  )
     ) (   )  (  (
     ( )  (    ) )
     _____________
    <_____________> ___
    |             |/ _ \
    |               | | |
    |               |_| |
 ___|             |\___/
/    \___________/    \
\_____________________/
"#;

#[derive(Default)]
struct Opts {
    force_memfd: bool,
    force_mmap: bool,
    info_only: bool,


}

fn parse_args(raw: &[String]) -> Result<(Opts, usize), String> {
    let mut opts = Opts::default();
    let mut i = 1usize;

    Ok((opts, i))
}

fn main() {
    println!("Hello, world!");
}
