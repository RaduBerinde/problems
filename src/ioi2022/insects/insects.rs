use std::{
    collections::HashMap,
    collections::HashSet,
    fs::{self, File},
    io::BufRead,
    io::BufReader,
};

use rand::seq::SliceRandom;
use rand::thread_rng;

trait Machine {
    fn move_inside(&mut self, i: usize);
    fn move_outside(&mut self, i: usize);
    fn press_button(&mut self) -> usize;
}

fn main() {
    println!("{:?}", run(vec![5, 8, 9, 5, 9, 9]));
    //return;
    //test("src/ioi2022/fish/tests/bandsonedim-01.in"); return;
    let paths = fs::read_dir("src/ioi2022/insects/tests").unwrap();
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

struct Testcase {
    types: Vec<i32>,

    machine: HashSet<usize>,
    num_calls: (i32, i32, i32),
}

impl Machine for Testcase {
    fn move_inside(&mut self, i: usize) {
        self.num_calls.0 += 1;
        self.machine.insert(i);
    }

    fn move_outside(&mut self, i: usize) {
        self.num_calls.1 += 1;
        self.machine.remove(&i);
    }

    fn press_button(&mut self) -> usize {
        self.num_calls.2 += 1;
        let mut typs = HashMap::<i32, usize>::new();
        for n in self.machine.iter() {
            let t = self.types[*n];
            let count = typs.entry(t).or_insert(0);
            *count += 1;
        }
        *typs.iter().map(|(u, v)| v).max().unwrap_or(&0)
    }
}

fn test(in_file: &str) {
    let types = read_input(&in_file);
    let n = types.len() as i32;

    let answer = types
        .iter()
        .fold(HashMap::<i32, usize>::new(), |mut m, x| {
            *m.entry(*x).or_default() += 1;
            m
        })
        .into_iter()
        .map(|(_, count)| count)
        .min()
        .unwrap();

    let (result, num_calls) = run(types);
    let threshold = 3 * n;
    if result != answer {
        println!(
            "{} WRONG ANSWER ({} instead of {})",
            in_file, result, answer
        );
    } else if num_calls <= threshold {
        println!(
            "{} OK! {} calls, threshold is {}",
            in_file, num_calls, threshold
        );
    } else {
        println!(
            "{} TOO MANY CALLS: {}  threshold {}",
            in_file, num_calls, threshold
        );
    }
}

fn read_input(path: &str) -> Vec<i32> {
    let f = File::open(path).unwrap();
    let r = BufReader::new(f);
    let mut l = r.lines().flatten();
    let l1s = l.next().unwrap();
    let mut l1 = l1s.split_whitespace().map(|s| s.parse::<i32>().unwrap());
    let n = l1.next().unwrap();
    let ls = l.next().unwrap();
    let types: Vec<i32> = ls
        .split_whitespace()
        .map(|s| s.parse::<i32>().unwrap())
        .collect();
    assert_eq!(types.len(), n as usize);
    types
}

fn run(types: Vec<i32>) -> (usize, i32) {
    let mut m = Testcase {
        types: types,
        machine: HashSet::new(),
        num_calls: (0, 0, 0),
    };
    let result = solve(m.types.len(), &mut m);
    let num_calls = m.num_calls.0.max(m.num_calls.1).max(m.num_calls.2);
    (result, num_calls)
}

#[derive(Eq, PartialEq, Copy, Clone)]
enum State {
    Inside,
    Outside,
    Ignored,
}

fn solve(n: usize, machine: &mut impl Machine) -> usize {
    let mut state = Vec::with_capacity(n);
    let mut num_inside = 0;
    for _ in 0..n {
        state.push(State::Outside);
    }
    let mut perm = Vec::with_capacity(n);
    for i in 0..n {
        perm.push(i);
    }
    let mut rng = thread_rng();
    perm.shuffle(&mut rng);
    // Find number of distinct types.
    for &i in perm.iter() {
        machine.move_inside(i);
        let c = machine.press_button();
        if c > 1 {
            machine.move_outside(i);
        } else {
            state[i] = State::Inside;
            num_inside += 1;
        }
    }

    // let num_inside = || {
    //     state
    //         .iter()
    //         .map(|s| if *s == State::Inside { 1 } else { 0 })
    //         .sum::<usize>()
    // };

    // We have D types of insects.
    let D = num_inside;

    // Keep the insects inside the machine forever.
    for i in 0..n {
        if state[i] == State::Inside {
            state[i] = State::Ignored
        }
    }

    // Binary search for the largest value such that all insects have at least that frequency.
    let mut l = 1;
    let mut r = n / D;
    while l < r {
        let m = (l + r + 1) / 2;
        // Let's see if all insect types have frequency at least m.
        let mut filled = false;
        perm.shuffle(&mut rng);
        for &i in perm.iter() {
            if state[i] != State::Ignored {
                machine.move_inside(i);
                let c = machine.press_button();
                if c > m {
                    machine.move_outside(i);
                } else {
                    state[i] = State::Inside;
                    num_inside += 1;
                }
            }
            if num_inside == D * m {
                filled = true;
                break;
            }
        }
        if filled {
            // There are at least m of each type. Keep the insects that are in the machine forever -
            // all subsequent checks will be will larger m.
            for i in 0..n {
                if state[i] == State::Inside {
                    state[i] = State::Ignored
                }
            }
            l = m;
        } else {
            // Ignore all the insects outside the machine - they all belong to types that are
            // more frequent. Take the insects we just added back out.
            for i in 0..n {
                match state[i] {
                    State::Inside => {
                        machine.move_outside(i);
                        num_inside -= 1;
                        state[i] = State::Outside;
                    }
                    State::Outside => {
                        state[i] = State::Ignored;
                    }
                    _ => {}
                }
            }
            r = m - 1;
        }
    }

    l
}
