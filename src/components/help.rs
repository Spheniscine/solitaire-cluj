use dioxus::prelude::*;

use crate::{components::{VIDEO_GAMEPLAY, rem}, game::{GameState, ScreenState}};

#[component]
fn Emph(children: Element) -> Element {
    rsx! {
        strong {
            color: "#ff0",
            {children}
        }
    }
}

#[component]
pub fn Help(game_state: Signal<GameState>) -> Element {
    // let st = game_state.read();
    // let skin = st.skin;

    rsx! {
        div {
            style: "display: flex; flex-direction: column; align-items: center; font-size: 4.5rem; color: #fff; padding: 4rem;",
            class: "help",

            div {
                text_align: "left",

                p {
                    margin_top: "0",
                    "The deck has 36 cards: 9 ranks in 4 suits. Suits are ignored in this game and can be disabled in settings."
                }

                p {
                    "Cards are stacked by ",Emph{"descending rank"},". Cards in order can be moved as a stack."
                }

                p {
                    Emph {"NOTE:"}, " To move cards, click to select a card or stack, then click the destination. ", Emph{"“Drag and drop” is not required."}
                }

                p {
                    "A stack of 9 descending cards, alone in a column, will collapse into a ",Emph{"locked stack"},"."
                }

                p {
                    "You may ",Emph{"cheat"}," by moving single cards to invalid positions. Cheated cards are denoted with a dark background."
                }

                p {
                    "You may not stack further cards on cheated cards. You may only move a cheated card to a valid position, which will restore it to normal."
                }

                p {
                    "To ",Emph{"win the game"},", collapse all cards into 4 locked stacks."
                }

                div {
                    position: "absolute",
                    bottom: rem(2.),
                    width: "92rem",
                    display: "flex",
                    justify_content: "center",

                    a {
                        href: VIDEO_GAMEPLAY,
                        target: "_blank",
                        text_decoration: "none",
                        margin_right: rem(4.),
                        div {
                            width: rem(30.),
                            position: "relative",
                            class: "game-button",
                            "Example video"
                        }
                    }

                    div {
                        width: rem(30.),
                        position: "relative",
                        class: "game-button",
                        onclick: move |_| game_state.write().screen_state = ScreenState::Game,
                        "Back to game"
                    }
                }
                
            }
        }
        
    }
}