use lazy_static::lazy_static;

use rand::{Rng, rng};
use std::{
    cmp::min,
    env,
    fs::create_dir_all,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use captcha_rs::{Captcha, CaptchaBuilder};
use either::{self, Either};
use rayon::prelude::*;

const OUTPUT_POSTFIX: &str = "dist/";
const OUTPUT_FORMAT: &str = "png";
const OUTPUT_W: u32 = 250;
const OUTPUT_H: u32 = 100;
const OUTPUT_INDEX_FROM: usize = 0;

const NOISE: u32 = 1;
const REPEAT: usize = 1;
const LENGTH: usize = 4;
const DROPOUT: f64 = 0.9999;

const SOURCE: &str = "1234567890QWERTYUIOPASDFGHJKLZXCVBNM";

lazy_static! {
    static ref TOTAL: RwLock<usize> = RwLock::new(SOURCE.len().pow(LENGTH as u32) as usize);
    static ref CURRENT: Arc<RwLock<usize>> = Arc::new(RwLock::new(0));
    static ref OUTPUT_ROOT: PathBuf = {
        let mut cwd = env::current_dir().expect("Unable to get `CWD`");
        cwd.push(OUTPUT_POSTFIX);
        create_dir_all(&cwd).expect(format!("Failed to mkdir `{}`", &cwd.display()).as_str());
        cwd
    };
    static ref PRINT_EVERY: Arc<RwLock<usize>> =
        Arc::new(RwLock::new(min(10000, *TOTAL.read().unwrap() / 100,)));
}

fn main() {
    // RUNTIME TEST

    assert_eq!(SOURCE.len(), 10 + 26);

    //
    let targets: Vec<String> = build(String::new(), true)
        .right()
        .expect("Unexpected result type from fn `build`");

    // init `CURRENT` for use while `save`ing
    *CURRENT.write().unwrap() = 0;

    let _: Vec<_> = targets
        .into_par_iter()
        .panic_fuse()
        .map(|target| {
            // Type-Checker require type-annotation
            let target: String = target.to_string();

            for i in (OUTPUT_INDEX_FROM)..(OUTPUT_INDEX_FROM + REPEAT) {
                let path = make_path(&target, i);
                //println!("  - Building: `{target}` [ ./{OUTPUT_POSTFIX} {:?} ]", path.file_name().expect("Invalid File Path"));

                let capt = string_to_captcha(target.clone());

                capt.save(&path);
            }

            *CURRENT.write().unwrap() += 1;
            if *CURRENT.read().unwrap() % *PRINT_EVERY.read().unwrap() == 0 {
                let prg =
                    (100_f64) * (*CURRENT.read().unwrap() as f64) / (*TOTAL.read().unwrap() as f64);
                println!(
                    "  - Progress: `{prg:.02?}`% ({}/{})",
                    *CURRENT.read().unwrap(),
                    *TOTAL.read().unwrap()
                );
            }
        })
        .collect();

    // print `100.00 %` after finish
    let cur = *CURRENT.read().unwrap();
    let tot = *TOTAL.read().unwrap();
    let prg = (100_f64) * (cur as f64) / (tot as f64);
    println!("  - Progress: `{prg:.02?}`% ({cur}/{tot})");
}

fn string_to_captcha(s: String) -> Captcha {
    CaptchaBuilder::new()
        .length(LENGTH)
        .width(OUTPUT_W)
        .height(OUTPUT_H)
        .dark_mode(false)
        .complexity(NOISE)
        .text(s)
        .build()
}

fn make_path<S: AsRef<Path> + std::fmt::Display>(captcha: S, iteration: usize) -> PathBuf {
    OUTPUT_ROOT
        .as_path()
        .join(format!("{captcha}.{iteration}.{OUTPUT_FORMAT}"))
}

fn build(prefix: String, is_root: bool) -> Either<String, Vec<String>> {
    if prefix.len() >= LENGTH {
        *CURRENT.write().unwrap() += 1;
        /*
        let cur = CURRENT.read().unwrap();
        if *cur % PRINT_EVERY == 0 {
            let prg = (100_f64) * (*cur as f64) / (*TOTAL as f64);
            println!("  - Progress: `{prg:.02?}`% ({cur}/{})", *TOTAL);
        }
        */
        return either::Left(prefix);
    }

    let results: RwLock<Vec<String>> = RwLock::new(Vec::new());
    for c in SOURCE.chars() {
        build(format!("{prefix}{c}"), false).map_either(
            |s| results.write().unwrap().push(s),
            |v| results.write().unwrap().extend(v),
        );
    }
    let mut results: Vec<String> = results
        .into_inner()
        .expect("Failed to strip `RwLock` from `results`");

    if is_root {
        let mut rng = rng();
        results = results
            .clone()
            .into_iter()
            .filter(|&_| rng.random::<f64>() > DROPOUT)
            .collect();

        // Overwrite `TOTAL`
        println!(
            "  - Target is built: ( Expected `{}`, After Dropout `{}` )",
            TOTAL.read().unwrap(),
            results.len()
        );
        *TOTAL.write().unwrap() = results.len();

        // Recalculate `PRINT_EVERY`
        let print_every = *PRINT_EVERY.read().unwrap(); // prevent Mutex locked forever
        *PRINT_EVERY.write().unwrap() = min(print_every, *TOTAL.read().unwrap() / 100);
    }

    either::Right(results)
}
