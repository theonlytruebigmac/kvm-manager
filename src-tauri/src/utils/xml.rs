use crate::utils::error::AppError;
use quick_xml::escape::{escape, unescape};
use quick_xml::events::{BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer, XmlVersion};

const MAX_DOCUMENT_BYTES: usize = 1024 * 1024;
const MAX_FIELD_CHARS: usize = 4096;

/// Validates an application supplied identifier before it becomes an XML name, resource name, or
/// other structured host input. Values intended as paths or free text use their own validator.
pub fn validate_identifier(value: &str, field: &str) -> Result<(), AppError> {
    if value.trim().is_empty() {
        return Err(AppError::InvalidConfig(format!(
            "{} must not be empty",
            field
        )));
    }
    if value.len() > 128 || value.chars().any(char::is_control) {
        return Err(AppError::InvalidConfig(format!(
            "{} has an invalid length or character",
            field
        )));
    }
    Ok(())
}

pub fn validate_text(value: &str, field: &str) -> Result<(), AppError> {
    if value.len() > MAX_FIELD_CHARS || value.chars().any(|character| character == '\0') {
        return Err(AppError::InvalidConfig(format!(
            "{} has an invalid length or character",
            field
        )));
    }
    Ok(())
}

pub fn escaped_text(value: &str, field: &str) -> Result<String, AppError> {
    validate_text(value, field)?;
    Ok(escape(value).into_owned())
}

/// Escapes an XML attribute value after applying the same bounded text rules used for element
/// content. `quick_xml` escapes both quote styles, so callers can safely use either XML quote
/// delimiter without allowing a supplied value to create a second attribute or element.
pub fn escaped_attribute(value: &str, field: &str) -> Result<String, AppError> {
    escaped_text(value, field)
}

pub fn xml_text_element(name: &str, value: &str) -> Result<String, AppError> {
    validate_identifier(name, "XML element name")?;
    Ok(format!(
        "<{}>{}</{}>",
        name,
        escaped_text(value, name)?,
        name
    ))
}

/// Rejects an oversized, malformed, or wrong-root raw XML document before libvirt receives it.
pub fn validate_document_root(document: &str, expected_root: &str) -> Result<(), AppError> {
    if document.len() > MAX_DOCUMENT_BYTES {
        return Err(AppError::InvalidConfig(
            "XML document exceeds the supported size".to_string(),
        ));
    }

    let mut reader = Reader::from_str(document);
    reader.config_mut().trim_text(false);
    let mut elements = Vec::new();
    let mut root_seen = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(element)) => {
                let local_name = element.local_name();
                let name = local_name.as_ref().to_string();
                if !root_seen {
                    if name != expected_root {
                        return Err(AppError::InvalidConfig(format!(
                            "Expected a <{}> document, received <{}>",
                            expected_root, name
                        )));
                    }
                    root_seen = true;
                } else if elements.is_empty() {
                    return Err(AppError::InvalidConfig(
                        "XML document has more than one root element".to_string(),
                    ));
                }
                elements.push(name);
            }
            Ok(Event::Empty(element)) => {
                let local_name = element.local_name();
                let name = local_name.as_ref();
                if !root_seen {
                    if name != expected_root {
                        return Err(AppError::InvalidConfig(format!(
                            "Expected a <{}> document, received <{}>",
                            expected_root, name
                        )));
                    }
                    root_seen = true;
                } else if elements.is_empty() {
                    return Err(AppError::InvalidConfig(
                        "XML document has more than one root element".to_string(),
                    ));
                }
            }
            Ok(Event::End(element)) => {
                let local_name = element.local_name();
                let name = local_name.as_ref();
                match elements.pop() {
                    Some(open) if open == name => {}
                    Some(_) | None => {
                        return Err(AppError::InvalidConfig(
                            "XML document has mismatched elements".to_string(),
                        ));
                    }
                }
            }
            Ok(Event::Text(text))
                if (!root_seen || elements.is_empty()) && !text.as_ref().trim().is_empty() =>
            {
                return Err(AppError::InvalidConfig(
                    "XML document has content outside its root element".to_string(),
                ));
            }
            Ok(Event::Eof) => {
                if !root_seen {
                    return Err(AppError::InvalidConfig(
                        "XML document has no root element".to_string(),
                    ));
                }
                if !elements.is_empty() {
                    return Err(AppError::InvalidConfig(
                        "XML document ended before all elements were closed".to_string(),
                    ));
                }
                return Ok(());
            }
            Ok(_) => {}
            Err(error) => {
                return Err(AppError::InvalidConfig(format!(
                    "Malformed XML document: {}",
                    error
                )));
            }
        }
    }
}

/// Reads an attribute from the first matching element using the XML event stream. This accepts
/// either quote style and decodes XML entities; it deliberately does not use substring matching.
pub fn first_element_attribute(
    document: &str,
    expected_root: &str,
    element_name: &str,
    attribute_name: &str,
) -> Result<Option<String>, AppError> {
    validate_identifier(element_name, "XML element name")?;
    validate_identifier(attribute_name, "XML attribute name")?;
    validate_document_root(document, expected_root)?;

    let mut reader = Reader::from_str(document);
    loop {
        match reader.read_event() {
            Ok(Event::Start(element)) | Ok(Event::Empty(element))
                if element.local_name().as_ref() == element_name =>
            {
                for attribute in element.attributes().with_checks(true) {
                    let attribute = attribute.map_err(|error| {
                        AppError::InvalidConfig(format!("Malformed XML attribute: {}", error))
                    })?;
                    if attribute.key.local_name().as_ref() == attribute_name {
                        return attribute
                            .normalized_value(XmlVersion::Implicit1_0)
                            .map(|value| Some(value.into_owned()))
                            .map_err(|error| {
                                AppError::InvalidConfig(format!(
                                    "Malformed XML attribute value: {}",
                                    error
                                ))
                            });
                    }
                }
            }
            Ok(Event::Eof) => return Ok(None),
            Ok(_) => {}
            Err(error) => {
                return Err(AppError::InvalidConfig(format!(
                    "Malformed XML document: {}",
                    error
                )));
            }
        }
    }
}

/// Counts matching elements using the event stream after validating the document root. This is
/// intentionally namespace-tolerant for libvirt extension documents while avoiding markup-like
/// text being mistaken for an element.
pub fn count_elements(
    document: &str,
    expected_root: &str,
    element_name: &str,
) -> Result<usize, AppError> {
    validate_identifier(element_name, "XML element name")?;
    validate_document_root(document, expected_root)?;

    let mut reader = Reader::from_str(document);
    let mut count = 0;
    loop {
        match reader.read_event() {
            Ok(Event::Start(element)) | Ok(Event::Empty(element))
                if element.local_name().as_ref() == element_name =>
            {
                count += 1;
            }
            Ok(Event::Eof) => return Ok(count),
            Ok(_) => {}
            Err(error) => {
                return Err(AppError::InvalidConfig(format!(
                    "Malformed XML document: {}",
                    error
                )));
            }
        }
    }
}

/// Reads the first direct text value from a matching XML element without depending on indentation
/// or byte offsets. Entity references are decoded before returning the application value.
pub fn first_element_text(
    document: &str,
    expected_root: &str,
    element_name: &str,
) -> Result<Option<String>, AppError> {
    validate_identifier(element_name, "XML element name")?;
    validate_document_root(document, expected_root)?;

    let mut reader = Reader::from_str(document);
    let mut depth = 0usize;
    let mut target_depth = None;
    loop {
        match reader.read_event() {
            Ok(Event::Start(element)) => {
                depth += 1;
                if target_depth.is_none() && element.local_name().as_ref() == element_name {
                    target_depth = Some(depth);
                }
            }
            Ok(Event::Text(text)) if target_depth == Some(depth) => {
                return unescape(text.as_ref())
                    .map(|value| Some(value.into_owned()))
                    .map_err(|error| {
                        AppError::InvalidConfig(format!("Malformed XML text value: {}", error))
                    });
            }
            Ok(Event::End(_)) => {
                if target_depth == Some(depth) {
                    return Ok(Some(String::new()));
                }
                depth = depth.saturating_sub(1);
            }
            Ok(Event::Eof) => return Ok(None),
            Ok(_) => {}
            Err(error) => {
                return Err(AppError::InvalidConfig(format!(
                    "Malformed XML document: {}",
                    error
                )));
            }
        }
    }
}

/// Rewrites only the first direct text child with `target_name`, copying every other XML event.
/// This provides a safe migration path for narrow existing-definition edits while preserving unknown
/// elements and namespaces.
pub fn rewrite_first_text_element(
    document: &str,
    target_name: &str,
    replacement: &str,
) -> Result<String, AppError> {
    validate_identifier(target_name, "XML element name")?;
    validate_text(replacement, target_name)?;
    validate_document_root(document, "domain")?;

    let mut reader = Reader::from_str(document);
    reader.config_mut().trim_text(false);
    let mut writer = Writer::new(Vec::new());
    let mut depth = 0usize;
    let mut target_depth = None;
    let mut rewritten = false;

    loop {
        let event = reader.read_event().map_err(|error| {
            AppError::InvalidConfig(format!("Malformed XML document: {}", error))
        })?;
        match event {
            Event::Start(element) => {
                depth += 1;
                let local_name = element.local_name();
                let is_target = local_name.as_ref() == target_name;
                if !rewritten && is_target {
                    target_depth = Some(depth);
                }
                writer
                    .write_event(Event::Start(element.into_owned()))
                    .map_err(xml_write_error)?;
            }
            Event::Text(_) if target_depth == Some(depth) && !rewritten => {
                writer
                    .write_event(Event::Text(BytesText::new(replacement)))
                    .map_err(xml_write_error)?;
                rewritten = true;
            }
            Event::End(element) => {
                writer
                    .write_event(Event::End(element.into_owned()))
                    .map_err(xml_write_error)?;
                if target_depth == Some(depth) {
                    target_depth = None;
                }
                depth = depth.saturating_sub(1);
            }
            Event::Eof => break,
            event => writer
                .write_event(event.into_owned())
                .map_err(xml_write_error)?,
        }
    }

    if !rewritten {
        return Err(AppError::InvalidConfig(format!(
            "No <{}> element was found",
            target_name
        )));
    }
    String::from_utf8(writer.into_inner())
        .map_err(|_| AppError::InvalidConfig("XML writer produced invalid UTF-8".to_string()))
}

