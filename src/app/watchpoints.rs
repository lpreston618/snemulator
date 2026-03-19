pub mod editor;
pub mod types;

// use crate::core;

// #[derive(Clone, Copy)]
// enum WatchpointPort {
//     In1,
//     In2,
// }

// // trait WatchpointTriggerable {
// //     /// Checks the watchpoint condition and returns `true` if it is met.
// //     fn check(&mut self, snem_core: &core::snemcore::Snemulator) -> bool;
// //     fn connect(&mut self, out_id: Option<WatchpointID>);
// // }

// // trait WatchpointLogical {
// //     /// Update the state of this watchpoint logic block based on the input.
// //     /// Propogates the update to connected logic blocks and returns `true` if a break should occur.
// //     fn update(&mut self, id: WatchpointID, value: bool) -> bool;
// //     fn connect(&mut self, id: Option<WatchpointID>, port: WatchpointPort);
// // }

// // struct WatchpointBreak {
// //     in_id: Option<WatchpointID>,
// // }
// // struct WatchpointAnd {
// //     in1_id: Option<WatchpointID>,
// //     in2_id: Option<WatchpointID>,
// //     out_id: Option<WatchpointID>,
// //     in1: bool,
// //     in2: bool,
// //     out: bool,
// // }
// // struct WatchpointOr {
// //     in1_id: Option<WatchpointID>,
// //     in2_id: Option<WatchpointID>,
// //     out_id: Option<WatchpointID>,
// //     in1: bool,
// //     in2: bool,
// //     out: bool,
// // }
// // struct WatchpointNot {
// //     in_id: Option<WatchpointID>,
// //     out_id: Option<WatchpointID>,
// //     out: bool,
// // }

// // impl WatchpointLogical for WatchpointAnd {
// //     fn update(&mut self, input: WatchpointPort, value: bool) -> bool {
// //         match input {
// //             WatchpointPort::In1 => self.in1 = value,
// //             WatchpointPort::In2 => self.in2 = value,
// //         }
// //         if self.in1 && self.in2 {
// //             if let Some(conn) = &mut self.connection.out {
// //                 return conn.update(self.connection.out_port, true);
// //             }
// //         }
// //         false // And block does not trigger break
// //     }
// // }

// // impl WatchpointLogical for WatchpointOr {
// //     fn update(&mut self, input: WatchpointPort, value: bool) -> bool {
// //         match input {
// //             WatchpointPort::In1 => self.in1 = value,
// //             WatchpointPort::In2 => self.in2 = value,
// //         }
// //         if self.in1 || self.in2 {
// //             if let Some(conn) = &mut self.connection.out {
// //                 return conn.update(self.connection.out_port, true);
// //             }
// //         }
// //         false // Or block does not trigger break
// //     }
// // }

// // impl WatchpointLogical for WatchpointNot {
// //     fn update(&mut self, input: WatchpointPort, value: bool) -> bool {
// //         match input {
// //             WatchpointPort::In1 => self.in1 = value,
// //             _ => {}
// //         }
// //         if !self.in1 {
// //             if let Some(conn) = &mut self.connection.out {
// //                 return conn.update(self.connection.out_port, true);
// //             }
// //         }
// //         false // Not block does not trigger break
// //     }
// // }

// // impl WatchpointLogical for WatchpointBreak {
// //     fn update(&mut self, _input: WatchpointPort, _value: bool) -> bool {
// //         true
// //     }
// // }



// // struct WatchpointTrigger {
// //     trigger: Box<dyn WatchpointTriggerable>,
// // }

// // struct WatchpointLogic {
// //     logic: Box<dyn WatchpointLogical>,
// // }

// #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
// pub struct WatchpointID(usize);

// enum WatchpointTrigger {
//     ManualToggle {
//         out_id: Option<WatchpointID>,
//         out: bool,
//     },
// }

