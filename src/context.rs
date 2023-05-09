use std::{cell::Cell, rc::Rc};

use crate::{cpu::CPU, gpu::GPU, joypad::JoyPad, mmu::MMU, rtc::RTC};

pub struct Context {
    pub cpu: Rc<Cell<CPU>>,
    pub mmu: Rc<Cell<MMU>>,
    pub rtc: Rc<Cell<RTC>>,
    pub gpu: Rc<Cell<GPU>>,
    pub joypad: Rc<Cell<JoyPad>>,
}
