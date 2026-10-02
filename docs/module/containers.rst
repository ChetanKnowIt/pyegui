Containers
===================

egui draws nothing until the frame is asked for a container, so an app states
which panels it wants and in what order. :func:`central_panel` and
:func:`window` are the two available so far; see ``TODO.md`` for the rest.

Draw side, top and bottom panels first, and ``central_panel`` last, so it takes
whatever space is left over.

.. py:function:: central_panel(ctx, contents, **options)

   Draw a central panel and run ``contents`` inside it.

   ``contents`` runs with the egui frame pushed, so any widget can be called
   from it without a ``Ui`` object.

   .. code:: python

      def update_func(ctx):
          central_panel(ctx, main_contents)

   Options map onto egui's ``CentralPanel`` setters. An unknown option raises
   ``ValueError`` naming every valid option.

.. py:function:: window(ctx, title, contents, **options)

   Draw a floating window and run ``contents`` inside it.

   Returns whether the window is open, matching what egui's
   ``Window::new(...).open(...)`` returns, so it can drive visibility:

   .. code:: python

      show_log = Bool(False)

      def update_func(ctx):
          if window(ctx, "Log", log_contents, open=show_log):
              show_log.value = False

   Pass ``open=`` a :class:`Bool` to let egui own the open state and write it
   back each frame. Options map onto egui's ``Window`` setters, including
   ``default_pos``, ``default_size``, ``collapsible`` and ``resizable``.

Nested layout
-------------

These push an egui ``Ui`` and affect the widgets drawn inside them.

.. autoclass:: pyegui.Layout
.. autoclass:: pyegui.LayoutType
.. autoclass:: pyegui.Scope
.. autoclass:: pyegui.Group