// impl WatchpointTrigger {
//     pub fn check(&mut self, snem_core: &core::snemcore::Snemulator) {
//         match self {
//             WatchpointTrigger::ManualToggle { .. } => {}
//         }
//     }
//     pub fn value(&self) -> bool {
//         match self {
//             WatchpointTrigger::ManualToggle { out, .. } => {
//                 *out
//             }
//         }
//     }
//     fn out_id(&self) -> Option<WatchpointID> {
//         match self {
//             WatchpointTrigger::ManualToggle { out_id, .. } => {
//                 *out_id
//             }
//         }
//     }
// }

// enum WatchpointLogic {
//     WatchpointBreak {
//         in_id: Option<WatchpointID>,
//     },
//     WatchpointAnd {
//         in1_id: Option<WatchpointID>,
//         in2_id: Option<WatchpointID>,
//         out_id: Option<WatchpointID>,
//         in1: bool,
//         in2: bool,
//         out: bool,
//     },
//     WatchpointOr {
//         in1_id: Option<WatchpointID>,
//         in2_id: Option<WatchpointID>,
//         out_id: Option<WatchpointID>,
//         in1: bool,
//         in2: bool,
//         out: bool,
//     },
//     WatchpointNot {
//         in_id: Option<WatchpointID>,
//         out_id: Option<WatchpointID>,
//         out: bool,
//     },
// }

// impl WatchpointLogic {
//     fn update(&mut self, input_id: WatchpointID, input_val: bool) {
//         match self {
//             WatchpointLogic::WatchpointBreak { .. } => {}
//             WatchpointLogic::WatchpointAnd { in1_id, in2_id, in1, in2, out, .. } => {
//                 if in1_id.is_some() && in1_id.unwrap() == input_id {
//                     *in1 = input_val;
//                 }
//                 if in2_id.is_some() && in2_id.unwrap() == input_id {
//                     *in2 = input_val;
//                 }
//                 *out = *in1 && *in2;
//             },
//             WatchpointLogic::WatchpointOr { in1_id, in2_id, in1, in2, out, .. } => {
//                 if in1_id.is_some() && in1_id.unwrap() == input_id {
//                     *in1 = input_val;
//                 }
//                 if in2_id.is_some() && in2_id.unwrap() == input_id {
//                     *in2 = input_val;
//                 }
//                 *out = *in1 || *in2;
//             },
//             WatchpointLogic::WatchpointNot { in_id, out, .. } => {
//                 if in_id.is_some() && in_id.unwrap() == input_id {
//                     *out = !input_val;
//                 }
//             },
//         }
//     }
//     fn value(&self) -> bool {
//         match self {
//             WatchpointLogic::WatchpointBreak { .. } => true,
//             WatchpointLogic::WatchpointAnd { out, .. } => *out,
//             WatchpointLogic::WatchpointOr { out, .. } => *out,
//             WatchpointLogic::WatchpointNot { out, .. } => *out,
//         }
//     }
//     fn out_id(&self) -> Option<WatchpointID> {
//         match self {
//             WatchpointLogic::WatchpointBreak { .. } => None,
//             WatchpointLogic::WatchpointAnd { out_id, .. } => *out_id,
//             WatchpointLogic::WatchpointOr { out_id, .. } => *out_id,
//             WatchpointLogic::WatchpointNot { out_id, .. } => *out_id,
//         }
//     }
// }

// pub enum WatchpointTriggerKind {
//     ManualToggle,
// }

// pub enum WatchpointLogicKind {
//     WatchpointBreak,
//     WatchpointAnd,
//     WatchpointOr,
//     WatchpointNot,
// }

// pub struct WatchpointView {
//     next_id: WatchpointID,
//     triggerables: std::collections::HashMap<WatchpointID, WatchpointTrigger>,
//     logic: std::collections::HashMap<WatchpointID, WatchpointLogic>,
// }