/// Removes the first complete element with `target_name` while copying every other event. This is
/// used for clone-only identifiers such as a domain UUID, where the replacement is intentionally
/// absent rather than interpolated into the document.
pub fn remove_first_element(document: &str, target_name: &str) -> Result<String, AppError> {
    validate_identifier(target_name, "XML element name")?;
    validate_document_root(document, "domain")?;

    let mut reader = Reader::from_str(document);
    reader.config_mut().trim_text(false);
    let mut writer = Writer::new(Vec::new());
    let mut skipping_depth = None;

    loop {
        let event = reader.read_event().map_err(|error| {
            AppError::InvalidConfig(format!("Malformed XML document: {}", error))
        })?;
        match event {
            Event::Start(element)
                if skipping_depth.is_none() && element.local_name().as_ref() == target_name =>
            {
                skipping_depth = Some(1usize);
            }
            Event::Start(_) if skipping_depth.is_some() => {
                skipping_depth = skipping_depth.map(|depth| depth + 1);
            }
            Event::End(_) if skipping_depth.is_some() => {
                if let Some(depth) = skipping_depth {
                    skipping_depth = (depth > 1).then_some(depth - 1);
                }
            }
            Event::Empty(element)
                if skipping_depth.is_none() && element.local_name().as_ref() == target_name => {}
            Event::Eof => break,
            event if skipping_depth.is_none() => writer
                .write_event(event.into_owned())
                .map_err(xml_write_error)?,
            _ => {}
        }
    }

    String::from_utf8(writer.into_inner())
        .map_err(|_| AppError::InvalidConfig("XML writer produced invalid UTF-8".to_string()))
}

/// Inserts a text element immediately after the first completed `anchor_name` element while
/// preserving every unowned event and namespace in the domain document.
pub fn insert_text_element_after_first(
    document: &str,
    anchor_name: &str,
    element_name: &str,
    value: &str,
) -> Result<String, AppError> {
    validate_identifier(anchor_name, "XML anchor name")?;
    validate_identifier(element_name, "XML element name")?;
    validate_text(value, element_name)?;
    validate_document_root(document, "domain")?;

    let mut reader = Reader::from_str(document);
    reader.config_mut().trim_text(false);
    let mut writer = Writer::new(Vec::new());
    let mut depth = 0usize;
    let mut anchor_depth = None;
    let mut inserted = false;

    loop {
        let event = reader.read_event().map_err(|error| {
            AppError::InvalidConfig(format!("Malformed XML document: {}", error))
        })?;
        match event {
            Event::Start(element) => {
                depth += 1;
                if !inserted && element.local_name().as_ref() == anchor_name {
                    anchor_depth = Some(depth);
                }
                writer
                    .write_event(Event::Start(element.into_owned()))
                    .map_err(xml_write_error)?;
            }
            Event::End(element) => {
                writer
                    .write_event(Event::End(element.into_owned()))
                    .map_err(xml_write_error)?;
                if !inserted && anchor_depth == Some(depth) {
                    writer
                        .write_event(Event::Start(quick_xml::events::BytesStart::new(
                            element_name,
                        )))
                        .map_err(xml_write_error)?;
                    writer
                        .write_event(Event::Text(BytesText::new(value)))
                        .map_err(xml_write_error)?;
                    writer
                        .write_event(Event::End(quick_xml::events::BytesEnd::new(element_name)))
                        .map_err(xml_write_error)?;
                    inserted = true;
                }
                depth = depth.saturating_sub(1);
            }
            Event::Eof => break,
            event => writer
                .write_event(event.into_owned())
                .map_err(xml_write_error)?,
        }
    }

    if !inserted {
        return Err(AppError::InvalidConfig(format!(
            "No <{}> element was found",
            anchor_name
        )));
    }
    String::from_utf8(writer.into_inner())
        .map_err(|_| AppError::InvalidConfig("XML writer produced invalid UTF-8".to_string()))
}

/// Returns the first complete `element_name` whose descendant `child_name` has an attribute equal
/// to `attribute_value`. The returned XML is written from the parsed event stream, so quote style,
/// namespaces, and vendor-specific children remain intact.
pub fn first_element_with_descendant_attribute(
    document: &str,
    element_name: &str,
    child_name: &str,
    attribute_name: &str,
    attribute_value: &str,
) -> Result<Option<String>, AppError> {
    validate_identifier(element_name, "XML element name")?;
    validate_identifier(child_name, "XML child element name")?;
    validate_identifier(attribute_name, "XML attribute name")?;
    validate_text(attribute_value, "XML attribute value")?;
    validate_document_root(document, "domain")?;

    let mut reader = Reader::from_str(document);
    reader.config_mut().trim_text(false);
    let mut candidate: Option<Writer<Vec<u8>>> = None;
    let mut candidate_depth = 0usize;
    let mut matches = false;

    loop {
        let event = reader.read_event().map_err(|error| {
            AppError::InvalidConfig(format!("Malformed XML document: {}", error))
        })?;
        match event {
            Event::Start(element)
                if candidate.is_none() && element.local_name().as_ref() == element_name =>
            {
                if element.local_name().as_ref() == child_name
                    && has_attribute(&element, attribute_name, attribute_value)?
                {
                    matches = true;
                }
                let mut writer = Writer::new(Vec::new());
                writer
                    .write_event(Event::Start(element.into_owned()))
                    .map_err(xml_write_error)?;
                candidate = Some(writer);
                candidate_depth = 1;
            }
            Event::Start(element) if candidate.is_some() => {
                if element.local_name().as_ref() == child_name
                    && has_attribute(&element, attribute_name, attribute_value)?
                {
                    matches = true;
                }
                candidate_depth += 1;
                candidate
                    .as_mut()
                    .expect("candidate is present")
                    .write_event(Event::Start(element.into_owned()))
                    .map_err(xml_write_error)?;
            }
            Event::Empty(element) if candidate.is_some() => {
                if element.local_name().as_ref() == child_name
                    && has_attribute(&element, attribute_name, attribute_value)?
                {
                    matches = true;
                }
                candidate
                    .as_mut()
                    .expect("candidate is present")
                    .write_event(Event::Empty(element.into_owned()))
                    .map_err(xml_write_error)?;
            }
            Event::End(element) if candidate.is_some() => {
                candidate
                    .as_mut()
                    .expect("candidate is present")
                    .write_event(Event::End(element.into_owned()))
                    .map_err(xml_write_error)?;
                candidate_depth = candidate_depth.saturating_sub(1);
                if candidate_depth == 0 {
                    let candidate = candidate.take().expect("candidate is present");
                    if matches {
                        return String::from_utf8(candidate.into_inner())
                            .map(Some)
                            .map_err(|_| {
                                AppError::InvalidConfig(
                                    "XML writer produced invalid UTF-8".to_string(),
                                )
                            });
                    }
                    matches = false;
                }
            }
            Event::Eof => return Ok(None),
            event if candidate.is_some() => candidate
                .as_mut()
                .expect("candidate is present")
                .write_event(event.into_owned())
                .map_err(xml_write_error)?,
            _ => {}
        }
    }
}

/// Returns the first complete element with the requested local name, preserving its parsed XML.
pub fn first_element_fragment(
    document: &str,
    element_name: &str,
) -> Result<Option<String>, AppError> {
    validate_identifier(element_name, "XML element name")?;
    validate_document_root(document, "domain")?;
    let mut reader = Reader::from_str(document);
    reader.config_mut().trim_text(false);
    let mut writer: Option<Writer<Vec<u8>>> = None;
    let mut depth = 0usize;
    loop {
        let event = reader.read_event().map_err(|error| {
            AppError::InvalidConfig(format!("Malformed XML document: {}", error))
        })?;
        match event {
            Event::Start(element)
                if writer.is_none() && element.local_name().as_ref() == element_name =>
            {
                let mut current = Writer::new(Vec::new());
                current
                    .write_event(Event::Start(element.into_owned()))
                    .map_err(xml_write_error)?;
                writer = Some(current);
                depth = 1;
            }
            Event::Start(element) if writer.is_some() => {
                writer
                    .as_mut()
                    .expect("fragment writer present")
                    .write_event(Event::Start(element.into_owned()))
                    .map_err(xml_write_error)?;
                depth += 1;
            }
            Event::Empty(element) if writer.is_some() => writer
                .as_mut()
                .expect("fragment writer present")
                .write_event(Event::Empty(element.into_owned()))
                .map_err(xml_write_error)?,
            Event::End(element) if writer.is_some() => {
                writer
                    .as_mut()
                    .expect("fragment writer present")
                    .write_event(Event::End(element.into_owned()))
                    .map_err(xml_write_error)?;
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return String::from_utf8(
                        writer.take().expect("fragment writer present").into_inner(),
                    )
                    .map(Some)
                    .map_err(|_| {
                        AppError::InvalidConfig("XML writer produced invalid UTF-8".to_string())
                    });
                }
            }
            Event::Eof => return Ok(None),
            event if writer.is_some() => writer
                .as_mut()
                .expect("fragment writer present")
                .write_event(event.into_owned())
                .map_err(xml_write_error)?,
            _ => {}
        }
    }
}

/// Removes the first element whose descendant has the requested attribute value.
pub fn remove_element_with_descendant_attribute(
    document: &str,
    element_name: &str,
    child_name: &str,
    attribute_name: &str,
    attribute_value: &str,
) -> Result<String, AppError> {
    validate_identifier(element_name, "XML element name")?;
    validate_identifier(child_name, "XML child element name")?;
    validate_identifier(attribute_name, "XML attribute name")?;
    validate_text(attribute_value, "XML attribute value")?;
    validate_document_root(document, "domain")?;
    let mut reader = Reader::from_str(document);
    reader.config_mut().trim_text(false);
    let mut writer = Writer::new(Vec::new());
    let mut candidate: Option<Writer<Vec<u8>>> = None;
    let mut depth = 0usize;
    let mut matches = false;
    let mut removed = false;
    loop {
        let event = reader.read_event().map_err(|error| {
            AppError::InvalidConfig(format!("Malformed XML document: {}", error))
        })?;
        match event {
            Event::Start(element)
                if candidate.is_none()
                    && !removed
                    && element.local_name().as_ref() == element_name =>
            {
                let mut current = Writer::new(Vec::new());
                current
                    .write_event(Event::Start(element.into_owned()))
                    .map_err(xml_write_error)?;
                candidate = Some(current);
                depth = 1;
            }
            Event::Start(element) if candidate.is_some() => {
                if element.local_name().as_ref() == child_name
                    && has_attribute(&element, attribute_name, attribute_value)?
                {
                    matches = true;
                }
                candidate
                    .as_mut()
                    .expect("candidate present")
                    .write_event(Event::Start(element.into_owned()))
                    .map_err(xml_write_error)?;
                depth += 1;
            }
            Event::Empty(element) if candidate.is_some() => {
                if element.local_name().as_ref() == child_name
                    && has_attribute(&element, attribute_name, attribute_value)?
                {
                    matches = true;
                }
                candidate
                    .as_mut()
                    .expect("candidate present")
                    .write_event(Event::Empty(element.into_owned()))
                    .map_err(xml_write_error)?;
            }
            Event::End(element) if candidate.is_some() => {
                candidate
                    .as_mut()
                    .expect("candidate present")
                    .write_event(Event::End(element.into_owned()))
                    .map_err(xml_write_error)?;
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    let current = candidate.take().expect("candidate present");
                    if matches {
                        removed = true;
                    } else {
                        writer.get_mut().extend_from_slice(&current.into_inner());
                    }
                    matches = false;
                }
            }
            Event::Eof => break,
            event if candidate.is_some() => candidate
                .as_mut()
                .expect("candidate present")
                .write_event(event.into_owned())
                .map_err(xml_write_error)?,
            event => writer
                .write_event(event.into_owned())
                .map_err(xml_write_error)?,
        }
    }
    if !removed {
        return Err(AppError::InvalidConfig(format!(
            "No <{}> device matched",
            element_name
        )));
    }
    String::from_utf8(writer.into_inner())
        .map_err(|_| AppError::InvalidConfig("XML writer produced invalid UTF-8".to_string()))
}

