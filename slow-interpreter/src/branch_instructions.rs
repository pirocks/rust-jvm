//pub fn do_goto_w(code: &[u8]) -> ! {
//    let branchbyte1 = code[1] as u32;
//    let branchbyte2 = code[2] as u32;
//    let branchbyte3 = code[3] as u32;
//    let branchbyte4 = code[4] as u32;
//    let _offset = ((branchbyte1 << 24) | (branchbyte2 << 16)
//        | (branchbyte3 << 8) | branchbyte4) as i16;
//    unimplemented!("todo branching")
//}
//
//pub fn do_goto(code: &[u8]) -> ! {
//    let branchbyte1 = code[1] as u16;
//    let branchbyte2 = code[2] as u16;
//    let _offset = ((branchbyte1 << 8) | branchbyte2) as i16;
//    unimplemented!("todo branching")
//}
//
