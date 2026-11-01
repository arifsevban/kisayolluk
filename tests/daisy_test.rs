use kisayolluk::ui::radial_menu::{create_daisy_petals, PetalDefinition};

#[test]
fn test_daisy_4_petals_symmetry_and_bounds() {
    let petals_def = vec![
        PetalDefinition {
            id: "convert".to_string(),
            title: "Dönüştür".to_string(),
            subtitle: "Görsel formatı".to_string(),
            icon: "⚡".to_string(),
            enabled: true,
        },
        PetalDefinition {
            id: "action_2".to_string(),
            title: "İşlem 2".to_string(),
            subtitle: "Yakında".to_string(),
            icon: "📦".to_string(),
            enabled: false,
        },
        PetalDefinition {
            id: "action_3".to_string(),
            title: "İşlem 3".to_string(),
            subtitle: "Yakında".to_string(),
            icon: "🔒".to_string(),
            enabled: false,
        },
        PetalDefinition {
            id: "action_4".to_string(),
            title: "İşlem 4".to_string(),
            subtitle: "Yakında".to_string(),
            icon: "✂️".to_string(),
            enabled: false,
        },
    ];

    let petals = create_daisy_petals(&petals_def, 180.0, 180.0);
    assert_eq!(petals.len(), 4);

    // Verify all petals fit within the 360x360 window
    for p in &petals {
        assert!(p.pos_x >= 0.0, "pos_x ({}) should be >= 0", p.pos_x);
        assert!(p.pos_y >= 0.0, "pos_y ({}) should be >= 0", p.pos_y);
        assert!(p.pos_x + p.size <= 360.0, "pos_x + size should fit in 360");
        assert!(p.pos_y + p.size <= 360.0, "pos_y + size should fit in 360");
    }

    // Top petal should be centered horizontally at x = 180
    let top = &petals[0];
    let top_center_x = top.pos_x + top.size / 2.0;
    assert!((top_center_x - 180.0).abs() <= 1.0, "Top petal must be centered horizontally");
    assert!(top.pos_y < 180.0, "Top petal must be in the upper half");

    // Right petal should be centered vertically at y = 180
    let right = &petals[1];
    let right_center_y = right.pos_y + right.size / 2.0;
    assert!((right_center_y - 180.0).abs() <= 1.0, "Right petal must be centered vertically");
    assert!(right.pos_x > 180.0, "Right petal must be in the right half");

    // Bottom petal should be centered horizontally at x = 180
    let bottom = &petals[2];
    let bottom_center_x = bottom.pos_x + bottom.size / 2.0;
    assert!((bottom_center_x - 180.0).abs() <= 1.0, "Bottom petal must be centered horizontally");
    assert!(bottom.pos_y > 180.0, "Bottom petal must be in the lower half");

    // Left petal should be centered vertically at y = 180
    let left = &petals[3];
    let left_center_y = left.pos_y + left.size / 2.0;
    assert!((left_center_y - 180.0).abs() <= 1.0, "Left petal must be centered vertically");
    assert!(left.pos_x < 180.0, "Left petal must be in the left half");
}

#[test]
fn test_daisy_future_expandability_arbitrary_counts() {
    // Test with 5, 6, and 8 petals to prove dynamic expandability
    for count in [5, 6, 8] {
        let defs: Vec<PetalDefinition> = (0..count)
            .map(|i| PetalDefinition {
                id: format!("action_{}", i),
                title: format!("İşlem {}", i),
                subtitle: "".to_string(),
                icon: "⭐".to_string(),
                enabled: i == 0,
            })
            .collect();

        let petals = create_daisy_petals(&defs, 180.0, 180.0);
        assert_eq!(petals.len(), count);

        // Every petal must stay inside the window boundary and have positive dimensions
        for p in &petals {
            assert!(p.size > 50.0, "Petal size must be reasonable");
            assert!(p.pos_x >= 0.0 && p.pos_x + p.size <= 360.0);
            assert!(p.pos_y >= 0.0 && p.pos_y + p.size <= 360.0);
        }
    }
}

#[test]
fn test_annular_wedge_path_format() {
    let defs = vec![
        PetalDefinition {
            id: "convert".to_string(),
            title: "Dönüştür".to_string(),
            subtitle: "Görsel formatı".to_string(),
            icon: "⚡".to_string(),
            enabled: true,
        },
        PetalDefinition {
            id: "action_2".to_string(),
            title: "İşlem 2".to_string(),
            subtitle: "Yakında".to_string(),
            icon: "📦".to_string(),
            enabled: false,
        },
    ];

    let petals = create_daisy_petals(&defs, 180.0, 180.0);
    for p in &petals {
        let path = p.path_data.as_str();
        assert!(path.starts_with('M'), "Path must start with M (moveto)");
        assert!(path.ends_with('Z'), "Path must end with Z (closepath)");
        assert!(path.contains(" A "), "Path must contain elliptical arc command");
        assert!(path.contains(" Q "), "Path must contain quadratic bezier rounded corners");
    }

    assert_eq!(petals[0].arrow.as_str(), "▲");
    assert_eq!(petals[1].arrow.as_str(), "▶");
}