/// Collects attribute values from matching descendant elements without relying on formatting.
pub fn descendant_attribute_values(
    document: &str,
    parent_name: &str,
    child_name: &str,
    attribute_name: &str,
) -> Result<Vec<String>, AppError> {
    validate_identifier(parent_name, "XML parent element name")?;
    validate_identifier(child_name, "XML child element name")?;
    validate_identifier(attribute_name, "XML attribute name")?;
    validate_document_root(document, "domain")?;
    let mut reader = Reader::from_str(document);
    reader.config_mut().trim_text(false);
    let mut parent_depth: Option<usize> = None;
    let mut depth = 0usize;
    let mut values = Vec::new();
    loop {
        let event = reader.read_event().map_err(|error| {
            AppError::InvalidConfig(format!("Malformed XML document: {}", error))
        })?;
        match event {
            Event::Start(element) => {
                depth += 1;
                if parent_depth.is_none() && element.local_name().as_ref() == parent_name {
                    parent_depth = Some(depth);
                } else if parent_depth.is_some() && element.local_name().as_ref() == child_name {
                    if let Some(value) = attribute_value(&element, attribute_name)? {
                        values.push(value);
                    }
                }
            }
            Event::Empty(element) => {
                if parent_depth.is_some() && element.local_name().as_ref() == child_name {
                    if let Some(value) = attribute_value(&element, attribute_name)? {
                        values.push(value);
                    }
                }
            }
            Event::End(_) => {
                if parent_depth == Some(depth) {
                    parent_depth = None;
                }
                depth = depth.saturating_sub(1);
            }
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(values)
}

/// Replaces the direct boot entries in `<os>` while preserving all other domain XML events.
/// `bootmenu` is intentionally removed because its old state would otherwise conflict with the
/// reviewed explicit device order.
pub fn rewrite_domain_boot_order(
    document: &str,
    boot_devices: &[&str],
) -> Result<String, AppError> {
    const ALLOWED: &[&str] = &["hd", "cdrom", "network", "fd"];
    if boot_devices.is_empty() || boot_devices.iter().any(|device| !ALLOWED.contains(device)) {
        return Err(AppError::InvalidConfig(
            "Boot order contains an unsupported device".to_string(),
        ));
    }
    validate_document_root(document, "domain")?;

    let mut reader = Reader::from_str(document);
    reader.config_mut().trim_text(false);
    let mut writer = Writer::new(Vec::new());
    let mut depth = 0usize;
    let mut os_depth = None;
    let mut skipping_depth = None;
    let mut found_os = false;

    loop {
        let event = reader.read_event().map_err(|error| {
            AppError::InvalidConfig(format!("Malformed XML document: {}", error))
        })?;
        match event {
            Event::Start(element) => {
                let is_direct_os_child = os_depth == Some(depth);
                if skipping_depth.is_some() {
                    skipping_depth = skipping_depth.map(|value| value + 1);
                } else if is_direct_os_child
                    && matches!(element.local_name().as_ref(), "boot" | "bootmenu")
                {
                    skipping_depth = Some(1);
                } else {
                    depth += 1;
                    if element.local_name().as_ref() == "os" {
                        os_depth = Some(depth);
                        found_os = true;
                    }
                    writer
                        .write_event(Event::Start(element.into_owned()))
                        .map_err(xml_write_error)?;
                }
            }
            Event::Empty(element) => {
                let is_direct_os_child = os_depth == Some(depth);
                if skipping_depth.is_none()
                    && is_direct_os_child
                    && matches!(element.local_name().as_ref(), "boot" | "bootmenu")
                {
                    continue;
                }
                if skipping_depth.is_none() {
                    writer
                        .write_event(Event::Empty(element.into_owned()))
                        .map_err(xml_write_error)?;
                }
            }
            Event::End(element) => {
                if let Some(skip_depth) = skipping_depth {
                    skipping_depth = (skip_depth > 1).then_some(skip_depth - 1);
                    continue;
                }
                if os_depth == Some(depth) {
                    for device in boot_devices {
                        writer
                            .write_event(Event::Empty(
                                BytesStart::new("boot").with_attributes([("dev", *device)]),
                            ))
                            .map_err(xml_write_error)?;
                    }
                    os_depth = None;
                }
                writer
                    .write_event(Event::End(element.into_owned()))
                    .map_err(xml_write_error)?;
                depth = depth.saturating_sub(1);
            }
            Event::Eof => break,
            event if skipping_depth.is_none() => writer
                .write_event(event.into_owned())
                .map_err(xml_write_error)?,
            _ => {}
        }
    }

    if !found_os {
        return Err(AppError::InvalidConfig(
            "No <os> section found in VM XML".to_string(),
        ));
    }
    String::from_utf8(writer.into_inner())
        .map_err(|_| AppError::InvalidConfig("XML writer produced invalid UTF-8".to_string()))
}

/// Replaces only the first CPU topology child. Existing CPU attributes, model choices, feature
/// flags, namespaces, and all non-topology children are copied as XML events.
pub fn rewrite_cpu_topology(
    document: &str,
    sockets: u32,
    cores: u32,
    threads: u32,
) -> Result<String, AppError> {
    if sockets == 0 || cores == 0 || threads == 0 {
        return Err(AppError::InvalidConfig(
            "CPU topology values must be greater than zero".to_string(),
        ));
    }
    validate_document_root(document, "domain")?;
    let mut reader = Reader::from_str(document);
    reader.config_mut().trim_text(false);
    let mut writer = Writer::new(Vec::new());
    let mut depth = 0usize;
    let mut cpu_depth = None;
    let mut skipping_depth = None;
    let mut rewritten = false;

    loop {
        let event = reader.read_event().map_err(|error| {
            AppError::InvalidConfig(format!("Malformed XML document: {}", error))
        })?;
        match event {
            Event::Start(element) => {
                if skipping_depth.is_some() {
                    skipping_depth = skipping_depth.map(|value| value + 1);
                    continue;
                }
                let is_direct_topology =
                    cpu_depth == Some(depth) && element.local_name().as_ref() == "topology";
                if is_direct_topology {
                    skipping_depth = Some(1);
                    continue;
                }
                depth += 1;
                if !rewritten && element.local_name().as_ref() == "cpu" {
                    cpu_depth = Some(depth);
                }
                writer
                    .write_event(Event::Start(element.into_owned()))
                    .map_err(xml_write_error)?;
            }
            Event::Empty(element) => {
                if skipping_depth.is_none()
                    && cpu_depth == Some(depth)
                    && element.local_name().as_ref() == "topology"
                {
                    continue;
                }
                if skipping_depth.is_none() {
                    writer
                        .write_event(Event::Empty(element.into_owned()))
                        .map_err(xml_write_error)?;
                }
            }
            Event::End(element) => {
                if let Some(skip_depth) = skipping_depth {
                    skipping_depth = (skip_depth > 1).then_some(skip_depth - 1);
                    continue;
                }
                if cpu_depth == Some(depth) {
                    write_cpu_topology(&mut writer, sockets, cores, threads)?;
                    cpu_depth = None;
                    rewritten = true;
                }
                writer
                    .write_event(Event::End(element.into_owned()))
                    .map_err(xml_write_error)?;
                depth = depth.saturating_sub(1);
            }
            Event::Eof => break,
            event if skipping_depth.is_none() => writer
                .write_event(event.into_owned())
                .map_err(xml_write_error)?,
            _ => {}
        }
    }
    if !rewritten {
        return Err(AppError::InvalidConfig(
            "No <cpu> section found in VM XML".to_string(),
        ));
    }
    String::from_utf8(writer.into_inner())
        .map_err(|_| AppError::InvalidConfig("XML writer produced invalid UTF-8".to_string()))
}

/// Updates the first CPU mode and (for custom mode) model while preserving all unrelated
/// CPU attributes, features, and child elements.
pub fn rewrite_cpu_model(
    document: &str,
    mode: &str,
    model: Option<&str>,
) -> Result<String, AppError> {
    if !["host-passthrough", "host-model", "custom"].contains(&mode) {
        return Err(AppError::InvalidConfig("CPU mode is invalid".to_string()));
    }
    let escaped_model = model.unwrap_or("qemu64");
    validate_text(escaped_model, "CPU model")?;
    validate_document_root(document, "domain")?;
    let mut reader = Reader::from_str(document);
    reader.config_mut().trim_text(false);
    let mut writer = Writer::new(Vec::new());
    let mut depth = 0usize;
    let mut cpu_depth = None;
    let mut skipping_model = None;
    let mut rewritten = false;

    loop {
        let event = reader.read_event().map_err(|error| {
            AppError::InvalidConfig(format!("Malformed XML document: {}", error))
        })?;
        match event {
            Event::Start(element)
                if cpu_depth.is_none() && element.local_name().as_ref() == "cpu" =>
            {
                let mut updated = BytesStart::new(element.name().as_ref().to_string());
                for attribute in element.attributes().with_checks(false) {
                    let attribute = attribute.map_err(|error| {
                        AppError::InvalidConfig(format!("Malformed CPU attribute: {}", error))
                    })?;
                    let key = attribute.key.local_name();
                    if !matches!(key.as_ref(), "mode" | "match" | "check") {
                        updated.push_attribute((attribute.key.as_ref(), attribute.value.as_ref()));
                    }
                }
                updated.push_attribute(("mode", mode));
                if mode == "custom" {
                    updated.push_attribute(("match", "exact"));
                    updated.push_attribute(("check", "none"));
                }
                writer
                    .write_event(Event::Start(updated))
                    .map_err(xml_write_error)?;
                depth += 1;
                cpu_depth = Some(depth);
                rewritten = true;
            }
            Event::Start(element)
                if cpu_depth == Some(depth) && element.local_name().as_ref() == "model" =>
            {
                skipping_model = Some(1);
                depth += 1;
            }
            Event::Empty(element)
                if cpu_depth == Some(depth) && element.local_name().as_ref() == "model" => {}
            Event::Start(element) => {
                if let Some(value) = skipping_model.as_mut() {
                    *value += 1;
                } else {
                    writer
                        .write_event(Event::Start(element.into_owned()))
                        .map_err(xml_write_error)?;
                }
                depth += 1;
            }
            Event::Empty(element) => {
                if skipping_model.is_none() {
                    writer
                        .write_event(Event::Empty(element.into_owned()))
                        .map_err(xml_write_error)?;
                }
            }
            Event::End(element) => {
                if let Some(value) = skipping_model {
                    skipping_model = (value > 1).then_some(value - 1);
                    depth = depth.saturating_sub(1);
                    continue;
                }
                if cpu_depth == Some(depth) {
                    if mode == "custom" {
                        writer
                            .write_event(Event::Start(BytesStart::new("model")))
                            .map_err(xml_write_error)?;
                        writer
                            .write_event(Event::Text(BytesText::new(escaped_model)))
                            .map_err(xml_write_error)?;
                        writer
                            .write_event(Event::End(quick_xml::events::BytesEnd::new("model")))
                            .map_err(xml_write_error)?;
                    }
                    cpu_depth = None;
                }
                writer
                    .write_event(Event::End(element.into_owned()))
                    .map_err(xml_write_error)?;
                depth = depth.saturating_sub(1);
            }
            Event::Eof => break,
            event if skipping_model.is_none() => writer
                .write_event(event.into_owned())
                .map_err(xml_write_error)?,
            _ => {}
        }
    }
    if !rewritten {
        return Err(AppError::InvalidConfig(
            "No <cpu> section found in VM XML".to_string(),
        ));
    }
    String::from_utf8(writer.into_inner())
        .map_err(|_| AppError::InvalidConfig("XML writer produced invalid UTF-8".to_string()))
}

/// Rewrites the link state of the interface identified by its MAC address. Only the selected
/// interface and its direct `<link>` child are owned by this operation; every other event is
/// copied unchanged.
pub fn rewrite_interface_link_state(
    document: &str,
    mac_address: &str,
    link_state: &str,
) -> Result<String, AppError> {
    if !["up", "down"].contains(&link_state) {
        return Err(AppError::InvalidConfig(
            "Interface link state is invalid".to_string(),
        ));
    }
    validate_text(mac_address, "MAC address")?;
    validate_document_root(document, "domain")?;

    let mut reader = Reader::from_str(document);
    reader.config_mut().trim_text(false);
    let mut writer = Writer::new(Vec::new());
    let mut candidate: Option<Writer<Vec<u8>>> = None;
    let mut candidate_depth = 0usize;
    let mut mac_matches = false;
    let mut skip_depth = None;
    let mut found = false;

    loop {
        let event = reader.read_event().map_err(|error| {
            AppError::InvalidConfig(format!("Malformed XML document: {}", error))
        })?;
        match event {
            Event::Start(element)
                if candidate.is_none() && element.local_name().as_ref() == "interface" =>
            {
                let mut buffered = Writer::new(Vec::new());
                buffered
                    .write_event(Event::Start(element.into_owned()))
                    .map_err(xml_write_error)?;
                candidate = Some(buffered);
                candidate_depth = 1;
            }
            Event::Start(element) if candidate.is_some() => {
                if skip_depth.is_some() {
                    skip_depth = skip_depth.map(|depth| depth + 1);
                    continue;
                }
                if candidate_depth == 1 && element.local_name().as_ref() == "link" {
                    skip_depth = Some(1);
                    continue;
                }
                if element.local_name().as_ref() == "mac"
                    && has_attribute(&element, "address", mac_address)?
                {
                    mac_matches = true;
                }
                candidate_depth += 1;
                candidate
                    .as_mut()
                    .expect("interface candidate exists")
                    .write_event(Event::Start(element.into_owned()))
                    .map_err(xml_write_error)?;
            }
            Event::Empty(element) if candidate.is_some() => {
                if skip_depth.is_some() {
                    continue;
                }
                if candidate_depth == 1 && element.local_name().as_ref() == "link" {
                    continue;
                }
                if element.local_name().as_ref() == "mac"
                    && has_attribute(&element, "address", mac_address)?
                {
                    mac_matches = true;
                }
                candidate
                    .as_mut()
                    .expect("interface candidate exists")
                    .write_event(Event::Empty(element.into_owned()))
                    .map_err(xml_write_error)?;
            }
            Event::End(element) if candidate.is_some() => {
                if let Some(depth) = skip_depth {
                    skip_depth = (depth > 1).then_some(depth - 1);
                    continue;
                }
                if candidate_depth == 1 {
                    if mac_matches {
                        let mut link = BytesStart::new("link");
                        link.push_attribute(("state", link_state));
                        let buffered = candidate.as_mut().expect("interface candidate exists");
                        buffered
                            .write_event(Event::Empty(link))
                            .map_err(xml_write_error)?;
                        buffered
                            .write_event(Event::End(element.into_owned()))
                            .map_err(xml_write_error)?;
                        let buffered = candidate.take().expect("interface candidate exists");
                        let bytes = buffered.into_inner();
                        writer.get_mut().extend_from_slice(&bytes);
                        found = true;
                        candidate_depth = 0;
                        mac_matches = false;
                        continue;
                    }
                    candidate_depth = 0;
                    let buffered = candidate.take().expect("interface candidate exists");
                    writer.get_mut().extend_from_slice(&buffered.into_inner());
                } else {
                    candidate_depth -= 1;
                    candidate
                        .as_mut()
                        .expect("interface candidate exists")
                        .write_event(Event::End(element.into_owned()))
                        .map_err(xml_write_error)?;
                }
            }
            Event::Eof => break,
            event if candidate.is_some() => candidate
                .as_mut()
                .expect("interface candidate exists")
                .write_event(event.into_owned())
                .map_err(xml_write_error)?,
            event => writer
                .write_event(event.into_owned())
                .map_err(xml_write_error)?,
        }
    }
    if !found {
        return Err(AppError::InvalidConfig(
            "Network interface with the selected MAC was not found".to_string(),
        ));
    }
    String::from_utf8(writer.into_inner())
        .map_err(|_| AppError::InvalidConfig("XML writer produced invalid UTF-8".to_string()))
}

/// Replaces the bandwidth child of the interface identified by `mac_address`. The selected
/// interface is buffered until its identity is known, so an unrelated interface (including its
/// extension elements and namespaces) is copied byte-for-byte through the event writer.
#[allow(clippy::too_many_arguments)]
pub fn rewrite_interface_bandwidth(
    document: &str,
    mac_address: &str,
    inbound_average: Option<u64>,
    inbound_peak: Option<u64>,
    inbound_burst: Option<u64>,
    outbound_average: Option<u64>,
    outbound_peak: Option<u64>,
    outbound_burst: Option<u64>,
) -> Result<String, AppError> {
    validate_text(mac_address, "MAC address")?;
    validate_document_root(document, "domain")?;

    let mut reader = Reader::from_str(document);
    reader.config_mut().trim_text(false);
    let mut writer = Writer::new(Vec::new());
    let mut candidate: Option<Writer<Vec<u8>>> = None;
    let mut depth = 0usize;
    let mut matches_mac = false;
    let mut found = false;

    loop {
        let event = reader.read_event().map_err(|error| {
            AppError::InvalidConfig(format!("Malformed XML document: {}", error))
        })?;
        match event {
            Event::Start(element)
                if candidate.is_none() && element.local_name().as_ref() == "interface" =>
            {
                let mut buffered = Writer::new(Vec::new());
                buffered
                    .write_event(Event::Start(element.into_owned()))
                    .map_err(xml_write_error)?;
                candidate = Some(buffered);
                depth = 1;
            }
            Event::Start(element) if candidate.is_some() => {
                if element.local_name().as_ref() == "mac"
                    && has_attribute(&element, "address", mac_address)?
                {
                    matches_mac = true;
                }
                depth += 1;
                candidate
                    .as_mut()
                    .expect("interface candidate exists")
                    .write_event(Event::Start(element.into_owned()))
                    .map_err(xml_write_error)?;
            }
            Event::Empty(element) if candidate.is_some() => {
                if element.local_name().as_ref() == "mac"
                    && has_attribute(&element, "address", mac_address)?
                {
                    matches_mac = true;
                }
                candidate
                    .as_mut()
                    .expect("interface candidate exists")
                    .write_event(Event::Empty(element.into_owned()))
                    .map_err(xml_write_error)?;
            }
            Event::End(element) if candidate.is_some() => {
                depth = depth.saturating_sub(1);
                candidate
                    .as_mut()
                    .expect("interface candidate exists")
                    .write_event(Event::End(element.into_owned()))
                    .map_err(xml_write_error)?;
                if depth == 0 {
                    let bytes = candidate
                        .take()
                        .expect("interface candidate exists")
                        .into_inner();
                    if matches_mac {
                        let fragment = String::from_utf8(bytes).map_err(|_| {
                            AppError::InvalidConfig("XML writer produced invalid UTF-8".to_string())
                        })?;
                        writer.get_mut().extend_from_slice(
                            rewrite_interface_bandwidth_fragment(
                                &fragment,
                                inbound_average,
                                inbound_peak,
                                inbound_burst,
                                outbound_average,
                                outbound_peak,
                                outbound_burst,
                            )?
                            .as_bytes(),
                        );
                        found = true;
                    } else {
                        writer.get_mut().extend_from_slice(&bytes);
                    }
                    matches_mac = false;
                }
            }
            Event::Eof => break,
            event if candidate.is_some() => candidate
                .as_mut()
                .expect("interface candidate exists")
                .write_event(event.into_owned())
                .map_err(xml_write_error)?,
            event => writer
                .write_event(event.into_owned())
                .map_err(xml_write_error)?,
        }
    }
    if !found {
        return Err(AppError::InvalidConfig(
            "Network interface with the selected MAC was not found".to_string(),
        ));
    }
    String::from_utf8(writer.into_inner())
        .map_err(|_| AppError::InvalidConfig("XML writer produced invalid UTF-8".to_string()))
}

/// Rewrites only the owned driver attributes and I/O-tuning child of the disk identified by its
/// target. Source, addressing, vendor elements, and all unrelated disk attributes are retained.
#[allow(clippy::too_many_arguments)]
pub fn rewrite_disk_settings(
    document: &str,
    target: &str,
    cache: Option<&str>,
    io: Option<&str>,
    discard: Option<&str>,
    detect_zeroes: Option<&str>,
    read_iops_sec: Option<u64>,
    write_iops_sec: Option<u64>,
    read_bytes_sec: Option<u64>,
    write_bytes_sec: Option<u64>,
) -> Result<String, AppError> {
    validate_text(target, "Disk target")?;
    validate_document_root(document, "domain")?;
    let mut reader = Reader::from_str(document);
    reader.config_mut().trim_text(false);
    let mut writer = Writer::new(Vec::new());
    let mut candidate: Option<Writer<Vec<u8>>> = None;
    let mut depth = 0usize;
    let mut matches_target = false;
    let mut found = false;

    loop {
        let event = reader.read_event().map_err(|error| {
            AppError::InvalidConfig(format!("Malformed XML document: {}", error))
        })?;
        match event {
            Event::Start(element)
                if candidate.is_none() && element.local_name().as_ref() == "disk" =>
            {
                let mut buffered = Writer::new(Vec::new());
                buffered
                    .write_event(Event::Start(element.into_owned()))
                    .map_err(xml_write_error)?;
                candidate = Some(buffered);
                depth = 1;
            }
            Event::Start(element) if candidate.is_some() => {
                if element.local_name().as_ref() == "target"
                    && has_attribute(&element, "dev", target)?
                {
                    matches_target = true;
                }
                depth += 1;
                candidate
                    .as_mut()
                    .expect("disk candidate exists")
                    .write_event(Event::Start(element.into_owned()))
                    .map_err(xml_write_error)?;
            }
            Event::Empty(element) if candidate.is_some() => {
                if element.local_name().as_ref() == "target"
                    && has_attribute(&element, "dev", target)?
                {
                    matches_target = true;
                }
                candidate
                    .as_mut()
                    .expect("disk candidate exists")
                    .write_event(Event::Empty(element.into_owned()))
                    .map_err(xml_write_error)?;
            }
            Event::End(element) if candidate.is_some() => {
                depth = depth.saturating_sub(1);
                candidate
                    .as_mut()
                    .expect("disk candidate exists")
                    .write_event(Event::End(element.into_owned()))
                    .map_err(xml_write_error)?;
                if depth == 0 {
                    let bytes = candidate
                        .take()
                        .expect("disk candidate exists")
                        .into_inner();
                    if matches_target {
                        let fragment = String::from_utf8(bytes).map_err(|_| {
                            AppError::InvalidConfig("XML writer produced invalid UTF-8".to_string())
                        })?;
                        writer.get_mut().extend_from_slice(
                            rewrite_disk_settings_fragment(
                                &fragment,
                                cache,
                                io,
                                discard,
                                detect_zeroes,
                                read_iops_sec,
                                write_iops_sec,
                                read_bytes_sec,
                                write_bytes_sec,
                            )?
                            .as_bytes(),
                        );
                        found = true;
                    } else {
                        writer.get_mut().extend_from_slice(&bytes);
                    }
                    matches_target = false;
                }
            }
            Event::Eof => break,
            event if candidate.is_some() => candidate
                .as_mut()
                .expect("disk candidate exists")
                .write_event(event.into_owned())
                .map_err(xml_write_error)?,
            event => writer
                .write_event(event.into_owned())
                .map_err(xml_write_error)?,
        }
    }
    if !found {
        return Err(AppError::InvalidConfig(
            "Disk with the selected target was not found".to_string(),
        ));
    }
    String::from_utf8(writer.into_inner())
        .map_err(|_| AppError::InvalidConfig("XML writer produced invalid UTF-8".to_string()))
}

/// Changes a disk source file only when the parsed `<source>` attribute exactly matches the old
/// path. This avoids modifying descriptions, metadata, or extension text that happens to contain
/// the same path.
pub fn rewrite_disk_source_path(
    document: &str,
    old_path: &str,
    new_path: &str,
) -> Result<String, AppError> {
    validate_text(old_path, "Existing disk path")?;
    validate_text(new_path, "New disk path")?;
    validate_document_root(document, "domain")?;
    let mut reader = Reader::from_str(document);
    reader.config_mut().trim_text(false);
    let mut writer = Writer::new(Vec::new());
    let mut depth = 0usize;
    let mut disk_depth = None;
    let mut rewritten = false;

    loop {
        let event = reader.read_event().map_err(|error| {
            AppError::InvalidConfig(format!("Malformed XML document: {}", error))
        })?;
        match event {
            Event::Start(element) => {
                let is_disk = element.local_name().as_ref() == "disk";
                if disk_depth.is_some()
                    && element.local_name().as_ref() == "source"
                    && has_attribute(&element, "file", old_path)?
                {
                    writer
                        .write_event(Event::Start(rewrite_source_file_attribute(
                            &element, new_path,
                        )?))
                        .map_err(xml_write_error)?;
                    rewritten = true;
                } else {
                    writer
                        .write_event(Event::Start(element.into_owned()))
                        .map_err(xml_write_error)?;
                }
                depth += 1;
                if disk_depth.is_none() && is_disk {
                    disk_depth = Some(depth);
                }
            }
            Event::Empty(element) => {
                if disk_depth.is_some()
                    && element.local_name().as_ref() == "source"
                    && has_attribute(&element, "file", old_path)?
                {
                    writer
                        .write_event(Event::Empty(rewrite_source_file_attribute(
                            &element, new_path,
                        )?))
                        .map_err(xml_write_error)?;
                    rewritten = true;
                } else {
                    writer
                        .write_event(Event::Empty(element.into_owned()))
                        .map_err(xml_write_error)?;
                }
            }
            Event::End(element) => {
                if disk_depth == Some(depth) {
                    disk_depth = None;
                }
                writer
                    .write_event(Event::End(element.into_owned()))
                    .map_err(xml_write_error)?;
                depth = depth.saturating_sub(1);
            }
            Event::Eof => break,
            event => writer
                .write_event(event.into_owned())
                .map_err(xml_write_error)?,
        }
    }
    if !rewritten {
        return Err(AppError::InvalidConfig(
            "No disk source matched the existing path".to_string(),
        ));
    }
    String::from_utf8(writer.into_inner())
        .map_err(|_| AppError::InvalidConfig("XML writer produced invalid UTF-8".to_string()))
}

/// Updates the domain's direct memory entries without accidentally changing an extension element
/// with the same local name.
pub fn rewrite_domain_memory(document: &str, value: &str) -> Result<String, AppError> {
    validate_text(value, "Memory")?;
    validate_document_root(document, "domain")?;
    let mut reader = Reader::from_str(document);
    reader.config_mut().trim_text(false);
    let mut writer = Writer::new(Vec::new());
    let mut depth = 0usize;
    let mut replacing: Option<usize> = None;
    let mut replaced_memory = false;
    let mut replaced_current = false;
    loop {
        let event = reader.read_event().map_err(|error| {
            AppError::InvalidConfig(format!("Malformed XML document: {}", error))
        })?;
        match event {
            Event::Start(element) => {
                depth += 1;
                if depth == 2 && matches!(element.local_name().as_ref(), "memory" | "currentMemory")
                {
                    replacing = Some(depth);
                }
                writer
                    .write_event(Event::Start(element.into_owned()))
                    .map_err(xml_write_error)?;
            }
            Event::Text(_) if replacing == Some(depth) => {
                writer
                    .write_event(Event::Text(BytesText::new(value)))
                    .map_err(xml_write_error)?;
            }
            Event::End(element) => {
                if replacing == Some(depth) {
                    match element.local_name().as_ref() {
                        "memory" => replaced_memory = true,
                        "currentMemory" => replaced_current = true,
                        _ => {}
                    }
                    replacing = None;
                }
                writer
                    .write_event(Event::End(element.into_owned()))
                    .map_err(xml_write_error)?;
                depth = depth.saturating_sub(1);
            }
            Event::Eof => break,
            event => writer
                .write_event(event.into_owned())
                .map_err(xml_write_error)?,
        }
    }
    if !replaced_memory || !replaced_current {
        return Err(AppError::InvalidConfig(
            "VM XML is missing direct memory settings".to_string(),
        ));
    }
    String::from_utf8(writer.into_inner())
        .map_err(|_| AppError::InvalidConfig("XML writer produced invalid UTF-8".to_string()))
}

/// Inserts a CPU definition immediately after the direct domain vCPU entry. This deliberately
/// avoids string-position insertion and leaves all existing domain events untouched.
pub fn insert_cpu_after_vcpu(
    document: &str,
    mode: &str,
    model: Option<&str>,
) -> Result<String, AppError> {
    if !["host-passthrough", "host-model", "custom"].contains(&mode) {
        return Err(AppError::InvalidConfig("CPU mode is invalid".to_string()));
    }
    validate_text(model.unwrap_or("qemu64"), "CPU model")?;
    validate_document_root(document, "domain")?;
    let mut reader = Reader::from_str(document);
    reader.config_mut().trim_text(false);
    let mut writer = Writer::new(Vec::new());
    let mut depth = 0usize;
    let mut vcpu_depth = None;
    let mut inserted = false;
    loop {
        let event = reader.read_event().map_err(|error| {
            AppError::InvalidConfig(format!("Malformed XML document: {}", error))
        })?;
        match event {
            Event::Start(element) => {
                depth += 1;
                if depth == 2 && element.local_name().as_ref() == "vcpu" {
                    vcpu_depth = Some(depth);
                }
                writer
                    .write_event(Event::Start(element.into_owned()))
                    .map_err(xml_write_error)?;
            }
            Event::Empty(element) => {
                let is_vcpu = depth == 1 && element.local_name().as_ref() == "vcpu";
                writer
                    .write_event(Event::Empty(element.into_owned()))
                    .map_err(xml_write_error)?;
                if is_vcpu && !inserted {
                    write_cpu_definition(&mut writer, mode, model)?;
                    inserted = true;
                }
            }
            Event::End(element) => {
                let is_vcpu = vcpu_depth == Some(depth);
                writer
                    .write_event(Event::End(element.into_owned()))
                    .map_err(xml_write_error)?;
                if is_vcpu && !inserted {
                    write_cpu_definition(&mut writer, mode, model)?;
                    inserted = true;
                }
                if is_vcpu {
                    vcpu_depth = None;
                }
                depth = depth.saturating_sub(1);
            }
            Event::Eof => break,
            event => writer
                .write_event(event.into_owned())
                .map_err(xml_write_error)?,
        }
    }
    if !inserted {
        return Err(AppError::InvalidConfig(
            "No direct <vcpu> element was found in VM XML".to_string(),
        ));
    }
    String::from_utf8(writer.into_inner())
        .map_err(|_| AppError::InvalidConfig("XML writer produced invalid UTF-8".to_string()))
}

/// Replaces one direct child of an owned parent while copying every other XML event. A missing
/// child is appended to the parent; `None` removes it. This keeps sibling ordering and unknown
/// namespace content intact without relying on byte positions or quote style.
pub fn replace_direct_child(
    document: &str,
    parent_name: &str,
    child_name: &str,
    replacement: Option<&str>,
) -> Result<String, AppError> {
    validate_identifier(parent_name, "XML parent element name")?;
    validate_identifier(child_name, "XML child element name")?;
    validate_document_root(document, "domain")?;
    if let Some(fragment) = replacement {
        validate_document_root(fragment, child_name)?;
    }

    let mut reader = Reader::from_str(document);
    reader.config_mut().trim_text(false);
    let mut writer = Writer::new(Vec::new());
    let mut depth = 0usize;
    let mut parent_depth = None;
    let mut skipping_depth = None;
    let mut found_parent = false;
    let mut replaced = false;

    loop {
        let event = reader.read_event().map_err(|error| {
            AppError::InvalidConfig(format!("Malformed XML document: {}", error))
        })?;
        match event {
            Event::Start(element) => {
                if skipping_depth.is_some() {
                    skipping_depth = skipping_depth.map(|value| value + 1);
                    continue;
                }
                if parent_depth == Some(depth)
                    && !replaced
                    && element.local_name().as_ref() == child_name
                {
                    skipping_depth = Some(1);
                    replaced = true;
                    continue;
                }
                depth += 1;
                if parent_depth.is_none() && element.local_name().as_ref() == parent_name {
                    parent_depth = Some(depth);
                    found_parent = true;
                }
                writer
                    .write_event(Event::Start(element.into_owned()))
                    .map_err(xml_write_error)?;
            }
            Event::Empty(element) => {
                if skipping_depth.is_some() {
                    continue;
                }
                if parent_depth == Some(depth)
                    && !replaced
                    && element.local_name().as_ref() == child_name
                {
                    if let Some(fragment) = replacement {
                        write_fragment_events(&mut writer, fragment)?;
                    }
                    replaced = true;
                    continue;
                }
                writer
                    .write_event(Event::Empty(element.into_owned()))
                    .map_err(xml_write_error)?;
            }
            Event::End(element) => {
                if let Some(value) = skipping_depth {
                    skipping_depth = (value > 1).then_some(value - 1);
                    if skipping_depth.is_none() {
                        if let Some(fragment) = replacement {
                            write_fragment_events(&mut writer, fragment)?;
                        }
                    }
                    continue;
                }
                if parent_depth == Some(depth) {
                    if !replaced {
                        if let Some(fragment) = replacement {
                            write_fragment_events(&mut writer, fragment)?;
                        }
                    }
                    parent_depth = None;
                }
                writer
                    .write_event(Event::End(element.into_owned()))
                    .map_err(xml_write_error)?;
                depth = depth.saturating_sub(1);
            }
            Event::Eof => break,
            event => {
                if skipping_depth.is_none() {
                    writer
                        .write_event(event.into_owned())
                        .map_err(xml_write_error)?;
                }
            }
        }
    }
    if !found_parent {
        return Err(AppError::InvalidConfig(format!(
            "No <{}> section found in VM XML",
            parent_name
        )));
    }
    String::from_utf8(writer.into_inner())
        .map_err(|_| AppError::InvalidConfig("XML writer produced invalid UTF-8".to_string()))
}

/// Replaces or removes the `vcpupin` entry for one vCPU without rebuilding the containing
/// `cputune` section. Other scheduler settings and extension elements are copied unchanged.
pub fn rewrite_vcpu_pin(
    document: &str,
    vcpu: u32,
    cpuset: Option<&str>,
) -> Result<String, AppError> {
    validate_document_root(document, "domain")?;
    if let Some(cpuset) = cpuset {
        validate_text(cpuset, "CPU set")?;
        if cpuset.is_empty() {
            return Err(AppError::InvalidConfig(
                "CPU set must not be empty".to_string(),
            ));
        }
    }

    let mut reader = Reader::from_str(document);
    reader.config_mut().trim_text(false);
    let mut writer = Writer::new(Vec::new());
    let mut depth = 0usize;
    let mut cputune_depth = None;
    let mut skipping_depth = None;
    let mut found_cputune = false;
    let mut replaced = false;

    loop {
        let event = reader.read_event().map_err(|error| {
            AppError::InvalidConfig(format!("Malformed XML document: {}", error))
        })?;
        match event {
            Event::Start(element) => {
                if skipping_depth.is_some() {
                    skipping_depth = skipping_depth.map(|value| value + 1);
                    continue;
                }
                if cputune_depth == Some(depth)
                    && !replaced
                    && element.local_name().as_ref() == "vcpupin"
                    && has_attribute(&element, "vcpu", &vcpu.to_string())?
                {
                    skipping_depth = Some(1);
                    replaced = true;
                    continue;
                }
                depth += 1;
                if cputune_depth.is_none() && element.local_name().as_ref() == "cputune" {
                    cputune_depth = Some(depth);
                    found_cputune = true;
                }
                writer
                    .write_event(Event::Start(element.into_owned()))
                    .map_err(xml_write_error)?;
            }
            Event::Empty(element) => {
                if skipping_depth.is_some() {
                    continue;
                }
                if cputune_depth == Some(depth)
                    && !replaced
                    && element.local_name().as_ref() == "vcpupin"
                    && has_attribute(&element, "vcpu", &vcpu.to_string())?
                {
                    if let Some(cpuset) = cpuset {
                        write_vcpu_pin(&mut writer, vcpu, cpuset)?;
                    }
                    replaced = true;
                    continue;
                }
                writer
                    .write_event(Event::Empty(element.into_owned()))
                    .map_err(xml_write_error)?;
            }
            Event::End(element) => {
                if let Some(value) = skipping_depth {
                    skipping_depth = (value > 1).then_some(value - 1);
                    if skipping_depth.is_none() {
                        if let Some(cpuset) = cpuset {
                            write_vcpu_pin(&mut writer, vcpu, cpuset)?;
                        }
                    }
                    continue;
                }
                if cputune_depth == Some(depth) {
                    if !replaced {
                        if let Some(cpuset) = cpuset {
                            write_vcpu_pin(&mut writer, vcpu, cpuset)?;
                        }
                    }
                    cputune_depth = None;
                }
                writer
                    .write_event(Event::End(element.into_owned()))
                    .map_err(xml_write_error)?;
                depth = depth.saturating_sub(1);
            }
            Event::Eof => break,
            event => {
                if skipping_depth.is_none() {
                    writer
                        .write_event(event.into_owned())
                        .map_err(xml_write_error)?;
                }
            }
        }
    }

    if !found_cputune {
        return match cpuset {
            Some(cpuset) => replace_direct_child(
                document,
                "domain",
                "cputune",
                Some(&format!(
                    "<cputune><vcpupin vcpu='{vcpu}' cpuset='{}'/></cputune>",
                    escaped_attribute(cpuset, "CPU set")?
                )),
            ),
            None => Ok(document.to_string()),
        };
    }
    String::from_utf8(writer.into_inner())
        .map_err(|_| AppError::InvalidConfig("XML writer produced invalid UTF-8".to_string()))
}

fn write_fragment_events(writer: &mut Writer<Vec<u8>>, fragment: &str) -> Result<(), AppError> {
    let mut reader = Reader::from_str(fragment);
    reader.config_mut().trim_text(false);
    loop {
        let event = reader.read_event().map_err(|error| {
            AppError::InvalidConfig(format!("Malformed XML replacement fragment: {}", error))
        })?;
        match event {
            Event::Eof => break,
            event => writer
                .write_event(event.into_owned())
                .map_err(xml_write_error)?,
        }
    }
    Ok(())
}

fn write_vcpu_pin(writer: &mut Writer<Vec<u8>>, vcpu: u32, cpuset: &str) -> Result<(), AppError> {
    let mut pin = BytesStart::new("vcpupin");
    let vcpu = vcpu.to_string();
    let cpuset = escaped_attribute(cpuset, "CPU set")?;
    pin.push_attribute(("vcpu", vcpu.as_str()));
    pin.push_attribute(("cpuset", cpuset.as_str()));
    writer
        .write_event(Event::Empty(pin))
        .map_err(xml_write_error)
}

#[allow(clippy::too_many_arguments)]
fn rewrite_interface_bandwidth_fragment(
    fragment: &str,
    inbound_average: Option<u64>,
    inbound_peak: Option<u64>,
    inbound_burst: Option<u64>,
    outbound_average: Option<u64>,
    outbound_peak: Option<u64>,
    outbound_burst: Option<u64>,
) -> Result<String, AppError> {
    let mut reader = Reader::from_str(fragment);
    reader.config_mut().trim_text(false);
    let mut writer = Writer::new(Vec::new());
    let mut depth = 0usize;
    let mut skip_depth = None;
    loop {
        let event = reader.read_event().map_err(|error| {
            AppError::InvalidConfig(format!("Malformed interface XML: {}", error))
        })?;
        match event {
            Event::Start(element) => {
                if skip_depth.is_some() {
                    skip_depth = skip_depth.map(|value| value + 1);
                    continue;
                }
                if depth == 1 && element.local_name().as_ref() == "bandwidth" {
                    skip_depth = Some(1);
                    continue;
                }
                depth += 1;
                writer
                    .write_event(Event::Start(element.into_owned()))
                    .map_err(xml_write_error)?;
            }
            Event::Empty(element) => {
                if skip_depth.is_none()
                    && depth == 1
                    && element.local_name().as_ref() == "bandwidth"
                {
                    continue;
                }
                if skip_depth.is_none() {
                    writer
                        .write_event(Event::Empty(element.into_owned()))
                        .map_err(xml_write_error)?;
                }
            }
            Event::End(element) => {
                if let Some(value) = skip_depth {
                    skip_depth = (value > 1).then_some(value - 1);
                    continue;
                }
                if depth == 1 {
                    write_bandwidth(
                        &mut writer,
                        inbound_average,
                        inbound_peak,
                        inbound_burst,
                        outbound_average,
                        outbound_peak,
                        outbound_burst,
                    )?;
                }
                writer
                    .write_event(Event::End(element.into_owned()))
                    .map_err(xml_write_error)?;
                depth = depth.saturating_sub(1);
            }
            Event::Eof => break,
            event if skip_depth.is_none() => writer
                .write_event(event.into_owned())
                .map_err(xml_write_error)?,
            _ => {}
        }
    }
    String::from_utf8(writer.into_inner())
        .map_err(|_| AppError::InvalidConfig("XML writer produced invalid UTF-8".to_string()))
}

#[allow(clippy::too_many_arguments)]
fn rewrite_disk_settings_fragment(
    fragment: &str,
    cache: Option<&str>,
    io: Option<&str>,
    discard: Option<&str>,
    detect_zeroes: Option<&str>,
    read_iops_sec: Option<u64>,
    write_iops_sec: Option<u64>,
    read_bytes_sec: Option<u64>,
    write_bytes_sec: Option<u64>,
) -> Result<String, AppError> {
    let mut reader = Reader::from_str(fragment);
    reader.config_mut().trim_text(false);
    let mut writer = Writer::new(Vec::new());
    let mut depth = 0usize;
    let mut skip_depth = None;
    let mut driver_seen = false;
    loop {
        let event = reader
            .read_event()
            .map_err(|error| AppError::InvalidConfig(format!("Malformed disk XML: {}", error)))?;
        match event {
            Event::Start(element) => {
                if skip_depth.is_some() {
                    skip_depth = skip_depth.map(|value| value + 1);
                    continue;
                }
                if depth == 1 && element.local_name().as_ref() == "iotune" {
                    skip_depth = Some(1);
                    continue;
                }
                if depth == 1 && element.local_name().as_ref() == "driver" {
                    driver_seen = true;
                    writer
                        .write_event(Event::Start(rewrite_driver_attributes(
                            &element,
                            cache,
                            io,
                            discard,
                            detect_zeroes,
                        )?))
                        .map_err(xml_write_error)?;
                    depth += 1;
                    continue;
                }
                depth += 1;
                writer
                    .write_event(Event::Start(element.into_owned()))
                    .map_err(xml_write_error)?;
            }
            Event::Empty(element) => {
                if skip_depth.is_none() && depth == 1 && element.local_name().as_ref() == "iotune" {
                    continue;
                }
                if skip_depth.is_none() && depth == 1 && element.local_name().as_ref() == "driver" {
                    driver_seen = true;
                    writer
                        .write_event(Event::Empty(rewrite_driver_attributes(
                            &element,
                            cache,
                            io,
                            discard,
                            detect_zeroes,
                        )?))
                        .map_err(xml_write_error)?;
                } else if skip_depth.is_none() {
                    writer
                        .write_event(Event::Empty(element.into_owned()))
                        .map_err(xml_write_error)?;
                }
            }
            Event::End(element) => {
                if let Some(value) = skip_depth {
                    skip_depth = (value > 1).then_some(value - 1);
                    continue;
                }
                if depth == 1 {
                    if !driver_seen {
                        write_driver(&mut writer, cache, io, discard, detect_zeroes)?;
                    }
                    write_iotune(
                        &mut writer,
                        read_iops_sec,
                        write_iops_sec,
                        read_bytes_sec,
                        write_bytes_sec,
                    )?;
                }
                writer
                    .write_event(Event::End(element.into_owned()))
                    .map_err(xml_write_error)?;
                depth = depth.saturating_sub(1);
            }
            Event::Eof => break,
            event if skip_depth.is_none() => writer
                .write_event(event.into_owned())
                .map_err(xml_write_error)?,
            _ => {}
        }
    }
    String::from_utf8(writer.into_inner())
        .map_err(|_| AppError::InvalidConfig("XML writer produced invalid UTF-8".to_string()))
}

fn rewrite_driver_attributes(
    element: &BytesStart<'_>,
    cache: Option<&str>,
    io: Option<&str>,
    discard: Option<&str>,
    detect_zeroes: Option<&str>,
) -> Result<BytesStart<'static>, AppError> {
    let mut updated = BytesStart::new(element.name().as_ref().to_string());
    for attribute in element.attributes().with_checks(true) {
        let attribute = attribute.map_err(|error| {
            AppError::InvalidConfig(format!("Malformed disk driver attribute: {}", error))
        })?;
        if !matches!(
            attribute.key.local_name().as_ref(),
            "cache" | "io" | "discard" | "detect_zeroes"
        ) {
            updated.push_attribute((attribute.key.as_ref(), attribute.value.as_ref()));
        }
    }
    for (name, value) in [
        ("cache", cache),
        ("io", io),
        ("discard", discard),
        ("detect_zeroes", detect_zeroes),
    ] {
        if let Some(value) = value {
            updated.push_attribute((name, value));
        }
    }
    Ok(updated)
}

fn rewrite_source_file_attribute(
    element: &BytesStart<'_>,
    new_path: &str,
) -> Result<BytesStart<'static>, AppError> {
    let mut updated = BytesStart::new(element.name().as_ref().to_string());
    for attribute in element.attributes().with_checks(true) {
        let attribute = attribute.map_err(|error| {
            AppError::InvalidConfig(format!("Malformed disk source attribute: {}", error))
        })?;
        if attribute.key.local_name().as_ref() == "file" {
            updated.push_attribute((attribute.key.as_ref(), new_path));
        } else {
            updated.push_attribute((attribute.key.as_ref(), attribute.value.as_ref()));
        }
    }
    Ok(updated)
}

