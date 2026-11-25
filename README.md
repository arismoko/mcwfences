# Macaw's Fences and Walls for Sunrise

198 fences, gates, walls and hedges in every wood, in metal and in many stone finishes.

A port of [Macaw's Fences and Walls](https://github.com/sketchmacaw/Fences) by Sketch Macaw and Peachy Macaw to [Sunrise](https://playsunrise.net) (Minecraft Java 26.3), as a content mod with a small native part: the original's blocks, models, textures, recipes and strings, plus the wired fences' contact damage, the metal fences' joining and the curved gates.

## What it does

- Picket, stockade, horse and wired fences, highley and pyramid gates and modern railings in every wood, pale oak and poplar included.
- Pillared and grass-topped walls in stone finishes, and leaf hedges.
- Fourteen metal fence designs, and curved gates in every wood and in metal, which open together with the gates touching them.
- Fences and walls join their neighbours, gates face the way you place them, and fences can be waterlogged.
- Wired wood fences and the wooden cheval de frise hurt what stands in them (2 cactus damage), and the iron cheval de frise hurts more (4).
- 241 recipes (crafting and stonecutting), drops and a creative tab.

## Differences from the original

- None known: the wired fences' contact damage, the metal fences and the curved gates are ported.

## Building

With the Sunrise mod tool (`cargo sunrise-mod`, from the Sunrise SDK):

```sh
cargo sunrise-mod check
cargo sunrise-mod build    # target/sunrise-mod/mcwfences.smod
```

This version is made for the Sunrise Dev build `dev-2026-11-25-bdad86d`.

## Licence and credits

Macaw's Fences and Walls is by Sketch Macaw and Peachy Macaw (https://github.com/sketchmacaw/Fences). Its textures, models, recipes and strings are used here under the **MIT licence**, the original's; `LICENSE` holds the text and its copyright notice, and `LICENSE-NOTICE.md` says where it was read.

Carried to Minecraft 26.3 by unnamed0631 ([unnamed0631/mcwfences](https://github.com/unnamed0631/mcwfences)), and ported to Sunrise by arismoko. This port is distributed under the same licence.
