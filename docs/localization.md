# Language management

The Sketchdown fork starts in Italian when no language preference exists. The user can switch languages on the home screen before creating a project or in the File tab while working. The choice is saved between app sessions. A command-line `--lang` value takes precedence for that launch; the CLI and MCP server also accept `CCAD_LANG` and `language_set`.

The language choice changes interface text, command descriptions, formatted decimal separators and names assigned to *new* projects. Names already saved in a `.ccad` document remain project data. Switching languages invalidates cached drawings so localized labels are refreshed.

The 8 language files in `assets/i18n/` use one key per line and `{ $name }` placeholders. Italian is the first editorial reference for this fork; all languages must keep the same keys and placeholders. The tests in `cc_engine::i18n` reject missing, extra or duplicate keys. New features should add a key in every language file before they are merged. Keep stored codes (`it`, `en`, `fr`, `de`, `es`, `pt-BR`, `pl`, `uk-UA`) independent from translated names so saved preferences remain stable.

## Italian editorial choices

- Use **Navigatore di progetto** for the project browser, **tavola** for a sheet, **vista** for a view, **quote** for dimensions and **prospetto** for an elevation.
- Use familiar construction terms for model objects: **muro**, **solaio**, **copertura**, **controsoffitto**, **pilastro** and **trave**. Use the same term in commands, properties and hints.
- Keep file formats and technical identifiers such as IFC, PDF, SVG, `.ccad` and engine command names untranslated.
- Review text in context at a 1100 px wide window as well as in the translation file. Short ribbon labels matter; tooltips can explain more fully.

The first Italian revision covers navigation, language feedback and the main architecture and presentation vocabulary. It does not turn engine diagnostic details or names contained in imported files into translated text.
