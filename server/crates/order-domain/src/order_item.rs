use crate::{DomainError, MenuItemId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderItem {
    line_number: i32,
    menu_item_id: MenuItemId,
    name: String,
    unit_price_amount: i64,
    quantity: i32,
    line_total_amount: i64,
}

impl OrderItem {
    pub fn create(
        line_number: i32,
        menu_item_id: MenuItemId,
        name: impl Into<String>,
        unit_price_amount: i64,
        quantity: i32,
    ) -> Result<Self, DomainError> {
        if quantity <= 0 {
            return Err(DomainError::InvalidItemQuantity);
        }
        if unit_price_amount < 0 {
            return Err(DomainError::NegativeUnitPriceAmount);
        }

        Ok(Self {
            line_number,
            menu_item_id,
            name: normalize_name(name)?,
            unit_price_amount,
            quantity,
            line_total_amount: unit_price_amount * i64::from(quantity),
        })
    }

    pub fn rehydrate(
        line_number: i32,
        menu_item_id: MenuItemId,
        name: impl Into<String>,
        unit_price_amount: i64,
        quantity: i32,
        line_total_amount: i64,
    ) -> Result<Self, DomainError> {
        let item = Self::create(line_number, menu_item_id, name, unit_price_amount, quantity)?;
        if item.line_total_amount != line_total_amount {
            return Err(DomainError::InvalidLineTotalAmount);
        }
        Ok(item)
    }

    pub fn line_number(&self) -> i32 {
        self.line_number
    }

    pub fn menu_item_id(&self) -> &MenuItemId {
        &self.menu_item_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn unit_price_amount(&self) -> i64 {
        self.unit_price_amount
    }

    pub fn quantity(&self) -> i32 {
        self.quantity
    }

    pub fn line_total_amount(&self) -> i64 {
        self.line_total_amount
    }
}

fn normalize_name(value: impl Into<String>) -> Result<String, DomainError> {
    let value = value.into().trim().to_string();
    if value.is_empty() {
        return Err(DomainError::EmptyItemName);
    }
    Ok(value)
}
