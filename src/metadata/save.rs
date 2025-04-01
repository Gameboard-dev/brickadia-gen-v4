use std::fs::File;
use brickadia::{save::Brick, write::SaveWriter};

pub fn save_bricks(bricks: Vec<Brick>) {

    let (mut savedata, path) = super::headers::savedata();

    savedata.bricks = bricks;

    println!("Writing save to {} with {} bricks", path.to_string_lossy(), savedata.bricks.len());

    SaveWriter::new(File::create(path).unwrap(), savedata)
        .write()
        .unwrap();
}