fn write_driver(
    writer: &mut Writer<Vec<u8>>,
    cache: Option<&str>,
    io: Option<&str>,
    discard: Option<&str>,
    detect_zeroes: Option<&str>,
) -> Result<(), AppError> {
    let mut driver = BytesStart::new("driver");
    driver.push_attribute(("name", "qemu"));
    driver.push_attribute(("type", "qcow2"));
    for (name, value) in [
        ("cache", cache),
        ("io", io),
        ("discard", discard),
        ("detect_zeroes", detect_zeroes),
    ] {
        if let Some(value) = value {
            driver.push_attribute((name, value));
        }
    }
    writer
        .write_event(Event::Empty(driver))
        .map_err(xml_write_error)
}

#[allow(clippy::too_many_arguments)]
fn write_iotune(
    writer: &mut Writer<Vec<u8>>,
    read_iops_sec: Option<u64>,
    write_iops_sec: Option<u64>,
    read_bytes_sec: Option<u64>,
    write_bytes_sec: Option<u64>,
) -> Result<(), AppError> {
    let values = [
        ("read_iops_sec", read_iops_sec),
        ("write_iops_sec", write_iops_sec),
        ("read_bytes_sec", read_bytes_sec),
        ("write_bytes_sec", write_bytes_sec),
    ];
    if values.iter().all(|(_, value)| value.is_none()) {
        return Ok(());
    }
    writer
        .write_event(Event::Start(BytesStart::new("iotune")))
        .map_err(xml_write_error)?;
    for (name, value) in values {
        if let Some(value) = value {
            writer
                .write_event(Event::Start(BytesStart::new(name)))
                .map_err(xml_write_error)?;
            writer
                .write_event(Event::Text(BytesText::new(&value.to_string())))
                .map_err(xml_write_error)?;
            writer
                .write_event(Event::End(quick_xml::events::BytesEnd::new(name)))
                .map_err(xml_write_error)?;
        }
    }
    writer
        .write_event(Event::End(quick_xml::events::BytesEnd::new("iotune")))
        .map_err(xml_write_error)
}

