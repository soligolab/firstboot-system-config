use std::collections::HashMap;

use slint::{Image, ModelRc, SharedString, VecModel};

const DEFAULT_LANGUAGE_CODE: &str = "en";

#[derive(Clone, Debug)]
pub struct LanguagePack {
    pub code: String,
    pub native_name: String,
    pub flag_emoji: String,
    pub flag_svg: String,
    pub texts: HashMap<String, String>,
}

impl LanguagePack {
    pub fn text(&self, key: &str) -> SharedString {
        self.texts
            .get(key)
            .cloned()
            .unwrap_or_else(|| key.to_string())
            .into()
    }

    pub fn text_string(&self, key: &str) -> String {
        self.texts
            .get(key)
            .cloned()
            .unwrap_or_else(|| key.to_string())
    }

    pub fn flag_image(&self) -> Image {
        Image::load_from_svg_data(self.flag_svg.as_bytes()).unwrap_or_default()
    }
}

#[derive(Clone, Debug)]
pub struct LocalizationCatalog {
    languages: Vec<LanguagePack>,
    default_code: String,
}

impl LocalizationCatalog {
    pub fn load_embedded() -> Result<Self, String> {
        Self::from_xml(crate::web::languages_xml())
    }

    pub fn from_xml(xml: &str) -> Result<Self, String> {
        let languages_tag =
            extract_tag(xml, "languages").ok_or_else(|| "missing languages root".to_string())?;
        let default_code = extract_attr(languages_tag.open_tag, "default")
            .unwrap_or_else(|| DEFAULT_LANGUAGE_CODE.to_string());

        let mut languages = Vec::new();
        for language_block in extract_tag_blocks(languages_tag.inner, "language") {
            let code = extract_attr(language_block.open_tag, "code").unwrap_or_default();
            if code.is_empty() {
                continue;
            }

            let name =
                extract_attr(language_block.open_tag, "name").unwrap_or_else(|| code.clone());
            let native_name = extract_attr(language_block.open_tag, "native-name")
                .unwrap_or_else(|| name.clone());
            let flag = extract_tag(language_block.inner, "flag")
                .ok_or_else(|| format!("missing flag for language {code}"))?;
            let flag_emoji = extract_attr(flag.open_tag, "emoji").unwrap_or_default();
            let flag_svg = strip_cdata(flag.inner.trim());

            let mut texts = HashMap::new();
            for text_block in extract_tag_blocks(language_block.inner, "text") {
                if let Some(key) = extract_attr(text_block.open_tag, "key") {
                    texts.insert(key, strip_cdata(text_block.inner.trim()));
                }
            }

            languages.push(LanguagePack {
                code,
                native_name,
                flag_emoji,
                flag_svg,
                texts,
            });
        }

        if languages.is_empty() {
            return Err("no languages found".into());
        }

        Ok(Self {
            languages,
            default_code,
        })
    }

    pub fn default_index(&self) -> usize {
        self.index_by_code(&self.default_code).unwrap_or(0)
    }

    pub fn language(&self, index: usize) -> &LanguagePack {
        &self.languages[index.min(self.languages.len().saturating_sub(1))]
    }

    pub fn language_names_model(&self) -> ModelRc<SharedString> {
        ModelRc::new(VecModel::from(
            self.languages
                .iter()
                .map(|language| SharedString::from(language.native_name.clone()))
                .collect::<Vec<_>>(),
        ))
    }

    pub fn len(&self) -> usize {
        self.languages.len()
    }

    fn index_by_code(&self, code: &str) -> Option<usize> {
        self.languages
            .iter()
            .position(|language| language.code == code)
    }
}

struct TagBlock<'a> {
    open_tag: &'a str,
    inner: &'a str,
}

fn extract_tag<'a>(xml: &'a str, tag: &str) -> Option<TagBlock<'a>> {
    extract_tag_blocks(xml, tag).into_iter().next()
}

fn extract_tag_blocks<'a>(xml: &'a str, tag: &str) -> Vec<TagBlock<'a>> {
    let mut blocks = Vec::new();
    let start_token = format!("<{tag}");
    let end_token = format!("</{tag}>");
    let mut rest = xml;

    while let Some(start_idx) = rest.find(&start_token) {
        let after_start = &rest[start_idx..];
        let open_end_rel = match after_start.find('>') {
            Some(idx) => idx,
            None => break,
        };
        let inner_start = start_idx + open_end_rel + 1;
        let after_open = &rest[inner_start..];
        let inner_end_rel = match after_open.find(&end_token) {
            Some(idx) => idx,
            None => break,
        };
        let block_end = inner_start + inner_end_rel + end_token.len();
        blocks.push(TagBlock {
            open_tag: &rest[start_idx..start_idx + open_end_rel + 1],
            inner: &rest[inner_start..inner_start + inner_end_rel],
        });
        rest = &rest[block_end..];
    }

    blocks
}

fn extract_attr(tag: &str, attr: &str) -> Option<String> {
    let token = format!("{attr}=\"");
    let start = tag.find(&token)? + token.len();
    let remainder = &tag[start..];
    let end = remainder.find('"')?;
    Some(decode_xml_entities(&remainder[..end]))
}

fn strip_cdata(value: &str) -> String {
    let trimmed = value.trim();
    if let Some(inner) = trimmed
        .strip_prefix("<![CDATA[")
        .and_then(|rest| rest.strip_suffix("]]>"))
    {
        inner.to_string()
    } else {
        decode_xml_entities(trimmed)
    }
}

fn decode_xml_entities(value: &str) -> String {
    value
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}
