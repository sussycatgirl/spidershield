use std::{fs::File, io::{BufRead, BufReader, BufWriter}};

use markov_generator::{AddEdges, HashChain};

const DEPTH: usize = 6;

pub fn init_chain() -> HashChain<char> {
    match File::open("data.chain") {
        Ok(chain_file) => {
            println!("Opening existing markov chain");
            cbor4ii::serde::from_reader(BufReader::new(chain_file))
                .expect("Couldn't open markov chain")
        },
        Err(_) => {
            // this is mostly copied from the crate's readme
            // i don't really care how it works, i just need it to work
            println!("Generating new markov chain");
            let mut chain = HashChain::new(DEPTH);

            let file = File::open("corpus.txt").unwrap();
            let mut reader = BufReader::new(file);
            let mut line = String::new();
            let mut prev = Vec::<char>::new();

            while let Ok(1..) = reader.read_line(&mut line) {
                if line.trim().is_empty() { continue; }

                chain.add_all(line.chars(), AddEdges::Both);

                chain.add_all(
                    prev.iter().copied().chain(line.chars().take(DEPTH)),
                    AddEdges::Neither,
                );

                if let Some((n, (i, _c))) =
                    line.char_indices().rev().enumerate().take(DEPTH).last()
                {
                    prev.drain(0..prev.len().saturating_sub(DEPTH - n - 1));
                    prev.extend(line[i..].chars());
                }
                line.clear();
            }

            println!("Markov chain generated");

            // todo control with env var or smth
            if true {
                println!("Writing chain to file");
                let out_file = File::create_new("data.chain").unwrap();
                let writer = BufWriter::new(out_file);
                cbor4ii::serde::to_writer(writer, &chain).unwrap();
            }

            return chain;
        }
    }
}

pub fn generate(chain: &HashChain<char>) -> String {
    let mut data = String::new();
    for &c in chain.generate() {
        data.push(c);
    };

    data
}
