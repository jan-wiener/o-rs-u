


use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "myapp", about = "An example app")]
pub struct Args {
    /// Should Use CLI?
    #[arg(short, long, default_value_t = false)]
    cli: bool,

    /// Osu File
    #[arg(short, long)]
    osufile: Option<String>,

    /// Music File
    #[arg(short, long)]
    musicfile: Option<String>,




    // /// Your age (optional)
    // #[arg(short, long, default_value_t = 0)]
    // age: u32,
}

impl Args {

    pub fn do_the_thing(&self) {
        if !self.cli {
            return;
        }

        *(crate::BEATMAP_PATH.lock().unwrap()) = self.osufile.as_ref().unwrap().to_string();

        *(crate::MUSIC_PATH.lock().unwrap()) = self.musicfile.as_ref().unwrap().to_string();


    }

    

}


