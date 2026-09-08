#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Reg {
    RAX = 0,
    RCX = 1,
    RDX = 2,
    RBX = 3,
    RSP = 4,
    RBP = 5,
    RSI = 6,
    RDI = 7,
    R8 = 8,
    R9 = 9,
    R10 = 10,
    R11 = 11,
    R12 = 12,
    R13 = 13,
    R14 = 14,
    R15 = 15,
}

impl Reg {
    pub fn code(self) -> u8 {
        (self as u8) & 7
    }

    pub fn is_ext(self) -> u8 {
        ((self as u8) >> 3) & 1
    }
}

pub const SYSV_ARG_REGS: [Reg; 6] = [Reg::RDI, Reg::RSI, Reg::RDX, Reg::RCX, Reg::R8, Reg::R9];

pub struct Encoder {
    pub code: Vec<u8>,
}

impl Encoder {
    pub fn new() -> Self {
        Self { code: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.code.len()
    }

    pub fn emit_byte(&mut self, byte: u8) {
        self.code.push(byte);
    }

    pub fn emit_bytes(&mut self, bytes: &[u8]) {
        self.code.extend_from_slice(bytes);
    }

    pub fn emit_u32_le(&mut self, val: u32) {
        self.emit_bytes(&val.to_le_bytes());
    }

    pub fn emit_i32_le(&mut self, val: i32) {
        self.emit_bytes(&val.to_le_bytes());
    }

    pub fn emit_u64_le(&mut self, val: u64) {
        self.emit_bytes(&val.to_le_bytes());
    }

    pub fn patch_i32_le(&mut self, pos: usize, val: i32) {
        let bytes = val.to_le_bytes();
        self.code[pos..pos + 4].copy_from_slice(&bytes);
    }

    // Function Prologue & Epilogue
    pub fn push_rbp(&mut self) {
        self.emit_byte(0x55);
    }

    pub fn pop_rbp(&mut self) {
        self.emit_byte(0x5D);
    }

    pub fn mov_rbp_rsp(&mut self) {
        self.emit_bytes(&[0x48, 0x89, 0xE5]);
    }

    pub fn leave(&mut self) {
        self.emit_byte(0xC9);
    }

    pub fn ret(&mut self) {
        self.emit_byte(0xC3);
    }

    pub fn sub_rsp_imm32(&mut self, amount: u32) {
        self.emit_bytes(&[0x48, 0x81, 0xEC]);
        self.emit_u32_le(amount);
    }

    // Reg-Imm MOV
    pub fn mov_reg_imm64(&mut self, reg: Reg, val: i64) {
        let rex = 0x48 | reg.is_ext();
        let opcode = 0xB8 | reg.code();
        self.emit_byte(rex);
        self.emit_byte(opcode);
        self.emit_u64_le(val as u64);
    }

    // Reg-Reg MOV: dst = src
    pub fn mov_reg_reg(&mut self, dst: Reg, src: Reg) {
        let rex = 0x48 | (src.is_ext() << 2) | dst.is_ext();
        let modrm = 0xC0 | (src.code() << 3) | dst.code();
        self.emit_byte(rex);
        self.emit_byte(0x89);
        self.emit_byte(modrm);
    }

    // Stack MOV: [rbp - disp32] = src
    pub fn store_rbp_disp32(&mut self, disp32: i32, src: Reg) {
        let rex = 0x48 | (src.is_ext() << 2);
        let modrm = 0x80 | (src.code() << 3) | Reg::RBP.code();
        self.emit_byte(rex);
        self.emit_byte(0x89);
        self.emit_byte(modrm);
        self.emit_i32_le(-disp32);
    }

    // Stack MOV: dst = [rbp - disp32]
    pub fn load_rbp_disp32(&mut self, dst: Reg, disp32: i32) {
        let rex = 0x48 | (dst.is_ext() << 2);
        let modrm = 0x80 | (dst.code() << 3) | Reg::RBP.code();
        self.emit_byte(rex);
        self.emit_byte(0x8B);
        self.emit_byte(modrm);
        self.emit_i32_le(-disp32);
    }

    // Arithmetic
    pub fn add_reg_reg(&mut self, dst: Reg, src: Reg) {
        let rex = 0x48 | (src.is_ext() << 2) | dst.is_ext();
        let modrm = 0xC0 | (src.code() << 3) | dst.code();
        self.emit_byte(rex);
        self.emit_byte(0x01);
        self.emit_byte(modrm);
    }

    pub fn sub_reg_reg(&mut self, dst: Reg, src: Reg) {
        let rex = 0x48 | (src.is_ext() << 2) | dst.is_ext();
        let modrm = 0xC0 | (src.code() << 3) | dst.code();
        self.emit_byte(rex);
        self.emit_byte(0x29);
        self.emit_byte(modrm);
    }

    pub fn imul_reg_reg(&mut self, dst: Reg, src: Reg) {
        let rex = 0x48 | (dst.is_ext() << 2) | src.is_ext();
        let modrm = 0xC0 | (dst.code() << 3) | src.code();
        self.emit_byte(rex);
        self.emit_byte(0x0F);
        self.emit_byte(0xAF);
        self.emit_byte(modrm);
    }

    pub fn cqo(&mut self) {
        self.emit_bytes(&[0x48, 0x99]);
    }

    pub fn idiv_reg(&mut self, src: Reg) {
        let rex = 0x48 | src.is_ext();
        let modrm = 0xC0 | (7 << 3) | src.code();
        self.emit_byte(rex);
        self.emit_byte(0xF7);
        self.emit_byte(modrm);
    }

    // Bitwise Ops
    pub fn and_reg_reg(&mut self, dst: Reg, src: Reg) {
        let rex = 0x48 | (src.is_ext() << 2) | dst.is_ext();
        let modrm = 0xC0 | (src.code() << 3) | dst.code();
        self.emit_byte(rex);
        self.emit_byte(0x21);
        self.emit_byte(modrm);
    }

    pub fn or_reg_reg(&mut self, dst: Reg, src: Reg) {
        let rex = 0x48 | (src.is_ext() << 2) | dst.is_ext();
        let modrm = 0xC0 | (src.code() << 3) | dst.code();
        self.emit_byte(rex);
        self.emit_byte(0x09);
        self.emit_byte(modrm);
    }

    pub fn xor_reg_reg(&mut self, dst: Reg, src: Reg) {
        let rex = 0x48 | (src.is_ext() << 2) | dst.is_ext();
        let modrm = 0xC0 | (src.code() << 3) | dst.code();
        self.emit_byte(rex);
        self.emit_byte(0x31);
        self.emit_byte(modrm);
    }

    // Comparisons & SetCC
    pub fn cmp_reg_reg(&mut self, reg1: Reg, reg2: Reg) {
        let rex = 0x48 | (reg2.is_ext() << 2) | reg1.is_ext();
        let modrm = 0xC0 | (reg2.code() << 3) | reg1.code();
        self.emit_byte(rex);
        self.emit_byte(0x39);
        self.emit_byte(modrm);
    }

    pub fn setcc_movzx(&mut self, set_opcode: u8, dst: Reg) {
        let rex_set = 0x40 | dst.is_ext();
        let modrm_set = 0xC0 | dst.code();
        self.emit_byte(rex_set);
        self.emit_byte(0x0F);
        self.emit_byte(set_opcode);
        self.emit_byte(modrm_set);

        let rex_movzx = 0x48 | (dst.is_ext() << 2) | dst.is_ext();
        let modrm_movzx = 0xC0 | (dst.code() << 3) | dst.code();
        self.emit_byte(rex_movzx);
        self.emit_byte(0x0F);
        self.emit_byte(0xB6);
        self.emit_byte(modrm_movzx);
    }

    pub fn sete_movzx(&mut self, dst: Reg) {
        self.setcc_movzx(0x94, dst);
    }

    pub fn setne_movzx(&mut self, dst: Reg) {
        self.setcc_movzx(0x95, dst);
    }

    pub fn setl_movzx(&mut self, dst: Reg) {
        self.setcc_movzx(0x9C, dst);
    }

    pub fn setge_movzx(&mut self, dst: Reg) {
        self.setcc_movzx(0x9D, dst);
    }

    pub fn setle_movzx(&mut self, dst: Reg) {
        self.setcc_movzx(0x9E, dst);
    }

    pub fn setg_movzx(&mut self, dst: Reg) {
        self.setcc_movzx(0x9F, dst);
    }

    // Jumps & Calls
    pub fn jmp_rel32(&mut self, offset: i32) -> usize {
        self.emit_byte(0xE9);
        let patch_pos = self.code.len();
        self.emit_i32_le(offset);
        patch_pos
    }

    pub fn je_rel32(&mut self, offset: i32) -> usize {
        self.emit_byte(0x0F);
        self.emit_byte(0x84);
        let patch_pos = self.code.len();
        self.emit_i32_le(offset);
        patch_pos
    }

    pub fn jne_rel32(&mut self, offset: i32) -> usize {
        self.emit_byte(0x0F);
        self.emit_byte(0x85);
        let patch_pos = self.code.len();
        self.emit_i32_le(offset);
        patch_pos
    }

    pub fn jge_rel32(&mut self, offset: i32) -> usize {
        self.emit_byte(0x0F);
        self.emit_byte(0x8D);
        let patch_pos = self.code.len();
        self.emit_i32_le(offset);
        patch_pos
    }

    pub fn jle_rel32(&mut self, offset: i32) -> usize {
        self.emit_byte(0x0F);
        self.emit_byte(0x8E);
        let patch_pos = self.code.len();
        self.emit_i32_le(offset);
        patch_pos
    }

    pub fn jl_rel32(&mut self, offset: i32) -> usize {
        self.emit_bytes(&[0x0F, 0x8C]);
        let patch_pos = self.code.len();
        self.emit_i32_le(offset);
        patch_pos
    }

    pub fn jg_rel32(&mut self, offset: i32) -> usize {
        self.emit_bytes(&[0x0F, 0x8F]);
        let patch_pos = self.code.len();
        self.emit_i32_le(offset);
        patch_pos
    }

    pub fn call_rel32(&mut self, offset: i32) -> usize {
        self.emit_byte(0xE8);
        let patch_pos = self.code.len();
        self.emit_i32_le(offset);
        patch_pos
    }

    pub fn syscall(&mut self) {
        self.emit_bytes(&[0x0F, 0x05]);
    }

    // Helper Emission Routines
    pub fn emit_print_str_helper(&mut self) -> usize {
        let start_pos = self.len();
        self.push_rbp();
        self.mov_rbp_rsp();

        // RDI = buf_ptr
        self.mov_reg_reg(Reg::RSI, Reg::RDI);
        self.mov_reg_imm64(Reg::RCX, 0);

        let loop_pos = self.len();
        // cmp byte ptr [rsi + rcx], 0
        self.emit_bytes(&[0x80, 0x3C, 0x0E, 0x00]);
        let patch_je = self.je_rel32(0);

        // inc rcx
        self.emit_bytes(&[0x48, 0xFF, 0xC1]);
        let patch_jmp = self.jmp_rel32(0);
        self.patch_i32_le(patch_jmp, (loop_pos as i64 - (patch_jmp + 4) as i64) as i32);

        let write_pos = self.len();
        self.patch_i32_le(patch_je, (write_pos as i64 - (patch_je + 4) as i64) as i32);

        // sys_write(1, rsi, rcx)
        self.mov_reg_reg(Reg::RDX, Reg::RCX);
        self.mov_reg_imm64(Reg::RDI, 1);
        self.mov_reg_imm64(Reg::RAX, 1);
        self.syscall();

        self.leave();
        self.ret();
        start_pos
    }

    pub fn emit_print_int_helper(&mut self) -> usize {
        let start_pos = self.len();
        self.push_rbp();
        self.mov_rbp_rsp();
        self.sub_rsp_imm32(32);
        self.store_rbp_disp32(32, Reg::RBX);

        self.mov_reg_reg(Reg::RAX, Reg::RDI);
        self.emit_bytes(&[0x48, 0x8D, 0x5D, 0xFF]); // lea rbx, [rbp - 1]
        self.emit_bytes(&[0xC6, 0x03, 0x0A]); // mov byte ptr [rbx], '\n'
        self.mov_reg_imm64(Reg::RCX, 1);

        self.mov_reg_imm64(Reg::R10, 0);
        self.cmp_reg_reg(Reg::RAX, Reg::R10);
        let patch_jge = self.jge_rel32(0);

        self.emit_bytes(&[0x48, 0xF7, 0xD8]); // neg rax
        self.mov_reg_imm64(Reg::R8, 1);
        let patch_skip_pos = self.jmp_rel32(0);

        let pos_offset = self.len();
        self.patch_i32_le(
            patch_jge,
            (pos_offset as i64 - (patch_jge + 4) as i64) as i32,
        );
        self.mov_reg_imm64(Reg::R8, 0);

        let skip_pos_offset = self.len();
        self.patch_i32_le(
            patch_skip_pos,
            (skip_pos_offset as i64 - (patch_skip_pos + 4) as i64) as i32,
        );

        let loop_offset = self.len();
        self.mov_reg_imm64(Reg::RDX, 0);
        self.mov_reg_imm64(Reg::R10, 10);
        self.emit_bytes(&[0x49, 0xF7, 0xF2]); // div r10

        self.emit_bytes(&[0x80, 0xC2, 0x30]); // add dl, '0'
        self.emit_bytes(&[0x48, 0xFF, 0xCB]); // dec rbx
        self.emit_bytes(&[0x88, 0x13]); // mov [rbx], dl
        self.emit_bytes(&[0x48, 0xFF, 0xC1]); // inc rcx

        self.mov_reg_imm64(Reg::R11, 0);
        self.cmp_reg_reg(Reg::RAX, Reg::R11);
        let patch_jne = self.jne_rel32(0);
        self.patch_i32_le(
            patch_jne,
            (loop_offset as i64 - (patch_jne + 4) as i64) as i32,
        );

        self.mov_reg_imm64(Reg::R11, 1);
        self.cmp_reg_reg(Reg::R8, Reg::R11);
        let patch_jne_write = self.jne_rel32(0);

        self.emit_bytes(&[0x48, 0xFF, 0xCB]); // dec rbx
        self.emit_bytes(&[0xC6, 0x03, 0x2D]); // mov byte ptr [rbx], '-'
        self.emit_bytes(&[0x48, 0xFF, 0xC1]); // inc rcx

        let write_offset = self.len();
        self.patch_i32_le(
            patch_jne_write,
            (write_offset as i64 - (patch_jne_write + 4) as i64) as i32,
        );

        self.mov_reg_imm64(Reg::RAX, 1);
        self.mov_reg_imm64(Reg::RDI, 1);
        self.mov_reg_reg(Reg::RSI, Reg::RBX);
        self.mov_reg_reg(Reg::RDX, Reg::RCX);
        self.syscall();

        self.load_rbp_disp32(Reg::RBX, 32);
        self.leave();
        self.ret();
        start_pos
    }

    pub fn emit_print_int_inline_helper(&mut self) -> usize {
        let start_pos = self.len();
        self.push_rbp();
        self.mov_rbp_rsp();
        self.sub_rsp_imm32(32);
        self.store_rbp_disp32(32, Reg::RBX);

        self.mov_reg_reg(Reg::RAX, Reg::RDI);
        self.emit_bytes(&[0x48, 0x8D, 0x5D, 0xFF]);
        self.mov_reg_imm64(Reg::RCX, 0);

        self.mov_reg_imm64(Reg::R10, 0);
        self.cmp_reg_reg(Reg::RAX, Reg::R10);
        let patch_jge = self.jge_rel32(0);
        self.emit_bytes(&[0x48, 0xF7, 0xD8]);
        self.mov_reg_imm64(Reg::R8, 1);
        let patch_skip_pos = self.jmp_rel32(0);
        let pos_offset = self.len();
        self.patch_i32_le(
            patch_jge,
            (pos_offset as i64 - (patch_jge + 4) as i64) as i32,
        );
        self.mov_reg_imm64(Reg::R8, 0);
        let skip_pos_offset = self.len();
        self.patch_i32_le(
            patch_skip_pos,
            (skip_pos_offset as i64 - (patch_skip_pos + 4) as i64) as i32,
        );

        let loop_offset = self.len();
        self.mov_reg_imm64(Reg::RDX, 0);
        self.mov_reg_imm64(Reg::R10, 10);
        self.emit_bytes(&[0x49, 0xF7, 0xF2]);
        self.emit_bytes(&[0x80, 0xC2, 0x30]);
        self.emit_bytes(&[0x48, 0xFF, 0xCB]);
        self.emit_bytes(&[0x88, 0x13]);
        self.emit_bytes(&[0x48, 0xFF, 0xC1]);
        self.mov_reg_imm64(Reg::R11, 0);
        self.cmp_reg_reg(Reg::RAX, Reg::R11);
        let patch_jne = self.jne_rel32(0);
        self.patch_i32_le(
            patch_jne,
            (loop_offset as i64 - (patch_jne + 4) as i64) as i32,
        );

        self.mov_reg_imm64(Reg::R11, 1);
        self.cmp_reg_reg(Reg::R8, Reg::R11);
        let patch_jne_write = self.jne_rel32(0);
        self.emit_bytes(&[0x48, 0xFF, 0xCB]);
        self.emit_bytes(&[0xC6, 0x03, 0x2D]);
        self.emit_bytes(&[0x48, 0xFF, 0xC1]);
        let write_offset = self.len();
        self.patch_i32_le(
            patch_jne_write,
            (write_offset as i64 - (patch_jne_write + 4) as i64) as i32,
        );
        self.mov_reg_imm64(Reg::RAX, 1);
        self.mov_reg_imm64(Reg::RDI, 1);
        self.mov_reg_reg(Reg::RSI, Reg::RBX);
        self.mov_reg_reg(Reg::RDX, Reg::RCX);
        self.syscall();
        self.load_rbp_disp32(Reg::RBX, 32);
        self.leave();
        self.ret();
        start_pos
    }

    pub fn emit_pow_int_helper(&mut self) -> usize {
        let start_pos = self.len();
        self.push_rbp();
        self.mov_rbp_rsp();

        // RDI = base, RSI = exp
        self.mov_reg_imm64(Reg::RAX, 1); // result = 1

        let loop_offset = self.len();
        self.mov_reg_imm64(Reg::R10, 0);
        self.cmp_reg_reg(Reg::RSI, Reg::R10);
        let patch_jle = self.jle_rel32(0);

        self.imul_reg_reg(Reg::RAX, Reg::RDI); // result *= base
        self.mov_reg_imm64(Reg::R10, 1);
        self.sub_reg_reg(Reg::RSI, Reg::R10); // exp -= 1

        let patch_jmp_loop = self.jmp_rel32(0);
        self.patch_i32_le(
            patch_jmp_loop,
            (loop_offset as i64 - (patch_jmp_loop + 4) as i64) as i32,
        );

        let done_offset = self.len();
        self.patch_i32_le(
            patch_jle,
            (done_offset as i64 - (patch_jle + 4) as i64) as i32,
        );

        self.leave();
        self.ret();
        start_pos
    }

    pub fn emit_input_int_helper(&mut self) -> usize {
        let start_pos = self.len();
        self.push_rbp();
        self.mov_rbp_rsp();
        self.sub_rsp_imm32(64);

        // read(0, rbp - 64, 64)
        self.mov_reg_imm64(Reg::RAX, 0);
        self.mov_reg_imm64(Reg::RDI, 0);
        self.emit_bytes(&[0x48, 0x8D, 0x75, 0xC0]);
        self.mov_reg_imm64(Reg::RDX, 64);
        self.syscall();

        // Parse an optional sign followed by decimal digits.
        self.emit_bytes(&[0x48, 0x8D, 0x75, 0xC0]); // lea rsi, [rbp - 64]
        self.mov_reg_imm64(Reg::RAX, 0); // result
        self.mov_reg_imm64(Reg::RCX, 0); // negative flag
        self.emit_bytes(&[0x80, 0x3E, 0x2D]); // cmp byte [rsi], '-'
        let patch_digits = self.jne_rel32(0);
        self.mov_reg_imm64(Reg::RCX, 1);
        self.emit_bytes(&[0x48, 0xFF, 0xC6]); // inc rsi
        let digits_offset = self.len();
        self.patch_i32_le(
            patch_digits,
            (digits_offset as i64 - (patch_digits + 4) as i64) as i32,
        );

        let loop_offset = self.len();
        self.emit_bytes(&[0x0F, 0xB6, 0x16]); // movzx edx, byte [rsi]
        self.emit_bytes(&[0x80, 0xFA, 0x30]); // cmp dl, '0'
        let patch_done_low = self.jl_rel32(0);
        self.emit_bytes(&[0x80, 0xFA, 0x39]); // cmp dl, '9'
        let patch_done_high = self.jg_rel32(0);
        self.emit_bytes(&[0x48, 0x6B, 0xC0, 0x0A]); // imul rax, rax, 10
        self.emit_bytes(&[0x80, 0xEA, 0x30]); // sub dl, '0'
        self.emit_bytes(&[0x48, 0x0F, 0xB6, 0xD2]); // movzx rdx, dl
        self.emit_bytes(&[0x48, 0x01, 0xD0]); // add rax, rdx
        self.emit_bytes(&[0x48, 0xFF, 0xC6]); // inc rsi
        let patch_loop = self.jmp_rel32(0);
        self.patch_i32_le(
            patch_loop,
            (loop_offset as i64 - (patch_loop + 4) as i64) as i32,
        );

        let done_offset = self.len();
        self.patch_i32_le(
            patch_done_low,
            (done_offset as i64 - (patch_done_low + 4) as i64) as i32,
        );
        self.patch_i32_le(
            patch_done_high,
            (done_offset as i64 - (patch_done_high + 4) as i64) as i32,
        );
        self.mov_reg_imm64(Reg::RDX, 0);
        self.cmp_reg_reg(Reg::RCX, Reg::RDX);
        let patch_positive = self.je_rel32(0);
        self.emit_bytes(&[0x48, 0xF7, 0xD8]); // neg rax
        let positive_offset = self.len();
        self.patch_i32_le(
            patch_positive,
            (positive_offset as i64 - (patch_positive + 4) as i64) as i32,
        );
        self.leave();
        self.ret();
        start_pos
    }
}
