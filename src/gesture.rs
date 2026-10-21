//! Pure gesture and policy logic, shared by the input service and its tests.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Phase {
    #[default]
    Idle,
    Pending,
    Fired,
    Pass,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Action {
    #[default]
    Pass,
    Consume,
    ReplayClick,
    ReplayDown,
    Show,
}

#[derive(Default)]
pub struct Hold {
    phase: Phase,
    pub origin: Point,
    start: u64,
}

impl Hold {
    pub fn down(&mut self, allowed: bool, at: Point, now: u64) -> Action {
        self.origin = at;
        self.start = now;
        self.phase = if allowed { Phase::Pending } else { Phase::Pass };
        if allowed {
            Action::Consume
        } else {
            Action::Pass
        }
    }
    pub fn pending(&self) -> bool {
        self.phase == Phase::Pending
    }
    pub fn tick(&mut self, allowed: bool, now: u64, delay: u32) -> Action {
        if self.pending() && !allowed {
            self.phase = Phase::Fired;
        }
        if self.pending() && now.saturating_sub(self.start) >= delay as u64 {
            self.phase = Phase::Fired;
            return Action::Show;
        }
        Action::Pass
    }
    pub fn moved(&mut self, at: Point) -> Action {
        if self.pending()
            && ((at.x as i64 - self.origin.x as i64).abs() > 8
                || (at.y as i64 - self.origin.y as i64).abs() > 8)
        {
            self.phase = Phase::Pass;
            return Action::ReplayDown;
        }
        Action::Pass
    }
    pub fn cancel(&mut self) {
        if self.pending() {
            self.phase = Phase::Fired;
        }
    }
    pub fn up(&mut self) -> Action {
        match std::mem::take(&mut self.phase) {
            Phase::Pending => Action::ReplayClick,
            Phase::Fired => Action::Consume,
            _ => Action::Pass,
        }
    }
}

pub fn exe_name(path: &str) -> Option<String> {
    let name = path
        .trim()
        .trim_matches('"')
        .rsplit(['\\', '/'])
        .next()?
        .trim()
        .to_lowercase();
    (name.len() > 4 && name.ends_with(".exe")).then_some(name)
}

pub fn excluded(exe: &str, apps: &[String]) -> bool {
    exe_name(exe).is_some_and(|name| apps.iter().any(|p| exe_name(p).as_deref() == Some(&name)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn short_click_and_hold_have_distinct_release_behavior() {
        let mut h = Hold::default();
        assert_eq!(h.down(true, Point::default(), 0), Action::Consume);
        assert_eq!(h.tick(true, 399, 400), Action::Pass);
        assert_eq!(h.up(), Action::ReplayClick);
        h.down(true, Point::default(), 1000);
        assert_eq!(h.tick(true, 1400, 400), Action::Show);
        assert_eq!(h.tick(true, 1800, 400), Action::Pass);
        assert_eq!(h.up(), Action::Consume);
        assert_eq!(h.up(), Action::Pass);
    }
    #[test]
    fn excluded_clicks_and_dragging_keep_native_input() {
        let mut h = Hold::default();
        assert_eq!(h.down(false, Point::default(), 0), Action::Pass);
        assert_eq!(h.tick(true, 1000, 400), Action::Pass);
        assert_eq!(h.up(), Action::Pass);
        h.down(true, Point { x: -20, y: 30 }, 0);
        assert_eq!(h.moved(Point { x: -10, y: 30 }), Action::ReplayDown);
        assert_eq!(h.tick(true, 1000, 400), Action::Pass);
        assert_eq!(h.up(), Action::Pass);
    }
    #[test]
    fn focus_change_or_pause_cancels_pending_hold_without_orphan_release() {
        let mut h = Hold::default();
        h.down(true, Point::default(), 0);
        h.cancel();
        assert_eq!(h.moved(Point { x: 100, y: 100 }), Action::Pass);
        assert_eq!(h.tick(true, 500, 400), Action::Pass);
        assert_eq!(h.up(), Action::Consume);
        h.down(true, Point::default(), 0);
        h.tick(false, 500, 400);
        assert_eq!(h.up(), Action::Consume);
    }
    #[test]
    fn exclusions_are_exact_case_insensitive_executable_names() {
        let apps = vec!["CS2.EXE".into(), "sekiro.exe".into()];
        assert!(excluded(r"D:\Steam\cs2.exe", &apps));
        assert!(!excluded("my-cs2.exe", &apps));
        assert!(!excluded("steam.exe", &apps));
        assert_eq!(exe_name(r#" "D:\Games\CS2.EXE" "#), Some("cs2.exe".into()));
        assert_eq!(exe_name(""), None);
    }
}
