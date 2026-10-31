# ARCHLine.XP wall preset extraction

Source: `ARCHlineXP2024/Templates/Local_Ita/Wall/*.set` from the installed ARCHLine.XP library.
The source files were read only and are not included in this repository.
Widths in `AttrWall.Wids` are cumulative millimetres; adjacent differences form the layers below.
The first `Mats` entry is the face material; subsequent entries are the layer labels.
ConstructCAD retains the widths and labels. Material colors, hatches, render textures and thermal properties are approximations and should be reviewed before technical use.
Only names that explicitly indicate an exterior wall are assigned the exterior function; other imported walls default to interior and remain editable.

Imported: 35. Excluded: 1.

| Category | ARCHLine style | Layers (mm) | Material labels |
| --- | --- | --- | --- |
| Generic | 1 layered 30 wide wall | 300 = 300 | Brick3 |
| Generic | 1 layered 38 wide wall | 380 = 380 | Brick3 |
| Concrete | CEMENTO 20 | 200 = 200 | Prefab concrete |
| Exterior | EST.MATTONI 1 | 10 + 80 + 75 + 75 + 200 + 50 + 10 = 500 | Plaster1 / Brick3 / Palette04 / Palette041 / {DEF5A562-7AF8-B648-8AEF-81612107AAEC} / {DEF5A562-7AF8-B648-8AEF-81612107AAEC} / Plaster1 |
| Exterior | EST.MATTONI 2 | 20 + 75 + 50 + 15 + 150 + 20 = 330 | Plaster2 / Brick4 / Palette04 / Wood-beech / {DEF5A562-7AF8-B648-8AEF-81612107AAEC} / Plaster2 |
| Exterior | EST.MATTONI 30 | 75 + 50 + 9 + 15 + 150 + 1 = 300 | Brick4 / Palette04 / Palette041 / Wood-beech / {DEF5A562-7AF8-B648-8AEF-81612107AAEC} / Palette05 |
| Exterior | EST.MATTONI 3 | 100 + 70 + 1 + 10 + 152 + 1 + 16 = 350 | Palette0143 / Palette04 / Palette041 / Wood-beech / {DEF5A562-7AF8-B648-8AEF-81612107AAEC} / Brick4 / Plaster1 |
| Exterior | EST.MATTONI 50 | 10 + 80 + 75 + 75 + 200 + 50 + 10 = 500 | Plaster1 / Brick3 / Palette04 / Palette041 / {DEF5A562-7AF8-B648-8AEF-81612107AAEC} / {DEF5A562-7AF8-B648-8AEF-81612107AAEC} / Plaster1 |
| Exterior | ESTERNO 30 Strati | 20 + 230 + 30 + 20 = 300 | Plaster1 / Terca_0043 / Thermal Isolation / Plaster1 |
| Generic | GEN.MATTONI 15 | 150 = 150 | Masonry-running |
| Generic | GENERICO 15 | 150 = 150 | Brick4 |
| Generic | GENERICO 20 | 200 = 200 | Brick4 |
| Generic | GENERICO 25 | 250 = 250 | Brick4 |
| Generic | GENERICO 30 (Stucco21) | 300 = 300 | Terca_0043 |
| Generic | GENERICO 30 (Stucco2) | 300 = 300 | Terca_0040 |
| Generic | GENERICO 38 | 380 = 380 | Brick3 |
| Interior | INTERNO 10 Strati | 20 + 60 + 20 = 100 | Plaster1 / Brick3 / Plaster1 |
| Interior | INTERNO 12 Strati | 20 + 80 + 20 = 120 | Plaster1 / Terca_0043 / Plaster1 |
| Interior | INTERNO 15 Strati | 15 + 120 + 15 = 150 | Plaster1 / Terca_0043 / Plaster1 |
| Timber | LEGNO 33 | 30 + 40 + 30 + 200 + 30 = 330 | Brick3 / Beech / {755576E3-A851-A340-89E2-91A0F0D7B2EC} / HOSZIGVAK / Brick3 |
| Generic | PREDEFINITO 30 | 300 = 300 | Terca_0043 |
| Generic | Muro predefinito da 30 | 300 = 300 | Terca_0043 |
| Generic | MF-10 | 30 + 60 + 130 + 80 = 300 | Terca_0043 / Default material / Terca_0043 / Terca_0043 |
| Generic | MF-11 | 30 + 60 + 140 + 60 = 290 | Terca_0043 / Default material / Terca_0043 / Terca_0043 |
| Generic | MF-12 | 30 + 60 + 140 + 60 = 290 | Terca_0043 / Default material / Terca_0043 / Terca_0043 |
| Generic | MF-1 | 80 + 60 + 180 = 320 | Terca_0043 / Default material / Terca_0043 |
| Generic | MF-2 | 30 + 60 + 140 + 60 = 290 | Terca_0043 / Default material / Terca_0043 / Terca_0043 |
| Generic | MF-3 | 80 + 60 + 125 = 265 | Terca_0043 / Default material / Terca_0043 |
| Generic | MF-4 | 80 + 60 + 125 = 265 | Terca_0043 / Default material / Terca_0043 |
| Generic | MF-5 | 80 + 60 + 125 = 265 | Terca_0043 / Default material / Terca_0043 |
| Generic | MF-6 | 80 + 60 + 130 = 270 | Terca_0043 / Default material / Terca_0043 |
| Generic | MF-7 | 80 + 60 + 130 = 270 | Terca_0043 / Default material / Terca_0043 |
| Generic | MF-8 | 280 = 280 | Terca_0043 |
| Generic | MF-9 | 260 = 260 | Terca_0043 |
| Timber | Timber 200 | 30 + 40 + 30 + 200 + 30 = 330 | Brick3 / Beech / {755576E3-A851-A340-89E2-91A0F0D7B2EC} / HOSZIGVAK / Brick3 |

## Excluded

- Wall_U_Profile_31fitO.set: U-shaped section requires a profile model
