use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rustweb_core::{Attr, VNode};
use rustweb_dom::{diff, renderer::apply_patches_to_tree};
use rustweb_ssr::render_to_string;

fn keyed_list(n: usize) -> VNode {
    let children = (0..n)
        .map(|i| {
            let mut e = rustweb_core::Element::new("li", vec![VNode::text(format!("row {i}"))]);
            e.key = Some(i.to_string());
            e.attrs.push(Attr::new("class", "row"));
            VNode::Element(e)
        })
        .collect();
    VNode::element("ul", vec![Attr::new("aria-label", "rows")], children)
}

fn rotated(list: &VNode, by: usize) -> VNode {
    match list {
        VNode::Element(el) => {
            let mut kids = el.children.clone();
            kids.rotate_right(by);
            VNode::element(el.tag.clone(), el.attrs.clone(), kids)
        }
        other => other.clone(),
    }
}

fn bench_diff(c: &mut Criterion) {
    let mut g = c.benchmark_group("diff");
    for n in [10usize, 100, 1000, 5000] {
        let old = keyed_list(n);
        let new = rotated(&old, n / 2);
        g.bench_function(BenchmarkId::new("reorder", n), |b| {
            b.iter(|| diff(std::hint::black_box(&old), std::hint::black_box(&new)))
        });
    }
    g.finish();
}

fn bench_apply(c: &mut Criterion) {
    let mut g = c.benchmark_group("apply");
    for n in [10usize, 100, 1000, 5000] {
        let old = keyed_list(n);
        let new = rotated(&old, n / 2);
        let patches = diff(&old, &new);
        g.bench_function(BenchmarkId::new("reorder", n), |b| {
            b.iter(|| {
                apply_patches_to_tree(
                    std::hint::black_box(old.clone()),
                    std::hint::black_box(&patches),
                )
            })
        });
    }
    g.finish();
}

fn bench_ssr(c: &mut Criterion) {
    let mut g = c.benchmark_group("ssr");
    for n in [10usize, 100, 1000, 5000] {
        let tree = keyed_list(n);
        g.bench_function(BenchmarkId::new("render", n), |b| {
            b.iter(|| render_to_string(std::hint::black_box(&tree)))
        });
    }
    g.finish();
}

fn bench_hot_small(c: &mut Criterion) {
    let old = keyed_list(8);
    let mut new = old.clone();
    if let VNode::Element(el) = &mut new {
        if let Some(VNode::Element(li)) = el.children.first_mut() {
            li.attrs.push(Attr::new("aria-selected", "true"));
        }
    }
    let mut g = c.benchmark_group("hot_small");
    g.bench_function("diff_apply_8", |b| {
        b.iter(|| {
            let p = diff(std::hint::black_box(&old), std::hint::black_box(&new));
            apply_patches_to_tree(old.clone(), &p).unwrap()
        })
    });
    g.finish();
}

criterion_group!(benches, bench_diff, bench_apply, bench_ssr, bench_hot_small);
criterion_main!(benches);