// impl WatchpointView {
//     pub fn new() -> Self {
//         Self {
//             next_id: WatchpointID(0),
//             triggerables: std::collections::HashMap::new(),
//             logic: std::collections::HashMap::new(),
//         }
//     }
//     pub fn add_triggerable(&mut self, kind: WatchpointTriggerKind) -> WatchpointID {
//         let id = self.next_id;
//         self.next_id.0 += 1;
//         self.triggerables.insert(id, match kind {
//             WatchpointTriggerKind::ManualToggle => WatchpointTrigger::ManualToggle { out_id: None, out: false },
//         });
//         id
//     }
//     pub fn add_logic(&mut self, kind: WatchpointLogicKind) -> WatchpointID {
//         let id = self.next_id;
//         self.next_id.0 += 1;
//         self.logic.insert(id, match kind {
//             WatchpointLogicKind::WatchpointBreak => {
//                 WatchpointLogic::WatchpointBreak { in_id: None }
//             },
//             WatchpointLogicKind::WatchpointAnd => {
//                 WatchpointLogic::WatchpointAnd {
//                     in1_id: None,
//                     in2_id: None,
//                     out_id: None,
//                     in1: false,
//                     in2: false,
//                     out: false,
//                 }
//             },
//             WatchpointLogicKind::WatchpointOr => {
//                 WatchpointLogic::WatchpointOr {
//                     in1_id: None,
//                     in2_id: None,
//                     out_id: None,
//                     in1: false,
//                     in2: false,
//                     out: false,
//                 }
//             },
//             WatchpointLogicKind::WatchpointNot => {
//                 WatchpointLogic::WatchpointNot {
//                     in_id: None,
//                     out_id: None,
//                     out: true,
//                 }
//             },
//         });
//         id
//     }
//     pub fn connect(&mut self, from_id: WatchpointID, to_id: WatchpointID, port: WatchpointPort) {
//         if !self.logic.contains_key(&to_id) {
//             return;
//         }
        
//         if !self.triggerables.contains_key(&from_id) && !self.logic.contains_key(&from_id) {
//             return;
//         }
                
//         let from = self.triggerables.get_mut(&from_id);
        
//         if from.is_none() {
//             let from = self.logic.get_mut(&from_id).unwrap();
            
//             match from {
//                 WatchpointLogic::WatchpointAnd { out_id, .. } => {
//                     *out_id = Some(to_id);
//                 }
//                 WatchpointLogic::WatchpointOr { out_id, .. } => {
//                     *out_id = Some(to_id);
//                 }
//                 WatchpointLogic::WatchpointNot { out_id, .. } => {
//                     *out_id = Some(to_id);
//                 }
//                 WatchpointLogic::WatchpointBreak { .. } => {
//                     return;
//                 }
//             }
            
//             return;
//         }
        
//         let from = from.unwrap();
//         match from {
//             WatchpointTrigger::ManualToggle { out_id } => {
//                 *out_id = Some(to_id);
//             }
//             // WatchpointTrigger::Memory { addr, trigger_value } => {
//             //     match port {
//             //         WatchpointPort::In1 => in1_id,
//             //         WatchpointPort::In2 => in2_id,
//             //         // WatchpointPort::Out => out_id,
//             //     }
//             // }
//             // WatchpointTrigger::CpuRegister8 { reg, trigger } => {
//             //     match port {
//             //         WatchpointPort::In1 => in1_id,
//             //         WatchpointPort::In2 => in2_id,
//             //         // WatchpointPort::Out => out_id,
//             //     }
//             // }
//             // WatchpointTrigger::CpuRegister16 { reg, trigger } => {
//             //     match port {
//             //         WatchpointPort::In1 => in1_id,
//             //         WatchpointPort::In2 => in2_id,
//             //         // WatchpointPort::Out => out_id,
//             //     }
//             // }
//             // WatchpointTrigger::HwRegister { reg, trigger_value } => {
//             //     match port {
//             //         WatchpointPort::In1 => in1_id,
//             //         // WatchpointPort::Out => out_id,
//             //         _ => return,
//             //     }
//             // }
//             // _ => return,
//         }
        
