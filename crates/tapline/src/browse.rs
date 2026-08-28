//! Finding Workshop items, rather than being told their ids.
//!
//! tapline could always download an item once you knew its number. Getting that
//! number meant a browser, which is a poor fit for a tool whose whole point is
//! not shelling out to something else. `PublishedFile.QueryFiles` is the same
//! search the Workshop website runs, and it is reachable over the CM session
//! already open — no WebAPI key, and **no login**: an anonymous session queries
//! Garry's Mod's 1,982,745 items happily, checked on 2026-08-27.
//!
//! ```no_run
//! # async fn example() -> Result<(), tapline::InstallError> {
//! use tapline::{AppId, BrowseQuery, BrowseSort, Session};
//!
//! let mut session = Session::anonymous().await?;
//! let page = session
//!     .browse_workshop(&BrowseQuery {
//!         app: AppId(4000),
//!         text: Some("stargate".to_owned()),
//!         sort: BrowseSort::TextMatch,
//!         ..BrowseQuery::default()
//!     })
//!     .await?;
//!
//! for found in &page.items {
//!     println!("{} {}", found.item.id, found.item.title);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! A result carries a [`WorkshopItem`], which is the same value
//! [`Session::workshop_details`] returns, so anything found here can be handed
//! straight to a download with no second lookup.

use crate::{InstallError, WorkshopItem};
use tapline_ids::AppId;
use tapline_proto::steammessages_publishedfile_steamclient::{
    CPublishedFile_QueryFiles_Request, c_published_file_query_files_request::TagGroup,
};

/// How Steam should order the results.
///
/// The numbers are Valve's `EPublishedFileQueryType`, which is sparse — 2 and
/// several others do not exist — so this enum names the ones worth offering
/// rather than mirroring the whole thing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BrowseSort {
    /// Highest rated. Valve's default, and a reasonable one.
    #[default]
    Vote,
    /// Most recently published.
    Recent,
    /// Most recently updated.
    Updated,
    /// Trending: what is being subscribed to now rather than in total.
    Trend,
    /// Most subscribed, all time.
    Subscribed,
    /// Best match for the search text.
    ///
    /// Only meaningful with [`BrowseQuery::text`]; Steam returns an
    /// unpredictable order without it, which is why [`BrowseQuery::validate`]
    /// refuses the combination rather than letting it look like it worked.
    TextMatch,
}

impl BrowseSort {
    /// Valve's `EPublishedFileQueryType` value.
    const fn query_type(self) -> u32 {
        match self {
            Self::Vote => 0,
            Self::Recent => 1,
            Self::Trend => 3,
            Self::Subscribed => 9,
            Self::TextMatch => 12,
            Self::Updated => 21,
        }
    }

    /// Parses the name the CLI and the bindings use.
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "vote" | "rated" => Some(Self::Vote),
            "recent" | "new" => Some(Self::Recent),
            "updated" => Some(Self::Updated),
            "trend" | "trending" => Some(Self::Trend),
            "subscribed" | "popular" => Some(Self::Subscribed),
            "text" | "relevance" => Some(Self::TextMatch),
            _ => None,
        }
    }

    /// Every name [`BrowseSort::parse`] accepts, canonical form first.
    pub const NAMES: [&'static str; 6] =
        ["vote", "recent", "updated", "trend", "subscribed", "text"];
}

/// The cursor that starts a search.
///
/// Steam's paging is a cursor, not an offset, and the first page is the literal
/// `"*"` rather than an empty string.
pub const FIRST_PAGE: &str = "*";

/// The most items Steam will return in one page.
///
/// Asking for more is not an error and not honoured either — it silently
/// returns fewer, which is the kind of thing that turns into a bug report about
/// missing results.
pub const MAX_PER_PAGE: u32 = 100;

/// What to search for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowseQuery {
    /// Which app's Workshop.
    pub app: AppId,
    /// Free text to match, if any.
    pub text: Option<String>,
    /// Tags an item must carry.
    pub required_tags: Vec<String>,
    /// Groups of tags, of which an item must carry at least one from each.
    ///
    /// This is what Steam's own sidebar does. Ticking Scene and Video under
    /// Type and Anime under Genre means *(Scene or Video) and Anime*, which
    /// [`BrowseQuery::required_tags`] cannot express:
    /// [`BrowseQuery::match_all_tags`] is a single switch over the whole set,
    /// so it is every tag or any tag and nothing in between.
    ///
    /// Groups combine with `required_tags` rather than replacing it — a tag
    /// that must always be present is simpler as a required tag than as a
    /// group of one.
    pub tag_groups: Vec<Vec<String>>,
    /// Tags that exclude an item.
    pub excluded_tags: Vec<String>,
    /// Whether an item must carry *every* required tag rather than any of them.
    ///
    /// Applies to [`BrowseQuery::required_tags`] only. Groups are always
    /// any-within-group and all-across-groups.
    pub match_all_tags: bool,
    /// How to order results.
    pub sort: BrowseSort,
    /// How many to return, capped at [`MAX_PER_PAGE`].
    pub per_page: u32,
    /// Where to resume from. `None` starts at the beginning.
    pub cursor: Option<String>,
}

