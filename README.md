# `dwwb`, Dreamweaver's Wiki Builder

A static site generator for creating simple html sites from markdown articles using Pandoc.

Running this requires for [Pandoc](https://pandoc.org/installing.html) to be downloaded and installed.
Version 2.18 or newer should be compatible.

The currently implemented subcommands are:

* `help [SUBCOMMAND]`
    * Prints out the help information either generally or of the given subcommand
    * Can also be used with the `--help` flag, or the shorter `-h` flag for a shorter summary
* `new PATH`
    * Creates a new empty dwwb project
    * The resulting directory contains:
        * `dwwb.yaml`, the main configuration file
        * `index.md`, the index article
        * `templates/dwwb-article.html`, the article template
        * `templates/sidebar.html`, the sidebar template
        * `articles/example.md`, an example article
        * `scripts/main.js`, an empty script file
        * `style.css`, the default stylesheet
* `build`
    * Converts the markdown files into html files, and copies over the other files
* `clean`
    * Removes the html output directory
* `add PATH`
    * Adds a new article to the input article directory


## Installing

Currently, you can install the repository 2 ways, either getting a local copy of the repo and building it, or installing it directly from github with cargo.

To build this you need to have the Rust toolchain version 1.65 or higher installed.
You can find the latest version at <https://www.rust-lang.org/tools/install>.

Once I think this is stable enough I'll publish it as a crate to [crates.io](https://crates.io).


### Installing from github

To download and install the latest version from github with Cargo you need to run the following command:

```
cargo install --git https://github.com/WeaverTVSC/dwwb
```


### Installing from cloned repo

After cloning or downloading the repository, you need to run the following command in the code directory:

```
cargo install --path .
```

(Or you can replace `.` with the path if it's not your current working directory)


## `dwwb.yaml`

This is the main configuration file.

It contains important information about the wiki and some arguments that are passed straight to pandoc.
Think of it like as the `Cargo.toml` file of dwwb.

The current keys of `dwwb.yaml` are:

* `name`
    * The name of the wiki
* `inputs`
    * All the paths and path globs of the files that will be processed or copied into the final build
        * The globs contain the path to the base directory, followed by a list of glob patterns
    * The glob patterns follow the [.gitignore syntax](https://git-scm.com/docs/gitignore#_pattern_format)
    * `index`
        * Default: `index.md`
        * The path of the index article
    * `style`
        * Default: `style.css`
        * The path to the stylesheet
        * The given stylesheet will be used on all generated html files
    * `article-template`
        * Default: `templates/dwwb-article.html`
        * The path to the pandoc template to be used with the generated articles
    * `articles`
        * Default:

        * ```yaml
          articles:
            base: articles
            patterns:
            - '**/*.{md,markdown}'
          ```

        * The input folder for markdown articles
    * `scripts`
        * Default:

        * ```yaml
          articles:
            base: scripts
            patterns:
            - '**/*.js'
          ```

        * The input folder of script files to include within `<script>` tags into each generated html file
    * Any other input folders
        * You can have arbitrary input directories with their own patterns by using the same syntax as the above `articles` and `scripts` folders have
        * The names/keys of the input directories need to match exactly to their counterparts in the `outputs`
* `outputs`
    * All the paths for the files outputted by the `build` command
    * `root`
        * Default: `html`
        * The root directory for all of the other output paths
        * All of the other output paths are relative to this
    * `style`
        * Default: `style.css`
        * The output file for the stylesheet
    * `articles`
        * Default: `articles`
        * The output directory for the generated html articles
    * `scripts`
        * Default: `scripts`
        * The output directory for the copied script files
    * Any other output folders
        * The keys must match their counterparts in the `inputs`
* `articles-title`
    * Default: `Articles`
    * The title of the list of sub-articles in the sidebar
* `sub-articles-title`
    * Default: `Sub-Articles`
    * The title of the list of sub-articles in the sidebar
* `toc-title`
    * Default: `Table of Contents`
    * The title of the table of contents in the sidebar
* `toc-depth`
    * Default: `3`
    * The depth of how many articles deep the sidebar table of contents shows
* `math-renderer`
    * Optional
    * The math rendering for rendering TeX math between dollar signs, or double-dollar signs
        * <https://pandoc.org/MANUAL.html#extension-tex_math_dollars>
    * Has fields `engine`, and optional field `url` for the url of the engine if you don't want to use the default one
        * Possible engines:
            * [`mathjax`](https://www.mathjax.org/)
                * A JavaScript library which renders MathML, TeX, and ASCIImath inputs
                * Adds an online reference to the script by-default, which is not suitable for a fully offline wiki
            * [`mathml`](https://pandoc.org/MANUAL.html#option--mathml)
                * Converts TeX code to MathML
                * MathML is not supported natively by every browser
            * [`webtex`](https://pandoc.org/MANUAL.html#option--webtex)
                * Converts TeX code to `<img>` tags
            * [`katex`](https://katex.org/)
                * A fast JavaScript library which renders a limited subset of LaTeX's markup
                * Adds an online reference to the script by-default, which is not suitable for a fully offline wiki
            * [`gladtex`](https://humenda.github.io/GladTeX/)
                * A preprocessor software for converting LaTeX markup to images
                * Must be installed locally before running dwwb
                * Not yet properly implemented, you'll have to run GladTeX on the resulting files by yourself
                * Being an offline Python application, does not allow the `url` field
    * Example `math_renderer` value for using the default online MathJax-engine:

        * ```yaml
          math_renderer:
            engine: mathjax
          ```

* `debug-pandoc-cmd`
    * Optional, default: `false`
    * Whether to print the pandoc commandline invocation


## Writing articles

Articles are written with [pandoc flavored markdown syntax](https://pandoc.org/MANUAL.html#pandocs-markdown).

An article can have sub-articles if they're in a directory with the same name as the main article (without the file extension).
The index article is an exception, as all the top-level files in the input articles directory are its sub-articles.

By default all of the URLs in the markdown files are local to the directory they're located in.
If you want to refer to the root of the wiki, there is a special pandoc filter that's executed for all articles which replaces all occurrences of the string `%ROOT%` with the local URL path to the root directory, ie. `%ROOT%/img/pic.png` would become `../../img/pic.png` if it was used in an article 2 directories down from the root output directory.


### The article metadata

Every markdown article *must* have a YAML metadata block.
It is delimited by triple hyphens (`---`) at the start and triple hyphens (`---`) or triple dots (`...`) at the end.

By default pandoc allows you to define the metadata in other ways as well, but dwwb expects the YAML metadata block.
It also *must* contain the title of the article.

I recommend putting the metadata block at the start of the article, but it can occur anywhere, but if it's not at the beginning, it must be preceded by a blank line.

An example metadata block:

```YAML
---
title: Example
author: John Doe
keywords:
- foo
- bar
---
```


### Syntax

To get a comprehensive understanding of how the pandoc markdown differs from other flavors and to get the most out of pandoc, I encourage you to read the pandoc documentation, but here's a small list of a few noticeable features:

* Attributes
    * You can give headers, code blocks, links, and images custom HTML attributes by immediately following them with a curly brace delimited attribute block
    * Also it can be used to create custom `<span>` blocks with custom attributes
        * This can be used for example to create custom text stylings from the CSS stylesheet
    * Examples:
        * Header: `# My Heading{#header .class key="value"}`
        * Span: `[blah blah]{.dialogue}`
        * Link: `[my link](https://pandoc.org/MANUAL.html){.important}`
        * Image: `![image](foo.jpg){#image-id .class width=30px height=20px}`
        * Inline code block: `` `let x = 1;`{.rust} ``
    * Few special classes defined by the default pandoc stylesheet:
        * `.underline` - Underlines text
        * `.mark` - Highlights text
        * `.smallcaps` - Enables small caps
* Heading identifiers
    * Headings are given automatic HTML identifiers by-default, but they can also be given explicit identifiers
    * The syntax for this uses the curly braces like the other attributes, like so: `# My Heading {#header-identifier}`
* Implicit header references
    * You can reference to a heading with just writing it's name in a link, like `[My Heading]` or `(link)[my heading]`, instead of using the identifier explicitly, like `(link)[#header-identifier]`
* Strikeout
    * You can strike out text by enclosing it in double tildes, like `~~struck out~~`
* Superscript and subscript
    * You can make text appear as superscript by surrounding it in carets, like `10^5^`
    * Likewise, you can make text appear as subscript by surrounding it with tildes, like `H~2~O`
    * These can't contain spaces or newlines
* Escaped line breaks
    * A backslash followed by a newline becomes a hard line break, eg. a line like `Hello world!\` becomes `<p>Hello world!<br>...</p>`.


## Legal
Copyright 2022 WeaverTVSC (<weaver.imaginarium@proton.me>).

This is free software and provided "as is" without warranty of any kind.

Dwwb is released under the GNU General Public License version 3 or greater.

A copy of the GPLv3 license has been included in the file COPYING.
