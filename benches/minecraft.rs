use criterion::{black_box, criterion_group, criterion_main, Criterion, Bencher};
use measure_time::print_time;
use pinin_rs::pinin::PinIn;
use pinin_rs::searcher::{Searcher, SearcherLogic, TreeSearcher};

const SMALL: &str = include_str!("small");
const LARGE: &str = include_str!("small");

fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fibonacci(n-1) + fibonacci(n-2),
    }
}

fn load_file(s: &str) -> Vec<&str> {
    s.lines().collect()
}

fn small_build(context: &PinIn, searcher: &mut dyn Searcher<usize>) {
    LARGE.lines().enumerate().for_each(|(i, s)| {
        searcher.insert(context, s, i);
    });
}

fn criterion_benchmark(c: &mut Criterion) {

    let mut pinin = PinIn::new();
    {
        let time = std::time::Instant::now();
        pinin.load_default_dict();
        println!("load dict took {}ms", (std::time::Instant::now() - time).as_millis());
    }
    {
        let time = std::time::Instant::now();
        let mut searcher = TreeSearcher::new(SearcherLogic::Begin, pinin.accelerator.clone().unwrap());
        small_build(&pinin, &mut searcher);

        black_box(searcher);

        println!("build small dict took {}ms", (std::time::Instant::now() - time).as_millis());
    }


    c.bench_function("TreeSearcher build small", |b: &mut Bencher| {
        let mut pinin = PinIn::new();
        pinin.load_default_dict();
        b.iter(|| {
            let mut searcher = TreeSearcher::new(SearcherLogic::Begin, pinin.accelerator.clone().unwrap());
            small_build(&pinin, &mut searcher);
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);