//         let to = self.logic.get_mut(&to_id).unwrap();
        
//         let ptr = match to {
//             WatchpointLogic::WatchpointBreak { in_id } => {
//                 match port {
//                     WatchpointPort::In1 => in_id,
//                     _ => return,
//                 }
//             },
//             WatchpointLogic::WatchpointAnd { in1_id, in2_id, out_id, .. } => {
//                 match port {
//                     WatchpointPort::In1 => in1_id,
//                     WatchpointPort::In2 => in2_id,
//                     // WatchpointPort::Out => out_id,
//                 }
//             }
//             WatchpointLogic::WatchpointOr { in1_id, in2_id, out_id, .. } => {
//                 match port {
//                     WatchpointPort::In1 => in1_id,
//                     WatchpointPort::In2 => in2_id,
//                     // WatchpointPort::Out => out_id,
//                 }
//             }
//             WatchpointLogic::WatchpointNot { in_id, out_id, .. } => {
//                 match port {
//                     WatchpointPort::In1 => in_id,
//                     // WatchpointPort::Out => out_id,
//                     _ => return,
//                 }
//             }
//         };
//         *ptr = Some(from_id);
//     }
    
//     // Helper function to disconnect watchpoint a from watchpoint b (remove all connections pointing from a to b)
//     fn asymmetric_disconnect(&mut self, a: WatchpointID, b: WatchpointID) {
//         if let Some(wp) = self.logic.get_mut(&a) {
//             match wp {
//                 WatchpointLogic::WatchpointAnd { in1_id, in2_id, .. } => {
//                     if in1_id.is_some() && in1_id.unwrap() == b { *in1_id = None; }
//                     if in2_id.is_some() && in2_id.unwrap() == b { *in2_id = None; }
//                 }
//                 WatchpointLogic::WatchpointOr { in1_id, in2_id, .. } => {
//                     if in1_id.is_some() && in1_id.unwrap() == b { *in1_id = None; }
//                     if in2_id.is_some() && in2_id.unwrap() == b { *in2_id = None; }
//                 }
//                 WatchpointLogic::WatchpointNot { in_id, .. } => {
//                     if in_id.is_some() && in_id.unwrap() == b { *in_id = None; }
//                 }
//                 _ => {}
//             }
//         } else if let Some(wp) = self.triggerables.get_mut(&a) {
//             match wp {
//                 WatchpointTrigger::ManualToggle { out_id, .. } => {
//                     if out_id.is_some() && out_id.unwrap() == b { *out_id = None; }
//                 }
//                 // _ => {}
//             }
//         }
//     }
    
//     pub fn disconnect(&mut self, id1: WatchpointID, id2: WatchpointID) {
//         self.asymmetric_disconnect(id1, id2);
//         self.asymmetric_disconnect(id2, id1);
//     }
    
//     pub fn delete(&mut self, id: WatchpointID) {
//         let wp = self.logic.remove(&id);
        
//         if let Some(wp) = wp {
//             match wp {
//                 WatchpointLogic::WatchpointAnd { in1_id, in2_id, out_id, .. } => {
//                     if in1_id.is_some() { self.asymmetric_disconnect(in1_id.unwrap(), id); }
//                     if in2_id.is_some() { self.asymmetric_disconnect(in2_id.unwrap(), id); }
//                     if out_id.is_some() { self.asymmetric_disconnect(out_id.unwrap(), id); }
//                 }
//                 WatchpointLogic::WatchpointOr { in1_id, in2_id, out_id, .. } => {
//                     if in1_id.is_some() { self.asymmetric_disconnect(in1_id.unwrap(), id); }
//                     if in2_id.is_some() { self.asymmetric_disconnect(in2_id.unwrap(), id); }
//                     if out_id.is_some() { self.asymmetric_disconnect(out_id.unwrap(), id); }
//                 }
//                 WatchpointLogic::WatchpointNot { in_id, out_id, .. } => {
//                     if in_id.is_some() { self.asymmetric_disconnect(in_id.unwrap(), id); }
//                     if out_id.is_some() { self.asymmetric_disconnect(out_id.unwrap(), id); }
//                 }
//                 WatchpointLogic::WatchpointBreak { in_id } => {
//                     if in_id.is_some() { self.asymmetric_disconnect(in_id.unwrap(), id); }
//                 }
//             }
//         }
        
