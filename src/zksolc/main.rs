//!
//! YUL to LLVM compiler binary.
//!

pub mod arguments;

use self::arguments::Arguments;

///
/// The application entry point.
///
fn main() {
    std::process::exit(match main_inner() {
        Ok(()) => compiler_common::EXIT_CODE_SUCCESS,
        Err(error) => {
            eprintln!("{:?}", error);
            compiler_common::EXIT_CODE_FAILURE
        }
    })
}

///
/// The auxiliary `main` function to facilitate the `?` error conversion operator.
///
fn main_inner() -> Result<(), compiler_solidity::Error> {
    let mut arguments = Arguments::new();

    let dump_flags = compiler_solidity::DumpFlag::initialize(
        arguments.dump_yul,
        arguments.dump_ethir,
        arguments.dump_evm,
        arguments.dump_llvm,
        arguments.dump_assembly,
    );

    for path in arguments.input_files.iter_mut() {
        *path = path.canonicalize()?;
    }

    let solc_executable = arguments
        .solc
        .unwrap_or_else(|| compiler_solidity::SolcCompiler::DEFAULT_EXECUTABLE_NAME.to_string());
    let solc = compiler_solidity::SolcCompiler::new(solc_executable);
    let solc_version = match solc.version() {
        Ok(version) => version,
        Err(stderr) => {
            eprint!("{}", stderr);
            std::process::exit(compiler_common::EXIT_CODE_FAILURE);
        }
    };

    let output_selection =
        compiler_solidity::SolcStandardJsonInputSettings::get_output_selection(vec![
            compiler_solidity::SolcStandardJsonInputSettingsSelection::Yul,
            compiler_solidity::SolcStandardJsonInputSettingsSelection::EVM,
            compiler_solidity::SolcStandardJsonInputSettingsSelection::ABI,
        ]);
    let solc_input = if arguments.standard_json {
        let mut input: compiler_solidity::SolcStandardJsonInput =
            serde_json::from_reader(std::io::BufReader::new(std::io::stdin()))?;
        input.settings.output_selection = output_selection;
        input
    } else {
        compiler_solidity::SolcStandardJsonInput::try_from_paths(
            arguments.input_files.as_slice(),
            arguments.libraries,
            output_selection,
            true,
        )?
    };

    let libraries = solc_input.settings.libraries.clone().unwrap_or_default();
    let mut solc_output = match solc.standard_json(
        solc_input,
        arguments.base_path,
        arguments.include_paths,
        arguments.allow_paths,
    ) {
        Ok(standard_json) => standard_json,
        Err(stderr) => {
            eprint!("{}", stderr);
            std::process::exit(compiler_common::EXIT_CODE_FAILURE);
        }
    };

    if let Some(errors) = solc_output.errors.as_deref() {
        let mut cannot_compile = false;
        for error in errors.iter() {
            if error.severity.as_str() == "error" {
                cannot_compile = true;
                if arguments.standard_json {
                    serde_json::to_writer(std::io::stdout(), &solc_output)?;
                    return Ok(());
                }
            }

            eprintln!("{}", error);
        }

        if cannot_compile {
            std::process::exit(compiler_common::EXIT_CODE_FAILURE);
        }
    }

    compiler_solidity::initialize_target();
    let mut project = match solc_output.clone().try_into_project(
        libraries,
        compiler_solidity::SolcPipeline::Yul,
        solc_version,
        dump_flags.as_slice(),
    ) {
        Ok(standard_json) => standard_json,
        Err(error) => {
            eprint!("{}", error);
            std::process::exit(compiler_common::EXIT_CODE_FAILURE);
        }
    };
    match project.compile_all(arguments.optimize, dump_flags) {
        Ok(()) => {}
        Err(error) => {
            eprint!("{}", error);
            std::process::exit(compiler_common::EXIT_CODE_FAILURE);
        }
    }

    if arguments.standard_json {
        if let Some(contracts) = solc_output.contracts.as_mut() {
            for (path, contracts) in contracts.iter_mut() {
                for (name, contract) in contracts.iter_mut() {
                    if let Some(contract_data) =
                        project.contracts.get(format!("{}:{}", path, name).as_str())
                    {
                        let bytecode = hex::encode(
                            contract_data
                                .bytecode
                                .as_ref()
                                .expect("Bytecode always exists"),
                        );

                        contract.ir_optimized = None;
                        contract.evm = Some(compiler_solidity::SolcStandardJsonOutputContractEVM::new_zkevm_bytecode(bytecode));
                        contract.factory_dependencies =
                            Some(contract_data.factory_dependencies.clone());
                        contract.hash = contract_data.hash.clone();
                    }
                }
            }
        }

        serde_json::to_writer(std::io::stdout(), &solc_output)?;
        return Ok(());
    }

    let combined_json = if let Some(combined_json) = arguments.combined_json {
        match solc.combined_json(arguments.input_files.as_slice(), combined_json.as_str()) {
            Ok(combined_json) => Some(combined_json),
            Err(stderr) => {
                eprint!("{}", stderr);
                std::process::exit(compiler_common::EXIT_CODE_FAILURE);
            }
        }
    } else {
        None
    };

    if let Some(output_directory) = arguments.output_directory {
        std::fs::create_dir_all(&output_directory)?;

        if let Some(mut combined_json) = combined_json {
            project.write_to_combined_json(&mut combined_json)?;
            combined_json.write_to_directory(&output_directory, arguments.overwrite)?;
        } else {
            project.write_to_directory(
                &output_directory,
                arguments.output_assembly,
                arguments.output_binary,
                arguments.overwrite,
            )?;
        }

        eprintln!(
            "Compiler run successful. Artifact(s) can be found in directory {:?}.",
            output_directory
        );
    } else if let Some(mut combined_json) = combined_json {
        project.write_to_combined_json(&mut combined_json)?;
        println!(
            "{}",
            serde_json::to_string(&combined_json).expect("Always valid")
        );
    } else if arguments.output_assembly
        || arguments.output_binary
        || arguments.output_hashes
        || arguments.output_abi
    {
        for (path, contract) in project.contracts.into_iter() {
            if arguments.output_assembly {
                println!(
                    "Contract `{}` assembly:\n\n{}",
                    path,
                    contract.assembly.expect("Always exists")
                );
            }
            if arguments.output_binary {
                println!(
                    "Contract `{}` bytecode: 0x{}",
                    path,
                    hex::encode(contract.bytecode.expect("Always exists").as_slice())
                );
            }
        }

        match solc.extra_output(
            arguments.input_files.as_slice(),
            arguments.output_abi,
            arguments.output_hashes,
        ) {
            Ok(stdout) => {
                print!("{}", stdout);
            }
            Err(stderr) => {
                eprint!("{}", stderr);
                std::process::exit(compiler_common::EXIT_CODE_FAILURE);
            }
        }
    } else {
        eprintln!("Compiler run successful. No output requested. Use --asm and --bin flags.");
    }

    Ok(())
}
