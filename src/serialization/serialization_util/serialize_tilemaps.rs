use super::{tilemap::Tilemap, *};

pub fn full_path(sub_path: &str) -> String {
    format!(
        "{}/{}/tiles/{}",
        SCENE_DIRECTORY,
        CURRENT_SCENE.lock().unwrap(),
        sub_path
    )
}

pub fn sub_path(uuid: &str) -> String {
    format!("{}.tile", uuid)
}

pub fn serialize_tiles(tilemap: &Tilemap, uuid: &str) -> Result<FragmentedData<Vec<Option<usize>>>, Error> {
    let path = sub_path(uuid);
    let vec: Vec<Option<usize>> = tilemap.tiles.iter().map(|mt| mt.map(|mt| mt.into())).collect();

    save_file_bin(&vec, &full_path(&path))?;

    Ok(FragmentedData::new(path))
}

pub fn load_tiles(tile_fraged_data: &FragmentedData<Vec<Option<usize>>>) -> Result<Vec<Option<Tile>>, Error> {
    let vec: Vec<Option<usize>> = load_file_bin(&full_path(&tile_fraged_data.relative_path))?;

    Ok(vec.into_iter().map(|mu| mu.map(|mu| mu.into())).collect())
}
