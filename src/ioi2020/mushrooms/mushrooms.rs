use std::{
    fs::{self, File},
    io::BufRead,
    io::{BufReader, Write},
    process::{Command, Stdio},
};

fn main() {
    test("src/ioi2020/mushrooms/tests/0-01.in");
    // println!(
    //     "{:?}",
    //     solve(
    //         [10, 15, 13].into_iter().collect(),
    //         [(0, 2, 20), (0, 1, -11)].into_iter().collect(),
    //     ),
    // );
    // let paths = fs::read_dir("src/ioi2021/candies/tests").unwrap();
    // let mut paths: Vec<String> = paths
    //     .map(|p| p.unwrap().path().to_string_lossy().to_string())
    //     .collect();
    // paths.sort();
    // for p in paths {
    //     if !str::ends_with(&p, ".in") {
    //         continue;
    //     }
    //     test(&p);
    // }
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
    manager_in
        .flush()
        .expect("could not flush input file to manager");

    let mut instance = Instance {
        to_manager: File::create(&to_manager).unwrap(),
        from_manager: BufReader::new(File::open(&from_manager).unwrap()).lines(),
    };

    println!("query result: {}", instance.query(&vec![0, 1, 2]));

    _ = child.kill();
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
        println!("writing '{}'", &str);
        self.to_manager
            .write(str.as_bytes())
            .expect("could not write to manager");
        println!("Waiting for response");
        let response = self
            .from_manager
            .next()
            .unwrap()
            .expect("could not read from manager");
        response.trim().parse::<usize>().unwrap()
    }
}
