# Macaw's Fences and Walls for Sunrise

150 fences, gates, walls and hedges in eleven woods and many stone finishes.

A port of [Macaw's Fences and Walls](https://github.com/sketchmacaw/Fences) by Sketch Macaw and Peachy Macaw to [Sunrise](https://playsunrise.net) (Minecraft Java 26.3), as a content mod: the original's blocks, models, textures, recipes and strings, with no code.

## What it does

- Picket, stockade, horse and wired fences, highley and pyramid gates and modern railings in eleven woods.
- Pillared and grass-topped walls in stone finishes, and leaf hedges.
- Fences and walls join their neighbours, gates face the way you place them, and fences can be waterlogged.
- 192 recipes (crafting and stonecutting), drops and a creative tab.

## Differences from the original

- The wired fences don't hurt you when you walk into them; that is Java code.

## Building

With the Sunrise mod tool (`cargo sunrise-mod`, from the Sunrise SDK):

```sh
cargo sunrise-mod check
cargo sunrise-mod build    # target/sunrise-mod/mcwfences.smod
```

This version is made for the Sunrise Dev build `dev-2026-11-22-7cfdf81`.

## Licence and credits

Macaw's Fences and Walls is by Sketch Macaw and Peachy Macaw (https://github.com/sketchmacaw/Fences). Its textures, models, recipes and strings are used here under the **MIT licence**, the original's; `LICENSE` holds the text and its copyright notice, and `LICENSE-NOTICE.md` says where it was read.

Carried to Minecraft 26.3 by unnamed0631 ([unnamed0631/mcwfences](https://github.com/unnamed0631/mcwfences)), and ported to Sunrise by arismoko. This port is distributed under the same licence.
