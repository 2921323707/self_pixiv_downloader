use crate::domain::R18Policy;
use crate::domain::{PixivWork, PixivWorkRef};
use crate::errors::AppError;

pub trait PixivClient: Send + Sync {
    fn fetch_work(&self, pixiv_id: &str) -> Result<PixivWork, AppError>;
    fn fetch_author_works(
        &self,
        author_uid: &str,
        limit: u32,
    ) -> Result<Vec<PixivWorkRef>, AppError>;
    fn fetch_bookmarks(
        &self,
        limit: u32,
        r18_policy: R18Policy,
    ) -> Result<Vec<PixivWorkRef>, AppError>;
    fn search_works_by_tags(
        &self,
        tags: &[String],
        negative_tags: &[String],
        limit: u32,
        r18_policy: R18Policy,
    ) -> Result<Vec<PixivWorkRef>, AppError>;
    fn download_image(&self, url: &str) -> Result<Vec<u8>, AppError>;
}

pub mod http {
    use reqwest::blocking::Client;
    use reqwest::header::{
        ACCEPT, ACCEPT_LANGUAGE, COOKIE, HeaderMap, HeaderValue, REFERER, USER_AGENT,
    };
    use serde_json::Value;

    use crate::domain::{ImageCategory, PixivPage, PixivWork, PixivWorkRef, R18Policy};
    use crate::errors::{AppError, ErrorCode};
    use crate::pixiv::PixivClient;

    pub struct PixivHttpClient {
        client: Client,
        cookie: String,
    }

    impl PixivHttpClient {
        pub fn new(php_sessid: impl Into<String>) -> Result<Self, AppError> {
            let cookie = php_sessid.into();
            if cookie.trim().is_empty() {
                return Err(AppError::new(
                    ErrorCode::MissingPixivCookie,
                    "PIXIV_PHPSESSID is required",
                ));
            }

            let client = Client::builder()
                .default_headers(default_headers(&cookie)?)
                .build()?;

            Ok(Self { client, cookie })
        }

        fn artwork_referer(pixiv_id: &str) -> String {
            format!("https://www.pixiv.net/artworks/{pixiv_id}")
        }
    }

    impl PixivClient for PixivHttpClient {
        fn fetch_work(&self, pixiv_id: &str) -> Result<PixivWork, AppError> {
            let url = format!("https://www.pixiv.net/ajax/illust/{pixiv_id}");
            let value: Value = self
                .client
                .get(url)
                .header(REFERER, Self::artwork_referer(pixiv_id))
                .send()?
                .error_for_status()?
                .json()?;

            parse_work(pixiv_id, &value)
        }

        fn download_image(&self, url: &str) -> Result<Vec<u8>, AppError> {
            let bytes = self
                .client
                .get(url)
                .header(REFERER, "https://www.pixiv.net/")
                .header(COOKIE, format!("PHPSESSID={}", self.cookie))
                .send()?
                .error_for_status()?
                .bytes()?;

            Ok(bytes.to_vec())
        }

        fn fetch_author_works(
            &self,
            author_uid: &str,
            limit: u32,
        ) -> Result<Vec<PixivWorkRef>, AppError> {
            if author_uid.trim().is_empty() {
                return Err(AppError::validation("author_uid cannot be empty"));
            }
            if limit == 0 {
                return Err(AppError::validation("limit must be at least 1"));
            }

            let url = format!("https://www.pixiv.net/ajax/user/{author_uid}/profile/all");
            let value: Value = self
                .client
                .get(url)
                .header(REFERER, format!("https://www.pixiv.net/users/{author_uid}"))
                .send()?
                .error_for_status()?
                .json()?;

            parse_author_work_refs(&value, limit)
        }

