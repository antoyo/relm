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

use futures::{Future, IntoStream, Poll, Stream};

pub struct RelmStream<ERROR, ITEM, STREAM: Stream<Item=ITEM, Error=ERROR>> {
    stream: STREAM,
}

impl<ERROR, ITEM, STREAM: Stream<Item=ITEM, Error=ERROR>> Stream for RelmStream<ERROR, ITEM, STREAM> {
    type Item = ITEM;
    type Error = ERROR;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        self.stream.poll()
    }
}

pub trait ToStream<STREAM: Stream<Item=Self::Item, Error=Self::Error>> {
    type Error;
    type Item;

    fn to_stream(self) -> RelmStream<Self::Error, Self::Item, STREAM>;
}

impl<ERROR, ITEM, STREAM: Stream<Item=ITEM, Error=ERROR>> ToStream<STREAM> for STREAM {
    type Error = ERROR;
    type Item = ITEM;

    fn to_stream(self) -> RelmStream<ERROR, ITEM, STREAM> {
        RelmStream {
            stream: self,
        }
    }
}

impl<ERROR, FUTURE: Future<Item=ITEM, Error=ERROR>, ITEM> ToStream<IntoStream<FUTURE>> for FUTURE {
    type Error = ERROR;
    type Item = ITEM;

    fn to_stream(self) -> RelmStream<ERROR, ITEM, IntoStream<FUTURE>> {
        RelmStream {
            stream: self.into_stream(),
        }
    }
}
