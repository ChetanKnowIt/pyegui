Performance
===========

.. |import_ms| replace:: 10.265ms
.. |ratio_label| replace:: 1.37x
.. |ratio_label_range| replace:: 1.36x-1.37x
.. |ratio_text_edit_plain| replace:: 1.31x
.. |ratio_text_edit_plain_range| replace:: 1.28x-1.34x
.. |ratio_text_edit_hint| replace:: 1.5x
.. |ratio_text_edit_hint_range| replace:: 1.45x-1.51x
.. |frame_ms_python_side| replace:: 0.636
.. |last_verified_run| replace:: 37196277892
.. |last_verified_commit| replace:: e766e6b

Every number on this page is an RST substitution defined above, and every one
of them is checked against ``bench/results/combined.json`` by
``tests/check_performance_page.py``. The numbers below come from run
|last_verified_run|, committed at |last_verified_commit|.

That commit is the whole provenance. Committing the snapshot fires no workflow,
so nothing will prompt anyone to re-run the benchmark when the code moves on.
Until someone does, these are the numbers as of that commit.

What the binding costs
----------------------

For a bare ``label``, pyegui costs |ratio_label| what native egui costs at 2000
widgets per frame (|ratio_label_range| across trials). In absolute terms the
binding adds about 0.183us per widget on top of the label itself.

The README's Performance section has the full table across widget counts. It
quotes 1.32-1.33x, from an earlier run (37106337662) on a different hosted
machine. This page quotes the current snapshot. The two numbers are not in
conflict -- they are two measurements of the same code taken on two different
runners, and between-run variation on shared infrastructure is wide enough
(earlier single-trial runs have landed anywhere from 1.30x to 1.55x) that
treating either as the true value would be false precision. Read the shape, not
the third digit.

What ``**kwargs`` costs
------------------------

This is the finding that is not obvious, and it is worth stating precisely
because it is easy to assume the opposite.

Adding options is not free, and the cost is not a constant. At 2000 widgets per
frame:

* a bare ``TextEdit`` costs |ratio_text_edit_plain| what native egui costs
  (|ratio_text_edit_plain_range|), about 0.275us per widget of overhead;
* the same ``TextEdit`` with one extra option, ``hint_text``, costs
  |ratio_text_edit_hint| (|ratio_text_edit_hint_range|), about 0.449us per
  widget.

So one option moves the ratio from 1.31x to 1.50x, and adds roughly 0.17us per
widget -- on top of a widget that already costs about twice what a label does.
A UI with twenty configured widgets is not paying twenty times one widget's
price; the marginal option costs less than the widget, but it is not zero, and
across a frame full of configured widgets it adds up.

The bounds on that claim are worth as much as the claim. This is **one option**
on **one widget**, measured on **one hosted runner**. It is not a per-option
constant, it is not a statement about other widgets, and it is not a projection
for the twenty-plus options that are still to be implemented on ``Slider`` and
``TextEdit``. Treat it as a before-measurement: a baseline to be compared
against once those options exist, not a forecast of what they will cost.

What your own Python costs
---------------------------

The scenarios above measure pyegui against native egui. They say nothing about
the Python you write around the widgets, which is not the binding's doing and
not the binding's bill.

The ``python_side`` scenario measures that separately: 2000 widgets in a frame,
built and torn down in Python rather than in Rust, takes |frame_ms_python_side|
ms in the median, about 0.318us per widget. There is no ``comparison`` block
for it and there should not be one -- it has no Rust twin, so there is no ratio
to report. Any number of the form "pyegui is Nx slower" derived from this
scenario would be fabricated.

What is not claimed
-------------------

* **Launch time.** Not measured. Importing pyegui costs |import_ms| in the
  snapshot's measurement, but that is the import, not the process start, not
  the window, and not the first frame.
* **First frame.** The snapshot records a first-frame figure, but it is a single
  observation per run on a shared runner and is too noisy to publish as a
  figure of merit. Only steady-state per-frame numbers appear here.
* **A blended score.** There is no single number summarising "what pyegui
  costs". It depends on which widgets you use, how you configure them and how
  many you draw. The README's table and the scenarios above are the honest
  forms; a headline ratio would be a number nobody could act on.
* **That ``label`` is the cheapest widget.** It is the simplest one to compare,
  which is why it is the scenario, and it may flatter the binding. Widgets with
  more parameters show a larger ratio, not a smaller one.
* **That these numbers transfer.** They were measured on one hosted runner. The
  shape of the result should carry over; the digits should not be quoted as
  properties of the library.

If a number here disagrees with the snapshot, the gate in
``tests/check_performance_page.py`` says so in the ``examples`` workflow log as
a ``::warning::`` annotation. It warns rather than fails, because between-run
variation makes a hard gate go red on ordinary runs, and a gate that cries wolf
is worse than no gate. To refresh the page, re-run the ``benchmark`` workflow
and update the substitutions at the top of this file to match.