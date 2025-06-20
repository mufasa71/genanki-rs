use regex::Regex;

pub fn extract_word_from_text(word: &str) -> String {
    let noun_regex_pattern = Regex::new(r"^(?<artikle>der|die|das) (?<word>\w+)").unwrap();
    let sich_regex_pattern = Regex::new(r"^(\(sich\)) (?P<word>\w+)").unwrap();
    let verb_regex_pattern = Regex::new(r"^(?<word>\w+),").unwrap();
    let zusammen_regex_pattern = Regex::new(r"^(?<word>\w+)/").unwrap();
    let mehr_regex_pattern = Regex::new(r"^(?<word>\w+) \(.*\)").unwrap();

    if let Some(caps) = noun_regex_pattern.captures(word) {
        let artikle = caps.name("artikle").unwrap().as_str();
        let word = caps.name("word").unwrap().as_str();
        return format!("{} {}", artikle, word);
    }

    if let Some(caps) = sich_regex_pattern.captures(word) {
        let word = caps.name("word").unwrap().as_str();
        return word.to_string();
    }

    if let Some(caps) = verb_regex_pattern.captures(word) {
        let word = caps.name("word").unwrap().as_str();
        return word.to_string();
    }

    if let Some(caps) = zusammen_regex_pattern.captures(word) {
        let word = caps.name("word").unwrap().as_str();
        return word.to_string();
    }

    if let Some(caps) = mehr_regex_pattern.captures(word) {
        return String::from(caps.name("word").unwrap().as_str());
    }

    String::from(word)
}

pub fn extract_word_to_speach(word: &str) -> String {
    let noun_regex_pattern = Regex::new(r"^(?<artikle>der|die|das) (?<word>\w+)").unwrap();
    let remove_minus_regex_pattern = Regex::new(r"^(?<word>\w+)-").unwrap();
    let zusammen_regex_pattern = Regex::new(r"^(?<word>\w+)/").unwrap();
    let mehr_regex_pattern = Regex::new(r"^(?<word>\w+) \(.*\)").unwrap();

    if let Some(caps) = noun_regex_pattern.captures(word) {
        let artikle = caps.name("artikle").unwrap().as_str();
        let word = caps.name("word").unwrap().as_str();
        return format!("{} {}", artikle, word);
    }

    if let Some(caps) = remove_minus_regex_pattern.captures(word) {
        return String::from(caps.name("word").unwrap().as_str());
    }

    if let Some(caps) = zusammen_regex_pattern.captures(word) {
        return String::from(caps.name("word").unwrap().as_str());
    }

    if let Some(caps) = mehr_regex_pattern.captures(word) {
        return String::from(caps.name("word").unwrap().as_str());
    }
    String::from(word)
}

#[test]
fn test_extract_word_from_text() {
    assert_eq!(extract_word_from_text("das Poulet, -s (CH)"), "das Poulet");
    assert_eq!(extract_word_from_text("der Schutz"), "der Schutz");
    assert_eq!(extract_word_from_text("die Leitung, -en"), "die Leitung");
    assert_eq!(
        extract_word_from_text("(sich) schneiden, schneidet, schnitt, hat geschnitten"),
        "schneiden"
    );
    assert_eq!(
        extract_word_from_text("kochen, kocht, kochte, hat gekocht"),
        "kochen"
    );
    assert_eq!(
        extract_word_from_text("verlängern, verlängert, verlängerte, hat verlängert"),
        "verlängern"
    );
    assert_eq!(extract_word_from_text("Lieblings-"), "Lieblings-");
    assert_eq!(extract_word_from_text("zusammen/zusammen-"), "zusammen");
    assert_eq!(extract_word_from_text("zurück/zurück-"), "zurück");
    assert_eq!(extract_word_from_text("toll"), "toll");
    assert_eq!(extract_word_from_text("klug"), "klug");
    assert_eq!(extract_word_from_text("mehr (siehe auch viel)"), "mehr");
    assert_eq!(
        extract_word_from_text("sowohl … als auch"),
        "sowohl … als auch"
    );
}

#[test]
fn test_extract_word_to_speach() {
    assert_eq!(extract_word_to_speach("das Poulet, -s (CH)"), "das Poulet");
    assert_eq!(extract_word_to_speach("der Schutz"), "der Schutz");
    assert_eq!(extract_word_to_speach("die Leitung, -en"), "die Leitung");
    assert_eq!(
        extract_word_to_speach("(sich) schneiden, schneidet, schnitt, hat geschnitten"),
        "(sich) schneiden, schneidet, schnitt, hat geschnitten"
    );
    assert_eq!(
        extract_word_to_speach("kochen, kocht, kochte, hat gekocht"),
        "kochen, kocht, kochte, hat gekocht"
    );
    assert_eq!(
        extract_word_to_speach("verlängern, verlängert, verlängerte, hat verlängert"),
        "verlängern, verlängert, verlängerte, hat verlängert"
    );
    assert_eq!(extract_word_to_speach("Lieblings-"), "Lieblings");
    assert_eq!(extract_word_to_speach("zusammen/zusammen-"), "zusammen");
    assert_eq!(extract_word_to_speach("zurück/zurück-"), "zurück");
    assert_eq!(extract_word_to_speach("toll"), "toll");
    assert_eq!(
        extract_word_to_speach("sowohl … als auch"),
        "sowohl … als auch"
    );
    assert_eq!(extract_word_from_text("mehr (siehe auch viel)"), "mehr");
}
