/*
 * Copyright (c) 2017-2020 Boucher, Antoni <bouanto@zoho.com>
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

use std::collections::HashSet;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::iter::FromIterator;
use std::str::FromStr;
use std::sync::Mutex;

use lazy_static::lazy_static;
use proc_macro2::{Span, TokenTree};
use quote::{quote, ToTokens};
use syn::{
    Expr,
    ExprMacro,
    Ident,
    LitStr,
    Macro,
    Pat,
    Path,
    Type,
    braced,
    bracketed,
    parenthesized,
    parse,
    parse2,
    token,
    Token,
};
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;

use self::ChildItem::*;
use self::EventValue::*;
use self::EventValueReturn::*;
use self::EitherWidget::*;
use self::IdentOrEventValue::*;
use self::InitProperties::*;
use self::WidgetPath::*;
use self::SaveWidget::*;

// TODO: switch to thread_local?
lazy_static! {
    static ref NAMES_INDEX: Mutex<HashMap<String, u32>> = Mutex::new(HashMap::new());
}

macro_rules! catch_return {
    ($expr:expr) => {
        (|| {
            Ok($expr)
        })()
    };
}

type ChildEvents = HashMap<(Ident, Ident), Event>;
type ChildProperties = HashMap<(Ident, Ident), Expr>;

#[derive(PartialEq)]
enum SaveWidget {
    DontSave,
    Save,
}

#[derive(Debug)]
pub enum EventValueReturn {
    CallReturn(Expr),
    Return(Box<(Expr, Expr)>),
    WithoutReturn(Expr),
}

#[derive(Debug)]
pub enum EventValue {
    CurrentWidget(EventValueReturn),
    ForeignWidget(Ident, EventValueReturn),
    NoEventValue,
}

#[derive(Debug)]
pub struct Event {
    pub params: Vec<Pat>,
    pub shared_values: Vec<Ident>,
    pub use_self: bool,
    pub value: EventValue,
}

impl Event {
    fn new() -> Self {
        Event {
            params: vec![],
            shared_values: vec![],
            use_self: false,
            value: NoEventValue,
        }
    }
}

#[derive(Debug)]
pub struct Widget {
    pub child_events: ChildEvents, // TODO: does it make sense for a relm widget?
    pub child_properties: ChildProperties, // TODO: does it make sense for a relm widget?
    pub children: Vec<Widget>,
    pub container_type: Option<Option<String>>, // TODO: Why two Options?
    pub init_parameters: Vec<Expr>,
    pub is_container: bool,
    pub name: Ident,
    pub nested_views: HashMap<Ident, Widget>,
    pub parent_id: Option<String>,
    pub properties: HashMap<Ident, Expr>,
    pub save: bool,
    pub typ: Path,
    pub widget: EitherWidget,
    pub style_classes: Vec<String>,
}

impl Widget {
    #[allow(clippy::too_many_arguments)]
    fn new_gtk(widget: GtkWidget, typ: Path, init_parameters: Vec<Expr>, children: Vec<Widget>,
        properties: HashMap<Ident, Expr>, child_properties: ChildProperties, child_events: ChildEvents,
        nested_views: HashMap<Ident, Widget>) -> Self
    {
        let name = gen_widget_name(&typ);
        Widget {
            child_events,
            child_properties,
            children,
            container_type: None,
            init_parameters,
            is_container: false,
            name,
            nested_views,
            parent_id: None,
            properties,
            save: false,
            typ,
            widget: Gtk(widget),
            style_classes: vec![],
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn new_relm(widget: RelmWidget, typ: Path, init_parameters: Vec<Expr>, children: Vec<Widget>,
        properties: HashMap<Ident, Expr>, child_properties: ChildProperties, child_events: ChildEvents,
        nested_views: HashMap<Ident, Widget>) -> Self
    {
        let mut name = gen_widget_name(&typ);
        // Relm widgets are not used in the update() method; they are only saved to avoid dropping
        // their channel too soon.
        // So prepend an underscore to hide a warning.
        name = Ident::new(&format!("_{}", name), name.span());
        Widget {
            child_events,
            child_properties,
            children,
            container_type: None,
            init_parameters,
            is_container: false,
            name,
            nested_views,
            parent_id: None,
            properties,
            save: false,
            typ,
            widget: Relm(widget),
            style_classes: vec![],
        }
    }
}

#[derive(Debug)]
pub enum EitherWidget {
    Gtk(GtkWidget),
    Relm(RelmWidget),
}

#[derive(Debug)]
pub struct GtkWidget {
    pub construct_properties: HashMap<Ident, Expr>,
    pub events: HashMap<Ident, Event>,
    pub relm_name: Option<Type>,
}

impl GtkWidget {
    fn new() -> Self {
        GtkWidget {
            construct_properties: HashMap::new(),
            events: HashMap::new(),
            relm_name: None,
        }
    }
}

#[derive(Debug)]
pub struct RelmWidget {
    pub events: HashMap<Ident, Vec<Event>>,
    pub gtk_events: HashMap<Ident, Event>,
    pub messages: HashMap<Ident, Expr>,
}

impl RelmWidget {
    fn new() -> Self {
        RelmWidget {
            events: HashMap::new(),
            gtk_events: HashMap::new(),
            messages: HashMap::new(),
        }
    }
}

pub struct WidgetList {
    pub widgets: Vec<Widget>,
}

impl Parse for WidgetList {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(LitStr) {
            let literal: LitStr = input.parse()?;

            // TODO: also support glade file.
            let mut file = File::open(literal.value()).expect("File::open() in parse()");
            let mut file_content = String::new();
            file.read_to_string(&mut file_content)
                .expect("read_to_string() in parse()");
            let tokens = proc_macro::TokenStream::from_str(&file_content)
                .expect("convert string to TokenStream");
            let tokens = respan_with(tokens, literal.span().unwrap());

            syn::parse(tokens)
        } else if lookahead.peek(Ident) || lookahead.peek(token::Pound) {
            let mut widgets = vec![];

            while !input.is_empty() {
                widgets.push(input.parse()?);
            }

            Ok(WidgetList { widgets })
        } else {
            Err(lookahead.error())
        }
    }
}

enum InitProperties {
    ConstructProperties(HashMap<Ident, Expr>),
    InitParameters(Vec<Expr>),
    NoInitParameter,
}

struct HashKeyValue {
    ident: Ident,
    expr: Expr,
}

impl Parse for HashKeyValue {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse()?;
        let _colon: Token![:] = input.parse()?;
        Ok(HashKeyValue {
            ident,
            expr: input.parse()?,
        })
    }
}

struct Hash {
    key_values: InitProperties,
}

impl Parse for Hash {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let _brace = braced!(content in input);
        let key_values: Punctuated<HashKeyValue, Token![,]> = content.parse_terminated(HashKeyValue::parse)?;
        let key_values = ConstructProperties(key_values.into_iter()
            .map(|key_value| (key_value.ident, key_value.expr)).collect());
        Ok(Hash {
            key_values,
        })
    }
}

struct InitPropertiesParser {
    properties: InitProperties,
}

impl Parse for InitPropertiesParser {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        let properties =
            if lookahead.peek(token::Paren) {
                let content;
                let _parens = parenthesized!(content in input);
                let lookahead = content.lookahead1();
                if lookahead.peek(token::Brace) {
                    Hash::parse(&content)?.key_values
                }
                else {
                    InitParameters(ExprList::parse(&content)?.exprs)
                }
            }
            else {
                NoInitParameter
            };
        Ok(InitPropertiesParser {
            properties,
        })
    }
}

enum ChildItem {
    ChildEvent(Ident, Ident, Event),
    ItemChildProperties(ChildProperties),
    ItemEvent(Ident, Event),
    ChildWidget(Widget),
    NestedView(Ident, Widget),
    Property(Ident, Value),
    RelmMsg(Ident, Value),
    RelmMsgEvent(Ident, Event),
}

impl ChildItem {
    fn unwrap_widget(self) -> Widget {
        match self {
            ChildEvent(_, _, _) => panic!("Expected widget, found child event"),
            ItemEvent(_, _) => panic!("Expected widget, found event"),
            ItemChildProperties(_) => panic!("Expected widget, found child properties"),
            NestedView(_, _) => panic!("Expected widget, found nested view"),
            Property(_, _) => panic!("Expected widget, found property"),
            RelmMsg(_, _) => panic!("Expected widget, found relm msg"),
            RelmMsgEvent(_, _) => panic!("Expected widget, found relm msg event"),
            ChildWidget(widget) => widget,
        }
    }
}

struct AttributeValue {
    value: LitStr,
}

impl Parse for AttributeValue {
    fn parse(input: ParseStream) -> Result<Self> {
        let _equal: Token![=] = input.parse()?;
        Ok(AttributeValue {
            value: input.parse()?,
        })
    }
}

struct NameValue {
    name: Ident,
    value: Option<AttributeValue>,
}

impl Parse for NameValue {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(NameValue {
            name: input.parse()?,
            value: AttributeValue::parse(input).ok(),
        })
    }
}

struct Attribute {
    name_values: HashMap<String, Option<LitStr>>, // TODO: Use Ident instead?
}

impl Parse for Attribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let _hash: Token![#] = input.parse()?;
        let content;
        let _bracket = bracketed!(content in input);
        let name_values: Punctuated<NameValue, Token![,]> = content.parse_terminated(NameValue::parse)?;
        let name_values = name_values.into_iter()
            .map(|name_value| (name_value.name.to_string(), name_value.value.map(|value| value.value)))
            .collect();

        Ok(Attribute {
            name_values,
        })
    }
}

struct Attributes {
    name_values: HashMap<String, Option<LitStr>>,
    style_classes: HashSet<String>,
}

impl Parse for Attributes {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut name_values = HashMap::new();
        let mut style_classes = HashSet::new();
        loop {
            let lookahead = input.lookahead1();

            if lookahead.peek(Token![#]) {
                let attribute: Attribute = input.parse()?;
                match attribute.name_values.get("style_class") {
                    Some(Some(style_class)) => {
                        style_classes.insert(style_class.value());
                    },
                    Some(None) => panic!("Invalid style_class specification"),
                    None => name_values.extend(attribute.name_values)
                };
            }
            else {
                break;
            }
        }

        Ok(Attributes {
            name_values,
            style_classes
        })
    }
}

#[derive(Debug)]
enum WidgetPath {
    RelmPath(Path),
    GtkPath(Path),
}

impl WidgetPath {
    fn get_relm_path(&self) -> &Path {
        match *self {
            RelmPath(ref ident) => ident,
            GtkPath(_) => panic!("Expected ident"),
        }
    }

    fn get_gtk_path(&self) -> &Path {
        match *self {
            RelmPath(_) => panic!("Expected path"),
            GtkPath(ref path) => path,
        }
    }
}

struct WidgetPathParser {
    widget_path: WidgetPath,
}

impl Parse for WidgetPathParser {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        let widget_path =
            if lookahead.peek(token::Dollar) {
                let _token: token::Dollar = input.parse()?;
                let path: Path = input.parse()?;
                RelmPath(path)
            }
            else {
                let path: Path = input.parse()?;
                if path.segments.len() == 1 {
                    RelmPath(path)
                }
                else {
                    GtkPath(path)
                }
            };
        Ok(WidgetPathParser {
            widget_path,
        })
    }
}

struct ChildWidgetParser {
    widget: ChildItem,
    parent_id: Option<String>,
}

/*
 * First tokens:
 * * # (attributes)
 * * $ (absolute relm widget name (path))
 * * path
 * * * (
 * * * {
 */
