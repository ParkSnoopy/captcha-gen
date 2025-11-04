use std::{
    env,
    path::{ PathBuf, Path },
    sync::{ LazyLock, OnceLock, RwLock },
};

use captcha_rs::{
    CaptchaBuilder,
};
use either::{
    self,
    Either,
};

const OUTPUT_POSTFIX: &str = "dist/";

const REPEAT: usize = 2;
const LENGTH: usize = 4;

const SOURCE: &str  = "1234567890qwertyuiopasdfghjklzxcvbnmQWERTYUIOPASDFGHJKLZXCVBNM";



static TOTAL: LazyLock<usize> = LazyLock::new(|| {
    SOURCE.len().pow(LENGTH as u32).strict_mul(REPEAT) as usize
});
const PRINT_EVERY: usize = 1000000;
static CURRENT: RwLock<usize> = RwLock::new(0);
static OUTPUT_ROOT: OnceLock<PathBuf> = OnceLock::new();

fn main() {
    OUTPUT_ROOT.set({
        let mut cwd = env::current_dir().expect("Unable to get `CWD`");
        cwd.push(OUTPUT_POSTFIX);
        cwd
    }).expect("Failed to set `OUTPUT_ROOT`");

    let mut builder = CaptchaBuilder::new();
    builder
        .length(LENGTH)
        .width(200)
        .height(80)
        .dark_mode(false)
        .complexity(1);

    for target in build( String::new() ).right().expect("Unexpected result type from fn `build`").into_iter() {

        builder = builder.clone();
        builder.text(target.clone());

        for i in 0..REPEAT {
            let path = make_path(&target, i);
            //println!("  - Building: `{target}` [ ./{OUTPUT_POSTFIX} {:?} ]", path.file_name().expect("Invalid File Path"));

            let capt = builder.build();
            capt.image.save(&path).expect(format!("Failed to save image `{path:?}`").as_str());
        }
    };
    println!("  - Progress: `100.00`% ({}/{})", *CURRENT.read().unwrap(), *TOTAL);
}

fn make_path<S: AsRef<Path> + std::fmt::Display>(captcha: S, iteration: usize) -> PathBuf {
    OUTPUT_ROOT.get().expect("`OUTPUT_ROOT` is not initialized yet").as_path().join(
        format!("{captcha}.{iteration}.png")
    )
}

fn build(prefix: String) -> Either<String, Vec<String>> {
    if prefix.len() >= LENGTH {
        *CURRENT.write().unwrap() += REPEAT;
        let cur = CURRENT.read().unwrap();
        if *cur % PRINT_EVERY == 0 {
            let prg = (100_f64) * (*cur as f64) / (*TOTAL as f64);
            println!("  - Progress: `{prg:.02?}`% ({cur}/{})", *TOTAL);
        }
        return either::Left(prefix);
    }

    let results: RwLock<Vec<String>> = RwLock::new( Vec::new() );
    for c in SOURCE.chars() {
        build( format!("{prefix}{c}") ).map_either(
            |s| results.write().unwrap().push(s),
            |v| results.write().unwrap().extend(v),
        );
    }
    either::Right(
        results.into_inner().expect("Failed to strip `RwLock` from `results`")
    )
}
