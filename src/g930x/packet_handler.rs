use g930x::events::G930KeyEvent;

pub trait EventHandler {
    fn new() -> Self;
    fn handle(&mut self, event: &G930KeyEvent) -> Result<(), &'static str>;
}
