use std::ffi::c_void;
use std::ptr::null_mut;


use crate::Register;

#[repr(C)]
#[derive(Copy, Clone)]
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
        if let Some(SavedRegistersWithoutIPDiff {
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
                        xsave_area
                    }) = saved_registers_without_ip {
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
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct SavedRegistersWithoutIP {
    pub rax: *mut c_void,
    pub rbx: *mut c_void,
    pub rcx: *mut c_void,
    pub rdx: *mut c_void,
    pub rsi: *mut c_void,
    pub rdi: *mut c_void,
    pub rbp: *mut c_void,
    pub rsp: *mut c_void,
    pub r8: *mut c_void,
    pub r9: *mut c_void,
    pub r10: *mut c_void,
    pub r11: *mut c_void,
    pub r12: *mut c_void,
    pub r13: *mut c_void,
    pub r14: *mut c_void,
    pub xsave_area: [u64; 64],
}

impl SavedRegistersWithoutIP {
    pub fn new_with_all_zero() -> Self {
        Self {
            rax: null_mut(),
            rbx: null_mut(),
            rcx: null_mut(),
            rdx: null_mut(),
            rsi: null_mut(),
            rdi: null_mut(),
            rbp: null_mut(),
            rsp: null_mut(),
            r8: null_mut(),
            r9: null_mut(),
            r10: null_mut(),
            r11: null_mut(),
            r12: null_mut(),
            r13: null_mut(),
            r14: null_mut(),
            xsave_area: [0; 64],
        }
    }

    pub fn get_register(&self, register: Register) -> u64 {
        (match register.0 {
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
        }) as u64
    }
}

#[derive(Copy, Clone, Debug)]
pub struct SavedRegistersWithIPDiff {
    pub rip: Option<*const c_void>,
    pub saved_registers_without_ip: Option<SavedRegistersWithoutIPDiff>,
}

#[derive(Copy, Clone, Debug)]
pub struct SavedRegistersWithoutIPDiff {
    pub rax: Option<*mut c_void>,
    pub rbx: Option<*mut c_void>,
    pub rcx: Option<*mut c_void>,
    pub rdx: Option<*mut c_void>,
    pub rsi: Option<*mut c_void>,
    pub rdi: Option<*mut c_void>,
    pub rbp: Option<*mut c_void>,
    pub rsp: Option<*mut c_void>,
    pub r8: Option<*mut c_void>,
    pub r9: Option<*mut c_void>,
    pub r10: Option<*mut c_void>,
    pub r11: Option<*mut c_void>,
    pub r12: Option<*mut c_void>,
    pub r13: Option<*mut c_void>,
    pub r14: Option<*mut c_void>,
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
            xsave_area: None,
        }
    }

    //todo keep in sync with other get_registers
    pub fn get_register_mut(&mut self, register: Register) -> &mut Option<*mut c_void> {
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

    pub fn add_change(&mut self, register: Register, new_val: *mut c_void) {
        let register = self.get_register_mut(register);
        assert!(register.is_none());
        *register = Some(new_val);
    }
}