impl ChildWidgetParser {
    fn parse(root: SaveWidget, input: ParseStream) -> Result<Self> {
        let attributes = Attributes::parse(input)?;
        let typ: WidgetPathParser = input.parse()?;
        let typ = typ.widget_path;
        let save = attributes.name_values.contains_key("name") || root == Save;
        match typ {
            RelmPath(_) => {
                let relm_widget = RelmWidgetParser::parse(typ.get_relm_path().clone(), input)?.relm_widget;
                Ok(adjust_widget_with_attributes(relm_widget, &attributes.name_values, &attributes.style_classes, save))
            },
            GtkPath(_) => {
                let gtk_widget = GtkWidgetParser::parse(typ.get_gtk_path().clone(), input)?.gtk_widget;
                Ok(adjust_widget_with_attributes(gtk_widget, &attributes.name_values, &attributes.style_classes, save))
            },
        }
    }
}

struct GtkWidgetParser {
    gtk_widget: ChildItem,
}

impl GtkWidgetParser {
    fn parse(typ: Path, input: ParseStream) -> Result<Self> {
        let init_properties = InitPropertiesParser::parse(input)?.properties;
        let content;
        let _brace = braced!(content in input);
        let child_items: Punctuated<ChildGtkItem, Token![,]> = content.parse_terminated(ChildGtkItem::parse)?;

        let mut gtk_widget = GtkWidget::new();
        let mut init_parameters = vec![];
        let mut children = vec![];
        let mut properties = HashMap::new();
        let mut child_events = HashMap::new();
        let mut child_properties = HashMap::new();
        let mut nested_views = HashMap::new();
        for item in child_items.into_iter() {
            let item = item.item;
            match item {
                ChildEvent(event_name, child_name, event) => {
                    let _ = child_events.insert((child_name, event_name), event);
                },
                ItemChildProperties(child_props) => {
                    for (key, value) in child_props {
                        child_properties.insert(key, value);
                    }
                },
                ItemEvent(ident, event) => { let _ = gtk_widget.events.insert(ident, event); },
                ChildWidget(widget) => children.push(widget),
                NestedView(ident, widget) => { let _ = nested_views.insert(ident, widget); },
                Property(ident, value) => { let _ = properties.insert(ident, value.value); },
                RelmMsg(_, _) | RelmMsgEvent(_, _) => panic!("Unexpected relm msg in gtk widget"),
            }
        }
        match init_properties {
            ConstructProperties(construct_properties) => gtk_widget.construct_properties = construct_properties,
            InitParameters(init_params) => init_parameters = init_params,
            NoInitParameter => (),
        }
        Ok(GtkWidgetParser {
            gtk_widget: ChildWidget(Widget::new_gtk(gtk_widget, typ, init_parameters, children, properties,
                            child_properties, child_events, nested_views)),
        })
    }
}

