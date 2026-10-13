"""Sphinx configuration for the PlotVale documentation site.

The site is built by Read the Docs (see ``.readthedocs.yaml`` in the repository
root) and can be built locally with::

    python -m sphinx -b html docs docs/_build/html

All documentation pages are Markdown, parsed by MyST.
"""

project = "PlotVale"
author = "PlotVale contributors"
copyright = "2026, PlotVale contributors"

# Keep in sync with CHANGELOG.md and the released version (0.2.0).
release = "0.2.0"
version = "0.2"

extensions = [
    "myst_parser",
    "sphinx_copybutton",
]

# Markdown first: every page in docs/ is .md.
source_suffix = {
    ".md": "markdown",
    ".rst": "restructuredtext",
}
root_doc = "index"

# MyST: the extensions PlotVale's pages rely on.
myst_enable_extensions = [
    "colon_fence",  # ::: note  (alternative to ``` fences)
    "deflist",
    "fieldlist",
    "tasklist",  # - [x] checkboxes
    "attrs_inline",
    "substitution",
]
myst_heading_anchors = 3  # stable #anchors for every heading up to h3

html_theme = "sphinx_rtd_theme"
html_title = "PlotVale documentation"
html_short_title = "PlotVale"
html_logo = "_static/logo.png"
html_favicon = "_static/favicon.png"
html_static_path = ["_static"]
html_css_files = ["custom.css"]

html_theme_options = {
    "navigation_depth": 3,
    "collapse_navigation": False,
    "sticky_navigation": True,
    "prev_next_buttons_location": "bottom",
    "style_external_links": True,
}

# sphinx-copybutton: do not copy the shell prompt or line numbers.
copybutton_exclude = ".linenos, .gp, .go"
copybutton_prompt_text = "$ "

exclude_patterns = ["_build", "requirements.txt", "Thumbs.db", ".DS_Store"]
