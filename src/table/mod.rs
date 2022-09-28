use self::dnl::DeadNonceList;

pub mod dnl;

const DNL_MAX_LENGTH: usize = 4096;

pub struct Table {
    pub dnl: DeadNonceList,
}

impl Table {
    pub fn new() -> Table {
        Table {
            dnl: DeadNonceList::new(DNL_MAX_LENGTH),
        }
    }

    pub fn clean(&mut self) {
        self.dnl.clean();
    }
}