struct ChildRelmItem {
    child_item: ChildItem,
}

impl Parse for ChildRelmItem {
    fn parse(input: ParseStream) -> Result<Self> {
        let parser = input.fork();
        let child_item =
            if RelmPropertyOrEvent::parse(&parser).is_ok() {
                RelmPropertyOrEvent::parse(input)?.child_item
            }
            else {
                ChildWidgetParser::parse(DontSave, input)?.widget
            };
        Ok(ChildRelmItem {
            child_item,
        })
    }
}

struct RelmWidgetParser {
    relm_widget: ChildItem,
}

impl RelmWidgetParser {
    fn parse(typ: Path, input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        let init_parameters =
            if lookahead.peek(token::Paren) {
                let content;
                let _parens = parenthesized!(content in input);
                Some(ExprList::parse(&content)?.exprs)
            }
            else {
                None
            };
        let lookahead = input.lookahead1();
        let relm_widget =
            if lookahead.peek(token::Brace) {
                let content;
                let _brace = braced!(content in input);
                let child_items = Punctuated::<ChildRelmItem, Token![,]>::parse_terminated(&content)?
                    .into_iter()
                    .map(|relm_item| relm_item.child_item);

                let init_parameters = init_parameters.unwrap_or_default();
                let mut relm_widget = RelmWidget::new();
                let mut children = vec![];
                let mut child_properties = HashMap::new();
                let mut child_events = HashMap::new();
                let mut properties = HashMap::new();
                let mut nested_views = HashMap::new();
                for item in child_items {
                    match item {
                        ChildEvent(event_name, child_name, event) => {
                            let _ = child_events.insert((child_name, event_name), event);
                        },
                        ChildWidget(widget) => children.push(widget),
                        ItemEvent(ident, event) => { let _ = relm_widget.gtk_events.insert(ident, event); },
                        ItemChildProperties(child_props) => {
                            for (key, value) in child_props {
                                child_properties.insert(key, value);
                            }
                        },
                        NestedView(ident, widget) => { let _ = nested_views.insert(ident, widget); },
                        Property(ident, value) => { let _ = properties.insert(ident, value.value); },
                        RelmMsg(ident, value) => { let _ = relm_widget.messages.insert(ident, value.value); },
                        RelmMsgEvent(ident, event) => {
                            let events = relm_widget.events.entry(ident).or_default();
                            events.push(event);
                        },
                    }
                }
                ChildWidget(Widget::new_relm(relm_widget, typ, init_parameters, children, properties,
                    child_properties, child_events, nested_views))
            }
            else {
                let init_parameters = init_parameters.unwrap_or_else(Vec::new);
                ChildWidget(Widget::new_relm(RelmWidget::new(), typ, init_parameters, vec![], HashMap::new(),
                    HashMap::new(), HashMap::new(), HashMap::new()))
            };
        Ok(RelmWidgetParser {
            relm_widget,
        })
    }
}