#[allow(clippy::too_many_arguments)]
fn write_bandwidth(
    writer: &mut Writer<Vec<u8>>,
    inbound_average: Option<u64>,
    inbound_peak: Option<u64>,
    inbound_burst: Option<u64>,
    outbound_average: Option<u64>,
    outbound_peak: Option<u64>,
    outbound_burst: Option<u64>,
) -> Result<(), AppError> {
    if [
        inbound_average,
        inbound_peak,
        inbound_burst,
        outbound_average,
        outbound_peak,
        outbound_burst,
    ]
    .iter()
    .all(Option::is_none)
    {
        return Ok(());
    }
    writer
        .write_event(Event::Start(BytesStart::new("bandwidth")))
        .map_err(xml_write_error)?;
    write_bandwidth_direction(
        writer,
        "inbound",
        inbound_average,
        inbound_peak,
        inbound_burst,
    )?;
    write_bandwidth_direction(
        writer,
        "outbound",
        outbound_average,
        outbound_peak,
        outbound_burst,
    )?;
    writer
        .write_event(Event::End(quick_xml::events::BytesEnd::new("bandwidth")))
        .map_err(xml_write_error)
}

fn write_bandwidth_direction(
    writer: &mut Writer<Vec<u8>>,
    name: &str,
    average: Option<u64>,
    peak: Option<u64>,
    burst: Option<u64>,
) -> Result<(), AppError> {
    if average.is_none() && peak.is_none() && burst.is_none() {
        return Ok(());
    }
    let mut direction = BytesStart::new(name);
    for (attribute, value) in [("average", average), ("peak", peak), ("burst", burst)] {
        if let Some(value) = value {
            direction.push_attribute((attribute, value.to_string().as_str()));
        }
    }
    writer
        .write_event(Event::Empty(direction))
        .map_err(xml_write_error)
}

