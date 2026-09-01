use std::collections::HashMap;

/// Узел суффиксного бора доменных имен
#[derive(Debug, Clone)]
pub struct TrieNode<T> {
    /// Значение, ассоциированное с данным точным доменом (например, twitch.tv)
    pub value: Option<T>,
    /// Значение для wildcard поддоменов (*.twitch.tv)
    pub wildcard_value: Option<T>,
    /// Дочерние узлы (метки домена слева: "tv" -> "twitch" -> "gql")
    pub children: HashMap<String, TrieNode<T>>,
}

impl<T> Default for TrieNode<T> {
    fn default() -> Self {
        Self {
            value: None,
            wildcard_value: None,
            children: HashMap::new(),
        }
    }
}

/// Высокопроизводительный Reverse Suffix Trie для сопоставления доменов
#[derive(Debug, Clone)]
pub struct DomainTrie<T> {
    root: TrieNode<T>,
    count: usize,
}

impl<T> Default for DomainTrie<T> {
    fn default() -> Self {
        Self {
            root: TrieNode::default(),
            count: 0,
        }
    }
}

impl<T: Clone> DomainTrie<T> {
    pub fn new() -> Self {
        Self {
            root: TrieNode::default(),
            count: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Вставка доменного шаблона в бор
    /// Поддерживает:
    /// - Точные домены: "gql.twitch.tv"
    /// - Wildcard поддомены: "*.live-video.net"
    /// - Суффиксы: ".twitch.tv" (эквивалентно и точному, и wildcard)
    pub fn insert(&mut self, pattern: &str, value: T) {
        let pattern = pattern.trim().to_ascii_lowercase();
        if pattern.is_empty() {
            return;
        }

        let is_wildcard = pattern.starts_with("*.") || pattern.starts_with('.');
        let clean_pattern = pattern.trim_start_matches('*').trim_start_matches('.');

        let mut curr = &mut self.root;
        // Проход по меткам справа налево ("com" -> "example" -> "sub")
        for label in clean_pattern.rsplit('.') {
            if label.is_empty() {
                continue;
            }
            curr = curr.children.entry(label.to_string()).or_default();
        }

        if is_wildcard {
            curr.wildcard_value = Some(value.clone());
            if pattern.starts_with('.') {
                curr.value = Some(value);
            }
        } else {
            curr.value = Some(value);
        }

        self.count += 1;
    }

    /// Поиск наиболее специфичного правила для запрашиваемого FQDN
    /// Выполняется без динамических аллокаций памяти в hot-path
    pub fn find(&self, fqdn: &str) -> Option<&T> {
        let clean_fqdn = fqdn.trim().trim_end_matches('.');
        if clean_fqdn.is_empty() {
            return None;
        }

        let mut curr = &self.root;
        let mut best_wildcard: Option<&T> = None;

        let mut labels = clean_fqdn.rsplit('.');

        while let Some(label) = labels.next() {
            let lower_label = label.to_ascii_lowercase();

            // Если текущий узел задает wildcard (*.domain), то все его поддомены наследуют это правило
            if let Some(ref w_val) = curr.wildcard_value {
                best_wildcard = Some(w_val);
            }

            if let Some(next) = curr.children.get(&lower_label) {
                curr = next;
            } else {
                // Дочерней ветки нет — возвращаем последнее подходящее wildcard-правило
                return best_wildcard;
            }
        }

        // Если домен полностью совпал по всем меткам:
        // Точное совпадение (value) имеет приоритет, затем wildcard
        curr.value.as_ref().or(best_wildcard)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        let mut trie = DomainTrie::new();
        trie.insert("gql.twitch.tv", 1);
        trie.insert("usher.ttvnw.net", 2);

        assert_eq!(trie.find("gql.twitch.tv"), Some(&1));
        assert_eq!(trie.find("usher.ttvnw.net"), Some(&2));
        assert_eq!(trie.find("twitch.tv"), None);
        assert_eq!(trie.find("sub.gql.twitch.tv"), None);
    }

    #[test]
    fn test_wildcard_match() {
        let mut trie = DomainTrie::new();
        trie.insert("*.live-video.net", 10);
        trie.insert("specific.live-video.net", 20);

        // Wildcard матчит поддомены любой вложенности
        assert_eq!(trie.find("video-edge-1.live-video.net"), Some(&10));
        assert_eq!(trie.find("deep.sub.live-video.net"), Some(&10));

        // Точное совпадение специфичнее wildcard
        assert_eq!(trie.find("specific.live-video.net"), Some(&20));

        // Сам корневой домен без поддомена не матчится по *.
        assert_eq!(trie.find("live-video.net"), None);
    }

    #[test]
    fn test_suffix_match() {
        let mut trie = DomainTrie::new();
        // .domain.com означает и сам domain.com, и *.domain.com
        trie.insert(".crunchyroll.com", 100);

        assert_eq!(trie.find("crunchyroll.com"), Some(&100));
        assert_eq!(trie.find("api.crunchyroll.com"), Some(&100));
        assert_eq!(trie.find("beta.api.crunchyroll.com"), Some(&100));
        assert_eq!(trie.find("othercrunchyroll.com"), None);
    }

    #[test]
    fn test_case_insensitivity_and_trailing_dot() {
        let mut trie = DomainTrie::new();
        trie.insert("GQL.Twitch.TV", 42);

        assert_eq!(trie.find("gql.twitch.tv"), Some(&42));
        assert_eq!(trie.find("GQL.TWITCH.TV."), Some(&42));
    }
}
