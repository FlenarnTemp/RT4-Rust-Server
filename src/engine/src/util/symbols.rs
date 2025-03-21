use std::fs;
use strum::IntoEnumIterator;
use crate::script::script_opcode::ScriptOpcode;
use crate::script::script_opcode_pointers::initialize_script_opcode_pointers;
use crate::util::namemap::load_pack;

pub fn generate_server_symbols() {
    fs::create_dir_all("./data/symbols").expect("Failed to create symbols directory");
    
    let scripts = load_pack("./data/src/pack/script.pack");
    let mut script_symbols = String::new();
    for (i, script) in scripts.iter().enumerate() {
        if !script.is_empty() {
            script_symbols.push_str(&format!("{}\t{}\n", i, script));
        }
    }
    
    fs::write("./data/symbols/runescript.sym", script_symbols).expect("Failed to write to RuneScript symbols file");
    
    let mut command_symbols = String::new();
    
    for opcode in ScriptOpcode::iter() {
        let opcode_value = opcode as i32;
        let command_name = format!("{:?}", opcode).to_lowercase();
        
        let mut line = format!("{}\t{}", opcode_value, command_name);
        
        line.push('\n');
        command_symbols.push_str(&line);
    }
    fs::write("./data/symbols/commands.sym", command_symbols).expect("Failed to write to command symbols file");
}