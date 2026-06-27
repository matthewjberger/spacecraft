use crate::content::Sector;
use nightshade::prelude::*;

fn split_speaker(line: &str) -> (Option<&str>, &str) {
    if let Some((speaker, text)) = line.split_once(": ") {
        (Some(speaker), text)
    } else {
        (None, line)
    }
}

pub fn sector_cutscene(sector: &Sector, ship: Vec3) -> Cutscene {
    let focus = ship + Vec3::new(0.0, 0.2, 0.0);
    let shot_a =
        CutsceneShot::new(ship + Vec3::new(-6.5, 2.6, 7.5), focus).with_field_of_view(50.0);
    let shot_b = CutsceneShot::new(ship + Vec3::new(5.5, 1.4, 6.0), focus).with_field_of_view(46.0);
    let shot_c = CutsceneShot::new(
        ship + Vec3::new(0.6, 1.0, 5.0),
        focus + Vec3::new(0.0, 0.5, -7.0),
    )
    .with_field_of_view(56.0);

    let mut scene = Cutscene::new(sector.name).letterbox_in(0.0, 0.6).title(
        0.4,
        2.8,
        format!("{}  —  {}", sector.name, sector.subtitle),
    );

    let mut time = 2.2;
    for line in sector.briefing.split('\n') {
        let (speaker, text) = split_speaker(line);
        let duration = 2.0 + text.len() as f32 * 0.045;
        scene = scene.dialogue(time, duration, speaker, text);
        time += duration + 0.3;
    }

    let half = (time * 0.5).max(0.5);
    scene
        .camera(0.0, half, EasingFunction::SineInOut, shot_a, shot_b)
        .camera(
            half,
            (time - half).max(0.5),
            EasingFunction::SineInOut,
            shot_b,
            shot_c,
        )
        .handheld(0.0, time, 0.05, 0.035, 1.2)
        .letterbox_out(time, 0.6)
}

pub fn finale_cutscene(ship: Vec3) -> Cutscene {
    let focus = ship + Vec3::new(0.0, 0.2, 0.0);
    let shot_a = CutsceneShot::new(ship + Vec3::new(0.0, 1.0, 6.5), focus).with_field_of_view(48.0);
    let shot_b =
        CutsceneShot::new(ship + Vec3::new(-4.5, 3.2, 10.0), focus).with_field_of_view(38.0);

    Cutscene::new("Finale")
        .fade_in(0.0, 1.4)
        .letterbox_in(0.0, 0.8)
        .title(0.6, 3.0, "THE LOOP IS BROKEN")
        .camera(0.0, 9.2, EasingFunction::SineInOut, shot_a, shot_b)
        .handheld(0.0, 9.2, 0.04, 0.03, 1.0)
        .dialogue(
            2.0,
            3.2,
            Some("WREN"),
            "The Monarch's dark. The swarm's forgotten how to move.",
        )
        .dialogue(
            5.4,
            3.2,
            Some("TALON"),
            "The fleet's running the gap. Tesse can rest now, Ranger.",
        )
        .letterbox_out(8.8, 0.8)
}
