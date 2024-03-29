pub fn load_textures(use_gif: &bool) -> Vec<u8> {
    let skin_full: Vec<u8> = include_bytes!("../../resources/skin_full.png").to_vec();
    let skin_gif: Vec<u8> = include_bytes!("../../resources/skin_20.png").to_vec();

    if *use_gif {
        skin_gif
    } else {
        skin_full
    }
}
