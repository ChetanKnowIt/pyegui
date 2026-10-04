Performance
===========

.. |import_ms| replace:: 9.652ms
.. |ratio_label| replace:: 1.33x
.. |ratio_label_range| replace:: 1.14x-1.36x
.. |extra_us_label| replace:: 0.131us
.. |ratio_text_edit_plain| replace:: 1.26x
.. |ratio_text_edit_plain_range| replace:: 1.19x-1.27x
.. |extra_us_text_edit_plain| replace:: 0.182us
.. |ratio_text_edit_hint| replace:: 3.26x
.. |ratio_text_edit_hint_range| replace:: 3.11x-3.32x
.. |extra_us_text_edit_hint| replace:: 1.65us
.. |ratio_slider_many_options| replace:: 2.66x
.. |ratio_slider_many_options_range| replace:: 2.61x-2.96x
.. |extra_us_slider_many_options| replace:: 4.181us
.. |frame_ms_python_side| replace:: 0.5263
.. |per_widget_us_python_side| replace:: 0.263us
.. |last_verified_run| replace:: 37231335459
.. |last_verified_commit| replace:: 271997f

Every figure this page quotes *from the current snapshot* is an RST
substitution defined above, and every one of those is checked against
``bench/results/combined.json`` by ``tests/check_performance_page.py``:
the import time, each scenario's median ratio and across-trial range, each
scenario's per-widget overhead, and ``python_side``'s frame time and
per-widget cost. The snapshot behind them is run |last_verified_run|,
committed at |last_verified_commit|.

The figures that are *not* substitutions are quoted in prose, and are not
machine-checked, because the snapshot does not carry them: the per-widget
costs from the README's earlier run (1.32-1.33x, run 37106337662), the
between-run spread (1.30x-1.55x), and the arithmetic on the option's own
cost, which is the difference between two checked numbers rather than a
number of its own. They are checked by eye against the run they name, and
no gate can do better than that.

That commit is the whole provenance. Committing the snapshot fires no workflow,
so nothing will prompt anyone to re-run the benchmark when the code moves on.
Until someone does, these are the numbers as of that commit.

What the binding costs
----------------------

For a bare ``label``, pyegui costs |ratio_label| what native egui costs at 2000
widgets per frame (|ratio_label_range| across trials). In absolute terms the
binding adds |extra_us_label| per widget on top of the label itself.

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
  (|ratio_text_edit_plain_range|), and adds |extra_us_text_edit_plain| per
  widget of overhead;
* the same ``TextEdit`` with one extra option, ``hint_text``, costs
  |ratio_text_edit_hint| (|ratio_text_edit_hint_range|), and adds
  |extra_us_text_edit_hint| per widget;
* a ``Slider`` configured with every one of the nineteen options it accepts
  costs |ratio_slider_many_options| (|ratio_slider_many_options_range|), and
  adds |extra_us_slider_many_options| per widget.

**Read that last number, because it is the one that matters.** The
``slider_many_options`` scenario exists because one option measured
extrapolated to nineteen is a guess, and the guess would have been wrong by a
wide margin. |extra_us_text_edit_hint| buys a widget with *one* option;
|extra_us_slider_many_options| buys a widget with *nineteen*, which is about
|extra_us_slider_many_options| against |extra_us_label| for the bare widget --
roughly thirty times the per-widget overhead of an unconfigured label, on a
widget that only costs 6.7 us to call at all. And this is the *cheap*
direction of the shape: the slider scenario **passes all nineteen**, and the
binding looks up nineteen names whether or not the caller supplies any of
them, so a caller who configures one option on a slider pays the same toll as
one who configures all of them. That is the cost of the ``**options`` design,
stated as a measurement rather than as a worry.

The ratio moves the same way, and for the same reason. A bare ``TextEdit``
is *cheaper* than a ``label`` in ratio terms and a configured slider is
*three times* one, and neither number is about egui getting slower: it is the
binding's fixed per-option toll standing against a native baseline that
happens to be small. Per-widget binding overhead and the ratio are two
different numbers, and only the first one tells you what the binding charges.

