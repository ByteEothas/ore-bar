use crate::{
    circular::Circular,
    consts::{
        ACCOUNT_DETAIL_HIGHT, ACCOUNT_DETAIL_WIDTH, MAX_ITEMS_PER_ROW, MENU_CATEGORY_SPACING,
        MENU_ITEM_INDENT, MENU_ITEM_SPACING, MENU_SPAN_HEIGHT, SUBHEAD_TEXT,
    },
    easing,
    logic::FetchMode,
    style,
    utils::{abbreviate, get_domain},
    Account, ContentType, Dashboard, Message, ModalType,
};
use iced::widget::{
    button, center, checkbox, column, container, horizontal_space, mouse_area, opaque, pick_list,
    row, scrollable, stack, svg, text, text_input, vertical_space, Column, Row,
};
use iced::{padding, Color, Element, Length, Theme};
use ore_api::consts::MINT_ADDRESS;
use std::time::Duration;

impl Dashboard {
    pub fn view(&self) -> Element<Message> {
        let refresh_button = if !self.is_refreshed {
            button(get_svg_icon("refresh", 24, 24))
                .on_press(Message::Refresh)
                .style(button::text)
        } else {
            button(get_svg_icon("refresh-disabled", 24, 24)).style(button::text)
        };

        let left = column![
            row![text("Accounts").size(20), refresh_button].align_y(iced::Alignment::Center),
            row![
                column![
                    text("Number:"),
                    text("Balance:").height(MENU_SPAN_HEIGHT),
                    text("Stake:").height(MENU_SPAN_HEIGHT),
                    text("Status:"),
                    text("Mint Address:")
                ]
                .padding(padding::left(MENU_ITEM_INDENT))
                .spacing(MENU_ITEM_SPACING),
                column![
                    text(self.accounts.len()),
                    column![
                        text(&self.balance),
                        text(format!("${}", &self.balance_usd)).size(SUBHEAD_TEXT)
                    ]
                    .height(MENU_SPAN_HEIGHT)
                    .width(Length::Fill)
                    .align_x(iced::Alignment::End),
                    column![
                        text(&self.stake),
                        text(format!("${}", &self.stake_usd)).size(SUBHEAD_TEXT)
                    ]
                    .height(MENU_SPAN_HEIGHT)
                    .width(Length::Fill)
                    .align_x(iced::Alignment::End),
                    active_num_view(&self),
                    text(abbreviate(&MINT_ADDRESS.to_string()))
                ]
                .align_x(iced::Alignment::End)
                .spacing(MENU_ITEM_SPACING)
            ],
            column![
                checkbox("Auto Refresh", self.auto_refresh).on_toggle(Message::ToggleSubscription),
                checkbox(
                    "Parallel Mode",
                    match self.fetch_mode {
                        FetchMode::Parallel => true,
                        FetchMode::Serial => false,
                    }
                )
                .on_toggle(Message::ToggleFetchMode),
            ]
            .spacing(MENU_ITEM_SPACING),
            row![
                button(text("Add an account").align_x(iced::Alignment::Center))
                    .on_press(Message::SetModalView(None, add_account_view))
                    .width(Length::Fill)
            ],
            vertical_space(),
            // Themes
            row!(pick_list(
                Theme::ALL,
                Some(&self.theme),
                Message::ThemeSelected
            ))
            .width(100),
            row![text("Version:"), text(&self.version)],
        ]
        .spacing(MENU_CATEGORY_SPACING)
        .padding(padding::all(5).left(10))
        .align_x(iced::Alignment::Start);
        let content = get_content_list(self);
        let body = row![left.width(250), content];
        match &self.show_modal {
            ModalType::Sub => modal(body, (self.modal_view)(&self)),
            _ => body.into(),
        }
    }
}

pub fn modal<'a, Message>(
    base: impl Into<Element<'a, Message>>,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    stack![
        base.into(),
        mouse_area(center(opaque(content)).style(|_theme| {
            container::Style {
                background: Some(
                    Color {
                        a: 0.8,
                        ..Color::BLACK
                    }
                    .into(),
                ),
                ..container::Style::default()
            }
        }))
    ]
    .into()
}

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
                .style(container::rounded_box),
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
        .align_x(iced::Alignment::Center)
        .into();
    }
    if status.is_valid {
        column![
            row![
                text("Account:"),
                text(abbreviate(&status.authority.to_string())),
                horizontal_space(),
                get_svg(status.is_online),
            ]
            .spacing(5),
            row![text("Balance:"), text(&status.balance)].spacing(5),
            row![text("Stake:"), text(&status.stake)].spacing(5),
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
                    .style(button::success),
                button("Stake").on_press(Message::SetModalView(Some(index), stake_view)),
                button("Remove").on_press(Message::SetModalView(Some(index), remove_account_view)),
            ]
            .spacing(5)]
            .width(Length::Fill)
            .align_x(iced::Alignment::Center),
        ]
        .padding(5)
        .spacing(8)
        .height(Length::Fill)
        .width(Length::Fill)
        .align_x(iced::Alignment::Start)
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
                .align_x(iced::Alignment::Center)
        ]
        .padding(5)
        .spacing(8)
        .height(Length::Fill)
        .width(Length::Fill)
        .align_x(iced::Alignment::Start)
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
                .align_x(iced::Alignment::Center),
        ]
        .spacing(10),
    )
    .width(400)
    .padding(10)
    .style(container::rounded_box)
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
        .style(container::rounded_box)
        .into()
    } else {
        text("No account selected").into()
    }
}

pub fn claim_view<'a>(dashboard: &Dashboard) -> Element<'a, Message> {
    container(
        column![
            text("Claim ore to wallet").size(24).style(text::success),
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
    .style(container::rounded_box)
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
    .style(container::rounded_box)
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
    .style(container::rounded_box)
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
    .style(container::rounded_box)
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
    row.align_y(iced::Alignment::Center)
        .height(21)
        .spacing(1)
        .into()
}

pub fn dialog_view<'a>(dashboard: &Dashboard) -> Element<'a, Message> {
    let (text_color, command): (Box<dyn Fn(&Theme) -> text::Style>, Option<Box<Message>>) =
        match dashboard.dialog.content_type {
            ContentType::Normal => (Box::new(text::primary), Some(Box::new(Message::Refresh))),
            ContentType::Good => (Box::new(text::success), Some(Box::new(Message::Refresh))),
            ContentType::Warn => (Box::new(text::secondary), None),
            ContentType::Error => (Box::new(text::danger), None),
        };
    container(
        Column::new()
            .push(
                text(dashboard.dialog.content.clone())
                    .size(24)
                    .style(text_color),
            )
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
    .style(style::pane_pop)
    .into()
}
