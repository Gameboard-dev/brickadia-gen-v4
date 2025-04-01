
use super::assets::BrickAssets;
use brickadia::save::{BrickOwner, SaveData, User};
use std::{env, path::PathBuf};

pub fn location() -> PathBuf {
    env::var("LOCALAPPDATA")
        .map(PathBuf::from)
        .map(|p| p.join("Brickadia").join("Saved_Staging").join("Builds"))
        .unwrap_or_else(|e| {
            eprintln!("Warning: Failed to retrieve LOCALAPPDATA: {}", e);
            PathBuf::new()
        })
}

pub fn savedata() -> (SaveData, PathBuf) {

    let mut save = SaveData::default();

    let public = User {
        name: "BrickadiaGen".into(),
        id: "3f5108a0-c929-4e77-a115-21f65096887b".parse().unwrap(),
    };

    let name = if cfg!(test) {
        std::env::var("NAME").unwrap_or_else(|_| "NAME".into())
    } else {
        std::env::args().nth(1).unwrap_or_else(|| "NAME".into())
    };
    
    let name = format!("{}.brs", name);

    let path = location().join(name);

    save.header1.author = public.clone();
    save.header1.host = Some(public.clone());
    save.header1.description = "This was saved with BrickadiaGen!".into();
    
    save.header2.brick_assets = BrickAssets::names();
    save.header2.materials.push("BMC_Glass".into());

    save.header2.brick_owners.push(BrickOwner::from_user_bricks(public.clone(), 100));

    (save, path)
}