fn write_cpu_definition(
    writer: &mut Writer<Vec<u8>>,
    mode: &str,
    model: Option<&str>,
) -> Result<(), AppError> {
    let mut cpu = BytesStart::new("cpu");
    cpu.push_attribute(("mode", mode));
    cpu.push_attribute(("check", "none"));
    if mode == "custom" {
        cpu.push_attribute(("match", "exact"));
    }
    if mode == "custom" {
        writer
            .write_event(Event::Start(cpu))
            .map_err(xml_write_error)?;
        writer
            .write_event(Event::Start(BytesStart::new("model")))
            .map_err(xml_write_error)?;
        writer
            .write_event(Event::Text(BytesText::new(model.unwrap_or("qemu64"))))
            .map_err(xml_write_error)?;
        writer
            .write_event(Event::End(quick_xml::events::BytesEnd::new("model")))
            .map_err(xml_write_error)?;
        writer
            .write_event(Event::End(quick_xml::events::BytesEnd::new("cpu")))
            .map_err(xml_write_error)
    } else {
        writer
            .write_event(Event::Empty(cpu))
            .map_err(xml_write_error)
    }
}

fn write_cpu_topology(
    writer: &mut Writer<Vec<u8>>,
    sockets: u32,
    cores: u32,
    threads: u32,
) -> Result<(), AppError> {
    let sockets = sockets.to_string();
    let cores = cores.to_string();
    let threads = threads.to_string();
    let mut topology = BytesStart::new("topology");
    topology.push_attribute(("sockets", sockets.as_str()));
    topology.push_attribute(("cores", cores.as_str()));
    topology.push_attribute(("threads", threads.as_str()));
    writer
        .write_event(Event::Empty(topology))
        .map_err(xml_write_error)
}

