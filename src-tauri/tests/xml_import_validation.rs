use kvm_manager_app_lib::utils::xml::{rewrite_first_text_element, validate_document_root};

#[test]
fn raw_import_documents_require_the_expected_root_and_well_formed_structure() {
    assert!(validate_document_root("<domain><name>guest</name></domain>", "domain").is_ok());
    assert!(validate_document_root("<filter name='safe'/>", "filter").is_ok());
    assert!(validate_document_root("<network/>", "domain").is_err());
    assert!(validate_document_root("<domain><name>guest</domain>", "domain").is_err());
}

#[test]
fn raw_import_accepts_namespaces_and_both_attribute_quote_styles() {
    let document = "<domain xmlns:vendor='urn:test'><name id=\"guest-1\">guest</name><vendor:extension mode='keep'/></domain>";
    assert!(validate_document_root(document, "domain").is_ok());
}

#[test]
fn targeted_transform_preserves_unknown_namespaced_elements() {
    let source = "<domain xmlns:vendor='urn:test'><name>old</name><vendor:extension flag='keep'>payload</vendor:extension></domain>";
    let transformed = rewrite_first_text_element(source, "name", "new & safe").unwrap();
    assert!(transformed.contains("<name>new &amp; safe</name>"));
    assert!(transformed.contains("vendor:extension"));
    assert!(transformed.contains("flag="));
    assert!(transformed.contains("keep"));
}
