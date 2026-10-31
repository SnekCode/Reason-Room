//! Deterministic, dependency-free scale benchmark for the pure canvas core.
//!
//! Run with:
//! `cargo run --release -p infinite-note-canvas-canvas --example scale_benchmark`

use std::hint::black_box;
use std::time::{Duration, Instant};

use infinite_note_canvas_canvas::{
    CONCEPT_COUNT, Camera, CanvasCommand, CanvasDataset, CanvasInteraction, Point,
    RELATIONSHIP_COUNT,
};

const TRACE_SAMPLES: usize = 512;
const VIEWPORT: Point = Point::new(1_440.0, 900.0);

fn main() {
    println!("infinite-note-canvas scale benchmark v1");
    println!("trace_samples={TRACE_SAMPLES}");

    let dataset_started = Instant::now();
    let dataset = CanvasDataset::generated();
    let dataset_elapsed = dataset_started.elapsed();

    assert_eq!(dataset.card_count(), CONCEPT_COUNT);
    assert_eq!(dataset.relationship_count(), RELATIONSHIP_COUNT);

    println!(
        "invariant dataset concepts={} relationships={}",
        dataset.card_count(),
        dataset.relationship_count()
    );
    print_single_timing("dataset_generation", dataset_elapsed);

    let viewport_results = benchmark_viewport_queries(&dataset);
    print_phase("viewport_queries", &viewport_results.timings);
    println!(
        "invariant viewport_queries samples={} visible_card_total={} visible_card_checksum={}",
        TRACE_SAMPLES, viewport_results.primary_total, viewport_results.checksum
    );

    let cluster_results = benchmark_overview_clusters(&dataset);
    print_phase("overview_cluster_queries", &cluster_results.timings);
    println!(
        "invariant overview_cluster_queries samples={} cluster_total={} clustered_card_total={} cluster_checksum={}",
        TRACE_SAMPLES,
        cluster_results.primary_total,
        cluster_results.secondary_total,
        cluster_results.checksum
    );

    let zoom_results = benchmark_zooms(&dataset);
    print_phase("pointer_anchored_zooms", &zoom_results.timings);
    println!(
        "invariant pointer_anchored_zooms events={} final_scale_bits={} anchor_error_total_micropixels={}",
        TRACE_SAMPLES, zoom_results.primary_total, zoom_results.secondary_total
    );

    let pan_results = benchmark_pans(&dataset);
    print_phase("pan_commands", &pan_results.timings);
    println!(
        "invariant pan_commands events={} final_offset_bits={} pan_checksum={}",
        TRACE_SAMPLES * 3,
        pan_results.primary_total,
        pan_results.checksum
    );

    let click_results = benchmark_clicks(&dataset);
    print_phase("click_selections", &click_results.timings);
    println!(
        "invariant click_selections clicks={} selected_card_total={} selected_id_checksum={}",
        TRACE_SAMPLES, click_results.primary_total, click_results.checksum
    );

    let marquee_results = benchmark_marquees(&dataset);
    print_phase("marquee_selections", &marquee_results.timings);
    println!(
        "invariant marquee_selections marquees={} selected_card_total={} selected_id_checksum={}",
        TRACE_SAMPLES, marquee_results.primary_total, marquee_results.checksum
    );
}

struct BenchmarkResults {
    timings: Vec<Duration>,
    primary_total: u64,
    secondary_total: u64,
    checksum: u64,
}

impl BenchmarkResults {
    fn new(
        timings: Vec<Duration>,
        primary_total: u64,
        secondary_total: u64,
        checksum: u64,
    ) -> Self {
        Self {
            timings,
            primary_total,
            secondary_total,
            checksum,
        }
    }
}

fn benchmark_viewport_queries(dataset: &CanvasDataset) -> BenchmarkResults {
    let mut interaction = CanvasInteraction::new(Camera::default());
    let mut timings = Vec::with_capacity(TRACE_SAMPLES);
    let mut visible_card_total = 0_u64;
    let mut checksum = 0_u64;

    for sample in 0..TRACE_SAMPLES {
        let started = Instant::now();
        let visible_rect = interaction.camera().visible_canvas_rect(VIEWPORT);
        let cards = dataset.visible_cards(visible_rect);
        let card_count = cards.count() as u64;
        let elapsed = started.elapsed();

        visible_card_total += card_count;
        checksum = mix(checksum, card_count ^ sample as u64);
        black_box(card_count);
        timings.push(elapsed);

        move_camera_for_query_trace(&mut interaction, dataset, sample);
    }

    BenchmarkResults::new(timings, visible_card_total, 0, checksum)
}

