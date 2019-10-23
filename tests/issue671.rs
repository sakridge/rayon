#![type_length_limit = "500000"]

extern crate solana_rayon;

use solana_rayon::prelude::*;

#[test]
fn type_length_limit() {
    let _ = Vec::<Result<(), ()>>::new()
        .into_par_iter()
        .map(|x| x)
        .map(|x| x)
        .map(|x| x)
        .map(|x| x)
        .map(|x| x)
        .map(|x| x)
        .collect::<Result<(), ()>>();
}
