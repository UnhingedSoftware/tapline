use crate::{InstallError, WorkshopItem};
use tapline_ids::AppId;
use tapline_proto::enums_productinfo::EContentDescriptorID;
use tapline_proto::steammessages_publishedfile_steamclient::{
    CPublishedFile_QueryFiles_Request, EQueryFilesSearchTextTarget,
    c_published_file_query_files_request::{DateRange, TagGroup},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BrowseSort {
    #[default]
    Vote,
    Recent,
    Updated,
    Trend,
    Subscribed,
    TextMatch,
}

impl BrowseSort {
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

    pub const NAMES: [&'static str; 6] =
        ["vote", "recent", "updated", "trend", "subscribed", "text"];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentDescriptor {
    NudityOrSexual,
    ViolenceOrGore,
    AdultOnlySexual,
    GratuitousSexual,
    AnyMature,
}

impl ContentDescriptor {
    const fn id(self) -> i32 {
        match self {
            Self::NudityOrSexual => 1,
            Self::ViolenceOrGore => 2,
            Self::AdultOnlySexual => 3,
            Self::GratuitousSexual => 4,
            Self::AnyMature => 5,
        }
    }

    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "nudity" | "sexual" => Some(Self::NudityOrSexual),
            "violence" | "gore" => Some(Self::ViolenceOrGore),
            "adult-only" => Some(Self::AdultOnlySexual),
            "gratuitous" => Some(Self::GratuitousSexual),
            "mature" => Some(Self::AnyMature),
            _ => None,
        }
    }

    pub const NAMES: [&'static str; 5] =
        ["nudity", "violence", "adult-only", "gratuitous", "mature"];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextTarget {
    #[default]
    Everything,
    Title,
    Description,
}

impl TextTarget {
    const fn target(self) -> Option<i32> {
        match self {
            Self::Everything => None,
            Self::Title => Some(1),
            Self::Description => Some(2),
        }
    }

    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "all" | "everything" => Some(Self::Everything),
            "title" => Some(Self::Title),
            "description" | "body" => Some(Self::Description),
            _ => None,
        }
    }

    pub const NAMES: [&'static str; 3] = ["all", "title", "description"];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TimeRange {
    pub start: Option<u32>,
    pub end: Option<u32>,
}

impl TimeRange {
    const fn is_backwards(self) -> bool {
        match (self.start, self.end) {
            (Some(start), Some(end)) => start > end,
            _ => false,
        }
    }

    const fn to_wire(self) -> DateRange {
        DateRange {
            timestamp_start: self.start,
            timestamp_end: self.end,
        }
    }
}

pub const FIRST_PAGE: &str = "*";

pub const MAX_PER_PAGE: u32 = 100;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowseQuery {
    pub app: AppId,
    pub text: Option<String>,
    pub search_in: TextTarget,
    pub required_tags: Vec<String>,
    pub tag_groups: Vec<Vec<String>>,
    pub excluded_tags: Vec<String>,
    pub excluded_descriptors: Vec<ContentDescriptor>,
    pub match_all_tags: bool,
    pub sort: BrowseSort,
    pub created: Option<TimeRange>,
    pub updated: Option<TimeRange>,
    pub trend_days: Option<u32>,
    pub per_page: u32,
    pub cursor: Option<String>,
    pub page: Option<u32>,
}

impl Default for BrowseQuery {
    fn default() -> Self {
        Self {
            app: AppId(0),
            text: None,
            search_in: TextTarget::default(),
            required_tags: Vec::new(),
            tag_groups: Vec::new(),
            excluded_tags: Vec::new(),
            excluded_descriptors: Vec::new(),
            match_all_tags: false,
            sort: BrowseSort::default(),
            created: None,
            updated: None,
            trend_days: None,
            per_page: 20,
            cursor: None,
            page: None,
        }
    }
}

impl BrowseQuery {
    pub fn validate(&self) -> Result<(), BrowseError> {
        if self.app.get() == 0 {
            return Err(BrowseError::NoApp);
        }
        if self.sort == BrowseSort::TextMatch && self.text.is_none() {
            return Err(BrowseError::TextSortWithoutText);
        }
        if self.search_in != TextTarget::Everything && self.text.is_none() {
            return Err(BrowseError::TextTargetWithoutText);
        }
        if self.tag_groups.iter().any(Vec::is_empty) {
            return Err(BrowseError::EmptyTagGroup);
        }
        if self.page.is_some() && self.cursor.is_some() {
            return Err(BrowseError::CursorAndPage);
        }
        if self.trend_days.is_some() && self.sort != BrowseSort::Trend {
            return Err(BrowseError::TrendDaysWithoutTrendSort);
        }
        if self.created.is_some_and(TimeRange::is_backwards)
            || self.updated.is_some_and(TimeRange::is_backwards)
        {
            return Err(BrowseError::BackwardsTimeRange);
        }
        Ok(())
    }

    pub(crate) fn to_request(&self) -> CPublishedFile_QueryFiles_Request {
        CPublishedFile_QueryFiles_Request {
            query_type: Some(self.sort.query_type()),
            appid: Some(self.app.get()),
            days: self.trend_days,
            date_range_created: self.created.map(TimeRange::to_wire),
            date_range_updated: self.updated.map(TimeRange::to_wire),
            search_text: self.text.clone(),
            search_text_target: self.search_in.target().map(EQueryFilesSearchTextTarget),
            requiredtags: self.required_tags.clone(),
            taggroups: self
                .tag_groups
                .iter()
                .map(|tags| TagGroup { tags: tags.clone() })
                .collect(),
            excludedtags: self.excluded_tags.clone(),
            excluded_content_descriptors: self
                .excluded_descriptors
                .iter()
                .map(|descriptor| EContentDescriptorID(descriptor.id()))
                .collect(),
            match_all_tags: Some(self.match_all_tags),
            numperpage: Some(self.per_page.clamp(1, MAX_PER_PAGE)),
            page: self.page,
            cursor: if self.page.is_some() {
                None
            } else {
                Some(self.cursor.clone().unwrap_or_else(|| FIRST_PAGE.to_owned()))
            },
            return_tags: Some(true),
            return_short_description: Some(true),
            return_previews: Some(true),
            return_vote_data: Some(true),
            return_details: Some(true),
            strip_description_bbcode: Some(true),
            ..CPublishedFile_QueryFiles_Request::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Preview {
    pub url: Option<String>,
    pub youtube_id: Option<String>,
    pub kind: u32,
    pub order: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowseResult {
    pub item: WorkshopItem,
    pub description: String,
    pub tags: Vec<String>,
    pub preview_url: Option<String>,
    pub previews: Vec<Preview>,
    pub creator: Option<u64>,
    pub created: u32,
    pub subscriptions: u64,
    pub favorites: u64,
    pub views: u64,
    pub score: Option<f32>,
    pub votes_up: u64,
    pub votes_down: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowsePage {
    pub items: Vec<BrowseResult>,
    pub total: u32,
    pub next_cursor: Option<String>,
    pub skipped: Vec<(u64, String)>,
}

impl BrowsePage {
    #[must_use]
    pub fn has_more(&self) -> bool {
        self.next_cursor.is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowseError {
    NoApp,
    TextSortWithoutText,
    EmptyTagGroup,
    TrendDaysWithoutTrendSort,
    BackwardsTimeRange,
    TextTargetWithoutText,
    CursorAndPage,
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
            Self::CursorAndPage => write!(
                f,
                "a cursor and a page number are two ways of saying where a \
                 page starts, and they disagree; give one"
            ),
            Self::TextTargetWithoutText => write!(
                f,
                "narrowing the search to titles or descriptions needs search \
                 text; without it there is nothing to narrow"
            ),
            Self::BackwardsTimeRange => write!(
                f,
                "a time window that ends before it starts can contain nothing; \
                 an empty result would look like the search simply found none"
            ),
            Self::TrendDaysWithoutTrendSort => write!(
                f,
                "a trend window only applies to the trend sort; Steam ignores \
                 it for the others, which reads as the period having no effect"
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
                tag.display_name
                    .clone()
                    .filter(|name| !name.is_empty())
                    .or_else(|| tag.tag.clone())
                    .unwrap_or_default()
            })
            .filter(|tag| !tag.is_empty())
            .collect(),
        preview_url: details.preview_url.clone().filter(|url| !url.is_empty()),
        previews: details
            .previews
            .iter()
            .map(|preview| Preview {
                url: preview.url.clone().filter(|url| !url.is_empty()),
                youtube_id: preview.youtubevideoid.clone().filter(|id| !id.is_empty()),
                kind: preview.preview_type.unwrap_or(0),
                order: preview.sortorder.unwrap_or(0),
            })
            .collect(),
        creator: details.creator.filter(|creator| *creator != 0),
        created: details.time_created.unwrap_or(0),
        subscriptions: u64::from(details.subscriptions.unwrap_or(0)),
        favorites: u64::from(details.favorited.unwrap_or(0)),
        views: u64::from(details.views.unwrap_or(0)),
        score: details.vote_data.as_ref().and_then(|vote| vote.score),
        votes_up: details
            .vote_data
            .as_ref()
            .and_then(|vote| vote.votes_up)
            .map_or(0, u64::from),
        votes_down: details
            .vote_data
            .as_ref()
            .and_then(|vote| vote.votes_down)
            .map_or(0, u64::from),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tapline_proto::steammessages_publishedfile_steamclient::{
        PublishedFileDetails,
        published_file_details::{Preview as Preview_, Tag, VoteData},
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
        let request = BrowseQuery {
            app: AppId(4000),
            ..BrowseQuery::default()
        }
        .to_request();
        assert_eq!(request.cursor.as_deref(), Some("*"));
    }

    #[test]
    fn a_page_number_replaces_the_cursor_rather_than_joining_it() {
        let request = BrowseQuery {
            app: AppId(431_960),
            page: Some(7),
            ..BrowseQuery::default()
        }
        .to_request();
        assert_eq!(request.page, Some(7));
        assert!(request.cursor.is_none());
    }

    #[test]
    fn asking_for_both_a_cursor_and_a_page_is_refused() {
        let query = BrowseQuery {
            app: AppId(431_960),
            cursor: Some("AoIIQ0NjvXTE+Olo".to_owned()),
            page: Some(3),
            ..BrowseQuery::default()
        };
        assert_eq!(query.validate(), Err(BrowseError::CursorAndPage));
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
    fn narrowing_the_text_target_travels_and_the_default_does_not() {
        let default = BrowseQuery {
            app: AppId(431_960),
            text: Some("miku".to_owned()),
            ..BrowseQuery::default()
        }
        .to_request();
        assert!(default.search_text_target.is_none());

        let title = BrowseQuery {
            app: AppId(431_960),
            text: Some("miku".to_owned()),
            search_in: TextTarget::Title,
            ..BrowseQuery::default()
        }
        .to_request();
        assert_eq!(title.search_text_target.map(|t| t.value()), Some(1));
    }

    #[test]
    fn narrowing_without_text_is_refused() {
        let query = BrowseQuery {
            app: AppId(431_960),
            search_in: TextTarget::Title,
            ..BrowseQuery::default()
        };
        assert_eq!(query.validate(), Err(BrowseError::TextTargetWithoutText));
    }

    #[test]
    fn each_date_window_travels_in_its_own_field() {
        let request = BrowseQuery {
            app: AppId(431_960),
            created: Some(TimeRange {
                start: Some(1_000),
                end: None,
            }),
            updated: Some(TimeRange {
                start: None,
                end: Some(2_000),
            }),
            ..BrowseQuery::default()
        }
        .to_request();

        let created = request.date_range_created.expect("created window");
        assert_eq!(created.timestamp_start, Some(1_000));
        assert_eq!(created.timestamp_end, None);
        let updated = request.date_range_updated.expect("updated window");
        assert_eq!(updated.timestamp_start, None);
        assert_eq!(updated.timestamp_end, Some(2_000));
    }

    #[test]
    fn a_window_that_ends_before_it_starts_is_refused() {
        let query = BrowseQuery {
            app: AppId(431_960),
            updated: Some(TimeRange {
                start: Some(2_000),
                end: Some(1_000),
            }),
            ..BrowseQuery::default()
        };
        assert_eq!(query.validate(), Err(BrowseError::BackwardsTimeRange));
    }

    #[test]
    fn an_open_ended_window_is_not_backwards() {
        let query = BrowseQuery {
            app: AppId(431_960),
            updated: Some(TimeRange {
                start: Some(2_000),
                end: None,
            }),
            ..BrowseQuery::default()
        };
        assert_eq!(query.validate(), Ok(()));
    }

    #[test]
    fn excluded_descriptors_travel_as_valves_ids() {
        let request = BrowseQuery {
            app: AppId(431_960),
            excluded_descriptors: vec![
                ContentDescriptor::NudityOrSexual,
                ContentDescriptor::AnyMature,
            ],
            ..BrowseQuery::default()
        }
        .to_request();
        let sent: Vec<i32> = request
            .excluded_content_descriptors
            .iter()
            .map(|descriptor| descriptor.value())
            .collect();
        assert_eq!(sent, vec![1, 5]);
    }

    #[test]
    fn descriptor_names_round_trip_through_the_parser() {
        for name in ContentDescriptor::NAMES {
            assert!(
                ContentDescriptor::parse(name).is_some(),
                "{name} is listed but does not parse"
            );
        }
        assert_eq!(ContentDescriptor::parse("nonsense"), None);
    }

    #[test]
    fn a_trend_window_travels_as_days() {
        let request = BrowseQuery {
            app: AppId(431_960),
            sort: BrowseSort::Trend,
            trend_days: Some(180),
            ..BrowseQuery::default()
        }
        .to_request();
        assert_eq!(request.days, Some(180));
    }

    #[test]
    fn a_trend_window_on_another_sort_is_refused() {
        let query = BrowseQuery {
            app: AppId(431_960),
            sort: BrowseSort::Vote,
            trend_days: Some(7),
            ..BrowseQuery::default()
        };
        assert_eq!(
            query.validate(),
            Err(BrowseError::TrendDaysWithoutTrendSort)
        );
    }

    #[test]
    fn an_empty_tag_group_is_refused() {
        let query = BrowseQuery {
            app: AppId(431_960),
            tag_groups: vec![Vec::new()],
            ..BrowseQuery::default()
        };
        assert_eq!(query.validate(), Err(BrowseError::EmptyTagGroup));
    }

    #[test]
    fn the_page_size_is_clamped_rather_than_sent_as_asked() {
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
        let (id, why) = describe(&details(7, 9), None).expect_err("must be skipped");
        assert_eq!(id, 7);
        assert!(!why.is_empty(), "a skipped item must say why");
    }

    #[test]
    fn a_result_carries_what_a_tile_and_a_detail_pane_show() {
        let mut raw = details(1, 1);
        raw.creator = Some(76_561_197_960_287_930);
        raw.time_created = Some(1_600_000_000);
        raw.views = Some(4_242);
        raw.vote_data = Some(VoteData {
            score: Some(0.94),
            votes_up: Some(470),
            votes_down: Some(30),
            ..VoteData::default()
        });
        raw.previews = vec![
            Preview_ {
                url: Some("https://example.invalid/one.jpg".to_owned()),
                sortorder: Some(0),
                preview_type: Some(0),
                ..Preview_::default()
            },
            Preview_ {
                youtubevideoid: Some("abc123".to_owned()),
                sortorder: Some(1),
                preview_type: Some(1),
                ..Preview_::default()
            },
        ];

        let found = describe(&raw, Some(tapline_ids::DepotId(4001))).expect("describe");
        assert_eq!(found.creator, Some(76_561_197_960_287_930));
        assert_eq!(found.created, 1_600_000_000);
        assert_eq!(found.views, 4_242);
        assert_eq!(found.score, Some(0.94));
        assert_eq!((found.votes_up, found.votes_down), (470, 30));
        assert_eq!(found.previews.len(), 2);
        assert_eq!(
            found.previews[0].url.as_deref(),
            Some("https://example.invalid/one.jpg")
        );
        assert_eq!(found.previews[1].youtube_id.as_deref(), Some("abc123"));
    }

    #[test]
    fn a_creator_of_zero_is_nobody() {
        let mut raw = details(1, 1);
        raw.creator = Some(0);
        assert_eq!(
            describe(&raw, Some(tapline_ids::DepotId(4001)))
                .expect("describe")
                .creator,
            None
        );
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
