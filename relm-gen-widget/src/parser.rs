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
use std::fs::File;
use std::io::Read;
#[cfg(feature = "unstable")]
use std::iter::FromIterator;
use std::str::FromStr;
use std::sync::Mutex;

use proc_macro;
#[cfg(feature = "unstable")]
use proc_macro::TokenTree;
use proc_macro2::{TokenTree, TokenStream};
use quote::ToTokens;
use syn::{
    self,
    Expr,
    Ident,
    LitStr,
    Pat,
    Path,
    Type,
    parse,
    parse2,
};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;

use self::ChildItem::*;
use self::EventValue::*;
use self::EventValueReturn::*;
use self::EitherWidget::*;
use self::IdentOrEventValue::*;
use self::InitProperties::*;
use self::PathOrIdent::*;
use self::SaveWidget::*;

lazy_static! {
    static ref NAMES_INDEX: Mutex<HashMap<String, u32>> = Mutex::new(HashMap::new());
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
    Return(Expr, Expr),
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
    pub parent_id: Option<String>,
    pub properties: HashMap<Ident, Expr>,
    pub typ: Path,
    pub widget: EitherWidget,
}

impl Widget {
    fn new_gtk(widget: GtkWidget, typ: Path, init_parameters: Vec<Expr>, children: Vec<Widget>,
        properties: HashMap<Ident, Expr>, child_properties: ChildProperties, child_events: ChildEvents) -> Self
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
            parent_id: None,
            properties,
            typ,
            widget: Gtk(widget),
        }
    }

    fn new_relm(widget: RelmWidget, typ: Path, init_parameters: Vec<Expr>, children: Vec<Widget>,
        properties: HashMap<Ident, Expr>, child_properties: ChildProperties, child_events: ChildEvents) -> Self
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
            parent_id: None,
            properties,
            typ,
            widget: Relm(widget),
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
    pub save: bool,
}

