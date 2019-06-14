use std::{env, fs, process, str};
use std::sync::Arc;
use std::ffi::OsStr;
use failure::Error;
use serde::Deserialize;
use serde_json::{json, Value};
use warp::{Filter, Reply, Rejection};
use tempdir::TempDir;

// TODO:
// - proper error handling instead of unwraps
// - extensive and multilevel logging

const BODY_LIMIT: u64 = 64 * 1024;


#[derive(Deserialize)]
struct Config {
    server_port: u16,
    allow_origin: String,
    sany_cmd: Vec<String>,
}


fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <config.toml>", args[0]);
        process::exit(1);
    }
    let config_toml = fs::read_to_string(&args[1])?;
    let config: Config = toml::from_str(&config_toml)?;
    let config = Arc::new(config);

    let config_ref = Arc::clone(&config);
    let config_arg = warp::any().map(move || config_ref.clone());
    let sany = warp::post2()
        .and(config_arg.clone())
        .and(warp::path("tla2sany"))
        .and(warp::body::content_length_limit(BODY_LIMIT))
        .and(warp::body::json())
        .and_then(tla2sany);

    let pcal = warp::post2()
        .and(config_arg.clone())
        .and(warp::path("pcal.trans"))
        .and(warp::body::content_length_limit(BODY_LIMIT))
        .and(warp::body::json())
        .and_then(pcal_trans);

    let cors = warp::cors()
        .allow_origin(config.allow_origin.as_str())
        .allow_methods(vec!["POST"]);
    let api = sany.or(pcal).with(cors);

    warp::serve(api).run(([127, 0, 0, 1], config.server_port));
    Ok(())
}


fn tla2sany(config: Arc<Config>, data: Value) -> Result<impl Reply, Rejection> {
    // FIXME: sanity check filename e.g. regexp "\W{1,20}"
    let filename = &data["filename"].as_str().unwrap();
    let code = &data["code"].as_str().unwrap();
    let res = run_tla2sany(&filename, &code, &config.sany_cmd).unwrap();
    Ok(warp::reply::json(&json!({
        "output": res,
    })))
}


fn run_tla2sany<S: AsRef<OsStr>>(name: &str, code: &str, cmd: &[S])
    -> Result<String, Error>
{
    let tmp_dir = TempDir::new("tla")?;
    let file_path = tmp_dir.path().join(name);
    fs::write(&file_path, code)?;
    let res = process::Command::new(&cmd[0])
        .args(&cmd[1..])
        .arg(file_path)
        .output()?;

    let out_str = str::from_utf8(&res.stdout).map(|x| x.to_owned())?;
    Ok(out_str)
}


fn pcal_trans(config: Arc<Config>, data: Value) -> Result<impl Reply, Rejection> {
    Ok(warp::reply())
}
