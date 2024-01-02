[![Posting to Mastodon](https://github.com/phijor/fractalbot/actions/workflows/post.yml/badge.svg)](https://github.com/phijor/fractalbot/actions/workflows/post.yml)

---

# Fractalbot

A bot that generates a random fractal and posts it to Mastodon.

## How it works

The bot renders a [Julia set](https://en.wikipedia.org/wiki/Julia_set)
via a [distance-estimation method](https://iquilezles.org/articles/distancefractals/).
Each Julia set is determined by a complex function $f_c(z) = z^2 + c$ depending on parameter $c$:
it is the set of complex numbers $z$ for which the sequence
```math
    z_0 = z,\ \ z_{n+1} = f_c(z_n) = z_n^2 + c
```
stays bounded.

In order, the bot does the following:

1. Read the complex parameter $c$ from the command line (`-c`), or sample it randomly.
2. Approximate the _bounding box_ of the set via [inverse iteration](https://e.math.cornell.edu/people/bdozier/mat331-spr20/projects/project2/proj2.pdf).
    This method yields a sequence of points on the boundary of the set from which the bounding box is computed.
3. Scale the bounding box to 120% to add a slight margin.
4. Select a color palette, depending on whether the set is connected or not.
    A Julia set is connected if and only if iteration for $z_0 = 0+0i$ stays bounded
    (i.e. $c$ is contained in the Mandelbrot set).
5. For each pixel in the final image, estimate the distance of the corresponding
    complex point in the bounding box to the set.
    For distances $\leq 0$, the point is assumed to lie inside the set and
    is colored black, otherwise a color is sampled from the palette.
    Color palettes are generated following [Inigo Quilez' amazing tutorial](https://iquilezles.org/articles/palettes/).
    Rendering is parallelized using `rayon`.
6. Depending on the mode, save the image to disk or post it to Mastodon.
    To interact with Mastodon, [`megalodon`](https://docs.rs/megalodon/latest/megalodon/mastodon/index.html) is used.
