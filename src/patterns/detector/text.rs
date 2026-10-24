//! The report as short human text.

use std::fmt;

use super::{Report, Symmetry};

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_matches(f, &self.matches)?;
        write_fingerprint(f, &self.fingerprint)?;
        write_symmetry(f, &self.symmetry)
    }
}

fn write_symmetry(f: &mut fmt::Formatter<'_>, symmetry: &Symmetry) -> fmt::Result {
    let order = symmetry.order();
    writeln!(
        f,
        "symmetry: {order} of 48 operations fix it ({} rotations, {} mirrored)",
        symmetry.rotations(),
        order - symmetry.rotations()
    )?;
    writeln!(
        f,
        "  {} of them keep every colour, {} rename colours",
        symmetry.colour_preserving(),
        order - symmetry.colour_preserving()
    )?;
    let mut classes: Vec<(&str, Vec<&super::Operation>)> = Vec::new();
    for operation in &symmetry.operations {
        match classes.iter_mut().find(|(c, _)| *c == operation.class) {
            Some((_, list)) => list.push(operation),
            None => classes.push((operation.class, vec![operation])),
        }
    }
    for (class, operations) in classes {
        let mut axes: Vec<(String, usize)> = Vec::new();
        for o in operations.iter().filter(|o| !o.axis.is_empty()) {
            let text = match o.colour_swap.as_str() {
                "" => o.axis.clone(),
                swap => format!("{} renaming {swap}", o.axis),
            };
            match axes.iter_mut().find(|(t, _)| *t == text) {
                Some((_, n)) => *n += 1,
                None => axes.push((text, 1)),
            }
        }
        let swapped = operations.iter().filter(|o| !o.keeps_colours()).count();
        if axes.is_empty() || axes.len() > 8 {
            let note = if swapped == 0 {
                String::new()
            } else {
                format!(", {swapped} renaming colours")
            };
            writeln!(f, "  {} x {class}{note}", operations.len())?;
        } else {
            let list: Vec<String> = axes
                .iter()
                .map(|(t, n)| {
                    if *n > 1 {
                        format!("{t} (x{n})")
                    } else {
                        t.clone()
                    }
                })
                .collect();
            writeln!(f, "  {} x {class}: {}", operations.len(), list.join(", "))?;
        }
    }
    Ok(())
}

fn write_matches(f: &mut fmt::Formatter<'_>, matches: &[super::Match]) -> fmt::Result {
    if matches.is_empty() {
        return writeln!(f, "pattern: no library match");
    }
    for m in matches {
        writeln!(f, "pattern: {} ({}), {}", m.name, m.id, m.relation)?;
    }
    Ok(())
}

fn write_fingerprint(f: &mut fmt::Formatter<'_>, fingerprint: &super::Fingerprint) -> fmt::Result {
    writeln!(f, "fingerprint: {}", fingerprint.code())?;
    for orbit in &fingerprint.orbits {
        let what = match orbit.name.split_once(' ') {
            Some(("wings", depth)) => format!("wings (depth {depth})"),
            _ => orbit.name.clone(),
        };
        let unit = if orbit.name == "centres" {
            "centre stickers"
        } else {
            "pieces"
        };
        let mut parts = vec![format!("{} {unit}", orbit.pieces)];
        if orbit.name == "centres" {
            parts.push(format!("{} misplaced", orbit.displaced));
        } else {
            parts.push(format!("{} displaced", orbit.displaced));
            let word = if orbit.name == "corners" {
                "twisted"
            } else {
                "flipped"
            };
            parts.push(format!("{} {word}", orbit.oriented_wrong));
        }
        match &orbit.cycles {
            Some(c) if !c.is_empty() => {
                let list: Vec<String> = c.iter().map(ToString::to_string).collect();
                parts.push(format!("cycles of length {}", list.join(", ")));
            }
            _ => {}
        }
        writeln!(f, "  {what}: {}", parts.join(", "))?;
    }
    Ok(())
}
