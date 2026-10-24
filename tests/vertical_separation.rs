use slide_captcha_rs::{
    Builder, Captcha, RangeVal, Size, graph_vertical_y, with_gen_graph_number,
    with_graph_vertical_separation, with_image_size, with_range_graph_size,
};

fn intervals_overlap(a: i32, b: i32, h: i32) -> bool {
    a < b + h && b < a + h
}

#[test]
fn graph_vertical_y_uses_upper_band_when_only_upper_fits() {
    let mut rng = rand::rng();
    let target_y = 80;
    let graph_h = 50;
    let min_y = 0;
    let max_y = 100;
    for _ in 0..50 {
        let y = graph_vertical_y(&mut rng, target_y, graph_h, min_y, max_y);
        assert!(
            (min_y..=target_y - graph_h).contains(&y),
            "y={y} not in upper band"
        );
        assert!(!intervals_overlap(y, target_y, graph_h));
    }
}

#[test]
fn graph_vertical_y_uses_lower_band_when_only_lower_fits() {
    let mut rng = rand::rng();
    let target_y = 20;
    let graph_h = 50;
    let min_y = 0;
    let max_y = 100;
    for _ in 0..50 {
        let y = graph_vertical_y(&mut rng, target_y, graph_h, min_y, max_y);
        assert!(
            (target_y + graph_h..=max_y).contains(&y),
            "y={y} not in lower band"
        );
        assert!(!intervals_overlap(y, target_y, graph_h));
    }
}

#[test]
fn graph_vertical_y_picks_either_band_when_both_fit() {
    let mut rng = rand::rng();
    let target_y = 50;
    let graph_h = 40;
    let min_y = 0;
    let max_y = 120;
    let upper = min_y..=target_y - graph_h;
    let lower = target_y + graph_h..=max_y;
    let mut saw_upper = false;
    let mut saw_lower = false;
    for _ in 0..200 {
        let y = graph_vertical_y(&mut rng, target_y, graph_h, min_y, max_y);
        let in_upper = upper.contains(&y);
        let in_lower = lower.contains(&y);
        assert!(in_upper || in_lower, "y={y} not in either band");
        assert!(!intervals_overlap(y, target_y, graph_h));
        saw_upper |= in_upper;
        saw_lower |= in_lower;
    }
    assert!(saw_upper, "never picked the upper band");
    assert!(saw_lower, "never picked the lower band");
}

#[test]
fn graph_vertical_y_returns_target_when_neither_band_fits() {
    let mut rng = rand::rng();
    let target_y = 30;
    let graph_h = 80;
    let min_y = 0;
    let max_y = 100;
    for _ in 0..20 {
        let y = graph_vertical_y(&mut rng, target_y, graph_h, min_y, max_y);
        assert_eq!(y, target_y);
    }
}

fn two_graph_captcha(separation: bool, drag_drop: bool) -> Captcha {
    let builder = Builder::new().set_options([
        with_image_size(Size {
            width: 300,
            height: 220,
        }),
        with_range_graph_size(RangeVal::new(60, 60)),
        with_gen_graph_number(2),
        with_graph_vertical_separation(separation),
    ]);
    if drag_drop {
        builder.make_drag_drop()
    } else {
        builder.make()
    }
}

#[test]
fn default_off_keeps_shared_y_in_basic_and_drag_drop() {
    for drag_drop in [false, true] {
        let captcha = two_graph_captcha(false, drag_drop);
        for _ in 0..20 {
            let blocks = captcha.generate_graph_blocks().expect("blocks");
            assert_eq!(blocks.len(), 2);
            assert_eq!(
                blocks[0].y, blocks[1].y,
                "default off must keep a shared Y (drag_drop={drag_drop})"
            );
        }
    }
}

#[test]
fn vertical_separation_keeps_extra_off_target_row() {
    for drag_drop in [false, true] {
        let captcha = two_graph_captcha(true, drag_drop);
        for _ in 0..40 {
            let blocks = captcha.generate_graph_blocks().expect("blocks");
            assert_eq!(blocks.len(), 2);
            let h = i32::try_from(blocks[0].height).expect("h");
            assert_eq!(blocks[0].height, blocks[1].height);
            assert!(
                !intervals_overlap(blocks[0].y, blocks[1].y, h),
                "graphs overlap: y0={} y1={} h={h} drag_drop={drag_drop}",
                blocks[0].y,
                blocks[1].y
            );
        }
    }
}