struct RelmPropertyOrEvent {
    child_item: ChildItem,
}

impl Parse for RelmPropertyOrEvent {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: Ident = input.parse()?;
        let lookahead = input.lookahead1();
        let child_item =
            if lookahead.peek(Token![:]) {
                let _colon: Token![:] = input.parse()?;
                let result = ValueOrChildProperties::parse(input, &ident)?.child_item;

                if ident.to_string().chars().next().map(|char| char.is_lowercase()) == Some(false) {
                    // Uppercase is a msg to send.
                    match result {
                        Property(ident, value) => RelmMsg(ident, value),
                        _ => panic!("Expecting property"),
                    }
                }
                else {
                    // Lowercase is a gtk property.
                    result
                }
            }
            else if lookahead.peek(Token![.]) {
                let _colon: Token![.] = input.parse()?;
                let event_name: Ident = input.parse()?;
                let event = Event::parse(input)?;
                ChildEvent(event_name, ident, event)
            }
            else {
                let mut event = Event::parse(input)?;
                if ident.to_string().chars().next().map(|char| char.is_lowercase()) == Some(false) {
                    // Uppercase is a msg.
                    RelmMsgEvent(ident, event)
                }
                else {
                    // Lowercase is a gtk event.
                    if event.params.is_empty() {
                        event.params.push(wild_pat());
                    }
                    ItemEvent(ident, event)
                }
            };
        Ok(RelmPropertyOrEvent {
            child_item,
        })
    }
}

