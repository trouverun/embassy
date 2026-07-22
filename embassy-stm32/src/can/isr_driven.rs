//! ISR driven CAN driver. The caller provides mutual exclusion and calls
//! on_interrupt from its IT0 handler.

use heapless::Deque;

use super::common::InfoRef;
use super::fd::peripheral::Registers;
use super::frame::{Envelope, Frame};

/// FDCAN instance serviced from a caller owned interrupt handler.
pub struct IsrDrivenCan<const TX: usize, const RX: usize> {
    _info: InfoRef,
    regs: Registers,
    ns_per_timer_tick: u64,
    bus_off_recovery: bool,
    tx: Deque<Frame, TX>,
    rx: Deque<Envelope, RX>,
}

impl<const TX: usize, const RX: usize> IsrDrivenCan<TX, RX> {
    pub(crate) fn new(info: InfoRef, regs: Registers, ns_per_timer_tick: u64, bus_off_recovery: bool) -> Self {
        Self {
            _info: info,
            regs,
            ns_per_timer_tick,
            bus_off_recovery,
            tx: Deque::new(),
            rx: Deque::new(),
        }
    }

    /// Drains the rx fifos, flushes queued tx frames and recovers from bus-off.
    pub fn on_interrupt(&mut self) {
        let ir = self.regs.regs.ir().read();

        if ir.tc() {
            self.regs.regs.ir().write(|w| w.set_tc(true));
        }
        if ir.tefn() {
            self.regs.regs.ir().write(|w| w.set_tefn(true));
        }
        while !self.tx.is_empty() && !self.regs.tx_queue_is_full() {
            let frame = self.tx.pop_front().unwrap();
            let _ = self.regs.write(&frame);
        }

        for fifonr in 0..2 {
            if ir.rfn(fifonr) {
                self.regs.regs.ir().write(|w| w.set_rfn(fifonr, true));
                while let Some((frame, ts)) = self.regs.read::<Frame>(fifonr) {
                    let ts = self.regs.calc_timestamp(self.ns_per_timer_tick, ts);
                    let _ = self.rx.push_back(Envelope { ts, frame });
                }
            }
        }

        if ir.bo() {
            self.regs.regs.ir().write(|w| w.set_bo(true));
            if self.bus_off_recovery && self.regs.regs.psr().read().bo() {
                self.regs.regs.cccr().modify(|w| w.set_init(false));
            }
        }
    }

    /// Returns the oldest received frame, if any.
    pub fn receive(&mut self) -> Option<Envelope> {
        self.rx.pop_front()
    }

    /// Writes to the tx fifo, queueing for on_interrupt when full.
    /// Returns the frame back when the queue is also full.
    pub fn send(&mut self, frame: Frame) -> Result<(), Frame> {
        if self.tx.is_empty() && !self.regs.tx_queue_is_full() {
            let _ = self.regs.write(&frame);
            return Ok(());
        }
        self.tx.push_back(frame)
    }
}
