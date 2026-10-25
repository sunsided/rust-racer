# A little history

Racing games had a problem long before anyone had the hardware to solve it properly: a road
that disappears into the distance is a 3D object, and early-1980s arcade boards had no business
drawing 3D objects. What they could do was draw horizontal lines of pixels quickly, shift them
left and right, and (on the fancier boards) scale sprites up and down. Everything in this book
grows out of that constraint.

## The arcade years

[Namco's Pole Position](https://en.wikipedia.org/wiki/Pole_Position) (1982) set the template
most people picture: a road drawn as a trapezoid narrowing towards a horizon, stripes on the
roadside rushing towards you, other cars and billboards as sprites that grow as they approach.
The road wasn't a model of anything; it was a picture of a road, redrawn every frame so that
it *looked* like you were moving along it.

Sega pushed the idea to its peak with [Out Run](https://en.wikipedia.org/wiki/Out_Run) (1986),
designed by Yu Suzuki: the Ferrari, the branching route across Europe-inspired scenery, the
radio stations, and a road that didn't just bend left and right but rolled over hills and
dipped into valleys. Sega's arcade hardware of the time (the line of boards that gave us
Hang-On and Space Harrier, marketed as "Super Scaler" technology) could scale large numbers of
sprites in hardware, which is what made the dense roadside scenery possible. Out Run's road
is still the reference look for this whole genre, and it's the look Jake Gordon was going for
when he wrote javascript-racer, 26 years later.

## Lou's Pseudo 3D Page

For a long time, how those games did it was arcade folklore. The page that wrote it down is
[Lou's Pseudo 3D Page](http://www.extentofthejam.com/pseudo/) by Louis Gorenfeld (last updated
in 2013), which is still the best single explanation around. It covers two families of
technique:

- the *raster* approach the original hardware used: a lookup table (the "Z map") from screen
  line to distance, with curves made by adding an ever-growing horizontal offset per line and
  hills made by stretching and squashing the lines vertically, and
- a *segment* approach that projects actual 3D points of a road chopped into slices, which he
  calls "Realistic Hills Using 3d-Projected Segments".

The two have one trick in common that matters more than anything else in this book: **curves
are never modeled**. The road stays perfectly straight in world space, and the renderer adds
a growing sideways offset as it walks towards the horizon. That's addition only, no
trigonometry, no rotation; you'll meet it in the [curves chapter](curves.md).

## javascript-racer

In 2012, Jake Gordon wrote a [four-part series](https://jakesgordon.com/writing/javascript-racer/)
on building an Out Run-style racer in plain JavaScript on an HTML5 canvas: v1 with straight
roads, v2 adding curves, v3 adding hills, and v4 ("final") adding the sprites, traffic, and
lap timing. He credits Lou's page as "the main source of information (that I could find)
online about how to build a pseudo-3d racing game", and builds a variation of Lou's
3D-projected segments. Jake calls the result "just-enough" pseudo-3D: points are translated
and projected, but never rotated, since the curves are faked anyway. The code is deliberately
simple (his words: KISS), and that's exactly what makes it such a good thing to port.

## This port

Rust Racer is a fairly literal port of v4. The track layout, constants, physics, traffic AI,
colors, art and music are all Jake's, and if you put the two side by side they should drive
the same. What changed is everything around it: the JavaScript game loop became a
game-toolkit `Game`, the canvas became a wgpu-backed `Painter` with layers, the global state
and the one big update function became an ECS with seven systems, and the result runs both
natively and (via WebAssembly) in the browser. Porting it also turned up a handful of things
game-toolkit and sillyecs were missing, which got fixed upstream along the way (the
[Rust chapters](ecs.md) mention where).

So "pseudo" here means what it meant in 1982: no meshes, no depth buffer, no camera matrix.
There is a list of road slices, one perspective division per point, a sideways offset that
grows with distance, and the painter's algorithm. That's the whole engine.