Two honest limits on all of this. It is **nineteen options on one widget**,
measured on **one hosted runner** in **one run**, and it is not a per-option
constant -- the options are not uniform, because reading a `handle_shape`
word and validating an enum costs more than reading a bool. And the spread on
this scenario is the widest on the page (13.3%), which is what a measurement
this size looks like when the runner is shared; read the ratio as "somewhere
around 2.7x", not to two digits.

**What the option itself costs, with no options passed.** The scenario was
built to answer one question: if the plain and configured widgets both sit
near the same ratio, does the ``**kwargs`` path cost anything measurable per
call? It does, and the ratio hides it. A bare ``TextEdit`` is *cheaper* than
a ``label`` in ratio terms -- |ratio_text_edit_plain| against
|ratio_label|, and |extra_us_text_edit_plain| of per-widget overhead against
|extra_us_label|. Both of those calls pass no options at all; the signature
takes ``**kwargs`` whether or not the caller supplies anything. The roughly
0.05us the overhead difference represents is the cost of that path itself, so
the honest reading of "the ratio barely moved" is not "the kwargs path is
free" but "the binding's own overhead grew a little while the native baseline
underneath it grew a lot."

The bounds on that claim are worth as much as the claim. This is **one option**
on **one widget**, measured on **one hosted runner**. It is not a per-option
constant and it is not a statement about other widgets -- and the many-options
figure above is the reason it could not be extended into one. Treat it as a
before-measurement, and the slider as the after-measurement it was built to
be.

What your own Python costs
---------------------------

The scenarios above measure pyegui against native egui. They say nothing about
the Python you write around the widgets, which is not the binding's doing and
not the binding's bill.

The ``python_side`` scenario measures that separately: 2000 widgets in a frame,
built and torn down in Python rather than in Rust, takes |frame_ms_python_side|
ms in the median, |per_widget_us_python_side| per widget. There is no
``comparison`` block for it and there should not be one -- it has no Rust twin,
so there is no ratio to report. Any number of the form "pyegui is Nx slower"
derived from this scenario would be fabricated.

What is not claimed
-------------------

* **Launch time.** Not measured. Importing pyegui costs |import_ms| in the
  snapshot's measurement, but that is the import, not the process start, not
  the window, and not the first frame.
* **First frame.** The snapshot records a first-frame figure, five trials of
  it per run, but on a shared runner it is still too noisy to publish as a
  figure of merit. Only steady-state per-frame numbers appear here.
* **A blended score.** There is no single number summarising "what pyegui
  costs". It depends on which widgets you use, how you configure them and how
  many you draw. The README's table and the scenarios above are the honest
  forms; a headline ratio would be a number nobody could act on.
* **That ``label`` is the cheapest widget.** It is the simplest one to compare,
  which is why it is the scenario, and it may flatter the binding -- but not in
  the way a reader would assume. A heavier widget costs the binding *more* per
  widget than a label does (|extra_us_text_edit_plain| against
  |extra_us_label| for a plain ``TextEdit``), and still shows a *lower* ratio
  at |ratio_text_edit_plain| against |ratio_label|, because the native widget
  it wraps grew by more than the binding overhead did. Per-widget binding
  overhead and the ratio are two different numbers, and only the first one
  tells you what the binding charges.
* **That these numbers transfer.** They were measured on one hosted runner. The
  shape of the result should carry over; the digits should not be quoted as
  properties of the library.

If a number here disagrees with the snapshot, the gate in
``tests/check_performance_page.py`` says so in the ``examples`` workflow log as
a ``::warning::`` annotation. It warns rather than fails, because between-run
variation makes a hard gate go red on ordinary runs, and a gate that cries wolf
is worse than no gate. To refresh the page, re-run the ``benchmark`` workflow
and update the substitutions at the top of this file to match.