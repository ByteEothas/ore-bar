use crate::{
    circular::Circular,
    consts::{ACCOUNT_DETAIL_HIGHT, ACCOUNT_DETAIL_WIDTH, MAX_ITEMS_PER_ROW, SUBHEAD_TEXT},
    easing, style,
    utils::{abbreviate, get_domain},
    Account, ContentType, Dashboard, Message,
};
use iced::widget::{
    button, column, container, horizontal_space, row, scrollable, svg, text, text_input,
    vertical_space, Column, Row,
};
use iced::{theme, Element, Length};
use std::time::Duration;

pub fn get_svg<'a>(is_online: bool) -> Element<'a, Message> {
    let handle = svg::Handle::from_path(format!(
        "{}/resources/{}.svg",
        env!("CARGO_MANIFEST_DIR"),
        if is_online { "green" } else { "red" }
    ));
    Element::new(svg(handle).width(3).height(15))
}

pub fn get_svg_icon<'a>(name: &str, width: u16, height: u16) -> Element<'a, Message> {
    let handle = svg::Handle::from_path(format!(
        "{}/resources/{}.svg",
        env!("CARGO_MANIFEST_DIR"),
        name,
    ));
    Element::new(svg(handle).width(width).height(height))
}

/// Returns a list of account details.
pub fn get_content_list<'a>(dashboard: &'a Dashboard) -> Element<'a, Message> {
    let mut columns = column![];
    let mut rows = row![].spacing(5);
    let mut count = 0;

    // Load accounts
    for (i, a) in dashboard.accounts.iter().enumerate() {
        if count >= MAX_ITEMS_PER_ROW + dashboard.extend_items_per_row {
            columns = columns.push(rows);
            rows = row![].spacing(5);
            count = 0;
        }

        rows = rows.push(
            container(get_content2(i, &a))
                .width(ACCOUNT_DETAIL_WIDTH)
                .height(ACCOUNT_DETAIL_HIGHT)
                .style(style::rounded_box),
        );
        count += 1;
    }
    if count > 0 {
        columns = columns.push(rows);
    }
    scrollable(columns.width(Length::Fill).padding(5).spacing(5))
        .height(Length::Fill)
        .into()
}

