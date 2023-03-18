use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[repr(u8)]
pub enum CreditType {
    Auto,
    Mortgage,
    Micro,
    Education,
    Consumer,
    Overdraft,
    CreditCard,
    Other,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Credit {
    pub initial_fee: String,
    pub interest_rate: String,
    pub credit_period: String,
    pub max_sum: String,
    pub credit_type: CreditType,
    pub title: String,
    pub currency: String,
    // Ставка на просроченный основной долг
    pub overdue_interest_rate: String,
    // Форма предоставления
    // Перечислением на банковский счет продавца
    pub type_of_interest_rate: String,
    // Льготный период
    pub grace_period: String,
    // Необходимые документы
    pub document_required: String,
    // Досрочное погашение
    pub early_repayment: bool,
    // Обеспечение по кредиту
    pub loan_security: String,
    // Способ оформления кредита
    pub loan_processing_method: String,
    // Периодичность платежей
    pub frequency_of_payments: String,
    // Дополнительные условия
    pub additional_conditions: String,
}

impl Credit {
    pub fn new() -> Credit {
        Credit {
            initial_fee: String::new(),
            interest_rate: String::new(),
            credit_period: String::new(),
            max_sum: String::new(),
            credit_type: CreditType::Auto,
            title: String::new(),
            currency: String::new(),
            overdue_interest_rate: String::new(),
            type_of_interest_rate: String::new(),
            grace_period: String::new(),
            document_required: String::new(),
            early_repayment: false,
            loan_security: String::new(),
            loan_processing_method: String::new(),
            frequency_of_payments: String::new(),
            additional_conditions: String::new(),
        }
    }
}
