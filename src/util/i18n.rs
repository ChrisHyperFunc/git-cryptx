use fluent_bundle::{FluentBundle, FluentError, FluentResource};
use std::borrow::Cow;
use unic_langid::LanguageIdentifier;

pub fn load_locale(language: &str) -> FluentBundle<FluentResource> {
    let ftl_string = match language {
        "zh" => include_str!("../../locales/zh.ftl"),
        _ => include_str!("../../locales/en.ftl"),
    };

    let resource =
        FluentResource::try_new(ftl_string.to_string()).expect("Failed to parse FTL string");
    let lang_id: LanguageIdentifier = language.parse().expect("Invalid language identifier");
    let mut bundle = FluentBundle::new(vec![lang_id]);
    bundle
        .add_resource(resource)
        .expect("Failed to add FTL resources to the bundle");
    bundle
}

pub fn format_pattern<'a>(
    bundle: &'a FluentBundle<FluentResource>,
    id: &str,
    errors: &mut Vec<FluentError>,
) -> Cow<'a, str> {
    bundle.format_pattern(
        bundle
            .get_message(id)
            .expect("Message not found")
            .value()
            .expect("Message has no value"),
        None,
        errors,
    )
}