/// Displays detailed content for an account.
pub fn get_content2<'a>(index: usize, account: &'a Account) -> Element<'a, Message> {
    let status = &account.status;
    let prepared = account.prepared;
    if !prepared {
        return column!(Element::new(
            Circular::new()
                .easing(&easing::EMPHASIZED)
                .cycle_duration(Duration::from_secs_f32(2.0))
                .size(40.0)
                .bar_height(4.0),
        ))
        .align_items(iced::Alignment::Center)
        .into();
    }
    if status.is_valid {
        column![
            container(
                row![
                    text("Account:"),
                    text(abbreviate(&status.authority.to_string())),
                    horizontal_space(),
                    get_svg(status.is_online),
                ]
                .spacing(5)
            )
            .style(if status.is_online {
                style::rounded_box
            } else {
                style::warn_box
            }),
            row![text("Balance:"), text(&status.balance)].spacing(5),
            row![text("Stake:"), text(&status.stake)].spacing(5),
            row![
                text("SOL: ").size(SUBHEAD_TEXT),
                text(&status.sol_balance).size(SUBHEAD_TEXT)
            ]
            .spacing(5),
            row![
                text("Last hash time: ").size(SUBHEAD_TEXT),
                text(&status.last_hash_at).size(SUBHEAD_TEXT)
            ]
            .spacing(5),
            row![
                text("Last stake time:").size(SUBHEAD_TEXT),
                text(&status.last_stake_at).size(SUBHEAD_TEXT)
            ]
            .spacing(5),
            row![
                text("Total hashes:").size(SUBHEAD_TEXT),
                text(&status.total_hashes).size(SUBHEAD_TEXT)
            ]
            .spacing(5),
            row![
                text("Total rewards:").size(SUBHEAD_TEXT),
                text(&status.total_rewards).size(SUBHEAD_TEXT)
            ]
            .spacing(5),
            row![
                text("Rpc:").size(SUBHEAD_TEXT),
                text(get_domain(&account.json_rpc_url)).size(SUBHEAD_TEXT)
            ]
            .spacing(5),
            vertical_space(),
            column![row![
                button("Claim")
                    .on_press(Message::SetModalView(Some(index), claim_view))
                    .style(theme::Button::Positive),
                button("Stake").on_press(Message::SetModalView(Some(index), stake_view)),
                button("Remove").on_press(Message::SetModalView(Some(index), remove_account_view)),
            ]
            .spacing(5)]
            .width(Length::Fill)
            .align_items(iced::Alignment::Center),
        ]
        .padding(5)
        .spacing(8)
        .height(Length::Fill)
        .width(Length::Fill)
        .align_items(iced::Alignment::Start)
        .into()
    } else {
        column![
            row![
                text("Account:"),
                text(abbreviate(&status.authority.to_string())),
                horizontal_space(),
                get_svg(status.is_online),
            ]
            .spacing(5),
            row![
                text("Rpc:").size(SUBHEAD_TEXT),
                text(get_domain(&account.json_rpc_url)).size(SUBHEAD_TEXT)
            ]
            .spacing(5),
            text("Miner account doesn't exist"),
            vertical_space(),
            column![button("Remove").on_press(Message::RemoveAccount(index))]
                .width(Length::Fill)
                .align_items(iced::Alignment::Center)
        ]
        .padding(5)
        .spacing(8)
        .height(Length::Fill)
        .width(Length::Fill)
        .align_items(iced::Alignment::Start)
        .into()
    }
}

pub fn add_account_view<'a>(dashboard: &Dashboard) -> Element<'a, Message> {
    container(
        column![
            text("Add an account").size(24),
            text("Json rpc url"),
            text_input("", &dashboard.json_rpc_url).on_input(Message::JsonRpcUrl),
            text("Key pair").size(12),
            row![
                text_input("File path", &dashboard.keypair)
                    .on_input(Message::Keypair)
                    .on_submit(Message::AddAccount),
                button(text("Open")).on_press(Message::OpenFile)
            ]
            .spacing(10),
            text("Priority fee"),
            text_input("", &dashboard.priority_fee).on_input(Message::PriorityFee),
            container(button(text("Add")).on_press(Message::AddAccount))
                .align_x(iced::alignment::Horizontal::Center),
        ]
        .spacing(10),
    )
    .width(400)
    .padding(10)
    .into()
}

pub fn remove_account_view<'a>(dashboard: &'a Dashboard) -> Element<'a, Message> {
    if let Some(index) = dashboard.current_index {
        let account = dashboard.accounts.get(index).expect("No account selected");
        let pubkey = account.status.authority.to_string();
        container(
            Column::new()
                .push(text(format!("Remove {} account?",pubkey)).size(16))
                .push(text("Removing this account will only take it off the view list. You can add it back later if needed.").size(18))
                .spacing(20)
                .padding(20)
                .push(
                    Row::new()
                        .spacing(20)
                        .push(button(text("Yes")).on_press(Message::RemoveAccount(index)))
                        .push(button(text("Cancel")).on_press(Message::HideModal(None))),
                ),
        )
        .width(600)
        .padding(10)
        .into()
    } else {
        text("No account selected").into()
    }
}