impl Default for BrowseQuery {
    fn default() -> Self {
        Self {
            app: AppId(0),
            text: None,
            required_tags: Vec::new(),
            tag_groups: Vec::new(),
            excluded_tags: Vec::new(),
            match_all_tags: false,
            sort: BrowseSort::default(),
            per_page: 20,
            cursor: None,
        }
    }
}

impl BrowseQuery {
    /// Checks the query is one Steam can answer usefully.
    ///
    /// Only refuses what would silently return nonsense. An app id of zero
    /// searches every app's Workshop at once and returns an unusable mixture;
    /// sorting by text match with no text returns an arbitrary order that looks
    /// like a ranking.
    pub fn validate(&self) -> Result<(), BrowseError> {
        if self.app.get() == 0 {
            return Err(BrowseError::NoApp);
        }
        if self.sort == BrowseSort::TextMatch && self.text.is_none() {
            return Err(BrowseError::TextSortWithoutText);
        }
        if self.tag_groups.iter().any(Vec::is_empty) {
            return Err(BrowseError::EmptyTagGroup);
        }
        Ok(())
    }

    /// Builds the wire request.
    ///
    /// Separate from the sending so the mapping can be tested without a
    /// network, which is where the mistakes are: a tag in the wrong field
    /// returns plausible results for the wrong query.
    pub(crate) fn to_request(&self) -> CPublishedFile_QueryFiles_Request {
        CPublishedFile_QueryFiles_Request {
            query_type: Some(self.sort.query_type()),
            appid: Some(self.app.get()),
            search_text: self.text.clone(),
            requiredtags: self.required_tags.clone(),
            taggroups: self
                .tag_groups
                .iter()
                .map(|tags| TagGroup { tags: tags.clone() })
                .collect(),
            excludedtags: self.excluded_tags.clone(),
            match_all_tags: Some(self.match_all_tags),
            numperpage: Some(self.per_page.clamp(1, MAX_PER_PAGE)),
            cursor: Some(self.cursor.clone().unwrap_or_else(|| FIRST_PAGE.to_owned())),
            // Without these the reply carries ids and little else, and every
            // caller would need a second round trip to show a search result.
            return_tags: Some(true),
            return_short_description: Some(true),
            return_previews: Some(true),
            return_vote_data: Some(true),
            return_details: Some(true),
            // Descriptions are BBCode otherwise, which nothing downstream
            // renders and every consumer would have to strip itself.
            strip_description_bbcode: Some(true),
            ..CPublishedFile_QueryFiles_Request::default()
        }
    }
}

/// One search result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowseResult {
    /// The item, in the same shape a download takes.
    pub item: WorkshopItem,
    /// A short description, with BBCode already stripped.
    pub description: String,
    /// The item's tags, display names where Steam gives one.
    pub tags: Vec<String>,
    /// Where its preview image lives, when it has one.
    pub preview_url: Option<String>,
    /// Current subscribers.
    pub subscriptions: u64,
    /// Current favourites.
    pub favorites: u64,
}

/// A page of results.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowsePage {
    /// The items on this page.
    pub items: Vec<BrowseResult>,
    /// How many the whole search matched, which is usually far more than a page.
    pub total: u32,
    /// The cursor for the next page, or `None` at the end.
    pub next_cursor: Option<String>,
    /// Items Steam returned that could not be described, and why.
    ///
    /// Reported rather than dropped: a search that quietly returns nineteen of
    /// twenty results is worse than one that says which it could not read. A
    /// deleted or hidden item is the usual cause and is not a failure of the
    /// search.
    pub skipped: Vec<(u64, String)>,
}

impl BrowsePage {
    /// Whether another page exists.
    #[must_use]
    pub fn has_more(&self) -> bool {
        self.next_cursor.is_some()
    }
}

/// Why a search could not be run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowseError {
    /// No app was given.
    NoApp,
    /// Sorting by text relevance without any text to match.
    TextSortWithoutText,
    /// A tag group with no tags in it.
    EmptyTagGroup,
}

