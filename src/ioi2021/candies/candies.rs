use std::{
    fs::{self, File},
    io::BufRead,
    io::BufReader,
};

fn main() {
    //test("src/ioi2021/candies/tests/1-02.in");
    println!(
        "{:?}",
        solve(
            [10, 15, 13].into_iter().collect(),
            [(0, 2, 20), (0, 1, -11)].into_iter().collect(),
        ),
    );
    let paths = fs::read_dir("src/ioi2021/candies/tests").unwrap();
    let mut paths: Vec<String> = paths
        .map(|p| p.unwrap().path().to_string_lossy().to_string())
        .collect();
    paths.sort();
    for p in paths {
        if !str::ends_with(&p, ".in") {
            continue;
        }
        test(&p);
    }
}

fn test(in_file: &str) {
    let (c, days) = read_input(&in_file);
    let result = solve(c, days);
    let answer = read_answer(&str::replace(&in_file, ".in", ".out"));
    if result == answer {
        println!("{} OK!", in_file);
    } else {
        println!("{} FAIL!", in_file);
        //println!("{} FAILED: {:?}  expected:{:?}", in_file, result, answer);
    }
}

fn read_input(path: &str) -> (Vec<i32>, Vec<(i32, i32, i32)>) {
    let f = File::open(path).unwrap();
    let r = BufReader::new(f);

    let mut l = r.lines().map(|ll| ll.unwrap()).skip(1);

    let n = l.next().unwrap().parse::<i32>().unwrap();

    let c: Vec<i32> = l
        .next()
        .unwrap()
        .split_whitespace()
        .map(|s| s.parse::<i32>().unwrap())
        .collect();
    assert_eq!(n as usize, c.len());

    let q = l.next().unwrap().parse::<i32>().unwrap();

    let mut days = Vec::new();
    for _ in 0..q {
        let vals: Vec<i32> = l
            .next()
            .unwrap()
            .split_whitespace()
            .map(|s| s.parse::<i32>().unwrap())
            .collect();
        days.push((vals[0], vals[1], vals[2]));
    }
    (c, days)
}

fn read_answer(path: &str) -> Vec<i32> {
    let f = File::open(path).unwrap();
    let r = BufReader::new(f);
    r.lines()
        .skip(2)
        .next()
        .unwrap()
        .unwrap()
        .split_whitespace()
        .map(|e| e.parse::<i32>().unwrap())
        .collect()
}

fn solve(c: Vec<i32>, days: Vec<(i32, i32, i32)>) -> Vec<i32> {
    let n = c.len();

    let mut events: Vec<(i32, i32, i32)> = Vec::with_capacity(2 * n);
    for (day, &(l, r, v)) in days.iter().enumerate() {
        events.push((l, day as i32, v));
        events.push((r + 1, day as i32, -v));
    }
    events.sort();

    let q = days.len() as i32;
    let mut t = SegTree::new(q);
    let mut ev_idx = 0;
    let mut result: Vec<i32> = Vec::new();
    let mut last_value = 0;
    for (i, &c) in c.iter().enumerate() {
        while ev_idx < events.len() && events[ev_idx].0 == i as i32 {
            let (_, day, v) = events[ev_idx];
            t.add(day + 1, q, v as i64);
            last_value += v as i64;
            ev_idx += 1;
        }

        result.push(if t.nodes[1].max - t.nodes[1].min <= c as i64 {
            // No aliasing.
            (last_value - t.nodes[1].min) as i32
        } else {
            let (x_val, min, max) = t.find_point(c as i64);
            assert!(x_val == min || x_val == max);
            // Two cases: the point is either the minimum or the maximum.
            if x_val == min {
                // No matter how many candies we have in day x, we will reach max capacity at the
                // max point, after which we never go below 0.
                (last_value - max + c as i64) as i32
            } else {
                // No matter hwo many candies we have in day x, we will reach 0 capacity at the
                // min point, after which we never go above the capacity.
                (last_value - min) as i32
            }
        });
    }

    result
}

struct Node {
    min: i64,
    max: i64,
    delta: i64, // Applies to entire subtree
}

struct SegTree {
    n: i32,
    nodes: Vec<Node>,
}

const INF: i64 = 1 << 48;

impl SegTree {
    fn new(n: i32) -> Self {
        Self {
            n,
            nodes: (0..(2 * (n + 1) as usize).next_power_of_two())
                .map(|_| Node {
                    min: 0,
                    max: 0,
                    delta: 0,
                })
                .collect(),
        }
    }

    fn add(&mut self, l: i32, r: i32, val: i64) {
        self.add_internal(1, 0, self.n, l, r, val);
    }

    fn add_internal(
        &mut self,
        node_idx: usize,
        n_left: i32,
        n_right: i32,
        l: i32,
        r: i32,
        delta: i64,
    ) {
        if r < n_left || n_right < l {
            return;
        }
        if l <= n_left && n_right <= r {
            let node = &mut self.nodes[node_idx];
            node.delta += delta;
            node.min += delta;
            node.max += delta;
            return;
        }
        let n_mid = (n_left + n_right) / 2;
        self.add_internal(node_idx * 2, n_left, n_mid, l, r, delta);
        self.add_internal(node_idx * 2 + 1, n_mid + 1, n_right, l, r, delta);

        let min = i64::min(
            self.nodes[node_idx * 2].min,
            self.nodes[node_idx * 2 + 1].min,
        );
        let max = i64::max(
            self.nodes[node_idx * 2].max,
            self.nodes[node_idx * 2 + 1].max,
        );

        let node = &mut self.nodes[node_idx];
        node.min = node.delta + min;
        node.max = node.delta + max;
    }

    // find_point returns the right-most x such that the delta between the maximum and minimum of
    // all values to the right of and including x is at least d.
    // Returns (x_val, min, max)
    fn find_point(&self, d: i64) -> (i64, i64, i64) {
        let mut node_idx = 1;
        let (mut l, mut r) = (0, self.n);

        let mut delta = 0;
        let mut min = INF;
        let mut max = -INF;
        while l < r {
            let n = &self.nodes[node_idx];
            delta += n.delta;

            let m = (l + r) / 2;
            // See if the result is to the right.
            let mid_min = min.min(delta + self.nodes[node_idx * 2 + 1].min);
            let mid_max = max.max(delta + self.nodes[node_idx * 2 + 1].max);
            if mid_max - mid_min >= d {
                node_idx = node_idx * 2 + 1;
                l = m + 1;
            } else {
                node_idx = node_idx * 2;
                r = m;
                min = mid_min;
                max = mid_max;
            }
        }
        let n = &self.nodes[node_idx];
        (
            delta + n.delta,
            min.min(n.min + delta),
            max.max(n.max + delta),
        )
    }
}