pub fn claim_view<'a>(dashboard: &Dashboard) -> Element<'a, Message> {
    container(
        column![
            text("Claim ore to wallet").size(24),
            column![
                column![
                    text("Wallet address").size(12),
                    text_input("(optional)", &dashboard.claim_address)
                        .on_input(Message::ClaimAddress)
                        .on_submit(Message::SetModalView(None, claim_confirm_view))
                        .padding(5),
                    text("Amount").size(12),
                    text_input("(optional)", &dashboard.claim_amount)
                        .on_input(Message::ClaimAmount)
                        .on_submit(Message::SetModalView(None, claim_confirm_view))
                        .padding(5),
                ]
                .spacing(5),
                row![
                    button(text("Claim")).on_press(Message::SetModalView(None, claim_confirm_view)),
                    button(text("Cancel")).on_press(Message::HideModal(None))
                ]
                .spacing(10),
            ]
            .spacing(10)
        ]
        .spacing(20),
    )
    .width(300)
    .padding(10)
    .into()
}

pub fn claim_confirm_view<'a>(dashboard: &'a Dashboard) -> Element<'a, Message> {
    container(
        Column::new()
            .push(text("Confirm ore claim request").size(24))
            .push(text(&dashboard.claim_address).size(16))
            .push(row![
                text(if *dashboard.claim_amount == String::default() {
                    "MAX available"
                } else {
                    &dashboard.claim_amount
                }),
                text(" ORE")
            ])
            .spacing(20)
            .padding(20)
            .push(
                Row::new()
                    .spacing(20)
                    .push(
                        button(text("Yes")).on_press_maybe(if !&dashboard.is_claim_process {
                            Some(Message::Claim)
                        } else {
                            None
                        }),
                    )
                    .push(button(text("Cancel")).on_press(Message::HideModal(None))),
            ),
    )
    .width(500)
    .padding(10)
    .into()
}

pub fn stake_view<'a>(dashboard: &Dashboard) -> Element<'a, Message> {
    container(
        column![
            text("Stake ore to wallet").size(24),
            column![
                column![
                    text("Amount").size(12),
                    text_input("(optional)", &dashboard.stake_amount)
                        .on_input(Message::StakeAmount)
                        .on_submit(Message::SetModalView(None, stake_confirm_view))
                        .padding(5),
                ]
                .spacing(5),
                row![
                    button(text("Stake")).on_press(Message::SetModalView(None, stake_confirm_view)),
                    button(text("Cancel")).on_press(Message::HideModal(None))
                ]
                .spacing(10)
            ]
            .spacing(10)
        ]
        .spacing(20),
    )
    .width(300)
    .padding(10)
    .into()
}

pub fn stake_confirm_view<'a>(dashboard: &'a Dashboard) -> Element<'a, Message> {
    container(
        Column::new()
            .push(text("Confirm ore stake request").size(24))
            .push(row![
                text(if *dashboard.stake_amount == String::default() {
                    "MAX available"
                } else {
                    &dashboard.stake_amount
                }),
                text(" ORE")
            ])
            .spacing(20)
            .padding(20)
            .push(
                Row::new()
                    .spacing(20)
                    .push(
                        button(text("Yes")).on_press_maybe(if !&dashboard.is_stake_process {
                            Some(Message::Stake)
                        } else {
                            None
                        }),
                    )
                    .push(button(text("Cancel")).on_press(Message::HideModal(None))),
            ),
    )
    .width(500)
    .padding(10)
    .into()
}

pub fn active_num_view<'a>(dashboard: &'a Dashboard) -> Element<'a, Message> {
    let active_num = dashboard.active_num;
    let all_num = dashboard.accounts.len();
    let mut row = row![];
    for _ in 0..active_num {
        row = row.push(get_svg(true));
    }
    // TODO Attempted subtraction resulting in integer overflow
    for _ in 0..all_num - active_num {
        row = row.push(get_svg(false));
    }
    row.spacing(1).into()
}

pub fn dialog_view<'a>(dashboard: &Dashboard) -> Element<'a, Message> {
    let command = match dashboard.dialog.content_type {
        ContentType::Normal => Some(Box::new(Message::Refresh)),
        ContentType::Good => Some(Box::new(Message::Refresh)),
        ContentType::Error => None,
    };
    container(
        Column::new()
            .push(text(dashboard.dialog.content.clone()).size(24))
            .spacing(20)
            .padding(20)
            .push(
                Row::new()
                    .spacing(20)
                    .push(button(text("Ok")).on_press(Message::HideModal(command))),
            ),
    )
    .width(300)
    .padding(10)
    .style(style::rounded_box)
    .into()
}

