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

use std::collections::HashMap;
use std::sync::Mutex;

use quote::{Tokens, ToTokens};
use syn;
use syn::Delimited;
use syn::DelimToken::{Brace, Bracket, Paren};
use syn::Lit::Str;
use syn::StrStyle::Cooked;
use syn::TokenTree::{self, Token};
use syn::Token::{At, Colon, Comma, Eq, FatArrow, Ident, Literal, ModSep, Pound};

use self::DefaultParam::*;
use self::EventValue::*;
use self::EventValueReturn::*;
use self::Value::*;
use Widget::*;

lazy_static! {
    static ref NAMES_INDEX: Mutex<HashMap<String, u32>> = Mutex::new(HashMap::new());
}

#[derive(PartialEq)]
enum DefaultParam {
    DefaultNoParam,
    DefaultOneParam,
}

enum Value {
    ChildProperties(HashMap<String, Tokens>),
    Value(Tokens),
}

#[derive(Debug)]
pub enum EventValueReturn {
    Return(Tokens, Tokens),
    WithoutReturn(Tokens),
}

#[derive(Debug)]
pub enum EventValue {
    CurrentWidget(EventValueReturn),
    ForeignWidget(Tokens, EventValueReturn),
}

#[derive(Debug)]
pub struct Event {
    pub params: Vec<String>,
    pub value: EventValue,
}

impl Event {
    fn new() -> Self {
        Event {
            value: CurrentWidget(WithoutReturn(Tokens::new())),
            params: vec!["_".to_string()],
        }
    }
}

#[derive(Debug)]
pub enum Widget {
    Gtk(GtkWidget),
    Relm(RelmWidget),
}

impl Widget {
    pub fn name(&self) -> &syn::Ident {
        match *self {
            Gtk(ref widget) => &widget.name,
            Relm(ref widget) => &widget.name,
        }
    }

    pub fn typ(&self) -> &syn::Ident {
        match *self {
            Gtk(ref widget) => &widget.gtk_type,
            Relm(ref widget) => &widget.relm_type,
        }
    }
}

#[derive(Debug)]
pub struct GtkWidget {
    pub child_properties: HashMap<String, Tokens>,
    pub children: Vec<Widget>,
    pub events: HashMap<String, Event>,
    pub gtk_type: syn::Ident,
    pub init_parameters: Vec<String>,
    pub name: syn::Ident,
    pub properties: HashMap<String, Tokens>,
    pub relm_name: Option<syn::Ident>,
    pub save: bool,
}

impl GtkWidget {
    fn new(gtk_type: &str) -> Self {
        let name = syn::Ident::new(gen_widget_name(&gtk_type));
        GtkWidget {
            child_properties: HashMap::new(),
            children: vec![],
            events: HashMap::new(),
            gtk_type: syn::Ident::new(gtk_type),
            init_parameters: vec![],
            name: name,
            properties: HashMap::new(),
            relm_name: None,
            save: false,
        }
    }
}

#[derive(Debug)]
pub struct RelmWidget {
    pub events: HashMap<String, Vec<Event>>,
    pub name: syn::Ident,
    pub relm_type: syn::Ident,
}

impl RelmWidget {
    fn new(relm_type: String) -> Self {
        let mut name = gen_widget_name(&relm_type);
        // Relm widgets are not used in the update() method; they are only saved to avoid dropping
        // their channel too soon.std
        // So prepend an underscore to hide a warning.
        name.insert(0, '_');
        RelmWidget {
            events: HashMap::new(),
            name: syn::Ident::new(name),
            relm_type: syn::Ident::new(relm_type),
        }
    }
}

pub fn parse(tokens: &[TokenTree]) -> Widget {
    let (widget, _) = parse_widget(tokens);
    Gtk(widget)
}

