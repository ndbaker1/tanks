use std::{
    io::{self, Write},
    process::{exit, Command, Output},
};

fn main() {
    let wasm_lib = "tanks_wasm";
    let wasm_target = "wasm32-unknown-unknown";
    let wasm_path = ["target/", wasm_target, "/release/", wasm_lib, ".wasm"].join("");
    let output_dir = "dist";

    println!(
        "\n📦 Packaging [[ {} ]] lib into '{}' directory for serving through HTTP\n",
        wasm_lib, output_dir,
    );

    let mut command = Command::new("cargo");
    command
        .arg("build")
        .args(["-p", wasm_lib])
        .arg("--lib")
        .args(["--target", wasm_target])
        .arg("--release");

    validate_command_result(&mut command);

    let mut command = Command::new("wasm-bindgen");
    command
        .arg(wasm_path)
        .args(["--out-dir", output_dir])
        .args(["--target", "web"])
        .arg("--typescript");

    validate_command_result(&mut command);

    let mut command = Command::new("cp");
    command
        .arg(format!("{}/index.html", wasm_lib))
        .arg(format!("{}/", output_dir));

    validate_command_result(&mut command);

    println!(
        "\n⚡ Finished packaging browser resources into './{}' directory\n",
        output_dir
    );

    if let Ok(output) = Command::new("ls").arg(output_dir).output() {
        println!("-------- CONTENTS --------");
        write_all_feedback(&output);
        println!("--------------------------");
    }
}

fn validate_command_result(command: &mut Command) {
    println!("EXECUTING :: ( {:?} )", command);
    let output = command.output().expect("failed to execute");

    if output.status.success() {
        println!("SUCCESS ✔");
    } else {
        println!("FAILURE ✖");
        println!("---------------- FAILURE OUTPUT ----------------");
        write_all_feedback(&output);
        println!("------------------------------------------------");
        exit(1);
    }
}

fn write_all_feedback(output: &Output) {
    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();
}