pub mod modal {
    use iced::advanced::layout::{self, Layout};
    use iced::advanced::overlay;
    use iced::advanced::renderer;
    use iced::advanced::widget::{self, Widget};
    use iced::advanced::{self, Clipboard, Shell};
    use iced::alignment::Alignment;
    use iced::event;
    use iced::mouse;
    use iced::{Color, Element, Event, Length, Point, Rectangle, Size, Vector};

    /// A widget that centers a modal element over some base element
    pub struct Modal<'a, Message, Theme, Renderer> {
        base: Element<'a, Message, Theme, Renderer>,
        modal: Element<'a, Message, Theme, Renderer>,
        on_blur: Option<Message>,
    }

    impl<'a, Message, Theme, Renderer> Modal<'a, Message, Theme, Renderer> {
        /// Returns a new [`Modal`]
        pub fn new(
            base: impl Into<Element<'a, Message, Theme, Renderer>>,
            modal: impl Into<Element<'a, Message, Theme, Renderer>>,
        ) -> Self {
            Self {
                base: base.into(),
                modal: modal.into(),
                on_blur: None,
            }
        }

        /// Sets the message that will be produces when the background
        /// of the [`Modal`] is pressed
        pub fn on_blur(self, on_blur: Message) -> Self {
            Self {
                on_blur: Some(on_blur),
                ..self
            }
        }
    }

    impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
        for Modal<'a, Message, Theme, Renderer>
    where
        Renderer: advanced::Renderer,
        Message: Clone,
    {
        fn children(&self) -> Vec<widget::Tree> {
            vec![
                widget::Tree::new(&self.base),
                widget::Tree::new(&self.modal),
            ]
        }

        fn diff(&self, tree: &mut widget::Tree) {
            tree.diff_children(&[&self.base, &self.modal]);
        }

        fn size(&self) -> Size<Length> {
            self.base.as_widget().size()
        }

        fn layout(
            &self,
            tree: &mut widget::Tree,
            renderer: &Renderer,
            limits: &layout::Limits,
        ) -> layout::Node {
            self.base
                .as_widget()
                .layout(&mut tree.children[0], renderer, limits)
        }

        fn on_event(
            &mut self,
            state: &mut widget::Tree,
            event: Event,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            renderer: &Renderer,
            clipboard: &mut dyn Clipboard,
            shell: &mut Shell<'_, Message>,
            viewport: &Rectangle,
        ) -> event::Status {
            self.base.as_widget_mut().on_event(
                &mut state.children[0],
                event,
                layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            )
        }

        fn draw(
            &self,
            state: &widget::Tree,
            renderer: &mut Renderer,
            theme: &Theme,
            style: &renderer::Style,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            viewport: &Rectangle,
        ) {
            self.base.as_widget().draw(
                &state.children[0],
                renderer,
                theme,
                style,
                layout,
                cursor,
                viewport,
            );
        }

        fn overlay<'b>(
            &'b mut self,
            state: &'b mut widget::Tree,
            layout: Layout<'_>,
            _renderer: &Renderer,
            translation: Vector,
        ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
            Some(overlay::Element::new(Box::new(Overlay {
                position: layout.position() + translation,
                content: &mut self.modal,
                tree: &mut state.children[1],
                size: layout.bounds().size(),
                on_blur: self.on_blur.clone(),
            })))
        }

        fn mouse_interaction(
            &self,
            state: &widget::Tree,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            viewport: &Rectangle,
            renderer: &Renderer,
        ) -> mouse::Interaction {
            self.base.as_widget().mouse_interaction(
                &state.children[0],
                layout,
                cursor,
                viewport,
                renderer,
            )
        }

        fn operate(
            &self,
            state: &mut widget::Tree,
            layout: Layout<'_>,
            renderer: &Renderer,
            operation: &mut dyn widget::Operation<Message>,
        ) {
            self.base
                .as_widget()
                .operate(&mut state.children[0], layout, renderer, operation);
        }
    }

    struct Overlay<'a, 'b, Message, Theme, Renderer> {
        position: Point,
        content: &'b mut Element<'a, Message, Theme, Renderer>,
        tree: &'b mut widget::Tree,
        size: Size,
        on_blur: Option<Message>,
    }

    impl<'a, 'b, Message, Theme, Renderer> overlay::Overlay<Message, Theme, Renderer>
        for Overlay<'a, 'b, Message, Theme, Renderer>
    where
        Renderer: advanced::Renderer,
        Message: Clone,
    {
        fn layout(&mut self, renderer: &Renderer, _bounds: Size) -> layout::Node {
            let limits = layout::Limits::new(Size::ZERO, self.size)
                .width(Length::Fill)
                .height(Length::Fill);

            let child = self
                .content
                .as_widget()
                .layout(self.tree, renderer, &limits)
                .align(Alignment::Center, Alignment::Center, limits.max());

            layout::Node::with_children(self.size, vec![child]).move_to(self.position)
        }

        fn on_event(
            &mut self,
            event: Event,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            renderer: &Renderer,
            clipboard: &mut dyn Clipboard,
            shell: &mut Shell<'_, Message>,
        ) -> event::Status {
            let content_bounds = layout.children().next().unwrap().bounds();

            if let Some(message) = self.on_blur.as_ref() {
                if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) = &event {
                    if !cursor.is_over(content_bounds) {
                        shell.publish(message.clone());
                        return event::Status::Captured;
                    }
                }
            }

            self.content.as_widget_mut().on_event(
                self.tree,
                event,
                layout.children().next().unwrap(),
                cursor,
                renderer,
                clipboard,
                shell,
                &layout.bounds(),
            )
        }

        fn draw(
            &self,
            renderer: &mut Renderer,
            theme: &Theme,
            style: &renderer::Style,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
        ) {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: layout.bounds(),
                    ..renderer::Quad::default()
                },
                Color {
                    a: 0.80,
                    ..Color::BLACK
                },
            );

            self.content.as_widget().draw(
                self.tree,
                renderer,
                theme,
                style,
                layout.children().next().unwrap(),
                cursor,
                &layout.bounds(),
            );
        }

        fn operate(
            &mut self,
            layout: Layout<'_>,
            renderer: &Renderer,
            operation: &mut dyn widget::Operation<Message>,
        ) {
            self.content.as_widget().operate(
                self.tree,
                layout.children().next().unwrap(),
                renderer,
                operation,
            );
        }

        fn mouse_interaction(
            &self,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            viewport: &Rectangle,
            renderer: &Renderer,
        ) -> mouse::Interaction {
            self.content.as_widget().mouse_interaction(
                self.tree,
                layout.children().next().unwrap(),
                cursor,
                viewport,
                renderer,
            )
        }

        fn overlay<'c>(
            &'c mut self,
            layout: Layout<'_>,
            renderer: &Renderer,
        ) -> Option<overlay::Element<'c, Message, Theme, Renderer>> {
            self.content.as_widget_mut().overlay(
                self.tree,
                layout.children().next().unwrap(),
                renderer,
                Vector::ZERO,
            )
        }
    }

    impl<'a, Message, Theme, Renderer> From<Modal<'a, Message, Theme, Renderer>>
        for Element<'a, Message, Theme, Renderer>
    where
        Theme: 'a,
        Message: 'a + Clone,
        Renderer: 'a + advanced::Renderer,
    {
        fn from(modal: Modal<'a, Message, Theme, Renderer>) -> Self {
            Element::new(modal)
        }
    }
}
