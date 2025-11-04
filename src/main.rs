use std::{
    env,
    path::{ PathBuf, Path },
    sync::{ LazyLock, OnceLock, RwLock, Arc },
};

use captcha_rs::{
    CaptchaBuilder,
    Captcha,
};
use either::{
    self,
    Either,
};
use rayon::prelude::*;

const OUTPUT_POSTFIX: &str  = "dist/";
const OUTPUT_FORMAT : &str  = "jpg";
const OUTPUT_W: u32 = 200;
const OUTPUT_H: u32 = 80;

const REPEAT: usize = 2;
const LENGTH: usize = 4;

const SOURCE: &str  = "1234567890qwertyuiopasdfghjklzxcvbnmQWERTYUIOPASDFGHJKLZXCVBNM";



static TOTAL: LazyLock<usize> = LazyLock::new(|| {
    SOURCE.len().pow(LENGTH as u32) as usize
});
const PRINT_EVERY: usize = 50000;
static CURRENT: LazyLock<Arc<RwLock<usize>>> = LazyLock::new(|| {
    Arc::new(RwLock::new(0))
});
static OUTPUT_ROOT: OnceLock<PathBuf> = OnceLock::new();

fn main() {
    OUTPUT_ROOT.set({
        let mut cwd = env::current_dir().expect("Unable to get `CWD`");
        cwd.push(OUTPUT_POSTFIX);
        cwd
    }).expect("Failed to set `OUTPUT_ROOT`");

    let targets: Vec<String> = build( String::new(), true ).right().expect("Unexpected result type from fn `build`");

    assert!(targets.len() == *TOTAL, "Length of `targets` is unexpected. Expected `{}`, Actual `{}`", *TOTAL, targets.len());

    // init `CURRENT` for use while `save`ing
    *CURRENT.write().unwrap() = 0;

    let _: Vec<_> = targets
        .into_par_iter()
        .map(|target| {

            // Type-Checker require type-annotation
            let target: String = target.to_string();

            for i in 0..REPEAT {
                let path = make_path(&target, i);
                //println!("  - Building: `{target}` [ ./{OUTPUT_POSTFIX} {:?} ]", path.file_name().expect("Invalid File Path"));

                let capt = string_to_captcha(target.clone());

                capt.save(&path);

                //dbg!(target);
                //dbg!(capt);
                //panic!("intended crash");
            }

            *CURRENT.write().unwrap() += 1;
            if *CURRENT.read().unwrap() % PRINT_EVERY == 0 {
                let prg = (100_f64) * (*CURRENT.read().unwrap() as f64) / (*TOTAL as f64);
                println!("  - Progress: `{prg:.02?}`% ({}/{})", *CURRENT.read().unwrap(), *TOTAL);
            }
        })
        .collect();
}

fn string_to_captcha(s: String) -> Captcha {
    CaptchaBuilder::new()
        .length(LENGTH)
        .width(OUTPUT_W)
        .height(OUTPUT_H)
        .dark_mode(false)
        .complexity(1)
        .text(s)
        .build()
}

fn make_path<S: AsRef<Path> + std::fmt::Display>(captcha: S, iteration: usize) -> PathBuf {
    OUTPUT_ROOT.get().expect("`OUTPUT_ROOT` is not initialized yet").as_path().join(
        format!("{captcha}.{iteration}.{OUTPUT_FORMAT}")
    )
}

fn build(prefix: String, _is_root: bool) -> Either<String, Vec<String>> {
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

    let results: RwLock<Vec<String>> = RwLock::new( Vec::new() );
    for c in SOURCE.chars() {
        build( format!("{prefix}{c}"), false ).map_either(
            |s| results.write().unwrap().push(s),
            |v| results.write().unwrap().extend(v),
        );
    }
    /*
    if is_root {
        // print progress 100% if this is root function of recursion
        println!("  - Progress: `100.00`% ({}/{})", *CURRENT.read().unwrap(), *TOTAL);
    }
    */
    either::Right(
        results.into_inner().expect("Failed to strip `RwLock` from `results`")
    )
}
