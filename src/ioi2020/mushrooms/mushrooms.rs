use std::{
    fs::{File, OpenOptions},
    io::BufRead,
    io::{BufReader, Write},
    process::{Command, Stdio},
};

fn main() {
    //test("src/ioi2020/mushrooms/tests/1-16.in"); return;
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

    let result = solve_92(&mut instance, n);
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

fn push_into(x: usize, val: bool, if_true: &mut Vec<usize>, if_false: &mut Vec<usize>) {
    if val {
        if_true.push(x);
    } else {
        if_false.push(x);
    }
}

fn solve_92(instance: &mut Instance, n: usize) -> usize {
    let mut known_a = Vec::new();
    let mut known_b = Vec::new();
    known_a.push(0);

    push_into(
        1,
        instance.query(&vec![0, 1]) == 0,
        &mut known_a,
        &mut known_b,
    );

    if n == 2 {
        return known_a.len();
    }

    push_into(
        2,
        instance.query(&vec![0, 2]) == 0,
        &mut known_a,
        &mut known_b,
    );

    if n == 3 {
        return known_a.len();
    }

    let mut x = 3;
    while x < 150 && x + 1 < n {
        let (freq, other) = if known_a.len() > known_b.len() {
            (&mut known_a, &mut known_b)
        } else {
            (&mut known_b, &mut known_a)
        };
        let res = instance.query(&vec![freq[0], x, freq[1], x + 1]);
        push_into(x, res < 2, freq, other);
        push_into(x + 1, res % 2 == 0, freq, other);
        x += 2;
    }

    let mut num_a = 0;
    while x < n {
        let (freq, other, freq_is_a) = if known_a.len() > known_b.len() {
            (&mut known_a, &mut known_b, true)
        } else {
            (&mut known_b, &mut known_a, false)
        };
        let num = freq.len().min(n - x);

        let mut query = Vec::new();
        for i in 0..num {
            query.push(freq[i]);
            query.push(x + i);
        }
        let res = instance.query(&query);
        push_into(x + num - 1, res % 2 == 0, freq, other);
        if freq_is_a {
            num_a += (num - 1) - (res / 2);
        } else {
            num_a += res / 2;
        }
        x += num;
    }

    known_a.len() + num_a
}
