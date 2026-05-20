//! Market eligibility filter.
//!
//! Decides whether the bot should consider copying a trade given the
//! market's slug, category, and tags vs the configured allow/block lists.
//!
//! Behaviour matches the brief picked by the operator: strict allowlist
//! when any `*_allow` list is populated, otherwise allow-everything-minus-
//! blocklist. Blocklists always take precedence.

use crate::config::FiltersConfig;
use crate::service::market_cache::MarketInfo;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Eligibility {
    Allowed,
    Closed,
    BlockedBySlug,
    BlockedByCategory(String),
    BlockedByTag(String),
    NotInAllowlist,
}

pub fn check(filters: &FiltersConfig, market: &MarketInfo) -> Eligibility {
    if market.closed {
        return Eligibility::Closed;
    }

    // 1. Block lists always win.
    if filters.slug_block.iter().any(|s| s == &market.slug) {
        return Eligibility::BlockedBySlug;
    }
    if let Some(cat) = market.category.as_ref() {
        if filters.categories_block.iter().any(|c| ci_eq(c, cat)) {
            return Eligibility::BlockedByCategory(cat.clone());
        }
    }
    if let Some(blocked) = market
        .tags
        .iter()
        .find(|tag| filters.tags_block.iter().any(|b| ci_eq(b, tag)))
    {
        return Eligibility::BlockedByTag(blocked.clone());
    }

    // 2. If no allow lists are configured, accept everything that passed blocks.
    if !filters.is_strict() {
        return Eligibility::Allowed;
    }

    // 3. Strict allowlist — at least one allow rule must match across slug,
    //    category, or tags. The slug allowlist is exact; cat/tag matching is
    //    case-insensitive to forgive Gamma capitalisation drift.
    let slug_hit = filters.slug_allow.iter().any(|s| s == &market.slug);
    let cat_hit = market
        .category
        .as_ref()
        .map(|cat| filters.categories_allow.iter().any(|c| ci_eq(c, cat)))
        .unwrap_or(false);
    let tag_hit = market
        .tags
        .iter()
        .any(|tag| filters.tags_allow.iter().any(|t| ci_eq(t, tag)));

    if slug_hit || cat_hit || tag_hit {
        Eligibility::Allowed
    } else {
        Eligibility::NotInAllowlist
    }
}

fn ci_eq(a: &str, b: &str) -> bool {
    a.len() == b.len() && a.chars().zip(b.chars()).all(|(x, y)| {
        x.to_ascii_lowercase() == y.to_ascii_lowercase()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn market(slug: &str, category: Option<&str>, tags: &[&str]) -> MarketInfo {
        MarketInfo {
            slug: slug.into(),
            question: String::new(),
            yes_token_id: "1".into(),
            no_token_id: "2".into(),
            category: category.map(String::from),
            tags: tags.iter().map(|s| s.to_string()).collect(),
            closed: false,
        }
    }

    #[test]
    fn closed_market_always_blocked() {
        let mut m = market("x", Some("Politics"), &[]);
        m.closed = true;
        let r = check(&FiltersConfig::default(), &m);
        assert_eq!(r, Eligibility::Closed);
    }

    #[test]
    fn default_filters_allow_everything() {
        let r = check(&FiltersConfig::default(), &market("x", Some("Memes"), &[]));
        assert_eq!(r, Eligibility::Allowed);
    }

    #[test]
    fn category_block_wins_over_allow() {
        let f = FiltersConfig {
            categories_allow: vec!["Memes".into()],
            categories_block: vec!["Memes".into()],
            ..Default::default()
        };
        let r = check(&f, &market("x", Some("Memes"), &[]));
        matches!(r, Eligibility::BlockedByCategory(_));
    }

    #[test]
    fn strict_allowlist_rejects_unlisted_category() {
        let f = FiltersConfig {
            categories_allow: vec!["Politics".into()],
            ..Default::default()
        };
        let r = check(&f, &market("x", Some("Crypto"), &[]));
        assert_eq!(r, Eligibility::NotInAllowlist);
    }

    #[test]
    fn strict_allowlist_admits_listed_category() {
        let f = FiltersConfig {
            categories_allow: vec!["politics".into()],
            ..Default::default()
        };
        let r = check(&f, &market("x", Some("Politics"), &[]));
        assert_eq!(r, Eligibility::Allowed);
    }

    #[test]
    fn slug_allow_overrides_category_miss() {
        let f = FiltersConfig {
            categories_allow: vec!["Politics".into()],
            slug_allow: vec!["my-fav-market".into()],
            ..Default::default()
        };
        let r = check(&f, &market("my-fav-market", Some("Crypto"), &[]));
        assert_eq!(r, Eligibility::Allowed);
    }
}
