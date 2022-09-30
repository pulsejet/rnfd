use std::{cell::RefCell, rc::Rc};
use crate::table::pit::PITNode;

pub struct Table {
    pub rib: Rc<RefCell<PITNode>>
}