use dioxus::prelude::*;

use crate::{components::{Emoji, SkinTrait}, game::{Card, ColorMode, Skin, SuitSkin}};

pub const KATEX_MAIN: &str = "KaTeX_Main";

impl Skin {
    fn render_suit_internal(&self, card: &Card, text_mode: bool) -> Element {
        if self.suits == SuitSkin::Animals {
            rsx! {
                Emoji { 
                    text: self.suits.suit_symbol(card.suit)
                }
            }
        } else {
            rsx! {
                span {
                    font_family: self.suits.font(),
                    position: if !text_mode && (self.suits == SuitSkin::Shapes) {"relative"},
                    top: if !text_mode && self.suits == SuitSkin::Shapes {"0.11em"},
                    {self.suits.suit_symbol(card.suit)}
                }
            }
        }
    }
}

impl SkinTrait<Card> for Skin {
    fn get_color(&self, card: &Card, mode: ColorMode) -> String {
        self.colors.color(card.suit, mode).to_string()
    }

    fn render_rank(&self, card: &Card) -> Element {
        rsx! {
            span {
                font_family: KATEX_MAIN,
                {self.ranks.rank_text(card.rank)}
            }
        }
    }

    fn render_suit(&self, card: &Card) -> Element {
        self.render_suit_internal(card, false)
    }

    // fn render_suit_text(&self, card: &Card) -> Element {
    //     self.render_suit_internal(card, true)
    // }
}