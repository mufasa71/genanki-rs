use crate::crawlers::CreditType;
use scraper::element_ref::Text;

pub fn clean_text(text: Text) -> String {
    text.map(|x| x.trim())
        .filter(|x| !x.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

// based on credit title try to find credit type
pub fn find_credit_type(title: &str) -> CreditType {
    if title.contains("Автокредит") {
        return CreditType::Auto;
    }

    if title.contains("Ипотека") {
        return CreditType::Mortgage;
    }

    if title.contains("Микрокредит") || title.contains("Микрозайм") {
        return CreditType::Micro;
    }

    if title.contains("Образование") || title.contains("Образовательный")
    {
        return CreditType::Education;
    }

    if title.contains("Потребительский") {
        return CreditType::Consumer;
    }

    if title.contains("Расчетный") || title.contains("Овердрафт") {
        return CreditType::Overdraft;
    }

    if title.contains("Кредитная карта") {
        return CreditType::CreditCard;
    }

    CreditType::Other
}