        fn fetch_bookmarks(
            &self,
            limit: u32,
            r18_policy: R18Policy,
        ) -> Result<Vec<PixivWorkRef>, AppError> {
            if limit == 0 {
                return Err(AppError::validation("limit must be at least 1"));
            }

            let user_uid = self.fetch_current_user_uid()?;
            let mut refs = Vec::new();
            let rest_modes: &[&str] = match r18_policy {
                R18Policy::OnlyR18 => &["show", "hide"],
                R18Policy::Exclude | R18Policy::IncludeBlurred | R18Policy::IncludeVisible => {
                    &["show", "hide"]
                }
            };

            for rest in rest_modes {
                if refs.len() >= limit as usize {
                    break;
                }
                let remaining = limit.saturating_sub(refs.len() as u32);
                let url = format!(
                    "https://www.pixiv.net/ajax/user/{user_uid}/illusts/bookmarks?tag=&offset=0&limit={remaining}&rest={rest}"
                );
                let value: Value = self
                    .client
                    .get(url)
                    .header(
                        REFERER,
                        format!("https://www.pixiv.net/users/{user_uid}/bookmarks/artworks"),
                    )
                    .send()?
                    .error_for_status()?
                    .json()?;
                refs.extend(parse_bookmark_work_refs(&value, remaining)?);
            }

            refs.sort_by(|left, right| {
                let left_id = left.pixiv_id.parse::<u64>().unwrap_or(0);
                let right_id = right.pixiv_id.parse::<u64>().unwrap_or(0);
                right_id.cmp(&left_id)
            });
            refs.dedup_by(|left, right| left.pixiv_id == right.pixiv_id);
            refs.truncate(limit as usize);
            Ok(refs)
        }

        fn search_works_by_tags(
            &self,
            tags: &[String],
            negative_tags: &[String],
            limit: u32,
            r18_policy: R18Policy,
        ) -> Result<Vec<PixivWorkRef>, AppError> {
            if limit == 0 {
                return Err(AppError::validation("limit must be at least 1"));
            }
            let tags = normalize_search_tags(tags);
            if tags.is_empty() {
                return Err(AppError::validation("tags cannot be empty"));
            }
            let negative_tags = normalize_search_tags(negative_tags);
            let mut terms = tags.clone();
            terms.extend(negative_tags.iter().map(|tag| format!("-{tag}")));
            let word = terms.join(" ");
            let primary_tag = tags.first().map(String::as_str).unwrap_or(word.as_str());
            let mode = match r18_policy {
                R18Policy::Exclude => "safe",
                R18Policy::OnlyR18 => "r18",
                R18Policy::IncludeBlurred | R18Policy::IncludeVisible => "all",
            };
            let url = format!(
                "https://www.pixiv.net/ajax/search/artworks/{}?word={}&order=date_d&mode={mode}&p=1&s_mode=s_tag&type=all&lang=zh",
                percent_encode(primary_tag),
                percent_encode(&word),
            );
            let value: Value = self
                .client
                .get(url)
                .header(
                    REFERER,
                    format!(
                        "https://www.pixiv.net/tags/{}/artworks",
                        percent_encode(primary_tag)
                    ),
                )
                .send()?
                .error_for_status()?
                .json()?;

            parse_search_work_refs(&value, limit)
        }
    }

    impl PixivHttpClient {
        pub fn fetch_current_user_uid(&self) -> Result<String, AppError> {
            let html = self
                .client
                .get("https://www.pixiv.net/")
                .header(REFERER, "https://www.pixiv.net/")
                .send()?
                .error_for_status()?
                .text()?;

            extract_current_user_uid(&html).ok_or_else(|| {
                AppError::new(
                    ErrorCode::PixivParseError,
                    "Pixiv response did not expose current user id",
                )
            })
        }
    }