fn benchmark_overview_clusters(dataset: &CanvasDataset) -> BenchmarkResults {
    let mut interaction = CanvasInteraction::new(Camera::new(Point::new(80.0, 80.0), 0.25));
    let mut timings = Vec::with_capacity(TRACE_SAMPLES);
    let mut cluster_total = 0_u64;
    let mut clustered_card_total = 0_u64;
    let mut checksum = 0_u64;

    for sample in 0..TRACE_SAMPLES {
        let started = Instant::now();
        let visible_rect = interaction.camera().visible_canvas_rect(VIEWPORT);
        let clusters = dataset.overview_clusters(visible_rect);
        let cluster_count = clusters.len() as u64;
        let card_count = clusters
            .iter()
            .map(|cluster| cluster.card_count as u64)
            .sum::<u64>();
        let elapsed = started.elapsed();

        cluster_total += cluster_count;
        clustered_card_total += card_count;
        checksum = mix(
            checksum,
            cluster_count ^ card_count.rotate_left(17) ^ sample as u64,
        );
        black_box((cluster_count, card_count));
        timings.push(elapsed);

        move_camera_for_query_trace(&mut interaction, dataset, sample);
    }

    BenchmarkResults::new(timings, cluster_total, clustered_card_total, checksum)
}

fn benchmark_zooms(dataset: &CanvasDataset) -> BenchmarkResults {
    let mut interaction = CanvasInteraction::new(Camera::default());
    let mut timings = Vec::with_capacity(TRACE_SAMPLES);
    let mut anchor_error_total_bits = 0_u64;

    for sample in 0..TRACE_SAMPLES {
        let anchor = trace_viewport_point(sample);
        let world_anchor = interaction.camera().screen_to_world(anchor);
        let scale_factor = if sample.is_multiple_of(2) {
            1.2
        } else {
            1.0 / 1.2
        };
        let started = Instant::now();
        interaction.apply(
            dataset,
            CanvasCommand::ZoomAt {
                viewport_position: anchor,
                scale_factor,
            },
        );
        let anchored_screen_point = interaction.camera().world_to_screen(world_anchor);
        let anchor_error =
            (anchored_screen_point.x - anchor.x).abs() + (anchored_screen_point.y - anchor.y).abs();
        let elapsed = started.elapsed();

        anchor_error_total_bits += (anchor_error * 1_000_000.0).round() as u64;
        black_box(anchor_error);
        timings.push(elapsed);
    }

    BenchmarkResults::new(
        timings,
        u64::from(interaction.camera().scale.to_bits()),
        anchor_error_total_bits,
        0,
    )
}

fn benchmark_pans(dataset: &CanvasDataset) -> BenchmarkResults {
    let mut interaction = CanvasInteraction::new(Camera::default());
    let mut timings = Vec::with_capacity(TRACE_SAMPLES);
    let mut checksum = 0_u64;

    for sample in 0..TRACE_SAMPLES {
        let start = trace_viewport_point(sample);
        let end = Point::new(
            start.x + (sample % 13) as f32 - 6.0,
            start.y + (sample % 17) as f32 - 8.0,
        );
        let started = Instant::now();
        interaction.apply(
            dataset,
            CanvasCommand::BeginPan {
                viewport_position: start,
            },
        );
        interaction.apply(
            dataset,
            CanvasCommand::ContinuePan {
                viewport_position: end,
            },
        );
        interaction.apply(dataset, CanvasCommand::EndPan);
        let offset = interaction.camera().offset;
        let elapsed = started.elapsed();

        checksum = mix(
            checksum,
            u64::from(offset.x.to_bits()) ^ u64::from(offset.y.to_bits()).rotate_left(23),
        );
        black_box(offset);
        timings.push(elapsed);
    }

    let final_offset = interaction.camera().offset;
    BenchmarkResults::new(
        timings,
        u64::from(final_offset.x.to_bits()) ^ u64::from(final_offset.y.to_bits()).rotate_left(32),
        0,
        checksum,
    )
}

fn benchmark_clicks(dataset: &CanvasDataset) -> BenchmarkResults {
    let mut interaction = CanvasInteraction::new(Camera::default());
    let mut timings = Vec::with_capacity(TRACE_SAMPLES);
    let mut selected_card_total = 0_u64;
    let mut selected_id_checksum = 0_u64;

    for sample in 0..TRACE_SAMPLES {
        let position = card_center_in_viewport(sample * 37 % CONCEPT_COUNT);
        let started = Instant::now();
        interaction.apply(
            dataset,
            CanvasCommand::BeginSelection {
                viewport_position: position,
            },
        );
        interaction.apply(
            dataset,
            CanvasCommand::EndSelection {
                viewport_position: position,
                extend_selection: false,
            },
        );
        let selected_ids: Vec<_> = interaction.selected_card_ids().collect();
        let elapsed = started.elapsed();

        selected_card_total += selected_ids.len() as u64;
        selected_id_checksum = mix(
            selected_id_checksum,
            selected_ids.iter().copied().map(u64::from).sum(),
        );
        black_box(&selected_ids);
        timings.push(elapsed);
    }

    BenchmarkResults::new(timings, selected_card_total, 0, selected_id_checksum)
}