fn has_attribute(
    element: &BytesStart<'_>,
    attribute_name: &str,
    expected_value: &str,
) -> Result<bool, AppError> {
    for attribute in element.attributes().with_checks(true) {
        let attribute = attribute.map_err(|error| {
            AppError::InvalidConfig(format!("Malformed XML attribute: {}", error))
        })?;
        if attribute.key.local_name().as_ref() == attribute_name {
            let value = attribute
                .normalized_value(XmlVersion::Implicit1_0)
                .map_err(|error| {
                    AppError::InvalidConfig(format!("Malformed XML attribute value: {}", error))
                })?;
            return Ok(value.as_ref() == expected_value);
        }
    }
    Ok(false)
}

fn attribute_value(
    element: &BytesStart<'_>,
    attribute_name: &str,
) -> Result<Option<String>, AppError> {
    for attribute in element.attributes().with_checks(true) {
        let attribute = attribute.map_err(|error| {
            AppError::InvalidConfig(format!("Malformed XML attribute: {}", error))
        })?;
        if attribute.key.local_name().as_ref() == attribute_name {
            let value = attribute
                .normalized_value(XmlVersion::Implicit1_0)
                .map_err(|error| {
                    AppError::InvalidConfig(format!("Malformed XML attribute value: {}", error))
                })?;
            return Ok(Some(value.into_owned()));
        }
    }
    Ok(None)
}

