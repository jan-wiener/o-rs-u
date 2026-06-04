use crate::osuparser;
use bevy::prelude::*;
use clap::Parser;
use std::{fs, path::PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
enum ArgError {
    #[error("Cli was not used")]
    IsNotCli,
    #[error("Incorrect argument combination")]
    ArgumentNotSupplied,
    #[error("Path cannot be parsed")]
    BadArgument,
}

#[derive(Parser, Debug)]
#[command(name = "Orsu Cli", about = "Orsu, the legendary rust rewrite")]
pub struct Args {
    /// Osz File
    oszfile: Option<PathBuf>,
}

pub struct Cli {
    args: Args,

    pub uses_cli: bool,
}

pub fn set_static_values(bmp: &str) {
    *(crate::BEATMAP_PATH.lock().unwrap()) = bmp.to_string();
}

pub fn input() -> String {
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf).unwrap();
    buf = buf.trim().to_string();
    buf
}

impl Cli {
    pub fn from_args(args: Args) -> Self {
        Self {
            uses_cli: args.oszfile.is_some(),
            args,
        }
    }

    pub fn extract_osz_file(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.uses_cli {
            return Err(Box::new(ArgError::IsNotCli));
        }
        

        let binary_dir = if std::env::var("CARGO").is_ok() {
            std::env::current_dir().expect("Failed to get CWD")
        } else {
            std::env::current_exe()
                .expect("Failed to get binary path")
                .parent()
                .expect("Binary has no parent directory")
                .to_path_buf()
        };
        let path_unzip_pathbuf = binary_dir.join("assets/cli");

        let _ = fs::remove_dir_all(&path_unzip_pathbuf);


        let path_unzip = path_unzip_pathbuf
            .to_str()
            .ok_or(ArgError::BadArgument)?
            .to_owned();

        let path_zip = self
            .args
            .oszfile
            .as_ref()
            .ok_or(ArgError::ArgumentNotSupplied)?
            .to_str()
            .ok_or(ArgError::BadArgument)?;
        let osz_extract_path = osuparser::unzipper::unzip_osufile(path_zip, &path_unzip)?;

        println!("Path Zip: {} | Path Unzip : {}", path_zip, path_unzip);

        let osu_files =
            osuparser::unzipper::get_osu_files_from_extracted_osz_file(&osz_extract_path).unwrap();

        for (idx, file) in osu_files.iter().enumerate() {
            println!("{} - {:?}", idx, file);
        }

        let idx = input()
            .parse::<usize>()
            .expect("Index out of range/Bad Input");

        println!("f: {:?}", osu_files[idx]);

        let final_osu_path = path_unzip_pathbuf.join(osu_files[idx].file_name().to_str().unwrap());
        println!("FINAL: {:?}", final_osu_path);
        set_static_values(final_osu_path.to_str().ok_or(ArgError::BadArgument)?);

        // set_static_values(&format!(
        //     "assets/cli/{}",
        //     osu_files[idx].file_name().to_str().unwrap()
        // ));

        Ok(())
    }

    // pub fn copy_and_convert(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    //     fs::copy(self.args.osufile.as_ref().ok_or(ArgError::ArgumentNotSupplied)?, "assets/cli/beatmap.osu")?;
    //     fs::copy(self.args.musicfile.as_ref().ok_or(ArgError::ArgumentNotSupplied)?, "assets/cli/musicfile")?;
    //     self.converted = true;
    //     Ok(())
    // }
    // pub fn set_static_values_full_cli_mode(&self) -> Result<(), Box<dyn std::error::Error>> {
    //     if !self.uses_cli() {
    //         return Err(Box::new(ArgError::IsNotCli));
    //     }
    //     *(crate::BEATMAP_PATH.lock().unwrap()) = "assets/cli/beatmap.osu".to_string();
    //     *(crate::MUSIC_PATH.lock().unwrap()) = "cli/musicfile".to_string();
    //     Ok(())
    // }
}
