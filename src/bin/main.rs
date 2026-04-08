use std::{
    error::Error,
    io::{self, Write},
};

use particles::{
    bitmap::Bitmap,
    env::resolution,
    glitter_scene::{
        animated_view, GlitterScene, GlitterSceneSettings, DEFAULT_DURATION, DEFAULT_FPS,
    },
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut output = io::stdout().lock();
    let resolution = resolution()?;
    let scene = GlitterScene::new();
    let settings = GlitterSceneSettings::for_resolution(&resolution);
    let frame_count = (DEFAULT_DURATION * DEFAULT_FPS) as usize;
    let mut bitmap = Bitmap::new(resolution);

    for frame in 0..frame_count {
        let time = frame as f32 / DEFAULT_FPS;
        scene.render(&mut bitmap, time, settings, animated_view(time));
        output.write_all(bitmap.data())?;
        output.flush()?;
    }

    Ok(())
}