    fn default_headers(cookie: &str) -> Result<HeaderMap, AppError> {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 PixivPlatform/0.1",
            ),
        );
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/json,text/plain,*/*"),
        );
        headers.insert(
            ACCEPT_LANGUAGE,
            HeaderValue::from_static("zh-CN,zh;q=0.9,en;q=0.8,ja;q=0.7"),
        );
        headers.insert(
            COOKIE,
            HeaderValue::from_str(&format!("PHPSESSID={cookie}")).map_err(|error| {
                AppError::new(
                    ErrorCode::ValidationError,
                    format!("invalid Pixiv cookie header: {error}"),
                )
            })?,
        );
        Ok(headers)
    }

    fn parse_work(pixiv_id: &str, value: &Value) -> Result<PixivWork, AppError> {
        if value.get("error").and_then(Value::as_bool).unwrap_or(false) {
            let message = value
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("Pixiv returned an error");
            return Err(AppError::new(ErrorCode::PixivParseError, message));
        }

        let body = value.get("body").ok_or_else(|| {
            AppError::new(ErrorCode::PixivParseError, "Pixiv response missing body")
        })?;

        let title = body.get("title").and_then(Value::as_str).map(str::to_owned);
        let author_uid = body
            .get("userId")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .or_else(|| {
                body.get("userId")
                    .and_then(Value::as_u64)
                    .map(|id| id.to_string())
            });
        let author_name = body
            .get("userName")
            .and_then(Value::as_str)
            .map(str::to_owned);
        let category = parse_category(body);
        let tags = parse_tags(body);
        let pages = parse_pages(body)?;

        Ok(PixivWork {
            pixiv_id: pixiv_id.to_owned(),
            title,
            author_uid,
            author_name,
            tags,
            category,
            pages,
        })
    }

    fn parse_category(body: &Value) -> ImageCategory {
        let x_restrict = body.get("xRestrict").and_then(Value::as_i64).unwrap_or(0);
        let sl = body.get("sl").and_then(Value::as_i64).unwrap_or(0);

        if x_restrict > 0 {
            ImageCategory::R18
        } else if sl >= 6 {
            ImageCategory::Nsfw
        } else {
            ImageCategory::Normal
        }
    }

    fn parse_tags(body: &Value) -> Vec<String> {
        body.pointer("/tags/tags")
            .and_then(Value::as_array)
            .map(|tags| {
                tags.iter()
                    .filter_map(|tag| tag.get("tag").and_then(Value::as_str))
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default()
    }

    fn parse_pages(body: &Value) -> Result<Vec<PixivPage>, AppError> {
        if let Some(original) = body.pointer("/urls/original").and_then(Value::as_str) {
            return Ok(vec![PixivPage {
                page_index: 0,
                original_url: original.to_owned(),
                width: body
                    .get("width")
                    .and_then(Value::as_u64)
                    .and_then(|value| u32::try_from(value).ok()),
                height: body
                    .get("height")
                    .and_then(Value::as_u64)
                    .and_then(|value| u32::try_from(value).ok()),
                extension: extension_from_url(original).map(str::to_owned),
            }]);
        }

        let page_count = body
            .get("pageCount")
            .and_then(Value::as_u64)
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or(1);
        let first_regular = body
            .pointer("/urls/regular")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                AppError::new(
                    ErrorCode::PixivParseError,
                    "Pixiv response missing original or regular image URL",
                )
            })?;

        let first_original = regular_to_original(first_regular);
        let pages = (0..page_count)
            .map(|page_index| {
                let original_url = first_original
                    .replace("_p0.", &format!("_p{page_index}."))
                    .replace("_p0_", &format!("_p{page_index}_"));
                PixivPage {
                    page_index,
                    extension: extension_from_url(&original_url).map(str::to_owned),
                    original_url,
                    width: None,
                    height: None,
                }
            })
            .collect();

        Ok(pages)
    }

    fn parse_author_work_refs(value: &Value, limit: u32) -> Result<Vec<PixivWorkRef>, AppError> {
        if value.get("error").and_then(Value::as_bool).unwrap_or(false) {
            let message = value
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("Pixiv returned an error");
            return Err(AppError::new(ErrorCode::PixivParseError, message));
        }

        let body = value.get("body").ok_or_else(|| {
            AppError::new(ErrorCode::PixivParseError, "Pixiv response missing body")
        })?;
        let mut ids = Vec::new();
        collect_work_ids(body.get("illusts"), &mut ids);
        collect_work_ids(body.get("manga"), &mut ids);

        ids.sort_by(|left, right| {
            let left_id = left.parse::<u64>().unwrap_or(0);
            let right_id = right.parse::<u64>().unwrap_or(0);
            right_id.cmp(&left_id)
        });
        ids.dedup();
        Ok(ids
            .into_iter()
            .take(limit as usize)
            .map(|pixiv_id| PixivWorkRef { pixiv_id })
            .collect())
    }

    fn parse_bookmark_work_refs(value: &Value, limit: u32) -> Result<Vec<PixivWorkRef>, AppError> {
        if value.get("error").and_then(Value::as_bool).unwrap_or(false) {
            let message = value
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("Pixiv returned an error");
            return Err(AppError::new(ErrorCode::PixivParseError, message));
        }

        let body = value.get("body").ok_or_else(|| {
            AppError::new(ErrorCode::PixivParseError, "Pixiv response missing body")
        })?;
        let mut ids = Vec::new();
        if let Some(works) = body.get("works").and_then(Value::as_array) {
            for work in works {
                collect_bookmark_ids(work, &mut ids);
            }
        } else {
            collect_bookmark_ids(body, &mut ids);
        }
        ids.dedup();
        Ok(ids
            .into_iter()
            .take(limit as usize)
            .map(|pixiv_id| PixivWorkRef { pixiv_id })
            .collect())
    }

    fn parse_search_work_refs(value: &Value, limit: u32) -> Result<Vec<PixivWorkRef>, AppError> {
        if value.get("error").and_then(Value::as_bool).unwrap_or(false) {
            let message = value
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("Pixiv returned an error");
            return Err(AppError::new(ErrorCode::PixivParseError, message));
        }

        let body = value.get("body").ok_or_else(|| {
            AppError::new(ErrorCode::PixivParseError, "Pixiv response missing body")
        })?;
        let mut ids = Vec::new();
        for pointer in [
            "/illustManga/data",
            "/illust/data",
            "/manga/data",
            "/data",
            "/works",
        ] {
            if let Some(section) = body.pointer(pointer) {
                collect_search_ids(section, &mut ids);
            }
        }
        if ids.is_empty() {
            collect_search_ids(body, &mut ids);
        }
        ids.dedup();
        Ok(ids
            .into_iter()
            .take(limit as usize)
            .map(|pixiv_id| PixivWorkRef { pixiv_id })
            .collect())
    }

    fn collect_bookmark_ids(value: &Value, ids: &mut Vec<String>) {
        match value {
            Value::Object(map) => {
                if let Some(id) = bookmark_work_id(map) {
                    ids.push(id);
                    return;
                }

                for (key, value) in map {
                    if matches!(key.as_str(), "bookmarkData" | "user" | "profileImageUrl") {
                        continue;
                    }
                    collect_bookmark_ids(value, ids);
                }
            }
            Value::Array(values) => {
                for value in values {
                    collect_bookmark_ids(value, ids);
                }
            }
            _ => {}
        }
    }

    fn collect_search_ids(value: &Value, ids: &mut Vec<String>) {
        match value {
            Value::Object(map) => {
                if let Some(id) = search_work_id(map) {
                    ids.push(id);
                    return;
                }

                for (key, value) in map {
                    if matches!(
                        key.as_str(),
                        "bookmarkData" | "user" | "profileImageUrl" | "tags"
                    ) {
                        continue;
                    }
                    collect_search_ids(value, ids);
                }
            }
            Value::Array(values) => {
                for value in values {
                    collect_search_ids(value, ids);
                }
            }
            _ => {}
        }
    }

    fn search_work_id(map: &serde_json::Map<String, Value>) -> Option<String> {
        for key in ["illustId", "illust_id", "workId"] {
            if let Some(id) = map.get(key).and_then(value_to_numeric_string) {
                return Some(id);
            }
        }

        let looks_like_work = map.contains_key("title")
            || map.contains_key("illustTitle")
            || map.contains_key("pageCount")
            || map.contains_key("url")
            || map.contains_key("urls")
            || map.contains_key("illustType");
        if looks_like_work {
            return map.get("id").and_then(value_to_numeric_string);
        }

        None
    }

    fn bookmark_work_id(map: &serde_json::Map<String, Value>) -> Option<String> {
        for key in ["illustId", "illust_id", "workId"] {
            if let Some(id) = map.get(key).and_then(value_to_numeric_string) {
                return Some(id);
            }
        }

        let looks_like_work = map.contains_key("title")
            || map.contains_key("illustTitle")
            || map.contains_key("pageCount")
            || map.contains_key("url")
            || map.contains_key("urls");
        if looks_like_work {
            return map.get("id").and_then(value_to_numeric_string);
        }

        None
    }

    fn value_to_numeric_string(value: &Value) -> Option<String> {
        value
            .as_str()
            .filter(|id| !id.is_empty() && id.chars().all(|c| c.is_ascii_digit()))
            .map(str::to_owned)
            .or_else(|| value.as_u64().map(|id| id.to_string()))
    }

    fn extract_current_user_uid(html: &str) -> Option<String> {
        for marker in [
            r#""userData":{"id":""#,
            r#""userData":{"id":"#,
            r#""userId":""#,
            r#""userId":"#,
            r#"pixiv.user.id = ""#,
        ] {
            if let Some(start) = html.find(marker) {
                let after = &html[start + marker.len()..];
                let id: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
                if !id.is_empty() {
                    return Some(id);
                }
            }
        }
        None
    }

    fn collect_work_ids(value: Option<&Value>, ids: &mut Vec<String>) {
        match value {
            Some(Value::Object(map)) => {
                for key in map.keys() {
                    if key.chars().all(|c| c.is_ascii_digit()) {
                        ids.push(key.to_owned());
                    }
                }
            }
            Some(Value::Array(values)) => {
                for value in values {
                    if let Some(id) = value
                        .as_str()
                        .filter(|id| !id.is_empty() && id.chars().all(|c| c.is_ascii_digit()))
                    {
                        ids.push(id.to_owned());
                    }
                }
            }
            _ => {}
        }
    }

    fn normalize_search_tags(tags: &[String]) -> Vec<String> {
        let mut normalized = Vec::new();
        for tag in tags {
            let tag = tag.trim();
            if tag.is_empty() {
                continue;
            }
            if !normalized.iter().any(|existing: &String| existing == tag) {
                normalized.push(tag.to_owned());
            }
            if normalized.len() == 12 {
                break;
            }
        }
        normalized
    }

    fn percent_encode(value: &str) -> String {
        let mut encoded = String::new();
        for byte in value.as_bytes() {
            match *byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    encoded.push(*byte as char)
                }
                _ => encoded.push_str(&format!("%{byte:02X}")),
            }
        }
        encoded
    }

    fn regular_to_original(url: &str) -> String {
        url.replace("/img-master/", "/img-original/")
            .replace("_master1200", "")
    }

    fn extension_from_url(url: &str) -> Option<&str> {
        url.rsplit_once('.')
            .map(|(_, ext)| ext)
            .and_then(|ext| ext.split('?').next())
            .filter(|ext| !ext.is_empty())
    }

    #[cfg(test)]
    mod tests {
        use serde_json::json;

        use super::{parse_bookmark_work_refs, parse_search_work_refs};

        #[test]
        fn req_dl_002_parse_bookmarks_ignores_empty_strings_and_bookmark_record_ids() {
            let value = json!({
                "error": false,
                "body": {
                    "works": [
                        {
                            "id": "123456",
                            "title": "work one",
                            "bookmarkData": { "id": "999999" },
                            "extra": ""
                        },
                        {
                            "illustId": "222222",
                            "bookmarkData": { "id": "888888" }
                        },
                        {
                            "id": "",
                            "title": "deleted or unavailable work",
                            "bookmarkData": { "id": "777777" }
                        }
                    ],
                    "irrelevant": ["", "333333"]
                }
            });

            let refs = parse_bookmark_work_refs(&value, 10).unwrap();
            let ids: Vec<_> = refs.into_iter().map(|work| work.pixiv_id).collect();

            assert_eq!(ids, vec!["123456", "222222"]);
        }

        #[test]
        fn req_ai_002_parse_search_results_ignores_user_and_bookmark_ids() {
            let value = json!({
                "error": false,
                "body": {
                    "illustManga": {
                        "data": [
                            {
                                "id": "123456",
                                "title": "search result",
                                "user": { "id": "999999" },
                                "bookmarkData": { "id": "888888" }
                            },
                            { "illustId": "222222", "illustTitle": "alt search result" },
                            { "id": "", "title": "deleted result" }
                        ]
                    }
                }
            });

            let refs = parse_search_work_refs(&value, 10).unwrap();
            let ids: Vec<_> = refs.into_iter().map(|work| work.pixiv_id).collect();

            assert_eq!(ids, vec!["123456", "222222"]);
        }
    }
}

#[cfg(test)]
pub mod mock {
    use std::collections::HashMap;

    use crate::domain::{PixivWork, PixivWorkRef, R18Policy};
    use crate::errors::{AppError, ErrorCode};
    use crate::pixiv::PixivClient;

    #[derive(Default)]
    pub struct MockPixivClient {
        works: HashMap<String, PixivWork>,
        images: HashMap<String, Vec<u8>>,
        author_works: HashMap<String, Vec<PixivWorkRef>>,
        bookmarks: Vec<PixivWorkRef>,
        tag_searches: HashMap<String, Vec<PixivWorkRef>>,
    }

    impl MockPixivClient {
        pub fn with_work(mut self, work: PixivWork) -> Self {
            self.works.insert(work.pixiv_id.clone(), work);
            self
        }

        pub fn with_image(mut self, url: impl Into<String>, bytes: Vec<u8>) -> Self {
            self.images.insert(url.into(), bytes);
            self
        }

        pub fn with_author_works(
            mut self,
            author_uid: impl Into<String>,
            works: Vec<PixivWorkRef>,
        ) -> Self {
            self.author_works.insert(author_uid.into(), works);
            self
        }

        pub fn with_bookmarks(mut self, works: Vec<PixivWorkRef>) -> Self {
            self.bookmarks = works;
            self
        }

        pub fn with_tag_search(mut self, tags: Vec<String>, works: Vec<PixivWorkRef>) -> Self {
            self.tag_searches.insert(tags.join("\n"), works);
            self
        }
    }

    impl PixivClient for MockPixivClient {
        fn fetch_work(&self, pixiv_id: &str) -> Result<PixivWork, AppError> {
            self.works.get(pixiv_id).cloned().ok_or_else(|| {
                AppError::new(
                    ErrorCode::PixivNotFound,
                    format!("Pixiv work {pixiv_id} not found"),
                )
            })
        }

        fn download_image(&self, url: &str) -> Result<Vec<u8>, AppError> {
            self.images.get(url).cloned().ok_or_else(|| {
                AppError::new(
                    ErrorCode::PixivNetworkError,
                    format!("No mock bytes for {url}"),
                )
            })
        }

        fn fetch_author_works(
            &self,
            author_uid: &str,
            limit: u32,
        ) -> Result<Vec<PixivWorkRef>, AppError> {
            let works = self.author_works.get(author_uid).cloned().ok_or_else(|| {
                AppError::new(
                    ErrorCode::PixivNotFound,
                    format!("Pixiv author {author_uid} not found"),
                )
            })?;
            Ok(works.into_iter().take(limit as usize).collect())
        }

        fn fetch_bookmarks(
            &self,
            limit: u32,
            _r18_policy: R18Policy,
        ) -> Result<Vec<PixivWorkRef>, AppError> {
            Ok(self
                .bookmarks
                .iter()
                .take(limit as usize)
                .cloned()
                .collect())
        }

        fn search_works_by_tags(
            &self,
            tags: &[String],
            _negative_tags: &[String],
            limit: u32,
            _r18_policy: R18Policy,
        ) -> Result<Vec<PixivWorkRef>, AppError> {
            let works = self
                .tag_searches
                .get(&tags.join("\n"))
                .cloned()
                .ok_or_else(|| {
                    AppError::new(
                        ErrorCode::PixivNotFound,
                        format!("Pixiv tag search {:?} not found", tags),
                    )
                })?;
            Ok(works.into_iter().take(limit as usize).collect())
        }
    }
}
