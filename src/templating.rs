extern crate handlebars;
use super::model::Tag;
use chrono::NaiveDateTime;
use handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext, RenderError};
use log::debug;
use rust_embed::RustEmbed;
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;

#[derive(RustEmbed)]
#[folder = "src/templates/"]
struct Templates;

#[derive(Debug, Clone, Default)]
pub struct Templater {
    pub hb: Arc<Handlebars<'static>>,
}
// TODO: implement a render method on this struct to hide the details of the boxed ref

impl Templater {
    pub fn new() -> Templater {
        debug!("Creating template singleton");
        let mut reg = Handlebars::new();
        debug!("Registering prelude partial template");
        reg.register_partial(
            "prelude",
            std::str::from_utf8(Templates::get("prelude.hbs").unwrap().as_ref())
                .expect("Failed to load prelude.hbs"),
        )
        .expect("Failed to register prelude partial");
        debug!("Registering template files");
        reg.register_template_string(
            "display",
            std::str::from_utf8(Templates::get("display.hbs").unwrap().as_ref())
                .expect("Failed to load display.hbs"),
        )
        .expect("Failed to register display template");
        reg.register_template_string(
            "healthcheck",
            std::str::from_utf8(Templates::get("healthcheck.hbs").unwrap().as_ref())
                .expect("Failed to load healthcheck.hbs"),
        )
        .expect("Failed to register healthcheck template");
        reg.register_template_string(
            "tags",
            std::str::from_utf8(Templates::get("tags.hbs").unwrap().as_ref())
                .expect("Failed to load tags.hbs"),
        )
        .expect("Failed to register tag manager template");
        reg.register_template_string(
            "new_tag",
            std::str::from_utf8(Templates::get("new_tag.hbs").unwrap().as_ref())
                .expect("Failed to load new_tag.hbs"),
        )
        .expect("Failed to register new tag template");
        debug!("Registering template helpers");
        reg.register_helper("duration", Box::new(Templater::duration_helper));
        reg.register_helper("systime", Box::new(Templater::systime_helper));
        reg.register_helper("tag", Box::new(Templater::tag_helper));
        Templater { hb: Arc::new(reg) }
    }

    fn systime_helper(
        h: &Helper,
        _: &Handlebars,
        _: &Context,
        _rc: &mut RenderContext,
        out: &mut dyn Output,
    ) -> HelperResult {
        let param = h
            .param(0)
            .map(|v| v.value())
            .ok_or_else(|| RenderError::new("param not found"))?;
        let out_str = Templater::systime_inner(param)?;
        out.write(&out_str)?;
        Ok(())
    }

    pub(crate) fn systime_inner(v: &Value) -> Result<String, RenderError> {
        let systime: NaiveDateTime = serde_json::from_value(v.clone()).unwrap();
        Ok(systime.format("%c").to_string())
    }

    fn duration_helper(
        h: &Helper,
        _: &Handlebars,
        _: &Context,
        _rc: &mut RenderContext,
        out: &mut dyn Output,
    ) -> HelperResult {
        let d = h
            .param(0)
            .map(|v| v.value())
            .ok_or_else(|| RenderError::new("param not found"))?;
        let out_string = Templater::duration_inner(d)?;
        out.write(&out_string)?;
        Ok(())
    }

    pub(crate) fn duration_inner(d: &Value) -> Result<String, RenderError> {
        let this_duration: Duration = serde_json::from_value(d.clone()).unwrap();
        let this_in_sec = this_duration.as_secs();
        Ok(this_in_sec.to_string())
    }

    fn tag_helper(
        h: &Helper,
        _: &Handlebars,
        _: &Context,
        _rc: &mut RenderContext,
        out: &mut dyn Output,
    ) -> HelperResult {
        let tag_raw = h
            .param(0)
            .map(|v| v.value())
            .ok_or_else(|| RenderError::new("param not found"))?;
        let tag: Tag = serde_json::from_value(tag_raw.clone())?;
        let out_str = Templater::tag_inner(tag)?;
        out.write(&out_str)?;
        Ok(())
    }

    fn tag_inner(t: Tag) -> Result<String, RenderError> {
        let out_str = serde_json::to_string_pretty::<Tag>(&t)?;
        Ok(out_str)
    }
}
#[cfg(test)]
mod tests {
    use crate::templating::Templater;
    use ::chrono::{NaiveDate, NaiveDateTime};
    use serde_json::json;
    use std::time::Duration;

    #[test]
    fn test_duration_inner() {
        let value = Duration::from_millis(1234);
        let json_value = json!(Duration::from_millis(1234));
        let expected = value.as_secs().to_string();
        let result = Templater::duration_inner(&json_value)
            .expect("Json serialized duration should convert to string result");
        assert_eq!(result, expected);
    }

    #[test]
    fn test_systime_inner() {
        let value: NaiveDateTime = NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11);
        let json_value = json!(value);
        let expected = value.format("%c").to_string();
        let result = Templater::systime_inner(&json_value)
            .expect("Json serialized NaiveDateTime should convert to string result");
        assert_eq!(expected, result);
    }
}
