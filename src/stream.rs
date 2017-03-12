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

pub struct RelmStream<E, I, S: Stream<Item=I, Error=E>> {
    stream: S,
}

impl<E, I, S: Stream<Item=I, Error=E>> Stream for RelmStream<E, I, S> {
    type Item = I;
    type Error = E;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        self.stream.poll()
    }
}

pub trait ToStream<S: Stream<Item=Self::Item, Error=Self::Error>> {
    type Error;
    type Item;

    fn to_stream(self) -> RelmStream<Self::Error, Self::Item, S>;
}

impl<E, I, S: Stream<Item=I, Error=E>> ToStream<S> for S {
    type Error = E;
    type Item = I;

    fn to_stream(self) -> RelmStream<E, I, S> {
        RelmStream {
            stream: self,
        }
    }
}

impl<E, F: Future<Item=I, Error=E>, I> ToStream<IntoStream<F>> for F {
    type Error = E;
    type Item = I;

    fn to_stream(self) -> RelmStream<E, I, IntoStream<F>> {
        RelmStream {
            stream: self.into_stream(),
        }
    }
}