struct GtkChildPropertyOrEvent {
    child_item: ChildItem,
}

/*
 * First tokens:
 * * ident
 * * * :
 * * * .
 * * * ( (in Event)
 * * * with
 * * * =>
 */
impl Parse for GtkChildPropertyOrEvent {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: Ident = input.parse()?;
        let lookahead = input.lookahead1();
        let child_item =
            if lookahead.peek(Token![:]) {
                let _colon: Token![:] = input.parse()?;
                ValueOrChildProperties::parse(input, &ident)?.child_item
            }
            else if lookahead.peek(Token![.]) {
                let _colon: Token![.] = input.parse()?;
                let event_name: Ident = input.parse()?;
                let mut event = Event::parse(input)?;

                if event.params.is_empty() {
                    event.params.push(wild_pat());
                }
                ChildEvent(event_name, ident, event)
            }
            else {
                let mut event = Event::parse(input)?;
                if event.params.is_empty() {
                    event.params.push(wild_pat());
                }
                ItemEvent(ident, event)
            };
        Ok(GtkChildPropertyOrEvent {
            child_item,
        })
    }
}

struct ValueOrChildProperties {
    child_item: ChildItem,
}

impl ValueOrChildProperties {
    fn parse(input: ParseStream, ident: &Ident) -> Result<Self> {
        let lookahead = input.lookahead1();
        let child_item =
            if lookahead.peek(token::Brace) {
                let properties;
                let _brace = braced!(properties in input);
                let properties = ChildPropertiesParser::parse(&properties)?.properties;
                let properties = properties.into_iter()
                    .map(|(key, value)| ((ident.clone(), key), value))
                    .collect();
                ItemChildProperties(properties)
            }
            else {
                let value = Value::parse(input)?;
                let mut nested_view = None;
                if let Expr::Macro(ExprMacro { mac: Macro { ref path, ref tokens, .. }, .. }) = value.value {
                    if path.is_ident(&dummy_ident("view")) {
                        nested_view = Some(tokens.clone());
                    }
                }
                if let Some(tokens) = nested_view {
                    let widget: Widget = parse2(tokens)?;
                    NestedView(ident.clone(), widget)
                }
                else {
                    Property(ident.clone(), value)
                }
            };
        Ok(ValueOrChildProperties {
            child_item,
        })
    }
}

