use std::ffi::c_void;

use crate::Register;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct SavedRegistersWithIP {
    pub rip: *const c_void,
    pub saved_registers_without_ip: SavedRegistersWithoutIP,
}

impl SavedRegistersWithIP {
    pub fn apply_diff(&mut self, diff: SavedRegistersWithIPDiff) {
        let SavedRegistersWithIPDiff { rip, saved_registers_without_ip } = diff;
        if let Some(rip) = rip {
            self.rip = rip;
        }
        let SavedRegistersWithoutIPDiff {
            rax,
            rbx,
            rcx,
            rdx,
            rsi,
            rdi,
            rbp,
            rsp,
            r8,
            r9,
            r10,
            r11,
            r12,
            r13,
            r14,
            // r15,
            xsave_area: _
        } = saved_registers_without_ip;
        if let Some(rax) = rax {
            self.saved_registers_without_ip.rax = rax;
        }
        if let Some(rbx) = rbx {
            self.saved_registers_without_ip.rbx = rbx;
        }
        if let Some(rcx) = rcx {
            self.saved_registers_without_ip.rcx = rcx;
        }
        if let Some(rdx) = rdx {
            self.saved_registers_without_ip.rdx = rdx;
        }
        if let Some(rsi) = rsi {
            self.saved_registers_without_ip.rsi = rsi;
        }
        if let Some(rdi) = rdi {
            self.saved_registers_without_ip.rdi = rdi;
        }
        if let Some(rbp) = rbp {
            self.saved_registers_without_ip.rbp = rbp;
        }
        if let Some(rsp) = rsp {
            self.saved_registers_without_ip.rsp = rsp;
        }
        if let Some(r8) = r8 {
            self.saved_registers_without_ip.r8 = r8;
        }
        if let Some(r9) = r9 {
            self.saved_registers_without_ip.r9 = r9;
        }
        if let Some(r10) = r10 {
            self.saved_registers_without_ip.r10 = r10;
        }
        if let Some(r11) = r11 {
            self.saved_registers_without_ip.r11 = r11;
        }
        if let Some(r12) = r12 {
            self.saved_registers_without_ip.r12 = r12;
        }
        if let Some(r13) = r13 {
            self.saved_registers_without_ip.r13 = r13;
        }
        if let Some(r14) = r14 {
            self.saved_registers_without_ip.r14 = r14;
        }
        // if let Some(r15) = r15 {
        //     self.saved_registers_without_ip.r15 = r15;
        // }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct SavedRegistersWithoutIP {
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub rsp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    // pub r15: u64,
    pub xsave_area: [u64; 64],
}

impl SavedRegistersWithoutIP {
    pub fn new_with_all_zero() -> Self {
        Self {
            rax: 0,
            rbx: 0,
            rcx: 0,
            rdx: 0,
            rsi: 0,
            rdi: 0,
            rbp: 0,
            rsp: 0,
            r8: 0,
            r9: 0,
            r10: 0,
            r11: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            // r15: 0,
            xsave_area: [0; 64],
        }
    }

    pub fn get_register(&self, register: Register) -> u64 {
        match register.0 {
            0 => self.rax,
            1 => self.rbx,
            2 => self.rcx,
            3 => self.rdx,
            4 => self.r8,
            5 => self.r9,
            6 => self.r10,
            7 => self.r11,
            8 => self.r12,
            9 => self.r13,
            10 => self.r14,
            _ => todo!()
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct SavedRegistersWithIPDiff {
    pub rip: Option<*const c_void>,
    pub saved_registers_without_ip: SavedRegistersWithoutIPDiff,
}

impl SavedRegistersWithIPDiff {
    pub fn no_change() -> Self {
        Self {
            rip: None,
            saved_registers_without_ip: SavedRegistersWithoutIPDiff::no_change(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct SavedRegistersWithoutIPDiff {
    pub rax: Option<u64>,
    pub rbx: Option<u64>,
    pub rcx: Option<u64>,
    pub rdx: Option<u64>,
    pub rsi: Option<u64>,
    pub rdi: Option<u64>,
    pub rbp: Option<u64>,
    pub rsp: Option<u64>,
    pub r8: Option<u64>,
    pub r9: Option<u64>,
    pub r10: Option<u64>,
    pub r11: Option<u64>,
    pub r12: Option<u64>,
    pub r13: Option<u64>,
    pub r14: Option<u64>,
    // pub r15: Option<u64>,
    pub xsave_area: Option<[u64; 64]>,
}

impl SavedRegistersWithoutIPDiff {
    pub fn no_change() -> Self {
        Self {
            rax: None,
            rbx: None,
            rcx: None,
            rdx: None,
            rsi: None,
            rdi: None,
            rbp: None,
            rsp: None,
            r8: None,
            r9: None,
            r10: None,
            r11: None,
            r12: None,
            r13: None,
            r14: None,
            // r15: None,
            xsave_area: None,
        }
    }

    //todo keep in sync with other get_registers
    pub fn get_register_mut(&mut self, register: Register) -> &mut Option<u64> {
        match register.0 {
            0 => &mut self.rax,
            1 => &mut self.rbx,
            2 => &mut self.rcx,
            3 => &mut self.rdx,
            4 => &mut self.r8,
            5 => &mut self.r9,
            6 => &mut self.r10,
            7 => &mut self.r11,
            8 => &mut self.r12,
            9 => &mut self.r13,
            10 => &mut self.r14,
            _ => todo!()
        }
    }

    pub fn add_change(&mut self, register: Register, new_val: u64) {
        let register = self.get_register_mut(register);
        assert!(register.is_none());
        *register = Some(new_val);
    }
}
