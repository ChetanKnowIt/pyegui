# Configuration file for the Sphinx documentation builder.
#
# For the full list of built-in configuration values, see the documentation:
# https://www.sphinx-doc.org/en/master/usage/configuration.html

# -- Project information -----------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#project-information

project = 'pyegui'
copyright = '2026, GachiLord'
author = 'GachiLord'

# -- General configuration ---------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#general-configuration

extensions = ['sphinx.ext.autodoc', 'myst_parser']

# The README is Markdown and index.rst includes it. myst-parser is what makes
# that work; without it Sphinx treats ../README.md as a literal include and the
# build fails. myst_heading_anchors is needed because the README's links point
# at its own headings, which only exist as anchors if myst generates them.
myst_enable_extensions = ['colon_fence', 'deflist', 'fieldlist', 'linkify']
myst_heading_anchors = 3

templates_path = ['_templates']
exclude_patterns = ['_build', 'Thumbs.db', '.DS_Store']
# Must cover the Markdown: index.rst includes ../README.md, and an include
# outside include_patterns is dropped with a warning.
include_patterns = ['*.rst', '**.rst', '*.md', '**.md']

# -- Options for HTML output -------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#options-for-html-output

html_theme = "sphinx_rtd_theme"
html_static_path = ['_static']

# The README is Markdown and index.rst includes it, so myst-parser handles
# its syntax. Its screenshot paths are `docs/_static/...`, rooted at the
# repository rather than at docs/, because that is where GitHub resolves them
# from the repository root. Sphinx resolves an included file's relative paths
# against the file's own directory, so `docs/_static/x.png` from inside
# ../README.md resolves correctly and needs no suppression.
#
# docs/gallery.rst carries the same screenshots as `.. figure::` directives
# with `_static/` paths, which resolve relative to docs/ directly.
