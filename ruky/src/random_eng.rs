use crate::ruky::Ruky;
use std::cell::RefCell;
use uzi::eng::Eng;
use uzi::engtx::UziOut;
use uzi::err::UziErr;
use uzi::guicmd::Pos;

#[derive(Clone, Debug)]
struct RandomEng {
    ruky: Ruky,
    uzi_out: UziOut,
    pos: RefCell<Option<Pos>>,
}

impl Eng for RandomEng {
    fn position(&mut self, pos: &Pos) -> Result<(), UziErr> {
        self.pos.borrow_mut().replace(pos.clone());
        Ok(())
    }
}
