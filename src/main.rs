use std::env;
use std::process::{Command, Stdio};

fn main() -> std::io::Result<()> {
    let minim = "01_Min";
    let heat1 = "02_Heat";
    let heat2 = "03_Heat2";
    let hold = "04_Hold";

    let args: Vec<String> = env::args().collect();
    match args.len() {
        3 => {
            let prmtop = &args[1][..];
            let rst = &args[2][..];

            let mut minimization = Command::new("pmemd")
                .args(&[
                    "-O",
                    "-i",
                    &format!("{}{}", minim, ".in"),
                    "-p",
                    prmtop,
                    "-c",
                    rst,
                    "-r",
                    &format!("{}{}", minim, "rst7"),
                ])
                .stdout(Stdio::piped())
                .spawn()?;

            if let Some(minim_output) = minimization.stdout.take() {
                let mut first_heat = Command::new("pmemd.cuda")
                    .args(&[
                        "-O",
                        "-i",
                        &format!("{}{}", heat1, ".in"),
                        "-p",
                        prmtop,
                        "-c",
                        &format!("{}{}", minim, ".rst7"),
                        "-o",
                        &format!("{}{}", heat1, "mdout"),
                        "-r",
                        &format!("{}{}", heat1, "rst7"),
                        "-ref",
                        &format!("{}{}", minim, ".rst7"),
                        "-x",
                        &format!("{}{}", heat1, ".nc"),
                    ])
                    .stdout(Stdio::piped())
                    .spawn()?;

                minimization.wait()?;

                if let Some(heat_output) = first_heat.stdout.take() {
                    let mut second_heat = Command::new("pmemd.cuda")
                        .args(&[
                            "-O",
                            "-i",
                            &format!("{}{}", heat2, ".in"),
                            "-p",
                            prmtop,
                            "-c",
                            &format!("{}{}", heat1, ".rst7"),
                            "-o",
                            &format!("{}{}", heat2, "mdout"),
                            "-r",
                            &format!("{}{}", heat2, "rst7"),
                            "-ref",
                            &format!("{}{}", heat1, ".rst7"),
                            "-x",
                            &format!("{}{}", heat2, ".nc"),
                        ])
                        .stdout(Stdio::piped())
                        .spawn()?;

                    first_heat.wait()?;

                    if let Some(heat_output) = first_heat.stdout.take() {
                        let mut cnt: u8 = 1;
                        let cntmax: u8 = 10;

                        while cnt <= cntmax {
                            if cnt == 1 {
                                Command::new("pmemd.cuda")
                                    .args(&[
                                        "-O",
                                        "-i",
                                        &format!("{}{}", hold, ".in"),
                                        "-p",
                                        prmtop,
                                        "-c",
                                        &format!("{}{}", heat2, ".rst7"),
                                        "-o",
                                        &format!("{}{}{}", hold, cnt, "mdout"),
                                        "-r",
                                        &format!("{}{}{}", hold, cnt, "rst7"),
                                        "-x",
                                        &format!("{}{}{}", hold, cnt, ".rst7"),
                                    ])
                                    .stdout(Stdio::piped())
                                    .spawn()?;

                                cnt += 1;
                            } else {
                                Command::new("pmemd.cuda")
                                    .args(&[
                                        "-O",
                                        "-i",
                                        &format!("{}{}", hold, ".in"),
                                        "-p",
                                        prmtop,
                                        "-c",
                                        &format!("{}{}{}", hold, &cnt - 1, ".rst7"),
                                        "-o",
                                        &format!("{}{}{}", hold, cnt, "mdout"),
                                        "-r",
                                        &format!("{}{}{}", hold, cnt, "rst7"),
                                        "-x",
                                        &format!("{}{}{}", hold, cnt, ".rst7"),
                                    ])
                                    .stdout(Stdio::piped())
                                    .spawn()?;

                                cnt += 1;
                            }
                        }
                        second_heat.wait()?;
                    }
                }
            }
        }
        _ => {
            println!(
                "Wrong command line. Usage: {} prmtop_file rst7_file",
                args[0]
            );
        }
    }

    Ok(())
}
