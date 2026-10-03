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

extensions = ['sphinx.ext.autodoc']

templates_path = ['_templates']
exclude_patterns = ['_build', 'Thumbs.db', '.DS_Store']
include_patterns = ['*.rst', '**.rst']

# -- Options for HTML output -------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#options-for-html-output

html_theme = "sphinx_rtd_theme"
html_static_path = ['_static']

# index.rst pulls in ../README.rst. GitHub renders a .rst README as plain
# text -- no RST directive is processed -- so the README's screenshots are
# Markdown ![](docs/_static/...) links, which GitHub renders and which docutils
# passes through as literal text. Nothing to resolve here, and nothing to
# silence: an unresolved path is still a real warning.
#
# docs/gallery.rst is the page Sphinx actually renders figures on, using the
# `_static/` form that resolves relative to docs/. That is where the images
# appear in the built documentation.
#
# An earlier version of this file claimed the README used RST figure
# directives with GitHub-rooted paths, and suppressed image.not_readable to
# cover the ones Sphinx could not resolve. That was checked and is not what
# happens: `gh api repos/.../readme` returns the README with no img tags at all.
# The suppression is removed rather than left in place, since there is now
# nothing for it to hide.
