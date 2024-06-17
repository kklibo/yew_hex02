#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Diff {
    Same,
    Different,
    NoOther,
}

pub fn get_diffs(a: &[u8], b: &[u8]) -> (Vec<Diff>, Vec<Diff>) {
    let mut a = a.iter();
    let mut b = b.iter();
    let mut a_diff = Vec::new();
    let mut b_diff = Vec::new();
    loop {
        let a_next = a.next();
        let b_next = b.next();
        match (a_next, b_next) {
            (Some(a), Some(b)) => {
                if a == b {
                    a_diff.push(Diff::Same);
                    b_diff.push(Diff::Same);
                } else {
                    a_diff.push(Diff::Different);
                    b_diff.push(Diff::Different);
                }
            }
            (Some(_), None) => {
                a_diff.push(Diff::NoOther);
            }
            (None, Some(_)) => {
                b_diff.push(Diff::NoOther);
            }
            (None, None) => break,
        }
    }
    (a_diff, b_diff)
}