impl std::fmt::Display for BrowseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoApp => write!(
                f,
                "a Workshop search needs an app id; searching every app at once \
                 returns an unusable mixture"
            ),
            Self::TextSortWithoutText => write!(
                f,
                "sorting by text match needs search text; without it Steam \
                 returns an arbitrary order that looks like a ranking"
            ),
            Self::EmptyTagGroup => write!(
                f,
                "a tag group needs at least one tag; an empty group is a \
                 filter that matches nothing and reads as a broken search"
            ),
        }
    }
}

impl std::error::Error for BrowseError {}

impl From<BrowseError> for InstallError {
    fn from(error: BrowseError) -> Self {
        Self::Io(error.to_string())
    }
}

/// Turns one reply item into a result, given the app's workshop depot.
///
/// Split out so the mapping is testable without a network.
pub(crate) fn describe(
    details: &tapline_proto::steammessages_publishedfile_steamclient::PublishedFileDetails,
    workshop_depot: Option<tapline_ids::DepotId>,
) -> Result<BrowseResult, (u64, String)> {
    let id = details.publishedfileid.unwrap_or(0);
    let item = crate::workshop::classify(details, workshop_depot)
        .map_err(|error| (id, error.to_string()))?;

    Ok(BrowseResult {
        item,
        description: details.short_description.clone().unwrap_or_default(),
        tags: details
            .tags
            .iter()
            .map(|tag| {
                // The display name is what a person recognises; the raw tag is
                // what a filter matches. Prefer the former and fall back.
                tag.display_name
                    .clone()
                    .filter(|name| !name.is_empty())
                    .or_else(|| tag.tag.clone())
                    .unwrap_or_default()
            })
            .filter(|tag| !tag.is_empty())
            .collect(),
        preview_url: details.preview_url.clone().filter(|url| !url.is_empty()),
        subscriptions: u64::from(details.subscriptions.unwrap_or(0)),
        favorites: u64::from(details.favorited.unwrap_or(0)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tapline_proto::steammessages_publishedfile_steamclient::{
        PublishedFileDetails, published_file_details::Tag,
    };

    fn details(id: u64, result: u32) -> PublishedFileDetails {
        PublishedFileDetails {
            publishedfileid: Some(id),
            result: Some(result),
            consumer_appid: Some(4000),
            hcontent_file: Some(999),
            title: Some("An Addon".to_owned()),
            file_size: Some(1234),
            time_updated: Some(42),
            short_description: Some("does things".to_owned()),
            ..PublishedFileDetails::default()
        }
    }

    #[test]
    fn the_first_page_asks_for_the_cursor_steam_expects() {
        // An empty string is not the first page; Steam wants a literal star,
        // and sending "" returns nothing at all.
        let request = BrowseQuery {
            app: AppId(4000),
            ..BrowseQuery::default()
        }
        .to_request();
        assert_eq!(request.cursor.as_deref(), Some("*"));
    }

    #[test]
    fn a_cursor_is_passed_back_verbatim() {
        let request = BrowseQuery {
            app: AppId(4000),
            cursor: Some("AoIIQ0NjvXTE+Olo".to_owned()),
            ..BrowseQuery::default()
        }
        .to_request();
        assert_eq!(request.cursor.as_deref(), Some("AoIIQ0NjvXTE+Olo"));
    }

    #[test]
    fn tags_go_in_their_own_fields() {
        // Required and excluded tags in the wrong field returns plausible
        // results for the opposite query, which no test of "did it return
        // something" would catch.
        let request = BrowseQuery {
            app: AppId(4000),
            required_tags: vec!["Fun".to_owned()],
            excluded_tags: vec!["NSFW".to_owned()],
            match_all_tags: true,
            ..BrowseQuery::default()
        }
        .to_request();
        assert_eq!(request.requiredtags, vec!["Fun".to_owned()]);
        assert_eq!(request.excludedtags, vec!["NSFW".to_owned()]);
        assert_eq!(request.match_all_tags, Some(true));
    }

    #[test]
    fn tag_groups_are_their_own_field_and_not_flattened() {
        // Flattening (Scene or Video) and Anime into three required tags asks
        // for either all three or any of the three. Both return a plausible
        // page of the wrong query.
        let request = BrowseQuery {
            app: AppId(431_960),
            tag_groups: vec![
                vec!["Scene".to_owned(), "Video".to_owned()],
                vec!["Anime".to_owned()],
            ],
            ..BrowseQuery::default()
        }
        .to_request();

        assert!(
            request.requiredtags.is_empty(),
            "groups must not leak into the flat tag list"
        );
        assert_eq!(request.taggroups.len(), 2);
        assert_eq!(
            request.taggroups[0].tags,
            vec!["Scene".to_owned(), "Video".to_owned()]
        );
        assert_eq!(request.taggroups[1].tags, vec!["Anime".to_owned()]);
    }

    #[test]
    fn required_tags_and_groups_travel_together() {
        // A tag that must always be present is simpler flat than as a group of
        // one, so both fields have to survive the same request.
        let request = BrowseQuery {
            app: AppId(431_960),
            required_tags: vec!["Wallpaper".to_owned()],
            tag_groups: vec![vec!["Scene".to_owned()]],
            ..BrowseQuery::default()
        }
        .to_request();
        assert_eq!(request.requiredtags, vec!["Wallpaper".to_owned()]);
        assert_eq!(request.taggroups.len(), 1);
    }

    #[test]
    fn an_empty_tag_group_is_refused() {
        // Steam answers it with something; whatever that is, it is not the
        // filter anyone meant.
        let query = BrowseQuery {
            app: AppId(431_960),
            tag_groups: vec![Vec::new()],
            ..BrowseQuery::default()
        };
        assert_eq!(query.validate(), Err(BrowseError::EmptyTagGroup));
    }

    #[test]
    fn the_page_size_is_clamped_rather_than_sent_as_asked() {
        // Steam silently returns fewer than requested past its own limit, which
        // reads downstream as missing results.
        let big = BrowseQuery {
            app: AppId(4000),
            per_page: 5_000,
            ..BrowseQuery::default()
        }
        .to_request();
        assert_eq!(big.numperpage, Some(MAX_PER_PAGE));

        let zero = BrowseQuery {
            app: AppId(4000),
            per_page: 0,
            ..BrowseQuery::default()
        }
        .to_request();
        assert_eq!(zero.numperpage, Some(1), "zero would return nothing");
    }

    #[test]
    fn every_sort_maps_to_a_query_type_valve_defines() {
        // Valve's enum is sparse — 2 is not a query type — so a plausible
        // guess produces an empty result rather than an error.
        const DEFINED: [u32; 6] = [0, 1, 3, 9, 12, 21];
        for name in BrowseSort::NAMES {
            let sort = BrowseSort::parse(name).expect("a listed name must parse");
            assert!(
                DEFINED.contains(&sort.query_type()),
                "{name} maps to {}, which Valve does not define",
                sort.query_type()
            );
        }
    }

    #[test]
    fn sort_names_round_trip_through_the_parser() {
        assert_eq!(BrowseSort::parse("trend"), Some(BrowseSort::Trend));
        assert_eq!(BrowseSort::parse("popular"), Some(BrowseSort::Subscribed));
        assert_eq!(BrowseSort::parse("nonsense"), None);
    }

    #[test]
    fn a_search_without_an_app_is_refused() {
        let error = BrowseQuery::default().validate().expect_err("must refuse");
        assert_eq!(error, BrowseError::NoApp);
    }

    #[test]
    fn sorting_by_relevance_without_text_is_refused() {
        // It "works" and returns an arbitrary order, which is the worst kind of
        // wrong: it looks like a ranking.
        let query = BrowseQuery {
            app: AppId(4000),
            sort: BrowseSort::TextMatch,
            ..BrowseQuery::default()
        };
        assert_eq!(
            query.validate(),
            Err(BrowseError::TextSortWithoutText),
            "a text sort with no text should not be sent"
        );
    }

    #[test]
    fn a_result_carries_something_a_download_accepts() {
        let found = describe(&details(1, 1), Some(tapline_ids::DepotId(4001))).expect("describe");
        assert_eq!(found.item.id.get(), 1);
        assert_eq!(found.item.title, "An Addon");
        assert_eq!(found.description, "does things");
    }

    #[test]
    fn a_refused_item_is_reported_rather_than_failing_the_page() {
        // One deleted item in twenty must not lose the other nineteen.
        let (id, why) = describe(&details(7, 9), None).expect_err("must be skipped");
        assert_eq!(id, 7);
        assert!(!why.is_empty(), "a skipped item must say why");
    }

    #[test]
    fn tags_prefer_the_name_a_person_recognises() {
        let mut raw = details(1, 1);
        raw.tags = vec![
            Tag {
                tag: Some("fun".to_owned()),
                display_name: Some("Fun".to_owned()),
                adminonly: None,
            },
            Tag {
                tag: Some("weapon".to_owned()),
                display_name: None,
                adminonly: None,
            },
            Tag {
                tag: None,
                display_name: None,
                adminonly: None,
            },
        ];
        let found = describe(&raw, Some(tapline_ids::DepotId(4001))).expect("describe");
        assert_eq!(
            found.tags,
            vec!["Fun".to_owned(), "weapon".to_owned()],
            "display name first, raw tag as fallback, empties dropped"
        );
    }
}