fn xml_write_error(error: std::io::Error) -> AppError {
    AppError::IoError(error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_text_without_creating_xml_structure() {
        assert_eq!(
            xml_text_element("name", "guest</name><uuid>unexpected").unwrap(),
            "<name>guest&lt;/name&gt;&lt;uuid&gt;unexpected</name>"
        );
    }

    #[test]
    fn validates_raw_document_root() {
        assert!(validate_document_root("<domain><name>a</name></domain>", "domain").is_ok());
        assert!(validate_document_root("<network/>", "domain").is_err());
        assert!(validate_document_root("<domain><name>a</domain>", "domain").is_err());
        assert!(validate_document_root("<domain/>trailing", "domain").is_err());
    }

    #[test]
    fn rewrites_only_target_text_and_preserves_unknown_content() {
        let result = rewrite_first_text_element(
            "<domain xmlns:vendor='urn:test'><name>old</name><vendor:extra attr='keep'>value</vendor:extra></domain>",
            "name",
            "new & safe",
        )
        .unwrap();
        assert!(result.contains("<name>new &amp; safe</name>"));
        assert!(result.contains("vendor:extra"));
        assert!(result.contains("keep"));
    }

    #[test]
    fn reads_attributes_without_assuming_quote_style_or_formatting() {
        let document = "<network><bridge name=\"br&amp;0\"/><ip address='192.0.2.1'/></network>";
        assert_eq!(
            first_element_attribute(document, "network", "bridge", "name").unwrap(),
            Some("br&0".to_string())
        );
        assert_eq!(
            first_element_attribute(document, "network", "ip", "address").unwrap(),
            Some("192.0.2.1".to_string())
        );
    }

    #[test]
    fn clone_metadata_transforms_preserve_unowned_extensions() {
        let document = "<domain xmlns:vendor='urn:test'><name>old</name><uuid>old-id</uuid><vendor:extra>keep</vendor:extra></domain>";
        let without_uuid = remove_first_element(document, "uuid").unwrap();
        let result =
            insert_text_element_after_first(&without_uuid, "name", "description", "safe & useful")
                .unwrap();
        assert!(!result.contains("<uuid>"));
        assert!(result.contains("<description>safe &amp; useful</description>"));
        assert!(result.contains("vendor:extra"));
    }

    #[test]
    fn selects_device_and_rewrites_boot_order_without_format_assumptions() {
        let document = "<domain xmlns:vendor='urn:test'><os><type>hvm</type><boot dev=\"hd\"/><vendor:keep/></os><devices><disk device='disk'><target dev=\"vda\"/><vendor:keep/></disk></devices></domain>";
        let disk =
            first_element_with_descendant_attribute(document, "disk", "target", "dev", "vda")
                .unwrap()
                .unwrap();
        assert!(disk.contains("vendor:keep"));
        let rewritten = rewrite_domain_boot_order(document, &["cdrom", "hd"]).unwrap();
        assert!(rewritten.contains("vendor:keep"));
        assert!(rewritten.contains("boot dev=\"cdrom\""));
        assert_eq!(rewritten.matches("<boot ").count(), 2);
        assert!(
            rewritten.find("boot dev=\"cdrom\"").unwrap()
                < rewritten.find("boot dev=\"hd\"").unwrap()
        );
    }

    #[test]
    fn extracts_device_fragment_without_quote_or_format_assumptions() {
        let document = "<domain xmlns:vendor='urn:test'><devices><sound model=\"ich9\"><audio id='1'/></sound><vendor:extra/></devices></domain>";
        let sound = first_element_fragment(document, "sound").unwrap().unwrap();
        assert!(sound.contains("model=\"ich9\""));
        assert!(sound.contains("<audio id='1'/>"));
    }

    #[test]
    fn removes_only_the_selected_input_device() {
        let document = "<domain><devices><input type='evdev'><source dev=\"/dev/input/event7\"/></input><input type='evdev'><source dev='/dev/input/event8'/></input></devices></domain>";
        let rewritten = remove_element_with_descendant_attribute(
            document,
            "input",
            "source",
            "dev",
            "/dev/input/event7",
        )
        .unwrap();
        assert!(!rewritten.contains("event7"));
        assert!(rewritten.contains("event8"));
    }

    #[test]
    fn collects_nested_device_attributes_across_quote_styles() {
        let document = "<domain><devices><input><source dev=\"/dev/input/event7\"/></input><input><source dev='/dev/input/event8'/></input></devices></domain>";
        let values = descendant_attribute_values(document, "input", "source", "dev").unwrap();
        assert_eq!(values, vec!["/dev/input/event7", "/dev/input/event8"]);
    }

    #[test]
    fn rewrites_only_cpu_topology_and_keeps_cpu_extensions() {
        let document = "<domain xmlns:vendor='urn:test'><vcpu placement='auto'>2</vcpu><cpu mode='custom'><model>host</model><topology sockets='1' cores='2' threads='1'/><vendor:feature enabled='yes'/></cpu></domain>";
        let rewritten = rewrite_cpu_topology(document, 2, 3, 4).unwrap();
        assert!(rewritten.contains("mode="));
        assert!(rewritten.contains("custom"));
        assert!(rewritten.contains("vendor:feature"));
        assert!(rewritten.contains("sockets=\"2\" cores=\"3\" threads=\"4\""));
        assert_eq!(rewritten.matches("topology").count(), 1);
    }

    #[test]
    fn rewrites_cpu_mode_and_model_without_dropping_extensions() {
        let document = "<domain xmlns:vendor='urn:test'><cpu mode='host-model' check='partial' vendor:flag='keep'><model fallback='forbid'>old</model><feature policy='require' name='vmx'/><vendor:extra/></cpu></domain>";
        let rewritten = rewrite_cpu_model(document, "custom", Some("Skylake-Client")).unwrap();
        assert!(rewritten.contains("mode=\"custom\"") || rewritten.contains("mode='custom'"));
        assert!(rewritten.contains("Skylake-Client"));
        assert!(rewritten.contains("vendor:flag"));
        assert!(rewritten.contains("feature policy"));
        assert!(!rewritten.contains(">old</model>"));
    }

    #[test]
    fn rewrites_selected_interface_link_and_preserves_other_interfaces() {
        let document = "<domain xmlns:vendor='urn:test'><devices><interface type='network'><mac address='52:54:00:aa:bb:cc'/><vendor:keep/></interface><interface type='network'><mac address='52:54:00:11:22:33'/><link state='up'/></interface></devices></domain>";
        let rewritten =
            rewrite_interface_link_state(document, "52:54:00:11:22:33", "down").unwrap();
        assert!(rewritten.contains("52:54:00:aa:bb:cc"));
        assert!(rewritten.contains("vendor:keep"));
        assert!(rewritten.contains("state=\"down\"") || rewritten.contains("state='down'"));
        assert!(!rewritten.contains("state=\"up\""));
    }

    #[test]
    fn rewrites_disk_settings_without_dropping_extension_content() {
        let document = "<domain xmlns:vendor='urn:test'><devices><disk type='file' vendor:flag='keep'><driver name='qemu' type='qcow2' cache='none' vendor:mode='keep'/><source file='/safe/disk.qcow2'/><target dev=\"vda\" bus='virtio'/><vendor:keep/><iotune><read_iops_sec>1</read_iops_sec></iotune></disk><disk><target dev='vdb'/></disk></devices></domain>";
        let rewritten = rewrite_disk_settings(
            document,
            "vda",
            Some("writeback"),
            None,
            Some("unmap"),
            None,
            Some(42),
            None,
            None,
            None,
        )
        .unwrap();
        assert!(rewritten.contains("vendor:flag"));
        assert!(rewritten.contains("vendor:mode"));
        assert!(rewritten.contains("vendor:keep"));
        assert!(rewritten.contains("/safe/disk.qcow2"));
        assert!(rewritten.contains("cache=\"writeback\""));
        assert!(rewritten.contains("<read_iops_sec>42</read_iops_sec>"));
        assert!(rewritten.contains("dev=\"vdb\"") || rewritten.contains("dev='vdb'"));
    }

    #[test]
    fn rewrites_interface_bandwidth_and_direct_memory_by_events() {
        let document = "<domain xmlns:vendor='urn:test'><memory unit='MiB'>512</memory><currentMemory unit='MiB'>512</currentMemory><vendor:memory>keep</vendor:memory><devices><interface type='network'><mac address='52:54:00:aa:bb:cc'/><target dev='vnet7'/><vendor:keep/><bandwidth><inbound average='1'/></bandwidth></interface></devices></domain>";
        let with_memory = rewrite_domain_memory(document, "1024").unwrap();
        assert!(with_memory.contains("vendor:memory") && with_memory.contains("keep"));
        assert!(with_memory.contains("<memory") && with_memory.contains(">1024</memory>"));
        let rewritten = rewrite_interface_bandwidth(
            &with_memory,
            "52:54:00:aa:bb:cc",
            Some(12),
            Some(24),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        assert!(rewritten.contains("vendor:keep"));
        assert!(rewritten.contains("average=\"12\""));
        assert!(rewritten.contains("peak=\"24\""));
    }

    #[test]
    fn inserts_cpu_after_vcpu_without_string_positioning() {
        let document = "<domain xmlns:vendor='urn:test'><vcpu placement='static'>2</vcpu><vendor:keep/><devices/></domain>";
        let rewritten = insert_cpu_after_vcpu(document, "custom", Some("Skylake-Client")).unwrap();
        assert!(rewritten.contains("<vendor:keep"));
        assert!(rewritten.contains("Skylake-Client"));
        assert!(rewritten.find("<vcpu").unwrap() < rewritten.find("<cpu").unwrap());
    }

    #[test]
    fn removes_complete_device_events_without_losing_extensions() {
        let document = "<domain xmlns:vendor='urn:test'><devices><redirdev bus='usb' type='spicevmc'/><vendor:keep/><redirdev bus='usb' type='spicevmc'><alias name='redir1'/></redirdev></devices></domain>";
        let once = remove_first_element(document, "redirdev").unwrap();
        let rewritten = remove_first_element(&once, "redirdev").unwrap();
        assert!(!rewritten.contains("redirdev"));
        assert!(rewritten.contains("vendor:keep"));
    }

    #[test]
    fn replaces_owned_direct_child_without_affecting_extensions() {
        let document = "<domain xmlns:vendor='urn:test'><memory>512</memory><vendor:memory>keep</vendor:memory><memoryBacking><old/></memoryBacking><devices><vendor:keep/></devices></domain>";
        let rewritten = replace_direct_child(
            document,
            "domain",
            "memoryBacking",
            Some("<memoryBacking><hugepages/></memoryBacking>"),
        )
        .unwrap();
        assert!(rewritten.contains("vendor:memory"));
        assert!(rewritten.contains("vendor:keep"));
        assert!(rewritten.contains("hugepages"));
        assert!(!rewritten.contains("<old"));
    }

    #[test]
    fn rewrites_only_selected_vcpu_pin_and_preserves_tuning_extensions() {
        let document = "<domain xmlns:vendor='urn:test'><vcpu>2</vcpu><cputune><vcpupin vcpu=\"0\" cpuset=\"0\"/><vendor:policy value=\"keep\"/><vcpupin vcpu='1' cpuset='1'/></cputune></domain>";
        let rewritten = rewrite_vcpu_pin(document, 1, Some("2-3")).unwrap();
        assert!(rewritten.contains("vendor:policy"));
        assert!(rewritten.contains("vcpu=\"0\""));
        assert!(rewritten.contains("cpuset=\"2-3\""));
        let cleared = rewrite_vcpu_pin(&rewritten, 1, None).unwrap();
        assert!(cleared.contains("vendor:policy"));
        assert!(!cleared.contains("cpuset=\"2-3\""));
    }

    #[test]
    fn rewrites_disk_source_attribute_without_replacing_unowned_text() {
        let document = "<domain xmlns:vendor='urn:test'><description>/old/disk.qcow2</description><devices><disk type='file'><source file='/old/disk.qcow2'/><target dev='vda'/><vendor:note>/old/disk.qcow2</vendor:note></disk></devices></domain>";
        let rewritten =
            rewrite_disk_source_path(document, "/old/disk.qcow2", "/new/disk.qcow2").unwrap();
        assert!(rewritten.contains("file=\"/new/disk.qcow2\""));
        assert!(rewritten.contains("<description>/old/disk.qcow2</description>"));
        assert!(rewritten.contains("<vendor:note>/old/disk.qcow2</vendor:note>"));
    }
}
