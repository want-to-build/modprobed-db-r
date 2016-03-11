use std::io::BufReader;
use std::io::BufRead;
use std::io::BufWriter;
use std::io::Write;
use std::fs::File;
use std::path::PathBuf;
use std::env;
use std::collections::HashSet;
use std::fs::OpenOptions;
use std::io::SeekFrom;
use std::io::Seek;
use std::process::Command;
use std::fs;
use std::os::unix::fs::PermissionsExt;

fn help ()
{
	println!("init   : create empty db at usr/local/share/.modprobed");
	println!("list   : display the modules saved in db");
	println!("store  : store the modules currently loaded in db");
	println!("recall : call all modules in db currently unloaded");
}

fn cut (s: String, c: char) -> String
{
	let mut res = String::new();
	for current in s.chars() {
		if current == c
		{
			break;
		}
		res.push(current);
	}
	return res;
}

fn db_path () -> PathBuf
{
	let mut p = PathBuf::new();
	p.push("/usr/local/share/.modprobed");
	return p;
}

fn get_loaded () -> HashSet<String>
{
	println!("Reading /proc/modules");
	let mut modules: HashSet<String> = HashSet::new();
	let f = match File::open("/proc/modules") {
    Ok(file) => file,
    Err(e) => panic!("{}", e)
	};

	let file = BufReader::new(&f);
	for line in file.lines() {
		  let l = line.unwrap();
		  modules.insert(cut(l, ' '));
	}

	println!("{} modules currently loaded", modules.len());
	return modules;
}

fn get_saved () -> HashSet<String>
{
	let mut modules: HashSet<String> = HashSet::new();
	let p = db_path();

	let f = match File::open(&p) {
    Ok(file) => file,
    Err(e) => panic!("{}", e)
	};

	println!("Reading db file at {}", p.to_string_lossy());
	let file = BufReader::new(&f);
	for line in file.lines() {
		  let l = line.unwrap();
		  modules.insert(l);
	}

	println!("{} modules previously saved", modules.len());
	return modules;
}

fn store ()
{
	let saved = get_saved ();
	let loaded = get_loaded ();
	let unsaved = loaded.difference(&saved);
	let unloaded = saved.difference(&loaded);

	for module in unsaved.clone()
	{
		println!("saving   : {}", module);
	}
	for module in unloaded
	{
		println!("unloaded : {}", module);
	}

	let p = db_path();
	let mut file = match OpenOptions::new().write(true).append(true).open(&p) {
    Ok(file) => file,
    Err(e) => panic!("{}", e)
	};

	let _ = file.seek(SeekFrom::End(0));
	let mut writer = BufWriter::new(&file);
	for module in unsaved {
		let mut s = module.clone();
		s.push('\n');
		let _ = writer.write(s.as_bytes());
	}
	let _ = writer.flush();
}

fn recall ()
{
	let saved = get_saved ();
	let loaded = get_loaded ();
	let unsaved = loaded.difference(&saved);
	let unloaded = saved.difference(&loaded);

	for module in unloaded.clone()
	{
		println!("loading : {}", module);
	}
	for module in unsaved
	{
		println!("unsaved : {}", module);
	}
	for module in unloaded
	{
		print!("{}", String::from_utf8_lossy(&Command::new("modprobe")
																							.arg(module)
																							.output().unwrap_or_else(|e| { panic!("{}", e) })
																							.stderr));
	}

}

fn init()
{
	let p = db_path();
	let permissions = PermissionsExt::from_mode(0o666);
	println!("Creating db file at {}", p.to_string_lossy());

  match File::create(&p) {
		Err(e) => panic!("{}", e),
		Ok(_) => match fs::set_permissions(p, permissions) {
							Err(e) => panic!("{}", e),
							Ok(_) => println!("Success !")
		}
	};
}

fn main () {
	let mut args = env::args_os();

	match args.nth(1) {
		Some(arg) => match arg.into_string() {
			Ok(string) => match string.as_str() {
				"list" => {
					for module in get_saved () {
						println!("{}", module);
					}
				},
				"store" => store(),
				"recall" => recall(),
				"init" => init(),
				_ => help()
			},
			_ => help()
		},
		_ => help()
	};

}
