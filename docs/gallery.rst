Gallery
=======

Every image on this page is rendered by ``.github/workflows/screenshot.yml``,
which builds the wheel in CI, runs ``examples/gallery.py`` under Xvfb with
Mesa's software renderer, and captures each page. That job doubles as a render
smoke test: a widget that compiles and imports but draws nothing fails it,
which the export gate cannot catch.

The images live in ``docs/_static/``. To refresh them, push a change to the
gallery, download the ``gallery-screenshots`` artifact, and replace the files.

.. figure:: _static/overlays.png
   :alt: The overlays page: a modal dialog over a resizable area

   ``modal`` returns ``True`` when the backdrop is clicked, and ``resize``
   draws a box with a real resize grip. Both need an existing container:
   ``modal`` before the central panel, ``resize`` inside it.

.. figure:: _static/panels.png
   :alt: The panels page: a left side panel, a top panel, a bottom panel and a central panel

   Independent containers composed explicitly. egui asks for the side, top and
   bottom panels first, and ``central_panel`` last, which then takes whatever
   space is left over.

.. figure:: _static/response.png
   :alt: The Response page: a tooltip button, a checkbox and two sliders

   ``Response``: hover tooltips, focus, context menus and change detection.

.. figure:: _static/colours.png
   :alt: The colour pickers page: five colour swatches and a date picker

   All eight egui 0.31.1 colour spaces, plus the date picker.

.. figure:: _static/text.png
   :alt: The text page: headings, styled text, text fields and a code editor

   Text widgets, text fields and the code editor.

.. figure:: _static/selection.png
   :alt: The selection page: checkbox, radio buttons, selectable labels and a combo box

   Selection widgets and the combo box.

.. figure:: _static/numbers.png
   :alt: The numbers page: sliders, drag values, two angle dials, a progress bar and a spinner

   Sliders, drag values, angle dials, progress and spinner.

.. figure:: _static/buttons.png
   :alt: The buttons page: a button, a small button, an icon button and a group

   Buttons, links and groups.

.. figure:: _static/layout.png
   :alt: The layout page: horizontal buttons, a centred button, a collapsed header, a framed group, a scroll area and indented text

   Layout containers: ``Layout``, ``Group``, ``collapsing``, ``scroll_area_vertical``, ``indent``.

.. figure:: _static/state.png
   :alt: The state page: a normal button, a disabled button and a half-opacity button

   Ui state: ``add_enabled`` and ``set_opacity``.
