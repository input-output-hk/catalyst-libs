//! Validation tests.

use catalyst_signed_doc::*;

mod common;

#[tokio::test]
async fn test_check_category() {
  let (doc, _) = common::get_dummy_signed_doc();

  let provider = common::DummyCatSignDocProvider;

  let result = validator::validate(&doc, &provider).await;

  assert!(result.is_ok());
}

#[tokio::test]
async fn test_check_content_encoding() {
  
}

#[tokio::test]
async fn test_check_content_type() {
  
}

#[tokio::test]
async fn test_check_doc_ref() {
  
}

#[tokio::test]
async fn test_check_reply() {
  
}

#[tokio::test]
async fn test_check_section() {
  
}

#[tokio::test]
async fn test_check_template() {
  
}