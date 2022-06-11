use std::io::Read;

fn main() {
    let mut src = String::new();
    std::io::stdin().read_to_string(&mut src).unwrap();

    let node = minicc_parser::parse(&src);

    minicc_gen::gen(&mut std::io::stdout(), &node);
}
