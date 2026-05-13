

use std::fs;
use std::path::Path;
use zip::ZipArchive;

pub fn unzip_osufile(path_zip: &str, path_unzip: &str) -> Result<String, Box<dyn std::error::Error>>{
    let mut file = fs::File::open(path_zip)?;
    let mut archive = ZipArchive::new(file)?;

    archive.extract(format!("./assets/{}", path_unzip))?;

    Ok(path_unzip.into())
}


pub fn get_osu_files_from_extracted_osz_file(path_unzip: &str) -> Result<Vec<fs::DirEntry>, Box<dyn std::error::Error>> {
    let p = format!("assets/{}", path_unzip);
    println!("p: {}",p);
    let folder = fs::read_dir(p)?;
    let mut entries = vec![];
    for file_result in folder {
        let file = file_result?;
        
        
        let fname = file.file_name().to_str().unwrap().to_string();
        if fname.ends_with(".osu") {
            println!("file: {:?}", fname);
        }

        entries.push(file);
    }
    Ok(entries)
}