struct Tag;

impl Tag {
    fn parse(input: ParseStream, expected_ident: &str) -> Result<()> {
        let parser = input.fork();
        let ident: Ident = parser.parse()?;
        if ident != expected_ident {
            Err(Error::new(ident.span(), format!("Expected ident {}, but found {}", expected_ident, ident)))
        }
        else {
            let _ident: Ident = input.parse()?;
            Ok(())
        }
    }
}

struct SharedValues {
    shared_values: Option<Vec<Ident>>,
}

impl Parse for SharedValues {
    fn parse(input: ParseStream) -> Result<Self> {
        let shared_values = catch_return! {{
            Tag::parse(input, "with")?;
            let content;
            let _parens = parenthesized!(content in input);
            IdentList::parse(&content)?.idents
        }}.ok();
        Ok(SharedValues {
            shared_values,
        })
    }
}

enum IdentOrEventValue {
    MessageIdent(EventValueReturn, bool),
    MessageEventValue(Ident, EventValueReturn, bool),
}

fn expr_use_self(expr: &Expr) -> bool {
    let mut tokens = quote! {};
    expr.to_tokens(&mut tokens);
    tokens.into_iter().any(|token| {
        if let TokenTree::Ident(ident) = token {
            return ident == "self";
        }
        false
    })
}

struct Value {
    value: Expr,
    use_self: bool,
}

impl Parse for Value {
    fn parse(input: ParseStream) -> Result<Self> {
        let expr = Expr::parse(input)?;
        let use_self = expr_use_self(&expr);
        Ok(Value {
            value: expr,
            use_self,
        })
    }
}

struct EventValueParser {
    value_return: EventValueReturn,
    use_self: bool,
}

impl Parse for EventValueParser {
    fn parse(input: ParseStream) -> Result<Self> {
        let tag = Tag::parse(input, "return");
        let lookahead = input.lookahead1();
        if tag.is_ok() {
            let value = Value::parse(input)?;
            Ok(EventValueParser {
                value_return: CallReturn(value.value),
                use_self: value.use_self,
            })
        }
        else if lookahead.peek(token::Paren) {
            let content;
            let _parens = parenthesized!(content in input);
            let value1: Value = content.parse()?;
            let _comma: token::Comma = content.parse()?;
            let value2: Value = content.parse()?;
            Ok(EventValueParser {
                value_return: Return(Box::new((value1.value, value2.value))),
                use_self: value1.use_self || value2.use_self,
            })
        }
        else {
            let value = Value::parse(input)?;
            Ok(EventValueParser {
                value_return: WithoutReturn(value.value),
                use_self: value.use_self,
            })
        }
    }
}

struct MessageSent {
    ident_or_event_value: IdentOrEventValue,
}

