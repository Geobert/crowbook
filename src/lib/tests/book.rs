use super::test_eq;
use crate::book::Book;

#[test]
fn load_config() {
    let config = "
author: Author
title: Some title
description: >-
 A
 long
 description
epub.version: 3";
    let mut book = Book::new();
    book.read_config(config.as_bytes()).unwrap();
    test_eq(book.options.get_str("author").unwrap(), "Author");
    test_eq(book.options.get_str("title").unwrap(), "Some title");
    test_eq(
        book.options.get_str("description").unwrap(),
        "A long description",
    );
    assert_eq!(book.options.get_i32("epub.version").unwrap(), 3);
}
