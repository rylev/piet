# Piet: a 2D graphics abstraction

This repo holds an API for 2D graphics drawing.

The motivation for this crate is set forth in this [blog post]. Ideally it will become a layer to help [druid] become cross-platform.

This repo is structured as a core API crate, "piet" and a separate crate for each back-end, currently "piet-direct2d", "piet-cairo", and "piet-web". One motivation for this structure is that additional back-ends can be written without coupling to the main crate, and clients can opt in to the back-ends they need. In addition, it's possible use multiple back-ends, which will likely be useful for testing.

A companion for Bézier path representation and geometry is [kurbo].

The library is of course named after [Piet Mondrian]. It's abstract and hopefully will be used for drawing lots of rectangles.

Contributions are welcome! It's in early stages, so there are lots of opportunities to fill things out.

[blog post]: https://raphlinus.github.io/rust/graphics/2018/10/11/2d-graphics.html
[druid]: https://github.com/xi-editor/druid
[kurbo]: https://github.com/linebender/kurbo
[Piet Mondrian]: https://en.wikipedia.org/wiki/Piet_Mondrian
