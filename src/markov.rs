use markov::Chain;
use rand::Rng;

pub fn init_chain() -> Chain<String> {
    println!("Loading markov chain");
    match Chain::load("data.chain") {
        Ok(chain) => {
            println!("Loading chain from file");
            chain
        },
        Err(_) => {
            println!("Generating new markov chain");
            let mut chain: Chain<String> = Chain::new();
            chain.feed_file("corpus.txt").unwrap();
            println!("Persisting chain to file");
            chain.save("data.chain").unwrap();
            println!("Chain written to disk");

            chain
        }
    }
}

pub fn markov_generate<T: Rng>(chain: &Chain<String>, rng: &mut T) -> String {
    chain.generate_with_rng(rng).join(" ")
}
