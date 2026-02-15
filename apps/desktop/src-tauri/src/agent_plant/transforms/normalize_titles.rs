use opscinema_types::Step;

pub fn apply(steps: &mut [Step]) {
    for s in steps {
        s.title = s.title.trim().replace("  ", " ");
    }
}