fn parse_widget(tokens: &[TokenTree]) -> (GtkWidget, &[TokenTree]) {
    let (gtk_type, mut tokens) = parse_qualified_name(tokens);
    let mut widget = GtkWidget::new(&gtk_type);
    if let TokenTree::Delimited(Delimited { delim: Paren, ref tts }) = tokens[0] {
        let parameters = parse_comma_list(tts);
        widget.init_parameters = parameters;
        tokens = &tokens[1..];
    }
    if let TokenTree::Delimited(Delimited { delim: Brace, ref tts }) = tokens[0] {
        let mut tts = &tts[..];
        while !tts.is_empty() {
            if &tts[0] == &Token(Pound) || try_parse_name(tts).is_some() {
                let (name, new_tts) = try_parse_name_attribute(tts);
                tts = new_tts;
                // GTK+ widget.
                if tts[0] == Token(Ident(syn::Ident::new("gtk"))) {
                    let (mut child, new_tts) = parse_widget(tts);
                    if let Some(name) = name {
                        child.save = true;
                        child.name = syn::Ident::new(name);
                    }
                    tts = new_tts;
                    widget.children.push(Gtk(child));
                }
                // Relm widget.
                else {
                    let (mut child, new_tts) = parse_relm_widget(tts);
                    if let Some(name) = name {
                        child.name = syn::Ident::new(name);
                    }
                    tts = new_tts;
                    widget.children.push(Relm(child));
                }
            }
            else {
                // Property or event.
                let (ident, _) = parse_ident(tts);
                tts = &tts[1..];
                match tts[0] {
                    Token(Colon) => {
                        tts = &tts[1..];
                        let (value, new_tts) = parse_value_or_child_properties(tts);
                        tts = new_tts;
                        match value {
                            ChildProperties(child_properties) => widget.child_properties = child_properties,
                            Value(value) => { widget.properties.insert(ident, value); },
                        }
                    },
                    TokenTree::Delimited(Delimited { delim: Paren, .. }) | Token(FatArrow) => {
                        let (event, new_tts) = parse_event(tts, DefaultOneParam);
                        widget.events.insert(ident, event);
                        tts = new_tts;
                    },
                    _ => panic!("Expected `:` or `(` but found `{:?}` in view! macro", tts[0]),
                }
            }

            if tts.first() == Some(&Token(Comma)) {
                tts = &tts[1..];
            }
        }
    }
    else {
        panic!("Expected {{ but found `{:?}` in view! macro", tokens[0]);
    }
    (widget, &tokens[1..])
}

fn parse_ident(tokens: &[TokenTree]) -> (String, &[TokenTree]) {
    match tokens[0] {
        Token(Ident(ref ident)) => {
            (ident.to_string(), &tokens[1..])
        },
        _ => panic!("Expected ident but found `{:?}` in view! macro", tokens[0]),
    }
}

fn parse_qualified_name(tokens: &[TokenTree]) -> (String, &[TokenTree]) {
    try_parse_name(tokens)
        .unwrap_or_else(|| panic!("Expected qualified name but found `{:?}` in view! macro", tokens[0]))
}

fn try_parse_name(mut tokens: &[TokenTree]) -> Option<(String, &[TokenTree])> {
    let mut segments = vec![];
    while !tokens.is_empty() {
        match tokens[0] {
            Token(Ident(ref ident)) => {
                segments.push(ident.to_string());
            },
            Token(ModSep) => (), // :: is part of a name.
            _ => break,
        }
        tokens = &tokens[1..];
    }
    if segments.is_empty() || segments.last().unwrap().chars().next().unwrap().is_lowercase() {
        None
    }
    else {
        match tokens[0] {
            TokenTree::Delimited(_) | Token(Comma) => Some((segments.join("::"), tokens)),
            _ => None,
        }
    }
}

fn parse_comma_list(tokens: &[TokenTree]) -> Vec<String> {
    let mut params = vec![];
    let mut current_param = Tokens::new();
    for token in tokens {
        if *token == Token(Comma) {
            params.push(current_param.to_string());
            current_param = Tokens::new();
        }
        else {
            token.to_tokens(&mut current_param);
        }
    }
    params.push(current_param.to_string());
    params
}

fn parse_event(mut tokens: &[TokenTree], default_param: DefaultParam) -> (Event, &[TokenTree]) {
    let mut event = Event::new();
    if default_param == DefaultNoParam {
        event.params.clear();
    }
    if let TokenTree::Delimited(Delimited { delim: Paren, ref tts }) = tokens[0] {
        event.params = parse_comma_list(tts);
        tokens = &tokens[1..];
    }
    if tokens[0] != Token(FatArrow) {
        panic!("Expected `=>` but found `{:?}` in view! macro", tokens[0]);
    }
    tokens = &tokens[1..];
    event.value =
        if tokens.len() >= 2 && tokens[1] == Token(At) {
            let (event_value, new_tokens) = parse_event_value(&tokens[2..]);
            let (ident, _) = parse_ident(tokens);
            tokens = new_tokens;
            let mut ident_tokens = Tokens::new();
            ident_tokens.append(ident);
            ForeignWidget(ident_tokens, event_value)
        }
        else {
            let (event_value, new_tokens) = parse_event_value(tokens);
            tokens = new_tokens;
            CurrentWidget(event_value)
        };
    (event, tokens)
}

