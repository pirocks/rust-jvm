pub fn aload_n(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData, n: u16) -> impl Iterator<Item=IRInstr> {
    //todo have register allocator
    let temp = Register(0);
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.local_var_entry(current_instr_data.current_index, n), to: temp },
        IRInstr::StoreFPRelative { from: temp, to: method_frame_data.local_var_entry(current_instr_data.next_index, 0) }
    ])
}
