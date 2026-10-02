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

# index.rst pulls in ../README.rst, and GitHub renders a README's relative
# image paths against the repository root while docutils resolves them
# relative to docs/. No single spelling satisfies both -- docs/_static/x.png,
# _static/x.png and /docs/_static/x.png were each tried and each is
# unreadable from here. The README therefore uses the GitHub-rooted form and
# the docs gallery page (docs/gallery.rst) carries the Sphinx-correct one.
# Only that one unresolved-path warning is silenced; a genuinely missing
# image elsewhere in the tree still fails the build.
suppress_warnings = ['image.not_readable']