impl Parse for MessageSent {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: Result<Ident> = catch_return! {{
            let parser = input.fork();
            let ident: Ident = parser.parse()?;
            let _token: token::At = parser.parse()?;
            ident
        }};
        if ident.is_ok() {
            let ident: Ident = input.parse()?;
            let _token: token::At = input.parse()?;
            let event_value = EventValueParser::parse(input)?;
            Ok(MessageSent {
                ident_or_event_value: MessageEventValue(ident, event_value.value_return, event_value.use_self),
            })
        }
        else {
            let event_value = EventValueParser::parse(input)?;
            Ok(MessageSent {
                ident_or_event_value: MessageIdent(event_value.value_return, event_value.use_self),
            })
        }
    }
}

struct ExprList {
    exprs: Vec<Expr>,
}

impl Parse for ExprList {
    fn parse(input: ParseStream) -> Result<Self> {
        let exprs: Punctuated<Expr, Token![,]> = input.parse_terminated(Expr::parse)?;
        Ok(ExprList {
            exprs: exprs.into_iter().collect(),
        })
    }
}

struct IdentList {
    idents: Vec<Ident>,
}

impl Parse for IdentList {
    fn parse(input: ParseStream) -> Result<Self> {
        let idents: Punctuated<Ident, Token![,]> = input.parse_terminated(Ident::parse)?;
        Ok(IdentList {
            idents: idents.into_iter().collect(),
        })
    }
}

impl Parse for Event {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let lookahead = input.lookahead1();
        let params =
            if lookahead.peek(token::Paren) {
                let _parens = parenthesized!(content in input);
                Punctuated::<Pat, Token![,]>::parse_separated_nonempty(&content).ok()
            }
            else {
                None
            };
        let shared_values = SharedValues::parse(input)?.shared_values;
        let _token: Token![=>] = input.parse()?;
        let message_sent = MessageSent::parse(input)?.ident_or_event_value;

        let mut event = Event::new();
        if let Some(params) = params {
            event.params = params.into_iter().collect();
        }
        if let Some(shared_values) = shared_values {
            event.shared_values = shared_values;
        }
        match message_sent {
            MessageIdent(event_value, use_self) => {
                event.use_self = use_self;
                event.value = CurrentWidget(event_value);
            },
            MessageEventValue(ident, event_value, use_self) => {
                event.use_self = use_self;
                event.value = ForeignWidget(ident, event_value);
            },
        }
        Ok(event)
    }
}

fn is_property_or_event(input: &ParseStream) -> bool {
    let input = input.fork();
    // Attributes start with # and qualified name for relm widget starts with $ .
    if input.peek(Token![#]) || input.peek(Token![$]) {
        return false;
    }
    {
        let input = input.fork();
        let path = input.parse::<Path>();
        if let Ok(path) = path {
            if path.segments.len() > 1 {
                // Only a widget starts with a path that contains more than 1 segment.
                return false;
            }
        }
    }
    let _ident: Ident = input.parse().expect("should be an ident");
    if input.peek(token::Brace) {
        // Only a widget can have an ident followed by { .
        return false;
    }
    if input.peek(Token![=>]) || input.peek(Token![.]) || input.peek(Token![:]) {
        // Only an event can contain => .
        return true;
    }
    {
        let input = input.fork();
        if Tag::parse(&input, "with").is_ok() {
            // Only an event can contain with.
            return true;
        }
    }
    let result = catch_return! {{
        let _content;
        let _parens = parenthesized!(_content in input);
    }};
    result.expect("parse parenthesis");
    if input.peek(token::Brace) {
        // Only a widget can have an ident followed by { .
        return false;
    }
    if input.peek(Token![=>]) || input.peek(Token![.]) || input.peek(Token![:]) {
        // Only an event can contain => .
        return true;
    }
    {
        let input = input.fork();
        if Tag::parse(&input, "with").is_ok() {
            // Only an event can contain with.
            return true;
        }
    }

    // If the parens are not followed by either => or with, it's a widget.
    false
}

struct ChildGtkItem {
    item: ChildItem,
}

impl Parse for ChildGtkItem {
    fn parse(input: ParseStream) -> Result<Self> {
        if is_property_or_event(&input) {
            let item: GtkChildPropertyOrEvent = input.parse()?;
            Ok(ChildGtkItem {
                item: item.child_item,
            })
        }
        else {
            Ok(ChildGtkItem {
                item: ChildWidgetParser::parse(DontSave, input)?.widget
            })
        }
    }
}

struct ChildProp {
    name: Ident,
    value: Value,
}

impl Parse for ChildProp {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        let _token: Token![:] = input.parse()?;
        let value = Value::parse(input)?;
        Ok(ChildProp {
            name,
            value,
        })
    }
}

