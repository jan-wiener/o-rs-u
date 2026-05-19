use bevy::prelude::*;
use std::fs;
use clap::Parser;
use thiserror::Error;
use crate::osuparser;


#[derive(Error, Debug)]
enum ArgError {
    #[error("Cli was not used")]
    IsNotCli,
    #[error("Incorrect argument combination")]
    ArgumentNotSupplied,
}


#[derive(Parser, Debug)]
#[command(name = "Orsu Cli", about = "Orsu, the legendary rust rewrite")]
pub struct Args {

    /// Osz File
    oszfile: Option<String>,

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

    pub fn extract_osz_file(&mut self) -> Result<(), Box<dyn std::error::Error>>{
        if !self.uses_cli {
            return Err(Box::new(ArgError::IsNotCli));
        }
        let _ = fs::remove_dir_all("assets/cli");


        let osz_extract_path = osuparser::unzipper::unzip_osufile(&self.args.oszfile.as_ref().ok_or(ArgError::ArgumentNotSupplied)?, "assets/cli")?;

        let osu_files = osuparser::unzipper::get_osu_files_from_extracted_osz_file(&osz_extract_path).unwrap();


        for (idx, file) in osu_files.iter().enumerate() {
            println!("{} - {:?}", idx,file);
        }

        let idx = input().parse::<usize>().expect("Index out of range/Bad Input");


        println!("f: {:?}", osu_files[idx]);


        set_static_values(&format!("assets/cli/{}", osu_files[idx].file_name().to_str().unwrap()));


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


