# Further reading

- Jake Gordon, [javascript-racer](https://jakesgordon.com/writing/javascript-racer/) (2012): the
  four-part series this port follows, with
  [v1 straight roads](https://jakesgordon.com/writing/javascript-racer-v1-straight/),
  [v2 curves](https://jakesgordon.com/writing/javascript-racer-v2-curves/),
  [v3 hills](https://jakesgordon.com/writing/javascript-racer-v3-hills/) and
  [v4 final](https://jakesgordon.com/writing/javascript-racer-v4-final/), and the
  [source code](https://github.com/jakesgordon/javascript-racer).
- Louis Gorenfeld, [Lou's Pseudo 3D Page](http://www.extentofthejam.com/pseudo/): the raster and the
  segment techniques, explained from the arcade hardware up.
- Glenn Fiedler, [Fix Your Timestep!](https://gafferongames.com/post/fix_your_timestep/): why the
  simulation runs at a fixed rate.
- [game-toolkit](https://github.com/sunsided/game-toolkit): the toolkit the port runs on.
- [sillyecs](https://github.com/sunsided/sillyecs): the ECS code generator.
- [rust-racer](https://github.com/sunsided/rust-racer): this game. As usual, you can find the entire
  code on GitHub.

That's all there is to it: a list of road slices, one division per point, an offset that grows with
distance, and the order you paint things in. Thanks for making it here, and have fun on the road!
