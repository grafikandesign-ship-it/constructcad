# Sketchdown direction on the ConstructCAD fork

Decision recorded 27 October 2026. ConstructCAD is the product base for architectural construction, 3D visualization and presentation sheets. Sketchdown's free 3D modeling is a later, separate workspace. The previous plan to begin with a Sketchdown-to-ConstructCAD component bridge is superseded.

## Product order

1. **Construction:** keep ConstructCAD's native BIM model and fast plan workflow as the source of truth. Refine walls, openings, levels, stairs, roofs and renovation phases in the existing command engine.
2. **Visualization:** derive 3D views from that model; improve navigation, materials, textures, lighting and scene presentation without turning building elements into anonymous meshes.
3. **Presentation:** derive drawings and sheets from the same model and views; make the path from a design decision to a presentable sheet direct.
4. **Modeling:** later bring Sketchdown's free geometry tools into a dedicated workspace for custom furniture and other off-catalog objects, then publish reusable definitions to the component catalog.

Construction, visualization and presentation share one `.ccad` document. The fourth workspace edits component definitions and places their instances in that document. A fixed imported mesh must not be described as parametrically editable unless its source parameters and operations have been preserved.

## Interface direction

Preserve ConstructCAD's project browser, properties panel, ribbon, split plan/3D views, command palette and command-driven model. Reorganize access around **Construction → Visualization → Presentation** before adding a Modeling workspace. Tools for structure, MEP, annotations and management remain available; changing navigation must not remove their commands or break shortcuts and automation. Component *placement* belongs with construction; component *authoring* belongs in Modeling when that workspace exists.

Refresh the application identity and SVG icon language within the repository's existing icon and theme systems. Keep third-party attribution and the GPL license. UI text must use the existing translation files. Prioritize a clean drawing area, obvious active mode, consistent tool availability, fast switching between plan and 3D, and explicit messages for unavailable commands.

### First ribbon map

| Workspace | Dedicated icon | Commands brought together | Expected result |
| --- | --- | --- | --- |
| Architecture | Building envelope and doorway | Walls, openings, floors, roof, stairs, rooms and components | Edit the native building model. |
| Visualization | Eye containing a 3D volume | 3D view, materials, visibility, filters, view templates and temporary isolation | Inspect and style the same model. |
| Presentation | Sheet with view and caption | Plans, sections, elevations, legends, schedules, sheets, dimensions, annotations and PDF/SVG/image export | Produce drawings from current views. |

The File, Modify, Structure, MEP and Manage tabs stay available. The icons are original SVGs in the existing icon system. The command palette and keyboard shortcuts continue to resolve the same `UiCommand` entries.

## First reviewable milestone

- A GitHub fork under the user's account and a separate Windows checkout; preserve the upstream remote and original ConstructCAD credits.
- A proposed navigation and icon map based on actual commands, with no engine rewrite.
- One compiled interface increment that makes the Construction → Visualization → Presentation sequence visible and preserves every existing command.
- A short native workflow using a copy of the Bergem source material: create or adjust a wall and opening, inspect the derived 3D view, apply a visual material, place a view on a sheet, save and reopen.
- Keep renovation *phase* separate from geometric *confidence* for the existing basement and ground floor. Project drawings alone do not establish as-built structure.

Sketchdown's existing `.igz` work remains a reference and source for the later modeling workspace; it is not the new architectural file format. No migration of the Bergem model is assumed complete by creating this fork.

Upstream starting commit: `afbd26702c094215b63d83c0afec15ae5da2b136` (`francisreboucas/constructcad`, `main`).