impl GtkWidget {
    fn new() -> Self {
        GtkWidget {
            construct_properties: HashMap::new(),
            events: HashMap::new(),
            relm_name: None,
            save: false,
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

pub fn parse_widget(tokens: TokenStream) -> Widget {
    if let Ok(literal) = parse2::<LitStr>(tokens.clone()) {
        // TODO: also support glade file.
        let mut file = File::open(literal.value()).expect("File::open() in parse()");
        let mut file_content = String::new();
        file.read_to_string(&mut file_content).expect("read_to_string() in parse()");
        let tokens = proc_macro::TokenStream::from_str(&file_content).expect("convert string to TokenStream");
        #[cfg(feature = "unstable")]
        let tokens = respan_with(tokens, literal.span().unstable());
        parse(tokens).expect("parse() Widget")
    }
    else {
        parse2(tokens).expect("parse() Widget")
    }
}

enum InitProperties {
    ConstructProperties(HashMap<Ident, Expr>),
    InitParameters(Vec<Expr>),
    NoInitProperties,
}

macro_rules! separated_by0 {
    ($i:expr, $sep:ident!($($sep_args:tt)*), $submac:ident!( $($args:tt)* )) => {{
        let ret;
        let mut res   = ::std::vec::Vec::new();
        let mut input = $i;

        loop {
            if input.eof() {
                ret = ::std::result::Result::Ok((res, input));
                break;
            }

            match $submac!(input, $($args)*) {
                ::std::result::Result::Err(_) => {
                    ret = ::std::result::Result::Ok((res, input));
                    break;
                }
                ::std::result::Result::Ok((o, i)) => {
                    // loop trip must always consume (otherwise infinite loops)
                    if i == input {
                        ret = ::syn::parse_error();
                        break;
                    }

                    res.push(o);
                    input = i;
                }
            }

            match $sep!(input, $($sep_args)*) {
                ::std::result::Result::Err(_) => {
                    ret = ::std::result::Result::Ok((res, input));
                    break;
                },
                ::std::result::Result::Ok((_, i)) => {
                    // loop trip must always consume (otherwise infinite loops)
                    if i == input {
                        ret = ::syn::parse_error();
                        break;
                    }

                    input = i;
                },
            }
        }

        ret
    }};
}

named! { parse_hash -> InitProperties, map!(braces!(map!(separated_by0!(
    punct!(,),
    do_parse!(
        ident: syn!(Ident) >>
        punct!(:) >>
        expr: syn!(Expr) >>
        (ident, expr)
    )), |data| ConstructProperties(data.into_iter().collect()))),
    |(_, hash)| hash)
}

named! { init_properties -> InitProperties,
    alt!
    ( map!(parens!(alt!
        ( parse_hash
        | map!(expr_list, InitParameters)
        )
      ), |(_, hash)| hash)
    | epsilon!() => { |_| NoInitProperties }
    )
}

enum ChildItem {
    ChildEvent(Ident, Ident, Event),
    ItemChildProperties(ChildProperties),
    ItemEvent(Ident, Event),
    ChildWidget(Widget),
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
            Property(_, _) => panic!("Expected widget, found property"),
            RelmMsg(_, _) => panic!("Expected widget, found relm msg"),
            RelmMsgEvent(_, _) => panic!("Expected widget, found relm msg event"),
            ChildWidget(widget) => widget,
        }
    }
}

named! { attributes -> HashMap<String, Option<LitStr>>, map!(many0!(do_parse!(
    punct!(#) >>
    values: map!(brackets!(separated_by0!(punct!(,), do_parse!(
        name: syn!(Ident) >>
        value: option!(do_parse!(
            punct!(=) >>
            value: syn!(LitStr) >>
            (value)
        )) >>
        (name.to_string(), value)
    ))), |(_, values)|
        values
    ) >>
    (values))), |maps| {
        let mut attrs = HashMap::new();
        for map in maps {
            for (key, values) in map {
                attrs.insert(key, values);
            }
        }
        attrs
    })
}

#[derive(Debug)]
enum PathOrIdent {
    WidgetIdent(Path),
    WidgetPath(Path),
}

impl PathOrIdent {
    fn get_ident(&self) -> &Path {
        match *self {
            WidgetIdent(ref ident) => ident,
            WidgetPath(_) => panic!("Expected ident"),
        }
    }

    fn get_path(&self) -> &Path {
        match *self {
            WidgetIdent(_) => panic!("Expected path"),
            WidgetPath(ref path) => path,
        }
    }
}

named! { path_or_ident -> PathOrIdent,
    map!(syn!(Path), |path| {
        if path.segments.len() == 1 {
            WidgetIdent(path)
        }
        else {
            WidgetPath(path)
        }
    })
}

named! { child_widget(root: SaveWidget) -> (ChildItem, Option<String>), do_parse!(
    attributes: map!(option!(attributes), |attributes| attributes.unwrap_or_else(HashMap::new)) >>
    typ: path_or_ident >>
    relm_widget: cond!(match typ {
        WidgetIdent(_) => true,
        WidgetPath(_) => false,
    }, call!(relm_widget, typ.get_ident().clone())) >>
    gtk_widget: cond!(match typ {
        WidgetIdent(_) => false,
        WidgetPath(_) => true,
    }, call!(gtk_widget, typ.get_path().clone(), attributes.contains_key("name") || root == Save)) >>
    (match typ {
        WidgetIdent(_) => {
            let mut widget = relm_widget.expect("relm widget");
            adjust_widget_with_attributes(widget, &attributes)
        },
        WidgetPath(_) => {
            let mut widget = gtk_widget.expect("gtk widget");
            adjust_widget_with_attributes(widget, &attributes)
        },
    })
    )
}

named! { gtk_widget(typ: Path, save: bool) -> ChildItem, map!(do_parse!(
        init_properties: init_properties >>
        gtk_widget: braces!(do_parse!(
            child_items: separated_by0!(punct!(,), call!(child_gtk_item)) >>
            ({
                let mut gtk_widget = GtkWidget::new();
                gtk_widget.save = save;
                let mut init_parameters = vec![];
                let mut children = vec![];
                let mut properties = HashMap::new();
                let mut child_events = HashMap::new();
                let mut child_properties = HashMap::new();
                for item in child_items.into_iter() {
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
                        Property(ident, value) => { let _ = properties.insert(ident, value.value); },
                        RelmMsg(_, _) | RelmMsgEvent(_, _) => panic!("Unexpected relm msg in gtk widget"),
                    }
                }
                match init_properties {
                    ConstructProperties(construct_properties) => gtk_widget.construct_properties = construct_properties,
                    InitParameters(init_params) => init_parameters = init_params,
                    NoInitProperties => (),
                }
                ChildWidget(Widget::new_gtk(gtk_widget, typ, init_parameters, children, properties, child_properties,
                                     child_events))
            })
        )) >>
        (gtk_widget)
        ), |(_, widget)| widget)
}

named! { child_relm_item -> ChildItem, alt!
    ( relm_property_or_event
    | map!(call!(child_widget, DontSave), |(widget, _)| widget)
    )
}

named! { relm_widget(typ: Path) -> ChildItem, do_parse!(
        init_parameters: option!(map!(parens!(expr_list), |(_, exprs)| exprs)) >>
        relm_widget: map!(option!(braces!(do_parse!(
            child_items: separated_by0!(punct!(,), call!(child_relm_item)) >>
            ({
                let init_parameters = init_parameters.clone().unwrap_or_else(Vec::new);
                let mut relm_widget = RelmWidget::new();
                let mut children = vec![];
                let mut child_properties = HashMap::new();
                let mut child_events = HashMap::new();
                let mut properties = HashMap::new();
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
                        Property(ident, value) => { let _ = properties.insert(ident, value.value); },
                        RelmMsg(ident, value) => { let _ = relm_widget.messages.insert(ident, value.value); },
                        RelmMsgEvent(ident, event) => {
                            let events = relm_widget.events.entry(ident).or_insert_with(Vec::new);
                            events.push(event);
                        },
                    }
                }
                ChildWidget(Widget::new_relm(relm_widget, typ.clone(), init_parameters, children, properties, child_properties,
                                      child_events))
            })
        ))), |widget| {
            widget
                .map(|(_, widget)| widget)
                .unwrap_or_else(|| {
                    let init_parameters = init_parameters.unwrap_or_else(Vec::new);
                    ChildWidget(Widget::new_relm(RelmWidget::new(), typ, init_parameters, vec![], HashMap::new(),
                        HashMap::new(), HashMap::new()))
                })
        }) >>
        (relm_widget)
    )
}