fn benchmark_marquees(dataset: &CanvasDataset) -> BenchmarkResults {
    let mut interaction = CanvasInteraction::new(Camera::default());
    let mut timings = Vec::with_capacity(TRACE_SAMPLES);
    let mut selected_card_total = 0_u64;
    let mut selected_id_checksum = 0_u64;

    for sample in 0..TRACE_SAMPLES {
        let (start, end) = marquee_viewport_points(sample);
        let started = Instant::now();
        interaction.apply(
            dataset,
            CanvasCommand::BeginSelection {
                viewport_position: start,
            },
        );
        interaction.apply(
            dataset,
            CanvasCommand::ContinueSelection {
                viewport_position: end,
            },
        );
        interaction.apply(
            dataset,
            CanvasCommand::EndSelection {
                viewport_position: end,
                extend_selection: false,
            },
        );
        let selected_ids: Vec<_> = interaction.selected_card_ids().collect();
        let elapsed = started.elapsed();

        selected_card_total += selected_ids.len() as u64;
        selected_id_checksum = mix(
            selected_id_checksum,
            selected_ids.iter().copied().map(u64::from).sum(),
        );
        black_box(&selected_ids);
        timings.push(elapsed);
    }

    BenchmarkResults::new(timings, selected_card_total, 0, selected_id_checksum)
}

fn move_camera_for_query_trace(
    interaction: &mut CanvasInteraction,
    dataset: &CanvasDataset,
    sample: usize,
) {
    let start = Point::new(0.0, 0.0);
    let end = Point::new(
        -((sample * 53 % 2_400) as f32),
        -((sample * 31 % 1_200) as f32),
    );
    interaction.apply(
        dataset,
        CanvasCommand::BeginPan {
            viewport_position: start,
        },
    );
    interaction.apply(
        dataset,
        CanvasCommand::ContinuePan {
            viewport_position: end,
        },
    );
    interaction.apply(dataset, CanvasCommand::EndPan);
}

fn card_center_in_viewport(card_index: usize) -> Point {
    let column = card_index % 100;
    let row = card_index / 100;
    Camera::default().world_to_screen(Point::new(
        column as f32 * 220.0 + 90.0,
        row as f32 * 140.0 + 52.0,
    ))
}

fn marquee_viewport_points(sample: usize) -> (Point, Point) {
    let column = sample * 7 % 97;
    let row = sample * 11 % 97;
    let camera = Camera::default();
    let first = camera.world_to_screen(Point::new(
        column as f32 * 220.0 + 10.0,
        row as f32 * 140.0 + 10.0,
    ));
    let second = camera.world_to_screen(Point::new(
        (column + 2) as f32 * 220.0 + 170.0,
        (row + 2) as f32 * 140.0 + 94.0,
    ));
    (first, second)
}

fn trace_viewport_point(sample: usize) -> Point {
    Point::new(
        32.0 + (sample * 53 % 1_376) as f32,
        24.0 + (sample * 29 % 852) as f32,
    )
}

fn mix(previous: u64, value: u64) -> u64 {
    previous.rotate_left(7) ^ value.wrapping_mul(0x9e37_79b9_7f4a_7c15)
}

fn print_single_timing(name: &str, duration: Duration) {
    println!(
        "timing phase={name} samples=1 total_ms={:.3} p50_ms={:.3} p95_ms={:.3} p99_ms={:.3}",
        milliseconds(duration),
        milliseconds(duration),
        milliseconds(duration),
        milliseconds(duration),
    );
}

fn print_phase(name: &str, timings: &[Duration]) {
    let mut sorted = timings.to_vec();
    sorted.sort_unstable();
    let total = timings.iter().copied().sum::<Duration>();

    println!(
        "timing phase={name} samples={} total_ms={:.3} p50_ms={:.3} p95_ms={:.3} p99_ms={:.3}",
        timings.len(),
        milliseconds(total),
        milliseconds(percentile(&sorted, 50)),
        milliseconds(percentile(&sorted, 95)),
        milliseconds(percentile(&sorted, 99)),
    );
}

fn percentile(sorted: &[Duration], percentile: usize) -> Duration {
    assert!(!sorted.is_empty());
    assert!(percentile <= 100);

    sorted[(sorted.len() - 1) * percentile / 100]
}

fn milliseconds(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1_000.0
}