//         let wp = self.triggerables.remove(&id);
        
//         if let Some(wp) = wp {
//             match wp {
//                 WatchpointTrigger::ManualToggle { out_id } => {
//                     if out_id.is_some() { self.asymmetric_disconnect(out_id.unwrap(), id); }
//                 }
//             }
//         }
//     }
    
//     pub fn check_triggers(&mut self, snem_core: &core::snemcore::Snemulator) -> bool {
//         let mut needs_update = std::collections::HashMap::new();
        
//         for (id, wp) in self.triggerables.iter_mut() {
//             let old_val = wp.value();
//             wp.check(snem_core);
//             let new_val = wp.value();
//             if new_val != old_val {
//                 if let Some(out_id) = wp.out_id() {
//                     needs_update.insert(out_id, (*id, new_val));
//                 }
//             }
//         }
        
//         let mut brk_triggered = false;
        
//         for (out_id, (id, new_val)) in needs_update {
//             brk_triggered = self.update(out_id, id, new_val);
//         }
        
//         brk_triggered
//     }
    
//     fn update(&mut self, id: WatchpointID, in_id: WatchpointID, in_val: bool) -> bool {
//         let logic = self.logic.get_mut(&id).unwrap();
        
//         let old_val = logic.value();
//         logic.update(in_id, in_val);
//         let new_val = logic.value();
        
//         if new_val != old_val {
//             if let Some(out_id) = logic.out_id() {
//                 return self.update(out_id, id, new_val);
//             }
//         }

//         match logic {
//             WatchpointLogic::WatchpointBreak { .. } => true,
//             _ => false,
//         }
//     }
// }


// // pub enum WatchpointTriggerKind {
// //     Toggleable
// // }

// // pub enum WatchpointLogicKind {
// //     And,
// //     Or,
// //     Not,
// //     Break,
// // }

// // pub struct WatchpointBuilder {}

// // impl WatchpointBuilder {
// //     pub fn triggerable(self, kind: WatchpointTriggerKind) -> Self {
        
// //         self
// //     }
// // }




// // pub enum Watchpoint {
// //     CpuRegister8 { reg: CpuReg8, trigger: U8Condition },
// //     CpuRegister16 { reg: CpuReg16, trigger: U16Condition },
// //     HwRegister  { reg: HwReg,  trigger_value: Option<u8> },
// //     Memory      { addr: u32,   trigger_value: Option<u8> },
// //     Event       { kind: EventKind },
// // }

// // pub enum CpuReg8 {
// //     A, X, Y, SP, DB,
// //     P {
// //         flag: Option<core::scpu::Flag> 
// //     }
// // }
// // pub enum CpuReg16 {
// //     PC, DB, DP, A, X, Y
// // }
// // pub enum HwReg  { Nmitimen, Inidisp, /* ... */ }
// // pub enum EventKind { Nmi, Irq, Reset, BrkInstruction }

// // pub enum U8Condition {
// //     Equals { value: u8 },
// //     OrEquals { operand: u8, value: u8 },
// //     AndEquals { operand: u8, value: u8 },
// //     Changes,
// //     OrChanges { operand: u8 },
// //     AndChanges { operand: u8 },
// // }

// // pub enum U16Condition {
// //     Equals { value: u16 },
// //     OrEquals { operand: u16, value: u16 },
// //     AndEquals { operand: u16, value: u16 },
// //     Changes,
// //     OrChanges { operand: u16 },
// //     AndChanges { operand: u16 },
// // }