// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::{NoResources, load_str};
fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    assert_eq!(args.len(), 5);
    let xml = std::fs::read_to_string(&args[1]).unwrap();
    let count = args[2].parse::<usize>().unwrap();
    let iterations = args[3].parse::<usize>().unwrap();
    assert!(count > 0 && iterations > 0);
    let last = format!("bench::f{}", count - 1);
    let mut checksum = 0.;
    let mut start = std::time::Instant::now();
    // The first validated load/state dump warms the same path outside timing.
    for index in 0..=iterations {
        if index == 1 {
            start = std::time::Instant::now();
        }
        let root = load_str(&xml, "bench.sdf", &NoResources).unwrap();
        let model = &root.models[0];
        assert_eq!(model.links.len(), count);
        assert_eq!(model.frames.len(), count);
        assert_eq!(root.attached_body(&last).unwrap(), "bench::l0");
        let pose = root.pose_of(&last, "bench").unwrap();
        assert!((pose.translation[0] - 0.01 * count as f64).abs() <= 1e-9);
        assert_eq!([pose.translation[1], pose.translation[2]], [0.; 2]);
        assert_eq!(pose.rotation, [0., 0., 0., 1.]);
        if index != 0 {
            checksum += pose.translation[0];
        } else {
            std::fs::write(&args[4], format!("{root:?}")).unwrap();
        }
    }
    println!(
        "{{\"nanoseconds\":{},\"iterations\":{},\"checksum\":{}}}",
        start.elapsed().as_nanos(),
        iterations,
        checksum
    );
}
