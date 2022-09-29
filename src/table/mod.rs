use self::dnl::DeadNonceList;
use self::pit::PIT;

pub mod dnl;
pub mod pit;

const DNL_MAX_LENGTH: usize = 4096;

pub struct Table {
    pub dnl: DeadNonceList,
    pub pit: PIT,
}

impl Table {
    pub fn new() -> Table {
        Table {
            dnl: DeadNonceList::new(DNL_MAX_LENGTH),
            pit: PIT::new(),
        }
    }

    pub fn clean(&mut self) {
        self.dnl.clean();
    }
}