fn parse_event_value(tokens: &[TokenTree]) -> (EventValueReturn, &[TokenTree]) {
    if let TokenTree::Delimited(Delimited { delim: Paren, ref tts }) = tokens[0] {
        let (value1, tokens) = parse_value(tts);
        let (value2, tokens) = parse_value(&tokens[1..]);
        (Return(value1, value2), tokens)
    }
    else {
        let (value, tokens) = parse_value(tokens);
        (WithoutReturn(value), tokens)
    }
}

fn parse_value_or_child_properties<'a>(tokens: &'a [TokenTree]) -> (Value, &'a [TokenTree]) {
    match tokens[0] {
        TokenTree::Delimited(Delimited { delim: Brace, tts: ref child_tokens }) => {
            let child_properties = parse_child_properties(child_tokens);
            (ChildProperties(child_properties), &tokens[1..])
        },
        _ => {
            let (value, tts) = parse_value(tokens);
            (Value(value), tts)
        },
    }
}

fn parse_value(tokens: &[TokenTree]) -> (Tokens, &[TokenTree]) {
    let mut current_param = Tokens::new();
    let mut i = 0;
    while i < tokens.len() {
        match tokens[i] {
            Token(Comma) => break,
            ref token => token.to_tokens(&mut current_param),
        }
        i += 1;
    }
    (current_param, &tokens[i..])
}

fn gen_widget_name(name: &str) -> String {
    let name =
        if let Some(index) = name.rfind(':') {
            name[index + 1 ..].to_lowercase()
        }
        else {
            name.to_lowercase()
        };
    let mut hashmap = NAMES_INDEX.lock().unwrap();
    let index = hashmap.entry(name.clone()).or_insert(0);
    *index += 1;
    format!("{}{}", name, index)
}

fn parse_child_properties(mut tokens: &[TokenTree]) -> HashMap<String, Tokens> {
    // TODO: panic if the same child properties is set twice.
    // TODO: same for normal properties?
    let mut properties = HashMap::new();
    while !tokens.is_empty() {
        let (ident, _) = parse_ident(tokens);
        tokens = &tokens[1..];
        if let Token(Colon) = tokens[0] {
            tokens = &tokens[1..];
            let (value, new_tokens) = parse_value(tokens);
            tokens = new_tokens;
            properties.insert(ident, value);
        }

        if tokens.first() == Some(&Token(Comma)) {
            tokens = &tokens[1..];
        }
    }
    properties
}

fn parse_relm_widget(tokens: &[TokenTree]) -> (RelmWidget, &[TokenTree]) {
    let (relm_type, tokens) = parse_qualified_name(tokens);
    let mut widget = RelmWidget::new(relm_type);
    if let TokenTree::Delimited(Delimited { delim: Brace, ref tts }) = tokens[0] {
        let mut tts = &tts[..];
        while !tts.is_empty() {
            let (ident, _) = parse_ident(tts);
            match tts[1] {
                TokenTree::Delimited(Delimited { delim: Paren, .. }) | Token(FatArrow) => {
                    let (event, new_tts) = parse_event(&tts[1..], DefaultNoParam);
                    let mut entry = widget.events.entry(ident).or_insert_with(Vec::new);
                    entry.push(event);
                    tts = new_tts;
                },
                _ => panic!("Expected event, but found {:?}", tts[0]),
            }

            if tts.first() == Some(&Token(Comma)) {
                tts = &tts[1..];
            }
        }
    }
    (widget, &tokens[1..])
}

fn try_parse_name_attribute(tokens: &[TokenTree]) -> (Option<String>, &[TokenTree]) {
    if &tokens[0] == &Token(Pound) {
        if let TokenTree::Delimited(Delimited { delim: Bracket, ref tts }) = tokens[1] {
            if Token(Ident(syn::Ident::new("name"))) == tts[0] && Token(Eq) == tts[1] {
                if let Token(Literal(Str(ref name, Cooked))) = tts[2] {
                    return (Some(name.clone()), &tokens[2..]);
                }
            }
        }
    }
    (None, tokens)
}