named! { relm_property_or_event -> ChildItem, do_parse!(
    ident: syn!(Ident) >>
    item: alt!
    ( do_parse!(
        punct!(:) >>
        result: call!(value_or_child_properties, ident) >>
        (
            if ident.as_ref().chars().next().map(|char| char.is_lowercase()) == Some(false) {
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
        )
      )
    | do_parse!(
        punct!(.) >>
        event_name: syn!(Ident) >>
        event: event >>
        (ChildEvent(event_name, ident, event))
      )
    | map!(event, |mut event| {
        if ident.as_ref().chars().next().map(|char| char.is_lowercase()) == Some(false) {
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
    })
    ) >>
    (item)
)}

named! { gtk_child_property_or_event -> ChildItem, do_parse!(
    ident: syn!(Ident) >>
    item: alt!
    ( do_parse!(
        punct!(:) >>
        value: call!(value_or_child_properties, ident) >>
        (value)
      )
    | do_parse!(
        punct!(.) >>
        event_name: syn!(Ident) >>
        mut event: event >>
        ({
            if event.params.is_empty() {
                event.params.push(wild_pat());
            }
            ChildEvent(event_name, ident, event)
        })
    )
    | map!(event, |mut event| {
        if event.params.is_empty() {
            event.params.push(wild_pat());
        }
        ItemEvent(ident, event)
    })
    ) >>
    (item)
    )
}

named! { value_or_child_properties(ident: Ident) -> ChildItem, alt!
    ( map!(braces!(child_properties), |(_, properties)| {
        let properties = properties.into_iter()
            .map(|(key, value)| ((ident.clone(), key), value))
            .collect();
        ItemChildProperties(properties)
    })
    | map!(value, |value| Property(ident, value))
    )
}

named! { tag(expected_ident: String) -> (), do_parse!(
    ident: syn!(Ident) >>
    cond!(ident.as_ref() != expected_ident, map!(reject!(), |()| ())) >>
    cond!(ident.as_ref() == expected_ident, epsilon!()) >>
    ()
)}

named! { shared_values -> Option<Vec<Ident>>, option!(do_parse!(
    call!(tag, "with".to_string()) >>
    idents: map!(parens!(ident_list), |(_, idents)| idents) >>
    (idents)
    ))
}

enum IdentOrEventValue {
    MessageIdent(EventValueReturn, bool),
    MessageEventValue(Ident, EventValueReturn, bool),
}

fn expr_use_self(expr: &Expr) -> bool {
    let mut tokens = quote! {};
    expr.to_tokens(&mut tokens);
    tokens.into_iter().any(|token| {
        if let TokenTree::Term(term) = token {
            return term.as_str() == "self";
        }
        false
    })
}

struct Value {
    value: Expr,
    use_self: bool,
}

named! { value -> Value, do_parse!(
    expr: syn!(Expr) >>
    ({
        let use_self = expr_use_self(&expr);
        Value {
            value: expr,
            use_self,
        }
    })
)}

named! { event_value -> (EventValueReturn, bool),
    alt!
    ( do_parse!(
        call!(tag, "return".to_string()) >>
        value: value >>
        (CallReturn(value.value), value.use_self)
      )
    | map!(parens!(do_parse!(
        value1: value >>
        punct!(,) >>
        value2: value >>
        (Return(value1.value, value2.value), value1.use_self || value2.use_self)
      )), |(_, value)| value)
    | do_parse!(
        value: value >>
        (WithoutReturn(value.value), value.use_self)
      )
    )
}

named! { message_sent -> IdentOrEventValue, do_parse!(
    value: alt!
        ( map!(do_parse!(
            ident: syn!(Ident) >>
            punct!(@) >>
            event_value: event_value >>
            ((ident, event_value))
          ), |(ident, (value, use_self))| MessageEventValue(ident, value, use_self))
        | event_value => { |(value, use_self)| MessageIdent(value, use_self) }
        ) >>
    (value)
    )
}

named! { expr_list -> Vec<Expr>,
    map!(call!(Punctuated::parse_terminated), |exprs: Punctuated<Expr, Token![,]>|
         exprs.into_iter().collect())
}

named! { ident_list -> Vec<Ident>,
    map!(call!(Punctuated::parse_terminated), |idents: Punctuated<Ident, Token![,]>|
         idents.into_iter().collect())
}

named! { event -> Event, do_parse!(
    params: option!(map!(parens!(separated_by0!(punct!(,), syn!(syn::Pat))), |(_, params)| params)) >>
    shared_values: shared_values >>
    punct!(=>) >>
    message_sent: message_sent >>
    ({
        let mut event = Event::new();
        if let Some(params) = params {
            event.params = params;
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
        event
    })
)}

named! { child_gtk_item -> ChildItem,
    alt!
    ( gtk_child_property_or_event
    | map!(call!(child_widget, DontSave), |(widget, _)| widget)
    )
}

named! { child_properties -> HashMap<Ident, Expr>, map!(
    separated_by0!(punct!(,), do_parse!(
        ident: syn!(Ident) >>
        punct!(:) >>
        value: value >>
        ((ident, value.value))
    )),
    |properties| properties.into_iter().collect())
}

fn wild_pat() -> Pat {
    parse(quote! {
        _
    }.into())
        .expect("wildcard pattern")
}

impl syn::synom::Synom for Widget {
    named! { parse -> Self,
        do_parse!(
            widget: call!(child_widget, Save) >>
            option!(punct!(,)) >>
            ({
                let (widget, parent_id) = widget;
                let mut widget = widget.unwrap_widget();
                widget.parent_id = parent_id;
                widget
            })
        )
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
        string.push_str(segment.ident.as_ref());
    }
    string
}

fn adjust_widget_with_attributes(mut widget: ChildItem, attributes: &HashMap<String, Option<LitStr>>)
    -> (ChildItem, Option<String>)
{
    let parent_id;
    match widget {
        ChildWidget(ref mut widget) => {
            let container_type = attributes.get("container")
                .map(|typ| typ.as_ref().map(|lit| lit.value()));
            let name = attributes.get("name").and_then(|name| name.clone());
            if let Some(name) = name {
                widget.name = Ident::new(&name.value(), name.span());
            }
            widget.is_container = !widget.children.is_empty();
            widget.container_type = container_type;
            parent_id = attributes.get("parent").and_then(|opt_str| opt_str.as_ref().map(|lit| lit.value()));
        },
        _ => panic!("Expecting widget"),
    }
    (widget, parent_id)
}

#[cfg(feature = "unstable")]
pub fn respan_with(tokens: proc_macro::TokenStream, span: proc_macro::Span) -> proc_macro::TokenStream {
    let mut result = vec![];
    for mut token in tokens {
        match token.kind {
            proc_macro::TokenNode::Group(delimiter, inner_tokens) => {
                let new_tokens = respan_with(inner_tokens, span);
                result.push(TokenTree {
                    span,
                    kind: proc_macro::TokenNode::Group(delimiter, new_tokens),
                });
            },
            _ => {
                token.span = span;
                result.push(token);
            }
        }
    }
    FromIterator::from_iter(result.into_iter())
}
