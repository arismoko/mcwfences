#!/usr/bin/env python3
"""Sets each Macaw block item's furnace fuel to the original's.

usage: macaw_fuel.py <original checkout> <source subfolder> <port checkout> <namespace> <components report>

<components report> is a vanilla data report's `reports/minecraft/components/item` folder.

The original registers its burnable block items as `FuelItemBlock`, whose `getBurnTime` returns one
number; every other item is not fuel. A port block declared on a vanilla wooden template inherits
that template's `minecraft:cooking_fuel`, so stone walls, metal doors and nether wood would burn.
This writes `minecraft:cooking_fuel` with the original's burn time on the `FuelItemBlock` items and
removes it (`!minecraft:cooking_fuel`) on the other items the source registers whose template is fuel (read from a vanilla
component report). Items the source does not register
(added in later releases than the source) keep their template's fuel.
"""
import json, re, sys
from pathlib import Path

orig, sub, port, ns, REPORT = Path(sys.argv[1]), sys.argv[2], Path(sys.argv[3]), sys.argv[4], Path(sys.argv[5])
java = list((orig / sub).rglob('*.java'))
fuel_class = next(p for p in java if p.name == 'FuelItemBlock.java').read_text()
burn = int(re.search(r'getBurnTime[^{]*\{\s*return (\d+);', fuel_class).group(1))
fuel, registered = set(), set()
for p in java:
    for line in p.read_text(errors='replace').splitlines():
        m = re.search(r'ITEMS\.register\("([a-z0-9_]+)"', line)
        if m:
            registered.add(m.group(1))
            if 'new FuelItemBlock(' in line:
                fuel.add(m.group(1))
def template_fuel(template):
    report = json.loads((REPORT / (template.split(':')[1] + '.json')).read_text())
    return 'minecraft:cooking_fuel' in report.get('components', report)


blocks = sorted((port / 'data' / ns / 'sunrise' / 'block').glob('*.json'))
blocks += sorted((port / 'data' / ns / 'sunrise' / 'item').glob('*.json'))
names = {p.stem for p in blocks}
missing = fuel - names
if missing:
    print(f'{ns}: not in the port: {sorted(missing)}')
on = off = 0
for p in blocks:
    decl = json.loads(p.read_text())
    if decl.get('item') is False:
        continue
    comps = {k: v for k, v in decl.get('components', {}).items() if not k.endswith('minecraft:cooking_fuel')}
    if p.stem in fuel:
        comps['minecraft:cooking_fuel'] = {'burn_time': burn, 'speed_multiplier': 'minecraft:cooking/speed_default'}
        on += 1
    elif p.stem in registered and template_fuel(decl['template']):
        comps['!minecraft:cooking_fuel'] = {}
        off += 1
    if comps:
        decl['components'] = comps
    else:
        decl.pop('components', None)
    p.write_text(json.dumps(decl, indent=2) + '\n')
print(f'{ns}: {on} burn for {burn} ticks, {off} no longer burn')