struct ChildPropertiesParser {
    properties: HashMap<Ident, Expr>,
}

impl Parse for ChildPropertiesParser {
    fn parse(input: ParseStream) -> Result<Self> {
        let properties = Punctuated::<ChildProp, Token![,]>::parse_terminated(input)?;
        Ok(ChildPropertiesParser {
            properties: properties.into_iter()
                .map(|child_prop| (child_prop.name, child_prop.value.value))
                .collect(),
        })
    }
}

fn wild_pat() -> Pat {
    parse(quote! {
        _
    }.into())
        .expect("wildcard pattern")
}

impl Parse for Widget {
    fn parse(input: ParseStream) -> Result<Self> {
        let child_widget = ChildWidgetParser::parse(Save, input)?;
        let _token: Option<Token![,]> = input.parse().ok();

        let mut widget = child_widget.widget.unwrap_widget();
        widget.parent_id = child_widget.parent_id;
        Ok(widget)
    }
}

fn gen_widget_name(path: &Path) -> Ident {
    let name = path_to_string(path);
    let name =
        if let Some(index) = name.rfind(':') {
            name[index + 1 ..].to_lowercase()
        }
        else {
            name.to_lowercase()
        };
    let mut hashmap = NAMES_INDEX.lock().expect("lock() in gen_widget_name()");
    let index = hashmap.entry(name.clone()).or_insert(0);
    *index += 1;
    Ident::new(&format!("{}{}", name, index), path.span())
}

fn path_to_string(path: &Path) -> String {
    let mut string = String::new();
    for segment in &path.segments {
        string.push_str(&segment.ident.to_string());
    }
    string
}

fn adjust_widget_with_attributes(mut widget: ChildItem, attributes: &HashMap<String, Option<LitStr>>, style_classes: &HashSet<String>, save: bool)
    -> ChildWidgetParser
{
    let parent_id;
    match widget {
        ChildWidget(ref mut widget) => {
            widget.save = save;
            let container_type = attributes.get("container")
                .map(|typ| typ.as_ref().map(|lit| lit.value()));
            let name = attributes.get("name").and_then(|name| name.clone());
            if let Some(name) = name {
                widget.name = Ident::new(&name.value(), name.span());
            }
            // style_class attribute
            for style_class in style_classes {
                widget.style_classes.push((*style_class).clone());
            }
            widget.is_container = !widget.children.is_empty();
            widget.container_type = container_type;
            parent_id = attributes.get("parent").and_then(|opt_str| opt_str.as_ref().map(|lit| lit.value()));
        },
        _ => panic!("Expecting widget"),
    }
    ChildWidgetParser {
        widget,
        parent_id,
    }
}

pub fn respan_with(tokens: proc_macro::TokenStream, span: proc_macro::Span) -> proc_macro::TokenStream {
    let mut result = vec![];
    for mut token in tokens {
        match token {
            proc_macro::TokenTree::Group(group) => {
                let new_tokens = respan_with(group.stream(), span);
                let mut res = proc_macro::TokenTree::Group(proc_macro::Group::new(group.delimiter(), new_tokens));
                res.set_span(span);
                result.push(res);
            },
            _ => {
                token.set_span(span);
                result.push(token);
            }
        }
    }
    FromIterator::from_iter(result)
}

pub fn dummy_ident(ident: &str) -> Ident {
    Ident::new(ident, Span::call_site())
}
