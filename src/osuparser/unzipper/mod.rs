

use std::fs;
use std::path::Path;
use zip::ZipArchive;

pub fn unzip_osufile(path: &str) -> Result<String, Box<dyn std::error::Error>>{
    let mut file = fs::File::open(path)?;

    let mut archive = ZipArchive::new(file)?;

    archive.extract("./outputfolder")?;


    Ok("".into())
}
