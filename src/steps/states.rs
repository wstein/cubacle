//! `recover --icons`: the cube as it should be and as it is, side by side.

use cubacle::notation::orbit64;
use cubacle::puzzle::Facelets;
use cubacle::render::{Raster, Style};

use super::{Graphics, Pictures};
use crate::rows;
use crate::views::{kitty_block, pair_fits, soft_outline};

/// Pixels between the two pictures.
const BETWEEN: u32 = 16;

fn token_line(label: &str, state: &Facelets) -> Result<String, String> {
    let token = orbit64::encode(state).map_err(|error| error.to_string())?;
    Ok(format!("{label}: {token}\n"))
}

impl Pictures<'_> {
    /// The token lines `Should be: TOKEN` and `Is now: TOKEN`, then the two
    /// pictures at the size of the start picture, the second marked where it
    /// differs from the first. On a graphics terminal they are one image, left
    /// and right, when they fit the width (and `--columns` allows two); else
    /// they are stacked, each under its label (`should_label`, `Is now`), as
    /// they are in text.
    pub fn states_against(
        &self,
        (should_label, should): (&str, &Facelets),
        now: &Facelets,
    ) -> Result<String, String> {
        let mut out = token_line("Should be", should)?;
        out.push_str(&token_line("Is now", now)?);
        let views = self.options.views;
        let kitty = self.graphics == Graphics::Kitty;
        let style = if kitty {
            Style {
                size: self.style.size * 3 / 2,
                ..self.style.clone()
            }
        } else {
            self.style.clone()
        };
        let left = views.icons(should, None, &style);
        let right = views.icons(now, Some(should), &style);
        if kitty {
            let draw = |drawn: &[cubacle::render::Icon]| {
                let rasters: Vec<Raster> = drawn
                    .iter()
                    .map(|d| d.clone().unframed().to_rgba(style.size))
                    .collect();
                soft_outline(&Raster::hstack(&rasters, rows::GAP))
            };
            let (a, b) = (draw(&left), draw(&right));
            let both = Raster::hstack(&[a.clone(), b.clone()], BETWEEN);
            let columns = self.options.columns;
            if pair_fits(self.terminal.width_px, both.width, columns) {
                out.push_str(&format!("Left: {should_label}. Right: Is now.\n"));
                out.push_str(&kitty_block(&both, self.terminal.cell.1));
                return Ok(out);
            }
            for (label, raster) in [(should_label, &a), ("Is now", &b)] {
                out.push_str(&format!("{label}\n"));
                out.push_str(&kitty_block(raster, self.terminal.cell.1));
            }
            return Ok(out);
        }
        for (label, drawn) in [(should_label, &left), ("Is now", &right)] {
            out.push_str(&format!("{label}\n"));
            out.push_str(&self.picture(drawn, &style));
        }
        Ok(out)
    }
}
