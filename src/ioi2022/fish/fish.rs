use std::{
    fs::{self, File},
    io::BufRead,
    io::BufReader,
};

fn main() {
    println!(
        "{}",
        //solve(5, vec![(0, 2, 5), (1, 1, 2), (4, 4, 1), (3, 3, 3)])
        //solve(
        //    6,
        //    vec![
        //        (0, 0, 10),
        //        (1, 2, 20),
        //        (2, 2, 30),
        //        (3, 1, 50),
        //        (4, 2, 70),
        //        (4, 4, 20),
        //        (5, 3, 40)
        //    ]
        //),
        solve(5, vec![(0, 0, 10), (2, 0, 10), (4, 0, 10),]),
    );
    //return;
    //test("src/ioi2022/fish/tests/bandsonedim-01.in"); return;
    let paths = fs::read_dir("src/ioi2022/fish/tests").unwrap();
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
    let (n, fish) = read_input(&in_file);
    let result = solve(n, fish);
    let answer = read_answer(&str::replace(&in_file, ".in", ".out"));
    if result == answer {
        println!("{} OK!", in_file);
    } else {
        println!("{} FAILED: {}  expected:{}", in_file, result, answer);
    }
}

fn read_input(path: &str) -> (i64, Vec<(i64, i64, i64)>) {
    let f = File::open(path).unwrap();
    let r = BufReader::new(f);
    let mut l = r.lines().skip(1);
    let l1s = l.next().unwrap().unwrap();
    let mut l1 = l1s.split_whitespace().map(|s| s.parse::<i64>().unwrap());
    let n = l1.next().unwrap();
    let m = l1.next().unwrap();
    let mut v = Vec::with_capacity(m as usize);
    for _ in 0..m {
        let ls = l.next().unwrap().unwrap();
        let mut l = ls.split_whitespace().map(|s| s.parse::<i64>().unwrap());
        let x = l.next().unwrap();
        let y = l.next().unwrap();
        let w = l.next().unwrap();
        v.push((x, y, w));
    }
    (n, v)
}

fn read_answer(path: &str) -> i64 {
    let f = File::open(path).unwrap();
    let r = BufReader::new(f);
    let mut l = r.lines().skip(2);
    l.next().unwrap().unwrap().parse::<i64>().unwrap()
}

#[derive(Copy, Clone)]
struct Position {
    y: i64,

    // sum of fish weight on this column, up to (and including) this position.
    weight_sum: i64,

    // Sum of captured weights (up to and including) this column for the best solution that ends in a pier that ends in this position,
    // and pier length is either increasing or decreasing.
    dp_up: i64,
    dp_down: i64,
}

fn solve(n: i64, fish: Vec<(i64, i64, i64)>) -> i64 {
    let mut p = vec![
        vec![Position {
            y: 0,
            weight_sum: 0,
            dp_up: 0,
            dp_down: 0
        }];
        (n + 2) as usize
    ];

    let mut add = |x: i64, y: i64, weight: i64| {
        let last_pos = p[x as usize].last_mut().unwrap();
        let weight_sum = last_pos.weight_sum + weight;
        if last_pos.y == y {
            last_pos.weight_sum = weight_sum;
            return;
        }
        p[x as usize].push(Position {
            y,
            weight_sum: weight_sum,
            dp_up: 0,
            dp_down: 0,
        });
    };

    let mut fish = fish;
    fish.sort_by_key(|&f| f.1);

    for f in fish {
        let (x, y, w) = f;
        if x > 0 {
            add(x, y + 1, 0);
        }
        add(x + 1, y + 1, w);
        if x + 2 <= n {
            add(x + 2, y + 1, 0);
        }
    }

    for x in 1..(n as usize + 2) {
        // Calculate dp_up.
        let mut i = 0;
        let mut j = 0;

        // Initial best corresponds to starting a new sequence (with no piers in the previous two
        // columns).
        let mut best = p[x - 1][0].dp_up;
        let mut best_weight_sum = 0;
        while j < p[x].len() {
            let y = p[x][j].y;

            while i + 1 < p[x - 1].len() && p[x - 1][i + 1].y <= y {
                i += 1;
                let pos = p[x - 1][i];
                // Potential new best; it must be better even while losing some fish in the previous column.
                if best + pos.weight_sum - best_weight_sum < pos.dp_up {
                    best = pos.dp_up;
                    best_weight_sum = pos.weight_sum;
                }
            }
            // p[x-1][i].weight_sum is the number of fish in the column up to y.
            // The max here accounts for the case where this is the start of a sequence and the
            // previous sequence ends in a pier of height at least y.
            p[x][j].dp_up =
                (best + p[x - 1][i].weight_sum - best_weight_sum).max(p[x - 1][0].dp_down);
            j += 1;
        }
        // Calculate dp_down.
        let mut i = p[x - 1].len();
        let mut j = p[x].len();
        let mut best = 0;
        let mut best_weight_sum = 0;
        while j > 0 {
            j -= 1;
            let y = p[x][j].y;

            while i > 0 && p[x - 1][i - 1].y >= y {
                i -= 1;
                let pos = p[x - 1][i];
                // Potential new best; it must be better even while losing some fish in this
                // column. The use of p[x][j] is a bit counter-intuitive - iti is the weight of
                // this column up to y; we really want the weight up to p[x-1][i].y. But we know
                // that p[x] has a position entry for every fish in this column, so there can't be
                // any fish in-between y and p[x-1][i].y.
                if best + best_weight_sum - p[x][j].weight_sum < pos.dp_up.max(pos.dp_down) {
                    best = pos.dp_up.max(pos.dp_down);
                    best_weight_sum = p[x][j].weight_sum;
                }
            }
            p[x][j].dp_down = (best + best_weight_sum - p[x][j].weight_sum).max(0);
        }
    }

    p[n as usize + 1][0].dp_down
}
