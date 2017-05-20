/*
 * Copyright (c) 2017 Boucher, Antoni <bouanto@zoho.com>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of
 * this software and associated documentation files (the "Software"), to deal in
 * the Software without restriction, including without limitation the rights to
 * use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
 * the Software, and to permit persons to whom the Software is furnished to do so,
 * subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
 * FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
 * COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
 * IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 * CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */

#![feature(proc_macro)]

extern crate env_logger;
#[macro_use]
extern crate log;
extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate relm_gen_widget;
extern crate syn;

use std::env;
use std::io::Write;
use std::process::{Command, Stdio};

use env_logger::LogBuilder;
use log::LogRecord;
use proc_macro::TokenStream;
use relm_gen_widget::gen_widget;
use syn::parse_item;

#[proc_macro_attribute]
pub fn widget(_attributes: TokenStream, input: TokenStream) -> TokenStream {
    let format = |record: &LogRecord| {
        record.args().to_string()
    };
    let mut builder = LogBuilder::new();
    builder.format(format);
    if let Ok(rust_log) = env::var("RUST_LOG") {
        builder.parse(&rust_log);
    }
    builder.init().ok();

    let source = input.to_string();
    let ast = parse_item(&source).unwrap();
    let tokens = quote! {
        #ast
    };
    let expanded = gen_widget(tokens);
    log_formatted(expanded.parse::<String>().unwrap());
    expanded.parse().unwrap()
}

fn log_formatted(code: String) {
    let command = Command::new("rustfmt")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn();
    warn!("{}", code);
    if let Ok(mut child) = command {
        warn!("*************************");
        {
            let stdin = child.stdin.as_mut().expect("failed to get stdin");
            write!(stdin, "{}", code).expect("failed to write to stdin");
        }
        let result = String::from_utf8(child.wait_with_output().expect("failed to wait on child").stdout)
            .expect("failed to decode rustfmt output");
        warn!("{}", result);
    }
}
