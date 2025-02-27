use codspeed_criterion_compat::{Criterion, black_box, criterion_group, criterion_main};
use common::{
    card_data::Id,
    deck::{Deck, PartType},
};
use criterion::Throughput;

const STEPS: u64 = 10_000;

pub fn deck(c: &mut Criterion) {
    let mut group = c.benchmark_group("deck");
    group.throughput(Throughput::Elements(STEPS));

    group.bench_function("modify and iterate", |b| {
        b.iter(|| {
            fastrand::seed(0);

            let mut deck = Deck::default();

            for _ in 0..STEPS {
                // Random part type, biased towards playing part
                let part_type = match fastrand::u8(0..6) {
                    0 => PartType::Side,
                    _ => PartType::Playing,
                };

                // Limit the size of the set of ids to a reasonable number
                let id = Id::new(fastrand::u16(0..80));

                // Chose between add and subtract, biased to increment
                if fastrand::u8(0..32) > 15 {
                    deck.increment(id, part_type, 1);
                } else {
                    deck.decrement(id, part_type, 1);
                }

                deck.entries().for_each(|entry| {
                    black_box(entry);
                });
            }

            black_box(deck);
        });
    });
}

criterion_group!(benches, deck);
criterion_main!(benches);
