use crate::{
    items::Item,
    notice_board::{self, NoticeboardEntryRenderable},
};

pub const NUM_SLOTS_PLAYER: usize = 5 * 9;
pub const MAX_ITEMS_PER_SLOT: u32 = 255;

#[derive(Default)]
pub struct Inventory {
    items: Vec<Option<Box<dyn Item>>>,
    pub is_player: bool,
}

impl Clone for Inventory {
    fn clone(&self) -> Self {
        let mut new = Self::new(self.items.len(), self.is_player);
        for i in 0..self.items.len() {
            new.items[i] = match &self.items[i] {
                None => None,
                Some(item) => Some(item.clone_item()),
            }
        }
        new
    }
}

impl Inventory {
    pub fn size(&self) -> usize {
        self.items.len()
    }

    pub fn resize(&mut self, new_size: usize) {
        self.items.resize_with(new_size, || None);
    }

    pub fn new(size: usize, is_player: bool) -> Self {
        let mut items = Vec::with_capacity(size);

        for _ in 0..size {
            items.push(None);
        }

        Self { items, is_player }
    }

    pub fn switch_items(&mut self, slot_a: usize, slot_b: usize) -> bool {
        if slot_a >= self.items.len() || slot_b >= self.items.len() {
            return false;
        }

        let val_a = self.items[slot_a].take();
        let val_b = self.items[slot_b].take();
        self.items[slot_a] = val_b;
        self.items[slot_b] = val_a;

        true
    }

    pub fn take_item(&mut self, slot: usize) -> Option<Box<dyn Item>> {
        if slot >= self.items.len() {
            None
        } else {
            let item = self.items[slot].take();
            if let Some(item) = &item {
                if self.is_player {
                    notice_board::add_entry(
                        NoticeboardEntryRenderable::Joiner(
                            Box::new(NoticeboardEntryRenderable::NamedItem(item.clone_item())),
                            Box::new(NoticeboardEntryRenderable::String(format!(
                                "- {}",
                                if item.metadata_is_stack_size() {
                                    item.metadata()
                                } else {
                                    1
                                }
                            ))),
                        ),
                        5,
                    );
                }
            }
            item
        }
    }

    pub fn add_item(&mut self, mut item: Box<dyn Item>, slot: usize) -> Option<Box<dyn Item>> {
        if slot >= self.items.len() {
            return Some(item);
        }

        let orig_sz = if item.metadata_is_stack_size() {
            item.metadata()
        } else {
            1
        };
        match &mut self.items[slot] {
            None => {
                if self.is_player {
                    notice_board::add_entry(
                        NoticeboardEntryRenderable::Joiner(
                            Box::new(NoticeboardEntryRenderable::NamedItem(item.clone_item())),
                            Box::new(NoticeboardEntryRenderable::String(format!("+ {}", orig_sz))),
                        ),
                        5,
                    );
                }
                self.items[slot].replace(item)
            }
            Some(slot_item) => {
                if slot_item.identifier() == item.identifier() && slot_item.metadata_is_stack_size()
                {
                    if slot_item.metadata() >= MAX_ITEMS_PER_SLOT {
                        return Some(item);
                    }
                    let new_sz = slot_item.metadata() + item.metadata();
                    if new_sz > MAX_ITEMS_PER_SLOT {
                        slot_item.set_metadata(MAX_ITEMS_PER_SLOT);
                        item.set_metadata(new_sz - MAX_ITEMS_PER_SLOT);

                        if self.is_player {
                            notice_board::add_entry(
                                NoticeboardEntryRenderable::Joiner(
                                    Box::new(NoticeboardEntryRenderable::NamedItem(
                                        item.clone_item(),
                                    )),
                                    Box::new(NoticeboardEntryRenderable::String(format!(
                                        "+ {}",
                                        orig_sz - item.metadata()
                                    ))),
                                ),
                                5,
                            );
                        }
                        Some(item)
                    } else {
                        if self.is_player {
                            notice_board::add_entry(
                                NoticeboardEntryRenderable::Joiner(
                                    Box::new(NoticeboardEntryRenderable::NamedItem(
                                        item.clone_item(),
                                    )),
                                    Box::new(NoticeboardEntryRenderable::String(format!(
                                        "+ {orig_sz}"
                                    ))),
                                ),
                                5,
                            );
                        }
                        slot_item.set_metadata(new_sz);
                        None
                    }
                } else {
                    if self.is_player {
                        notice_board::add_entry(
                            NoticeboardEntryRenderable::Joiner(
                                Box::new(NoticeboardEntryRenderable::NamedItem(item.clone_item())),
                                Box::new(NoticeboardEntryRenderable::String(format!(
                                    "+ {orig_sz}"
                                ))),
                            ),
                            5,
                        );
                    }
                    self.items[slot].replace(item)
                }
            }
        }
    }

    pub fn get_item<'a>(&'a self, slot: usize) -> &'a Option<Box<dyn Item>> {
        &self.items[slot]
    }

    pub fn get_item_mut<'a>(&'a mut self, slot: usize) -> &'a mut Option<Box<dyn Item>> {
        &mut self.items[slot]
    }

    pub fn try_add_item(&mut self, mut item: Box<dyn Item>) -> Option<Box<dyn Item>> {
        let can_extend_amount = item.metadata_is_stack_size();
        let identifier = item.identifier();

        let mut orig_sz = if item.metadata_is_stack_size() {
            item.metadata()
        } else {
            1
        };
        for slot in 0..self.items.len() {
            match &mut self.items[slot] {
                None => {
                    if self.is_player {
                        notice_board::add_entry(
                            NoticeboardEntryRenderable::Joiner(
                                Box::new(NoticeboardEntryRenderable::NamedItem(item.clone_item())),
                                Box::new(NoticeboardEntryRenderable::String(format!(
                                    "+ {orig_sz}"
                                ))),
                            ),
                            5,
                        );
                    }
                    self.items[slot] = Some(item);
                    return None;
                }
                Some(other_item) => {
                    if other_item.identifier() == identifier && can_extend_amount {
                        if other_item.metadata() >= MAX_ITEMS_PER_SLOT {
                            continue;
                        }
                        let new_sz = other_item.metadata() + item.metadata();
                        if new_sz > MAX_ITEMS_PER_SLOT {
                            other_item.set_metadata(MAX_ITEMS_PER_SLOT);
                            item.set_metadata(new_sz - MAX_ITEMS_PER_SLOT);
                            if self.is_player {
                                notice_board::add_entry(
                                    NoticeboardEntryRenderable::Joiner(
                                        Box::new(NoticeboardEntryRenderable::NamedItem(
                                            item.clone_item(),
                                        )),
                                        Box::new(NoticeboardEntryRenderable::String(format!(
                                            "+ {}",
                                            orig_sz - item.metadata()
                                        ))),
                                    ),
                                    5,
                                );
                            }
                            orig_sz = item.metadata()
                        } else {
                            if self.is_player {
                                notice_board::add_entry(
                                    NoticeboardEntryRenderable::Joiner(
                                        Box::new(NoticeboardEntryRenderable::NamedItem(
                                            item.clone_item(),
                                        )),
                                        Box::new(NoticeboardEntryRenderable::String(format!(
                                            "+ {orig_sz}"
                                        ))),
                                    ),
                                    5,
                                );
                            }
                            other_item.set_metadata(new_sz);
                            return None;
                        }
                    }
                }
            }
        }
        if can_extend_amount && item.metadata() > 0 {
            Some(item)
        } else {
            None
        }
    }
}
