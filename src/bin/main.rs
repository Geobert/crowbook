// Copyright (C) 2016 Élisabeth HENRY.
//
// This file is part of Crowbook.
//
// Crowbook is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published
// by the Free Software Foundation, either version 2.1 of the License, or
// (at your option) any later version.
//
// Caribon is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Lesser General Public License for more details.
//
// You should have received ba copy of the GNU Lesser General Public License
// along with Crowbook.  If not, see <http://www.gnu.org/licenses/>.

extern crate crowbook;
extern crate clap;

mod helpers;
use helpers::*;

use crowbook::{Book};
use clap::ArgMatches;
use std::process::exit;
use std::fs::File;
use std::io;


/// Render a book to specific format
fn render_format(book: &mut Book, matches: &ArgMatches, format: &str) -> ! {
    if let Some(file) = matches.value_of("output") {
        let value = Some(file.to_owned());
        match format {
            "epub" => book.output_epub = value,
            "tex" => book.output_tex = value,
            "html" => book.output_html = value,
            "pdf" => book.output_pdf = value,
            "odt" => book.output_odt = value,
            _ => unreachable!()
        }
    }
    
    let option = match format {
        //                    if let &Some(ref file) = match format {
        "epub" => &book.output_epub,
        "tex" => &book.output_tex,
        "html" => &book.output_html,
        "pdf" => &book.output_pdf,
        "odt" => &book.output_odt,
        _ => unreachable!()
    };
    let result = match *option {
        None => {
            match format {
                "html" => book.render_html(&mut io::stdout()),
                "tex" => book.render_tex(&mut io::stdout()),
                _ => print_error(&format!("No output file specified, and book doesn't specify an output file for {}", format)),
            }
        },
        Some(ref file) => {
            match format {
                "epub" => book.render_epub(),
                "tex" => {
                    let mut f = File::create(file).unwrap();
                    book.render_tex(&mut f)
                },
                "html" => {
                    let mut f = File::create(file).unwrap();
                    book.render_html(&mut f)
                },
                "pdf" => book.render_pdf(),
                "odt" => book.render_odt(),
                _ => unreachable!()
            }
        }
    };
    match result {
        Err(err) => print_error(&format!("{}", err)),
        Ok(_) => {
                    println!("crowbook terminated successfully");
            exit(0);
        },
    }
}

/// sets the book options according to command line arguments
fn set_book_options(book: &mut Book, matches: &ArgMatches) {
     if let Some(autoclean) = matches.value_of("autoclean") {
        book.autoclean = match autoclean {
            "true" => true,
            "false" => false,
            _ => unreachable!()
        };
    }
    if let Some(numbering) = matches.value_of("numbering") {
        book.numbering = match numbering {
            "true" => true,
                        "false" => false,
            _ => unreachable!()
        };
    }

    if let Some(iter) = matches.values_of("set") {
        let v:Vec<_> = iter.collect();
        let name = v[0];
        let value = v[1];
        let dest = match name {
            "html.template" => &mut book.html_template,
            "html.css" => &mut book.html_css,
            "epub.css" => &mut book.epub_css,
                "epub.template" => &mut book.epub_template,
            "tex.template" => &mut book.tex_template,
            "decription" => &mut book.description,
            "subject" => &mut book.subject,
            "cover" => &mut book.cover,
            _ => print_error(&format!("{} is not a valid key.", name)),
        };
        *dest = Some(value.to_owned());
    }

}

fn main() {
    let matches = create_matches();

    if matches.is_present("create") {
        create_book(&matches);
    }

    // ok to unwrap since clap checks it's there
    let s = matches.value_of("BOOK").unwrap();
    let verbose = matches.is_present("verbose");
    match Book::new_from_file(s, verbose) {
        Err(err) => print_error(&format!("{}", err)),
        Ok(mut book) => {
            set_book_options(&mut book, &matches);
            
            if let Some(format) = matches.value_of("to") {
                render_format(&mut book, &matches, format);
            } else {
                match book.render_all() {
                    Err(err) => print_error(&format!("{}", err)),
                    Ok(_) => {
                        println!("crowbook terminated successfully");
                        exit(0);
                    }
                }
            }
        }
    }
}
