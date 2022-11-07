# `dwwb`, Dreamweaver's Wiki Builder

Converts a markdown wiki into a html-site.
Running this requires for [Pandoc](https://pandoc.org/installing.html) to be installed.

The currently implemented subcommands are:

* New
    * Creates a new empty wiki project
    * The resulting directory contains:
        * `dwwb.yaml`, the wiki configuration file
        * `index.md`, the index article
        * `articles/example.md`, an example article
        * `main.js`, an empty script file
        * `style.css`, the default stylesheet
* Build
    * Converts the markdown files into html files, and copies over the other files
* Clean
    * Removes the html output directory


## Installing

Currently, you can install the repository 2 ways, either getting a local copy of the repo and building it, or installing it directly from github with cargo.

To build this you need to have the Rust toolchain version 1.65 or higher installed.
You can find the latest version at <https://www.rust-lang.org/tools/install>.

Once I think this is stable enough I'll publish it as a crate to [crates.io](https://crates.io).

### Installing from github

To download and install the latest version from github with Cargo you need to run the following command:

```
cargo install --git https://github.com/DreamweaverOI/dwwb
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
* `index`
    * Default: `index.md`
    * The name of the index article
    * There should be only one article with this name in the project, and it should be at the same directory as `dwwb.yaml`
* `css`
    * Default: `style.css`
    * The path to the stylesheet
    * The given stylesheet will be used on all generated html files
* `script`
    * Default: `main.js`
    * The path to the script file
    * The given file will be included in all generated html files
* `sub_articles_title`
    * Default: `Sub-Articles`
    * The title of the list of sub-articles in the sidebar
* `toc_title`
    * Default: `Table of Contents`
    * The title of the table of contents in the sidebar
* `toc_depth`
    * Default: `3`
    * The depth of how many articles deep the sidebar table of contents shows
* `output_dir`
    * Default: `html`
    * The path to the output directory for the built html version


## Writing articles

Articles are written by using the [pandoc markdown syntax](https://pandoc.org/MANUAL.html#pandocs-markdown).
No markdown extensions are added or removed by default, but you can change them in `dwwb.yaml` like with the other pandoc flags and variables.

The file extension of the articles must be either `.md` or `.markdown`.

An article can have sub-articles if they're in a folder with the same name as the main article.
The index article is an exception.


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

By default all of the URLs in the markdown articles are local to the directory they're located in.
If you want to refer to the root of the wiki, there is a special pandoc filter that's executed for all articles which replaces all occurrences of the string `%ROOT%` with the path to the root directory.

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

