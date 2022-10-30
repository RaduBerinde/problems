use std::{
    fs::{self, File},
    io::BufRead,
    io::BufReader,
};

fn main() {
    test("src/ioi2021/dungeons/tests/0-01.in");
    println!(
        "{:?}",
        solve(
            3,
            [2, 6, 9].into_iter().collect(),
            [3, 1, 2].into_iter().collect(),
            [2, 2, 3].into_iter().collect(),
            [1, 0, 1].into_iter().collect(),
            [(0, 1)].into_iter().collect()
        )
    );
    let paths = fs::read_dir("src/ioi2021/dungeons/tests").unwrap();
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
    let (n, s, l, p, w, queries) = read_input(&in_file);
    let result = solve(n, s, l, p, w, queries);
    let answer = read_answer(&str::replace(&in_file, ".in", ".out"));
    if result == answer {
        println!("{} OK!", in_file);
    } else {
        println!("{} FAILED: {:?}  expected:{:?}", in_file, result, answer);
    }
}

fn read_input(path: &str) -> (i32, Vec<i32>, Vec<i32>, Vec<i32>, Vec<i32>, Vec<(i32, i32)>) {
    let f = File::open(path).unwrap();
    let r = BufReader::new(f);
    let mut l = r.lines().skip(1);
    let l1s = l.next().unwrap().unwrap();
    let mut l1 = l1s.split_whitespace().map(|s| s.parse::<i32>().unwrap());
    let n = l1.next().unwrap();
    let q = l1.next().unwrap();

    let ls = l.next().unwrap().unwrap();
    let s = ls
        .split_whitespace()
        .map(|s| s.parse::<i32>().unwrap())
        .collect();
    let ls = l.next().unwrap().unwrap();
    let p = ls
        .split_whitespace()
        .map(|s| s.parse::<i32>().unwrap())
        .collect();
    let ls = l.next().unwrap().unwrap();
    let w = ls
        .split_whitespace()
        .map(|s| s.parse::<i32>().unwrap())
        .collect();
    let ls = l.next().unwrap().unwrap();
    let ll = ls
        .split_whitespace()
        .map(|s| s.parse::<i32>().unwrap())
        .collect();
    let mut queries = Vec::with_capacity(q as usize);
    for _ in 0..q {
        let ls = l.next().unwrap().unwrap();
        let mut l = ls.split_whitespace().map(|s| s.parse::<i32>().unwrap());
        let x = l.next().unwrap();
        let w = l.next().unwrap();
        queries.push((x, w));
    }
    (n, s, p, w, ll, queries)
}

fn read_answer(path: &str) -> Vec<i64> {
    let f = File::open(path).unwrap();
    let r = BufReader::new(f);
    r.lines()
        .skip(2)
        .map(|e| e.unwrap().parse::<i64>().unwrap())
        .collect()
}

const J_BOUND: usize = 25;
const K_BOUND: usize = 25;

#[derive(Copy, Clone, Debug)]
struct Entry {
    ending_dungeon: i32,
    // This simulation is "valid" only for starting strengths >= 2^K and <= this value.
    max_starting_strength: i64,
    gained_strength: i64,
}

fn solve(
    n: i32,
    s: Vec<i32>,
    p: Vec<i32>,
    w: Vec<i32>,
    l: Vec<i32>,
    queries: Vec<(i32, i32)>,
) -> Vec<i64> {
    let n = n as usize;

    // dp[j][i] = simulation starting at i, with 2^j steps; no strength increase, we beat opponents
    //            iff their strength is <= 2^k.
    let mut dp: Vec<Vec<Entry>> = Vec::with_capacity(J_BOUND);
    for _ in 0..J_BOUND {
        let mut row = Vec::with_capacity(n + 1);
        for _ in 0..=n {
            row.push(Entry {
                ending_dungeon: 0,
                max_starting_strength: 0,
                gained_strength: 0,
            });
        }
        dp.push(row);
    }

    let mut queries: Vec<(i32, i64)> = queries.into_iter().map(|e| (e.0, e.1 as i64)).collect();
    for k in 0..K_BOUND {
        let strength = 1i32 << k;
        for i in 0..n {
            dp[0][i] = if s[i] <= strength {
                Entry {
                    ending_dungeon: w[i],
                    max_starting_strength: (1i64 << 48),
                    gained_strength: s[i] as i64,
                }
            } else {
                Entry {
                    ending_dungeon: l[i],
                    max_starting_strength: s[i] as i64 - 1,
                    gained_strength: p[i] as i64,
                }
            }
        }
        dp[0][n] = Entry {
            ending_dungeon: n as i32,
            max_starting_strength: (1i64 << 48),
            gained_strength: 0,
        };

        for j in 1..J_BOUND {
            for i in 0..=n {
                let e1 = dp[j - 1][i];
                let e2 = dp[j - 1][e1.ending_dungeon as usize];
                dp[j][i] = Entry {
                    ending_dungeon: e2.ending_dungeon,
                    max_starting_strength: i64::min(
                        e1.max_starting_strength,
                        e2.max_starting_strength - e1.gained_strength,
                    ),
                    gained_strength: e1.gained_strength + e2.gained_strength,
                }
            }
        }

        for q in &mut queries {
            let &mut (mut x, mut z) = q;
            if x == n as i32 {
                continue;
            }
            assert!(z >= strength as i64);

            for j in (0..J_BOUND).rev() {
                if x == n as i32 || (k < K_BOUND - 1 && z >= 2 * strength as i64) {
                    break;
                }
                let e = dp[j][x as usize];
                if e.max_starting_strength >= strength as i64 && z <= e.max_starting_strength as i64
                {
                    x = e.ending_dungeon;
                    z += e.gained_strength;
                }
            }
            if x != n as i32 {
                // We have pushed the simulation until the first enemy that has s > strength but
                // which we can defeat. Make one step, which will add at least <strength> to our
                // strength.
                if s[x as usize] as i64 <= z {
                    z += s[x as usize] as i64;
                    x = w[x as usize];
                }
            }
            (*q) = (x, z);
        }
    }

    queries
        .into_iter()
        .map(|q| {
            assert_eq!(q.0, n as i32);
            q.1
        })
        .collect()
}
