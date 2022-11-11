use std::{
    fs::{File, OpenOptions},
    io::BufRead,
    io::{BufReader, Write},
    ops::{Index, IndexMut},
    process::{Command, Stdio},
};

fn main() {
    //test("src/ioi2020/mushrooms/tests/1-02.in"); return;
    let paths = std::fs::read_dir("src/ioi2020/mushrooms/tests").unwrap();
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

fn tmp_filename() -> String {
    let output = Command::new("mktemp")
        .arg("-u")
        .output()
        .expect("failed to execute mktemp");
    assert!(output.status.success());
    String::from(String::from_utf8(output.stdout).unwrap().trim())
}

fn mkfifo(file: &str) {
    assert!(Command::new("mkfifo")
        .arg(file)
        .status()
        .expect("failed to execute mkfifo")
        .success());
}

fn test(in_file: &str) {
    println!("\n{}", in_file);

    let to_manager = tmp_filename();
    mkfifo(&to_manager);
    let from_manager = tmp_filename();
    mkfifo(&from_manager);

    // Spawn the manager process.
    let mut child = Command::new("src/ioi2020/mushrooms/grader/manager")
        .arg(&to_manager)
        .arg(&from_manager)
        .stdin(Stdio::piped())
        .spawn()
        .expect("could not spawn manager");

    let input = std::fs::read_to_string(in_file).expect("could not read input file");

    let mut manager_in = child.stdin.take().unwrap();
    manager_in
        .write(input.as_bytes())
        .expect("could not write input file to manager");
    drop(manager_in);

    // We must first open the pipe from the manager, to match the opening order in the manager;
    // otherwise we deadlock.
    let mut instance = Instance {
        from_manager: BufReader::new(File::open(&from_manager).unwrap()).lines(),
        to_manager: OpenOptions::new().append(true).open(&to_manager).unwrap(),
    };

    let n = instance
        .from_manager
        .next()
        .unwrap()
        .expect("could not read from manager")
        .trim()
        .parse::<usize>()
        .unwrap();

    let result = if true {
        solve(&mut instance, n)
    } else {
        solve_92(&mut instance, n)
    };
    instance.answer(result);

    let status = child.wait().expect("failed to wait for manager");
    assert!(status.success());

    _ = std::fs::remove_file(&to_manager);
    _ = std::fs::remove_file(&from_manager);
}

struct Instance {
    to_manager: File,
    from_manager: std::io::Lines<BufReader<File>>,
}

impl Instance {
    fn query(&mut self, idx: &Vec<usize>) -> usize {
        let mut str = format!("Q {}", idx.len());
        for &i in idx {
            str.push_str(&format!(" {}", i));
        }
        str.push_str("\n");
        self.to_manager
            .write(str.as_bytes())
            .expect("could not write to manager");
        let response = self
            .from_manager
            .next()
            .unwrap()
            .expect("could not read from manager");
        response.trim().parse::<usize>().unwrap()
    }

    fn answer(&mut self, res: usize) {
        let str = format!("A {}\n", res);
        self.to_manager
            .write(str.as_bytes())
            .expect("could not write answer to manager");
    }
}

struct Known([Vec<usize>; 2]);

impl Known {
    fn max_query(&self) -> usize {
        self.0[A].len().max(self.0[B].len())
    }

    // run_query returns how many of count_elems are of type A, and the type of the extra element.
    fn run_query(
        &mut self,
        instance: &mut Instance,
        count_elems: &Vec<usize>,
        extra: usize,
    ) -> (usize, Type) {
        let freq = a_if(self.0[A].len() > self.0[B].len());

        assert!(count_elems.len() < self.0[freq].len());
        let mut query = Vec::with_capacity(2 * count_elems.len() + 2);
        for (&k, &q) in self.0[freq]
            .iter()
            .zip(count_elems.iter().chain(std::iter::once(&extra)))
        {
            query.push(k);
            query.push(q);
        }
        let res = instance.query(&query);
        let count = match freq {
            A => count_elems.len() - res / 2,
            B => res / 2,
        };
        let extra_type = freq.xor(res % 2 == 1);
        self.0[extra_type].push(extra);
        (count, extra_type)
    }
}

fn solve_92(instance: &mut Instance, n: usize) -> usize {
    let mut known = Known([Vec::new(), Vec::new()]);
    known.0[A].push(0);

    known.run_query(instance, &vec![], 1);
    if n == 2 {
        return known.0[A].len();
    }

    known.run_query(instance, &vec![], 2);
    if n == 3 {
        return known.0[A].len();
    }

    let mut x = 3;
    while x < 150 && x + 1 < n {
        let (c, _) = known.run_query(instance, &vec![x], x + 1);
        known.0[a_if(c == 1)].push(x);
        x += 2;
    }

    let mut count = 0;
    while x < n {
        let num = known.max_query().min(n - x);
        let (c, _) = known.run_query(instance, &(x..(x + num - 1)).collect(), x + num - 1);
        count += c;
        x += num;
    }

    known.0[A].len() + count
}

#[derive(Debug)]
struct BitsProblem {
    // Number of bits we can determine.
    n: usize,
    // Minimum known elements.
    min_known: usize,
    // Last query is always sum of all bits.
    queries: Vec<Vec<usize>>,
}

// Returns the permutation of queries ordered by increasing length.
fn sorted(queries: &Vec<Vec<usize>>) -> Vec<usize> {
    let mut x: Vec<(usize, usize)> = queries
        .iter()
        .enumerate()
        .map(|(i, x)| (x.len(), i))
        .collect();
    x.sort();
    x.into_iter().map(|(_, i)| i).collect()
}

fn init_bits_problem() -> Vec<BitsProblem> {
    // bp[m] refers to what bits we can determine with 2^m queries.
    let mut bp = Vec::new();
    bp.push(BitsProblem {
        n: 1,
        // The initial number of known mushrooms that is needed to run the queries.
        min_known: 1,
        queries: vec![vec![0]],
    });

    for m in 1..10 {
        let last = &bp[m - 1];

        let n = last.n * 2 + (1 << (m - 1)) - 1;
        let mut queries = Vec::new();
        assert_eq!(last.queries.len(), (1 << (m - 1)));
        for q_idx in 0..(last.queries.len() - 1) {
            let q = &last.queries[q_idx];
            // First query: query the bits in both sets (sum).
            let mut sum = q.clone();
            for &i in q {
                sum.push(last.n + i);
            }

            queries.push(sum);
            // Second query: query the difference between the two queries, plus an extra bit.
            let mut diff = q.clone();
            let mut last_elem = 0;
            for &x in q.iter() {
                for i in last_elem..x {
                    diff.push(last.n + i);
                }
                last_elem = x + 1;
            }
            for i in last_elem..last.n {
                diff.push(last.n + i);
            }

            diff.push(last.n * 2 + q_idx);
            queries.push(diff);
        }
        queries.push((last.n..(2 * last.n)).collect());
        queries.push((0..n).collect());

        let min_known = sorted(&queries)
            .into_iter()
            .enumerate()
            .map(|(ord, q_idx)| queries[q_idx].len() as i32 * 2 + 1 - ord as i32)
            .max()
            .unwrap() as usize;

        bp.push(BitsProblem {
            n,
            min_known,
            queries,
        });
    }
    bp
}

fn solve_bits_problem(bp: &Vec<BitsProblem>, m: usize, results: Vec<usize>) -> Vec<bool> {
    if m == 0 {
        return vec![results[0] == 1];
    }

    let sum = results[(1 << m) - 1];
    let sum_r = results[(1 << m) - 2];

    let mut res_l = Vec::new();
    let mut res_r = Vec::new();
    let mut ret_end = Vec::new();
    for i in 0..((1 << (m - 1)) - 1) {
        let q_sum = results[2 * i] as i32;
        let mut q_diff = results[2 * i + 1] as i32 - sum_r as i32;
        let bit = (q_sum + q_diff) % 2 != 0;
        ret_end.push(bit);
        if bit {
            q_diff -= 1;
        }
        res_l.push((q_sum + q_diff) as usize / 2);
        res_r.push((q_sum - q_diff) as usize / 2);
    }
    res_l.push(sum - sum_r - ret_end.iter().filter(|&&x| x).count());
    res_r.push(sum_r);
    let mut ret_l = solve_bits_problem(bp, m - 1, res_l);
    let ret_r = solve_bits_problem(bp, m - 1, res_r);

    ret_l.extend(ret_r);
    ret_l.extend(ret_end);
    ret_l
}

fn solve(instance: &mut Instance, n: usize) -> usize {
    let bp = init_bits_problem();
    // for i in 0..5 {
    //     println!("{}: {:?}", 1 << i, bp[i]);
    // }
    // dp[i] = min moves to know i values, and last m.
    let mut dp = [10000; 1000];
    let mut dp_m = [0; 1000];
    dp[3] = 2;

    for i in 2..dp.len() {
        for m in 0..bp.len() {
            if i < bp[m].min_known {
                continue;
            }
            let x = i + bp[m].n + (1 << m); // last term is for extra elements.
            let cost: usize = dp[i] + (1 << m);
            if x < dp.len() && dp[x] > cost {
                dp[x] = cost;
                dp_m[x] = m;
            }
        }
    }
    // for (i, &v) in dp.iter().enumerate() {
    //     println!("{}: {}", i, v.0);
    // }

    let mut best = (0, 10000);
    for i in 3..dp.len().min(n) {
        let mut remaining = (n - i) as i32;
        let mut known = i as i32;
        let mut count = dp[i];
        while remaining > 0 {
            remaining -= (known + 1) / 2;
            known += 1;
            count += 1;
        }
        if best.1 > count {
            best = (i, count)
        }
    }
    //println!("n={}  best_known={} total={}", n, best.0, best.1);

    let mut known = Known([Vec::new(), Vec::new()]);
    known.0[A].push(0);

    known.run_query(instance, &vec![], 1);

    if n == 2 {
        return known.0[A].len();
    }

    known.run_query(instance, &vec![], 2);
    if n == 3 {
        return known.0[A].len();
    }

    let mut seq = Vec::new();
    let mut i = best.0;
    while i > 3 {
        let m = dp_m[i];
        seq.push(m);
        i -= bp[m].n + (1 << m);
    }
    seq.reverse();

    for m in seq {
        let start = known.0[A].len() + known.0[B].len();
        // Run queries to determine elements x through start + bp[m].n + (1 << m).
        let mut extra = start + bp[m].n;

        let mut res: Vec<usize> = std::iter::repeat(0).take(1 << m).collect();
        for q_idx in sorted(&bp[m].queries) {
            (res[q_idx], _) = known.run_query(
                instance,
                &bp[m].queries[q_idx].iter().map(|&x| start + x).collect(),
                extra,
            );
            extra += 1;
        }
        for (i, bit) in solve_bits_problem(&bp, m, res).into_iter().enumerate() {
            known.0[a_if(bit)].push(start + i);
        }
    }

    assert_eq!(best.0, known.0[A].len() + known.0[B].len());
    let mut x = best.0;
    let mut count = 0;
    while x < n {
        let num = known.max_query().min(n - x);
        let (c, _) = known.run_query(instance, &(x..(x + num - 1)).collect(), x + num - 1);
        count += c;
        x += num;
    }

    known.0[A].len() + count
}

#[derive(Copy, Clone)]
enum Type {
    A,
    B,
}
use Type::A;
use Type::B;

fn a_if(cond: bool) -> Type {
    if cond {
        A
    } else {
        B
    }
}

impl Type {
    fn xor(self, cond: bool) -> Type {
        a_if(matches!(self, A) ^ cond)
    }
}

impl<T> Index<Type> for [T] {
    type Output = T;

    fn index(&self, index: Type) -> &Self::Output {
        let idx: usize = match index {
            A => 0,
            B => 1,
        };
        &self[idx]
    }
}

impl<T> IndexMut<Type> for [T] {
    fn index_mut(&mut self, index: Type) -> &mut Self::Output {
        let idx: usize = match index {
            A => 0,
            B => 1,
        };
        &mut self[idx]
    }
}
