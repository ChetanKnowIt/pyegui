#![allow(static_mut_refs)]

use chrono::NaiveDate;
use eframe::egui::{FontData, FontDefinitions, FontFamily};
use eframe::{self, egui};
use egui_extras;
use log::debug;
use pyo3::prelude::*;
use pyo3::{
    exceptions::{PyOSError, PyRuntimeError, PyValueError},
    types::{PyAny, PyBool, PyDict, PyInt, PyString},
};
use std::sync::{Arc, Mutex};
use std::{fs, ptr};

// state

static mut UI: *mut Vec<*mut egui::Ui> = ptr::null_mut();
static mut APP_MUTEX: Mutex<()> = Mutex::new(());

// messages

static APP_MUTEX_ERR: &'static str = "run_simple_native has been called on a separate thread";
static UI_PTR_NULL_ERR: &'static str = "UI ptr is null. This is likely to be a problem with pyegui";
static UI_STACK_ERR: &'static str = "UI stack is empty. This is likely to be a problem with pyegui";
static UI_CALL_OUTSIDE_UPDATE_FUNC: &'static str = "UI functions should be called only within update_fun and on the same thread. update_fun should only be called by run_simple_native";

// classes

/// Context object that controls global aspects of your app
///
/// Usage::
///
///     def update_func(ctx):
///         ctx.set_light_theme
///         heading("Using light theme even if system's is dark")
///
///     run_native("My app", update_func)
#[pyclass]
struct Context(egui::Context);

#[pymethods]
impl Context {
    /// True when theme is dark
    #[getter]
    fn is_light_theme(&self) -> bool {
        self.0.theme() == egui::Theme::Light
    }

    /// True when theme is dark
    #[getter]
    fn is_dark_theme(&self) -> bool {
        self.0.theme() == egui::Theme::Dark
    }

    /// Sets light theme. Default is system's
    fn set_light_theme(&self) {
        self.0.set_theme(egui::ThemePreference::Light);
    }

    /// Sets dark theme. Default is system's
    fn set_dark_theme(&self) {
        self.0.set_theme(egui::ThemePreference::Dark);
    }

    /// Sets system's theme if it has been changed.
    fn set_system_theme(&self) {
        self.0.set_theme(egui::ThemePreference::System);
    }

    /// Tell egui which fonts to use.
    ///
    /// The default egui fonts only support latin and cyrillic alphabets, but you can call this to install additional fonts that support e.g. Japanese characters.
    ///
    /// The new fonts will become active at the start of the next pass. This will overwrite the existing fonts.
    ///  
    /// Example::
    ///
    ///   def update_func(ctx):
    ///     ctx.set_font("NotoSansJP-VariableFont_wght.ttf")
    ///     heading("天気の子")
    fn set_font(&self, source: String) -> PyResult<()> {
        let buf = fs::read(&source).map_err(|e| {
            eprintln!("Cannot open '{}': {}", source, e.to_string());
            e
        })?;

        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert(
            source.clone(),
            Arc::new(
                // .ttf and .otf supported
                FontData::from_owned(buf),
            ),
        );
        fonts
            .families
            .get_mut(&FontFamily::Monospace)
            .unwrap()
            .push(source.clone());

        fonts
            .families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, source);

        self.0.set_fonts(fonts);

        Ok(())
    }

    /// Open an URL in a browser.
    fn open_url(&self, url: &str) {
        self.0.open_url(egui::OpenUrl::new_tab(url));
    }

    /// Copy the given text to the system clipboard.
    fn copy_text(&self, text: String) {
        self.0.copy_text(text);
    }

    /// Close the window, which shuts the app down.
    ///
    /// Useful for a "quit" button, and for screenshots, where the app has to
    /// exit on its own once every frame has been captured.
    ///
    /// Example::
    ///
    ///   def update_func(ctx):
    ///     if button_clicked("quit"):
    ///         ctx.close()
    fn close(&self) {
        self.0
            .send_viewport_cmd(egui::viewport::ViewportCommand::Close);
    }

    /// Ask egui to redraw on the next frame.
    ///
    /// egui only repaints when something changes, so an app that draws
    /// something once and then sits still will show a stale window. Call this
    /// when you have changed something egui cannot see.
    ///
    /// Example::
    ///
    ///   def update_func(ctx):
    ///     label(f"frame {counter}")
    ///     counter.value += 1
    ///     ctx.request_repaint()
    fn request_repaint(&self) {
        self.0.request_repaint();
    }
}

/// Str stores string value that can be referenced
///
/// Usage::
///
///     data = Str("")
///     
///     def update_func():
///         heading(f"Value of the data is {data.value}")
///         if button_clicked("Add :)"):
///             data.value += ":) "
#[pyclass]
struct Str {
    #[pyo3(get, set)]
    value: String,
}

#[pymethods]
impl Str {
    #[new]
    fn new(value: String) -> Self {
        Str { value }
    }
}

/// Bool stores a boolean value that can be referenced
///
/// Usage::
///
///     data = Bool(False)
///     
///     def update_func():
///         heading(f"Value of the data is {data.value}")
///         # button will be shown only if the checkbox is checked
///         if data.value and button_clicked("set to False"):
///             # hiding the button
///             data.value = False
///         checkbox(data, "Check me")
#[pyclass]
struct Bool {
    #[pyo3(get, set)]
    value: bool,
}

#[pymethods]
impl Bool {
    #[new]
    fn new(value: bool) -> Self {
        Bool { value }
    }
}

/// Int stores integer value that can be referenced
///
/// Usage::
///
///     data = Int(69)
///     
///     def update_func():
///         heading(f"Value of the data is {data.value}")
///         if button_clicked("Increment"):
///             data.value += 1
#[pyclass]
struct Int {
    #[pyo3(get, set)]
    value: i32,
}

#[pymethods]
impl Int {
    #[new]
    fn new(value: i32) -> Self {
        Int { value }
    }
}

/// Float stores float value that can be referenced
///
/// Usage::
///
///     data = Float(69.0)
///     
///     def update_func():
///         heading(f"Value of the data is {data.value}")
///         if button_clicked("Increment"):
///             # what can go wrong?
///             data.value += 0.1
#[pyclass]
struct Float {
    #[pyo3(get, set)]
    value: f32,
}

#[pymethods]
impl Float {
    #[new]
    fn new(value: f32) -> Self {
        Float { value }
    }
}

/// Rgb color picker
///
/// Usage::
///
///     color_rgb = RGB(69, 69, 69)
///     color_edit_button_rgb(color_rgb)
#[pyclass]
struct RGB {
    #[pyo3(get, set)]
    r: f32,
    #[pyo3(get, set)]
    g: f32,
    #[pyo3(get, set)]
    b: f32,
}

#[pymethods]
impl RGB {
    #[new]
    fn new(r: f32, g: f32, b: f32) -> Self {
        RGB { r, g, b }
    }
}

/// Date picker
///
/// Usage::
///
///     date = Date(datetime.datetime.now())
///     date_picker_button(date)
#[pyclass]
struct Date {
    #[pyo3(get, set)]
    value: NaiveDate,
}

#[pymethods]
impl Date {
    #[new]
    fn new(value: NaiveDate) -> Self {
        Date { value }
    }
}

/// Values of this enum are used in Layout
#[pyclass]
#[derive(Clone)]
enum LayoutType {
    Horizontal,
    HorizontalCentered,
    HorizontalTop,
    HorizontalWrapped,
    Vertical,
    VerticalCentered,
    VerticalCenteredJustified,
    CenteredAndJustified,
}

/// Layout class can be used to specify layout of widgets that go after with statement.
///
/// Usage::
///
///     with Layout(LayoutType.VerticalCentered):
///        heading("This widget will be horizontally centered")
///
/// Layout types:
///
/// **Horizontal**
///
/// Start a ui with horizontal layout.
///
/// Elements will be centered on the Y axis, i.e. adjusted up and down to lie in the center of the horizontal layout. Centering is almost always what you want if you are planning to mix widgets or use different types of text.
///
/// If you don’t want the contents to be centered, use HorizontalTop instead.
///
/// **HorizontalCentered**
///
/// Like Horizontal, but allocates the full vertical height and then centers elements vertically.
///
/// **HorizontalTop**
///
/// Like Horizontal, but aligns content with top.
///
/// **HorizontalWrapped**
///
/// Start a ui with horizontal layout that wraps to a new row when it reaches the right edge.
///
/// Elements will be centered on the Y axis, i.e. adjusted up and down to lie in the center of the horizontal layout. Centering is almost always what you want if you are planning to mix widgets or use different types of text.
///
/// **Vertical**
///
/// Start a ui with vertical layout. Widgets will be left-justified.
///
/// **VerticalCentered**
///
/// Start a ui with vertical layout. Widgets will be horizontally centered.
///
/// **VerticalCenteredJustified**
///
/// Start a ui with vertical layout. Widgets will be horizontally centered and justified (fill full width).
///
/// **CenteredAndJustified**
///
/// This will make the next added widget centered and justified in the available space.
///
/// Only one child widget is allowed!
#[pyclass]
struct Layout {
    ui: egui::Ui,
}

#[pymethods]
impl Layout {
    #[new]
    fn __new__(layout_type: LayoutType) -> PyResult<Self> {
        unsafe {
            let parent_ui = current_ui(&UI)?;

            // build layout based on scope type
            let layout = match layout_type {
                LayoutType::HorizontalCentered => egui::Layout::left_to_right(egui::Align::Center)
                    .with_cross_align(egui::Align::Center),
                LayoutType::Horizontal => egui::Layout::left_to_right(egui::Align::Center)
                    .with_main_wrap(false)
                    .with_cross_align(egui::Align::Min),
                LayoutType::HorizontalTop => egui::Layout::left_to_right(egui::Align::Center)
                    .with_cross_align(egui::Align::Min),
                LayoutType::HorizontalWrapped => egui::Layout::left_to_right(egui::Align::Center)
                    .with_main_wrap(false)
                    .with_cross_align(egui::Align::Min),
                LayoutType::Vertical => egui::Layout::top_down(egui::Align::Min),
                LayoutType::VerticalCentered => egui::Layout::top_down(egui::Align::Center),
                LayoutType::VerticalCenteredJustified => {
                    egui::Layout::top_down(egui::Align::Center).with_cross_justify(true)
                }
                LayoutType::CenteredAndJustified => {
                    egui::Layout::centered_and_justified(egui::Direction::TopDown)
                }
            };

            let ui = parent_ui.new_child(egui::UiBuilder::new().layout(layout));

            Ok(Self { ui })
        }
    }

    #[staticmethod]
    fn __init__(scope_type: LayoutType) -> PyResult<Self> {
        Self::__new__(scope_type)
    }

    fn __enter__(&mut self) -> PyResult<()> {
        unsafe {
            let ui_stack = ui_stack(&UI)?;

            ui_stack.push(&raw mut self.ui);
            Ok(())
        }
    }

    fn __exit__(
        &self,
        _exception_type: Bound<'_, PyAny>,
        _exception_value: Bound<'_, PyAny>,
        _exception_traceback: Bound<'_, PyAny>,
    ) -> PyResult<()> {
        unsafe {
            let ui_stack = ui_stack(&UI)?;

            let child_ui = ui_stack
                .pop()
                .ok_or(PyRuntimeError::new_err(UI_CALL_OUTSIDE_UPDATE_FUNC))?
                .as_mut()
                .ok_or(PyRuntimeError::new_err(UI_PTR_NULL_ERR))?;

            let parent_ui = ui_stack
                .last()
                .ok_or(PyRuntimeError::new_err(UI_CALL_OUTSIDE_UPDATE_FUNC))?
                .as_mut()
                .ok_or(PyRuntimeError::new_err(UI_PTR_NULL_ERR))?;

            parent_ui.advance_cursor_after_rect(child_ui.min_rect());

            Ok(())
        }
    }
}

/// Create a scope for the contents.
/// You can use this to temporarily change the Style of a sub-region
///
/// Usage::
///
///     with Scope():
///        set_opacity(0.5)
///        heading("hi")
///        heading("there")
///
///     heading("normal opacity")
#[pyclass]
struct Scope {
    ui: egui::Ui,
}

#[pymethods]
impl Scope {
    #[new]
    fn __new__() -> PyResult<Self> {
        unsafe {
            let parent_ui = current_ui(&UI)?;
            let ui = parent_ui.new_child(egui::UiBuilder::new());
            Ok(Self { ui })
        }
    }

    #[staticmethod]
    fn __init__() -> PyResult<Self> {
        Self::__new__()
    }

    fn __enter__(&mut self) -> PyResult<()> {
        unsafe {
            let ui_stack = ui_stack(&UI)?;

            ui_stack.push(&raw mut self.ui);
            Ok(())
        }
    }

    fn __exit__(
        &self,
        _exception_type: Bound<'_, PyAny>,
        _exception_value: Bound<'_, PyAny>,
        _exception_traceback: Bound<'_, PyAny>,
    ) -> PyResult<()> {
        unsafe {
            let ui_stack = ui_stack(&UI)?;

            let child_ui = ui_stack
                .pop()
                .ok_or(PyRuntimeError::new_err(UI_CALL_OUTSIDE_UPDATE_FUNC))?
                .as_mut()
                .ok_or(PyRuntimeError::new_err(UI_PTR_NULL_ERR))?;

            let parent_ui = ui_stack
                .last()
                .ok_or(PyRuntimeError::new_err(UI_CALL_OUTSIDE_UPDATE_FUNC))?
                .as_mut()
                .ok_or(PyRuntimeError::new_err(UI_PTR_NULL_ERR))?;

            parent_ui.advance_cursor_after_rect(child_ui.min_rect());

            Ok(())
        }
    }
}

/// Visually groups the contents together.
///
/// Usage::
///
///     with Group():
///        heading("hi")
///        heading("there")
#[pyclass]
struct Group {
    prepared: egui::frame::Prepared,
}

#[pymethods]
impl Group {
    #[new]
    fn __new__() -> PyResult<Self> {
        unsafe {
            let parent_ui = current_ui(&UI)?;

            let frame = egui::Frame::group(&parent_ui.style());
            let prepared = frame.begin(parent_ui);

            Ok(Self { prepared })
        }
    }

    #[staticmethod]
    fn __init__() -> PyResult<Self> {
        Self::__new__()
    }

    fn __enter__(&mut self) -> PyResult<()> {
        unsafe {
            let ui_stack = ui_stack(&UI)?;

            ui_stack.push(&raw mut self.prepared.content_ui);
            Ok(())
        }
    }

    fn __exit__(
        &self,
        _exception_type: Bound<'_, PyAny>,
        _exception_value: Bound<'_, PyAny>,
        _exception_traceback: Bound<'_, PyAny>,
    ) -> PyResult<()> {
        unsafe {
            let ui_stack = ui_stack(&UI)?;

            let _ = ui_stack
                .pop()
                .ok_or(PyRuntimeError::new_err(UI_CALL_OUTSIDE_UPDATE_FUNC))?;

            let mut parent_ui = ui_stack
                .last()
                .ok_or(PyRuntimeError::new_err(UI_CALL_OUTSIDE_UPDATE_FUNC))?
                .as_mut()
                .ok_or(PyRuntimeError::new_err(UI_PTR_NULL_ERR))?;

            self.prepared.paint(&mut parent_ui);
            self.prepared.allocate_space(&mut parent_ui);
            // parent_ui.advance_cursor_after_rect(child_ui.min_rect());

            Ok(())
        }
    }
}

/// Linear RGBA color with alpha, each channel in the [0, 1] range.
///
/// This is egui's Rgba. It is the color space egui composites in, so it is
/// the right choice for anything that has to match what is on screen. The
/// alpha channel is not premultiplied unless the function name says so.
///
/// Usage::
///
///     color = RGBA(1.0, 0.0, 0.0, 1.0)
///     color_edit_button_rgba_unmultiplied(color)
#[pyclass]
struct RGBA {
    #[pyo3(get, set)]
    r: f32,
    #[pyo3(get, set)]
    g: f32,
    #[pyo3(get, set)]
    b: f32,
    #[pyo3(get, set)]
    a: f32,
}

#[pymethods]
impl RGBA {
    #[new]
    fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        RGBA { r, g, b, a }
    }

    /// The same color with alpha forced to 1.0.
    fn opaque(&self) -> RGBA {
        RGBA {
            r: self.r,
            g: self.g,
            b: self.b,
            a: 1.0,
        }
    }

    fn __repr__(&self) -> String {
        format!("RGBA({}, {}, {}, {})", self.r, self.g, self.b, self.a)
    }
}

/// Hue, saturation, value and alpha, each in the [0, 1] range.
///
/// This is egui's Hsva. HSV is how people describe colors ("a saturated
/// blue"), so it is the right choice for a color wheel or any UI where the
/// user picks a color by name rather than by number.
///
/// A negative alpha means an additive color, in which case alpha is ignored.
///
/// Usage::
///
///     color = HSVA(0.0, 1.0, 1.0, 1.0)  # pure red
///     color_edit_button_hsva(color)
#[pyclass]
struct HSVA {
    #[pyo3(get, set)]
    h: f32,
    #[pyo3(get, set)]
    s: f32,
    #[pyo3(get, set)]
    v: f32,
    #[pyo3(get, set)]
    a: f32,
}

#[pymethods]
impl HSVA {
    #[new]
    fn new(h: f32, s: f32, v: f32, a: f32) -> Self {
        HSVA { h, s, v, a }
    }

    /// The same color with alpha forced to 1.0.
    fn opaque(&self) -> HSVA {
        HSVA {
            h: self.h,
            s: self.s,
            v: self.v,
            a: 1.0,
        }
    }

    fn __repr__(&self) -> String {
        format!("HSVA({}, {}, {}, {})", self.h, self.s, self.v, self.a)
    }
}

/// sRGB color with alpha, each channel in the [0, 255] range.
///
/// This is egui's Color32, the format egui stores textures in. Use it when
/// you need the exact 8-bit value egui will paint, for example when reading
/// a pixel back out of an image.
///
/// Usage::
///
///     color = Color32(255, 0, 0, 255)
///     color_edit_button_srgba(color)
#[pyclass]
struct Color32 {
    #[pyo3(get, set)]
    r: u8,
    #[pyo3(get, set)]
    g: u8,
    #[pyo3(get, set)]
    b: u8,
    #[pyo3(get, set)]
    a: u8,
}

#[pymethods]
impl Color32 {
    #[new]
    fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color32 { r, g, b, a }
    }

    /// The same color with alpha forced to 255.
    fn opaque(&self) -> Color32 {
        Color32 {
            r: self.r,
            g: self.g,
            b: self.b,
            a: 255,
        }
    }

    /// egui's own debug format, e.g. "#FF0000FF".
    fn __repr__(&self) -> String {
        format!("#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a)
    }
}

/// sRGB color without alpha, each channel in the [0, 255] range.
///
/// Usage::
///
///     color = SRGB(255, 128, 0)
///     color_edit_button_srgb(color)
#[pyclass]
struct SRGB {
    #[pyo3(get, set)]
    r: u8,
    #[pyo3(get, set)]
    g: u8,
    #[pyo3(get, set)]
    b: u8,
}

#[pymethods]
impl SRGB {
    #[new]
    fn new(r: u8, g: u8, b: u8) -> Self {
        SRGB { r, g, b }
    }

    fn __repr__(&self) -> String {
        format!("SRGB({}, {}, {})", self.r, self.g, self.b)
    }
}

/// Mouse button that triggered an interaction.
///
/// Passed to Response.clicked_by and friends. Values are Primary (usually the
/// left button), Secondary (usually the right button), Middle (the scroll
/// wheel), Extra1 and Extra2 (the side buttons on some mice).
#[pyclass]
#[derive(Clone, Copy)]
enum PointerButton {
    Primary,
    Secondary,
    Middle,
    Extra1,
    Extra2,
}

impl From<PointerButton> for egui::PointerButton {
    fn from(button: PointerButton) -> Self {
        match button {
            PointerButton::Primary => egui::PointerButton::Primary,
            PointerButton::Secondary => egui::PointerButton::Secondary,
            PointerButton::Middle => egui::PointerButton::Middle,
            PointerButton::Extra1 => egui::PointerButton::Extra1,
            PointerButton::Extra2 => egui::PointerButton::Extra2,
        }
    }
}

/// The result of showing a widget.
///
/// Every widget in egui returns one of these; pyegui returns it from the
/// `*_response` functions so the full interaction state is reachable. The
/// existing boolean helpers (button_clicked and friends) are unchanged.
///
/// A Response describes one frame. Read it inside the update function, in the
/// same frame the widget was shown.
///
/// Usage::
///
///     def update_func(ctx):
///         response = button_response("click me")
///         if response.clicked:
///             print("clicked")
///         if response.hovered:
///             response.on_hover_text("i am a tooltip")
#[pyclass]
struct Response {
    inner: egui::Response,
}

#[pymethods]
impl Response {
    /// True if the widget was clicked this frame.
    #[getter]
    fn clicked(&self) -> bool {
        self.inner.clicked()
    }

    /// True if the widget was clicked with the given mouse button.
    fn clicked_by(&self, button: PointerButton) -> bool {
        self.inner.clicked_by(button.into())
    }

    /// True if the right mouse button was used.
    #[getter]
    fn secondary_clicked(&self) -> bool {
        self.inner.secondary_clicked()
    }

    /// True if the middle (scroll wheel) button was used.
    #[getter]
    fn middle_clicked(&self) -> bool {
        self.inner.middle_clicked()
    }

    /// True if the widget was double clicked.
    #[getter]
    fn double_clicked(&self) -> bool {
        self.inner.double_clicked()
    }

    /// True if the widget was double clicked with the given mouse button.
    fn double_clicked_by(&self, button: PointerButton) -> bool {
        self.inner.double_clicked_by(button.into())
    }

    /// True if the widget was triple clicked.
    #[getter]
    fn triple_clicked(&self) -> bool {
        self.inner.triple_clicked()
    }

    /// True if the widget was triple clicked with the given mouse button.
    fn triple_clicked_by(&self, button: PointerButton) -> bool {
        self.inner.triple_clicked_by(button.into())
    }

    /// True if a click happened elsewhere, i.e. this widget lost the click.
    #[getter]
    fn clicked_elsewhere(&self) -> bool {
        self.inner.clicked_elsewhere()
    }

    /// True if the widget was touched for longer than a click.
    #[getter]
    fn long_touched(&self) -> bool {
        self.inner.long_touched()
    }

    /// False if the widget is disabled, in which case it senses nothing.
    #[getter]
    fn enabled(&self) -> bool {
        self.inner.enabled()
    }

    /// True if the pointer is over the widget.
    #[getter]
    fn hovered(&self) -> bool {
        self.inner.hovered()
    }

    /// True if the widget rect contains the pointer, even where clipped.
    #[getter]
    fn contains_pointer(&self) -> bool {
        self.inner.contains_pointer()
    }

    /// True if the widget is highlighted, e.g. by being hovered.
    #[getter]
    fn highlighted(&self) -> bool {
        self.inner.highlighted()
    }

    /// True if the widget or its label has keyboard focus.
    #[getter]
    fn has_focus(&self) -> bool {
        self.inner.has_focus()
    }

    /// True if the widget gained focus this frame.
    #[getter]
    fn gained_focus(&self) -> bool {
        self.inner.gained_focus()
    }

    /// True if the widget lost focus this frame.
    #[getter]
    fn lost_focus(&self) -> bool {
        self.inner.lost_focus()
    }

    /// Give the widget keyboard focus.
    fn request_focus(&self) {
        self.inner.request_focus();
    }

    /// Remove keyboard focus from the widget.
    fn surrender_focus(&self) {
        self.inner.surrender_focus();
    }

    /// True if a drag gesture started on the widget this frame.
    #[getter]
    fn drag_started(&self) -> bool {
        self.inner.drag_started()
    }

    /// True if a drag started with the given mouse button.
    fn drag_started_by(&self, button: PointerButton) -> bool {
        self.inner.drag_started_by(button.into())
    }

    /// True if the widget is being dragged.
    #[getter]
    fn dragged(&self) -> bool {
        self.inner.dragged()
    }

    /// True if the widget is being dragged by the given mouse button.
    fn dragged_by(&self, button: PointerButton) -> bool {
        self.inner.dragged_by(button.into())
    }

    /// True if the drag ended on the widget this frame.
    #[getter]
    fn drag_stopped(&self) -> bool {
        self.inner.drag_stopped()
    }

    /// True if a drag with the given mouse button stopped on the widget.
    fn drag_stopped_by(&self, button: PointerButton) -> bool {
        self.inner.drag_stopped_by(button.into())
    }

    /// How far the widget was dragged this frame, as (dx, dy).
    #[getter]
    fn drag_delta(&self) -> (f32, f32) {
        let delta = self.inner.drag_delta();
        (delta.x, delta.y)
    }

    /// How much the pointer moved this frame, as (dx, dy).
    #[getter]
    fn drag_motion(&self) -> (f32, f32) {
        let motion = self.inner.drag_motion();
        (motion.x, motion.y)
    }

    /// Where the pointer was when the widget was clicked or dragged, as
    /// (x, y), or None if there was no interaction.
    #[getter]
    fn interact_pointer_pos(&self) -> Option<(f32, f32)> {
        self.inner.interact_pointer_pos().map(|p| (p.x, p.y))
    }

    /// Where the pointer is, as (x, y), or None if not over the widget.
    #[getter]
    fn hover_pos(&self) -> Option<(f32, f32)> {
        self.inner.hover_pos().map(|p| (p.x, p.y))
    }

    /// True if the mouse button is held down over the widget.
    #[getter]
    fn is_pointer_button_down_on(&self) -> bool {
        self.inner.is_pointer_button_down_on()
    }

    /// True if the widget's value changed this frame. For text fields and
    /// sliders, this is how you know the user edited something.
    #[getter]
    fn changed(&self) -> bool {
        self.inner.changed()
    }

    /// Force changed() to be true next frame. Use when you modify the value
    /// yourself and want dependent widgets to react.
    fn mark_changed(&mut self) {
        self.inner.mark_changed();
    }

    /// True if a tooltip is currently open for this widget.
    #[getter]
    fn is_tooltip_open(&self) -> bool {
        self.inner.is_tooltip_open()
    }

    /// True if a context menu was opened on the widget this frame.
    #[getter]
    fn context_menu_opened(&self) -> bool {
        self.inner.context_menu_opened()
    }

    /// Show a tooltip while the pointer is over the widget. Use text that
    /// explains the widget, not its current value.
    fn on_hover_text(&mut self, text: &str) {
        self.inner = self.inner.clone().on_hover_text(text);
    }

    /// Show a tooltip while the pointer is over a disabled widget.
    fn on_disabled_hover_text(&mut self, text: &str) {
        self.inner = self.inner.clone().on_disabled_hover_text(text);
    }

    /// Show a tooltip, but only if one is already open for this widget.
    /// Cheaper than on_hover_text when the tooltip content is expensive to
    /// build.
    fn show_tooltip_text(&self, text: &str) {
        self.inner.show_tooltip_text(text);
    }

    /// Show a tooltip whose contents are built by a function, called only
    /// while the pointer is over the widget.
    ///
    /// Usage::
    ///
    ///     def build_tooltip():
    ///         heading("details")
    ///         label("built only while hovered")
    ///
    ///     button_response("hover me").on_hover_ui(build_tooltip)
    fn on_hover_ui(&mut self, update_func: Bound<'_, PyAny>) {
        self.inner = self
            .inner
            .clone()
            .on_hover_ui(|ui| run_nested_update_func_lossy(ui, update_func.clone()));
    }

    /// Show a tooltip with pre-built contents, like on_hover_ui but without
    /// the lazy building.
    fn show_tooltip_ui(&self, update_func: Bound<'_, PyAny>) {
        self.inner
            .show_tooltip_ui(|ui| run_nested_update_func_lossy(ui, update_func));
    }

    /// Open a context menu when the widget is right clicked. The function is
    /// called with the menu contents, and only when the menu opens.
    ///
    /// Returns True if the menu is open after the call.
    ///
    /// Usage::
    ///
    ///     def menu_contents():
    ///         if button_clicked("cut"):
    ///             print("cut")
    ///
    ///     button_response("right click me").context_menu(menu_contents)
    fn context_menu(&self, update_func: Bound<'_, PyAny>) -> bool {
        self.inner
            .context_menu(|ui| run_nested_update_func_lossy(ui, update_func))
            .is_some()
    }
}

// Start function

struct PyeguiApp<'py> {
    update_func: Bound<'py, PyAny>,
}

impl eframe::App for PyeguiApp<'_> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let ctx_r = Context(ctx.clone());

        // Python owns frame composition, exactly as an egui app does. egui
        // requires CentralPanel to be added after every other top-level
        // panel, so pyegui cannot open one on Python's behalf without making
        // that order unenforceable. `update_func` therefore receives only a
        // Context and must call `central_panel(ctx, contents)` itself, last.
        //
        // `ctx` is eframe's own Context, already inside the frame that
        // `eframe::App::update` is called from, so it is passed straight
        // through. Do not wrap this in `egui::Context::run`: that is the
        // frame driver eframe itself calls to reach this method, not a
        // wrapper for use inside it.
        debug!("Execute update_func");

        Python::with_gil(|py| {
            if let Err(err) = self.update_func.call1((ctx_r,)) {
                err.display(py);
            }
        });

        debug!("Executed update_func");
    }
}

/// Creates a window and runs update_func.
/// This is an entrypoint for your GUI application.
///
/// Args:
///     app_name (str): name displayed at the header bar
///
///     update_func (Callable[[Context], None]): your function that draws UI
///
///     inner_height (float): the desired height of the window
///
///     inner_width (float): the desired width of the window
///
///     min_inner_height (float): min height of the window
///
///     min_inner_width (float): min width of the window
///
///     max_inner_height (float): max height of the window
///
///     max_inner_width (float): max width of the window
///
///     fullscreen (bool): whether to open app in fullscreen
///
///     maximized (bool): whether to open app maximized
///
///     resizable (bool): whether our app is resizable
///
///     transparent (bool): whether our app is transparent
///
///     icon_path (str): path to icon in rgba format
///
/// Examples::
///
///     name = Str("")
///     
///     def update_func(ctx):
///         heading(f"Hello, {name.value}!")
///         text_edit_singleline(name)
///         
///         if button_clicked("click me"):
///             print("clicked")
///     
///     run_native("My app", update_func)
///
#[pyfunction]
#[pyo3(signature = (app_name, update_func, **kwargs))]
unsafe fn run_native(
    app_name: &str,
    update_func: Bound<'_, PyAny>,
    kwargs: Option<&Bound<'_, PyDict>>,
) -> PyResult<()> {
    debug!("Trying to get the app lock");
    // ensure thread safety
    let _lock = APP_MUTEX
        .try_lock()
        .map_err(|_| PyRuntimeError::new_err(APP_MUTEX_ERR))?;
    // init UI stack
    debug!("Initialzing UI stack");
    let mut ui_stack = Vec::with_capacity(32);
    UI = &raw mut *&mut ui_stack;
    // parse kwargs
    let mut viewport = egui::viewport::ViewportBuilder::default();

    if let Some(kwargs) = kwargs {
        if let (Some(height), Some(width)) = (
            kwargs.get_item("inner_height")?,
            kwargs.get_item("inner_width")?,
        ) {
            viewport = viewport.with_inner_size([
                width.downcast::<PyInt>()?.extract()?,
                height.downcast::<PyInt>()?.extract()?,
            ]);
        }

        if let (Some(height), Some(width)) = (
            kwargs.get_item("min_inner_height")?,
            kwargs.get_item("min_inner_width")?,
        ) {
            viewport = viewport.with_min_inner_size([
                width.downcast::<PyInt>()?.extract()?,
                height.downcast::<PyInt>()?.extract()?,
            ]);
        }

        if let (Some(height), Some(width)) = (
            kwargs.get_item("max_inner_height")?,
            kwargs.get_item("max_inner_width")?,
        ) {
            viewport = viewport.with_max_inner_size([
                width.downcast::<PyInt>()?.extract()?,
                height.downcast::<PyInt>()?.extract()?,
            ]);
        }

        if let Some(fullscreen) = kwargs.get_item("fullscreen")? {
            viewport = viewport.with_fullscreen(fullscreen.downcast::<PyBool>()?.extract()?);
        }

        if let Some(maximized) = kwargs.get_item("maximized")? {
            viewport = viewport.with_maximized(maximized.downcast::<PyBool>()?.extract()?);
        }

        if let Some(resizable) = kwargs.get_item("resizable")? {
            viewport = viewport.with_resizable(resizable.downcast::<PyBool>()?.extract()?);
        }

        if let Some(transparent) = kwargs.get_item("transparent")? {
            viewport = viewport.with_transparent(transparent.downcast::<PyBool>()?.extract()?);
        }

        if let Some(icon_path) = kwargs.get_item("icon_path")? {
            let path = icon_path.downcast::<PyString>()?.extract::<String>()?;
            let buf = fs::read(path)?;

            let icon_data = eframe::icon_data::from_png_bytes(&buf)
                .map_err(|e| PyOSError::new_err(format!("Failed to decode png file: {}", e)))?;
            viewport = viewport.with_icon(icon_data);
        }
    }

    let options = eframe::NativeOptions {
        viewport,
        ..eframe::NativeOptions::default()
    };
    debug!("Creating a window");
    // create a window
    let result = eframe::run_native(
        app_name,
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::new(PyeguiApp {
                update_func: update_func,
            }))
        }),
    );

    match result {
        Ok(_) => Ok(()),
        Err(err) => Err(PyRuntimeError::new_err(format!(
            "Cannot create a window: {}",
            err.to_string()
        ))),
    }
}

// helpers

unsafe fn ui_stack(ui: &*mut Vec<*mut egui::Ui>) -> PyResult<&mut Vec<*mut egui::Ui>> {
    ui.as_mut()
        .ok_or(PyRuntimeError::new_err(UI_CALL_OUTSIDE_UPDATE_FUNC))
}

unsafe fn last_ui(ui_stack: &mut Vec<*mut egui::Ui>) -> PyResult<&mut egui::Ui> {
    let last_ui = ui_stack
        .last_mut()
        .ok_or(PyRuntimeError::new_err(UI_STACK_ERR))?;

    last_ui
        .as_mut()
        .ok_or(PyRuntimeError::new_err(UI_PTR_NULL_ERR))
}

unsafe fn current_ui(ui: &*mut Vec<*mut egui::Ui>) -> PyResult<&mut egui::Ui> {
    last_ui(ui_stack(ui)?)
}

/// Like `run_nested_update_func`, but for the callbacks egui types as
/// `impl FnOnce(&mut Ui)` rather than `impl FnOnce(&mut Ui) -> R`. Those
/// closures must return `()`, so the error is dropped here instead of being
/// propagated. The Python-side exception has already been displayed by
/// `run_nested_update_func` at that point.
fn run_nested_update_func_lossy(ui: &mut egui::Ui, update_fun: Bound<'_, PyAny>) {
    unsafe {
        let _ = run_nested_update_func(ui, update_fun);
    }
}

unsafe fn run_nested_update_func(ui: &mut egui::Ui, update_fun: Bound<'_, PyAny>) -> PyResult<()> {
    let ui_stack = ui_stack(&UI).unwrap_unchecked();

    ui_stack.push(&raw mut *ui);

    if let Err(err) = update_fun.call0() {
        Python::with_gil(|py| {
            err.display(py);
        });
    }

    match ui_stack.pop() {
        Some(_) => Ok(()),
        None => Err(PyRuntimeError::new_err(UI_STACK_ERR)),
    }
}

// UI functions

/// Show large text
///
/// Example::
///
///     heading("hello")
#[pyfunction]
unsafe fn heading(text: &str) -> PyResult<()> {
    heading_response(text)?;
    Ok(())
}

/// Returns the Response of the text. A text widget does not react to clicks,
/// but its Response carries the rect and can carry a tooltip.
///
/// Example::
///
///     heading("hello")
///
///     # with a tooltip:
///     heading_response("hover me").on_hover_text("an explanation")
#[pyfunction]
unsafe fn heading_response(text: &str) -> PyResult<Response> {
    let ui = current_ui(&UI)?;

    Ok(Response {
        inner: ui.heading(text),
    })
}

/// Show monospace (fixed width) text.
///
/// Example::
///
///     monospace("hello")
#[pyfunction]
unsafe fn monospace(text: &str) -> PyResult<()> {
    monospace_response(text)?;
    Ok(())
}

/// Returns the Response of the text. A text widget does not react to clicks,
/// but its Response carries the rect and can carry a tooltip.
///
/// Example::
///
///     monospace("codeish")
///
///     # with a tooltip:
///     monospace_response("hover me").on_hover_text("an explanation")
#[pyfunction]
unsafe fn monospace_response(text: &str) -> PyResult<Response> {
    let ui = current_ui(&UI)?;

    Ok(Response {
        inner: ui.monospace(text),
    })
}

/// Show small text.
///
/// Example::
///
///     small("hello")
#[pyfunction]
unsafe fn small(text: &str) -> PyResult<()> {
    small_response(text)?;
    Ok(())
}

/// Returns the Response of the text. A text widget does not react to clicks,
/// but its Response carries the rect and can carry a tooltip.
///
/// Example::
///
///     small("tiny")
///
///     # with a tooltip:
///     small_response("hover me").on_hover_text("an explanation")
#[pyfunction]
unsafe fn small_response(text: &str) -> PyResult<Response> {
    let ui = current_ui(&UI)?;

    Ok(Response {
        inner: ui.small(text),
    })
}

/// Show text that stand out a bit (e.g. slightly brighter).
///
/// Example::
///
///     strong("hello")
#[pyfunction]
unsafe fn strong(text: &str) -> PyResult<()> {
    strong_response(text)?;
    Ok(())
}

/// Returns the Response of the text. A text widget does not react to clicks,
/// but its Response carries the rect and can carry a tooltip.
///
/// Example::
///
///     strong("important")
///
///     # with a tooltip:
///     strong_response("hover me").on_hover_text("an explanation")
#[pyfunction]
unsafe fn strong_response(text: &str) -> PyResult<Response> {
    let ui = current_ui(&UI)?;

    Ok(Response {
        inner: ui.strong(text),
    })
}

/// Show text that is weaker (fainter color).
///
/// Example::
///
///     weak("hello")
#[pyfunction]
unsafe fn weak(text: &str) -> PyResult<()> {
    weak_response(text)?;
    Ok(())
}

/// Returns the Response of the text. A text widget does not react to clicks,
/// but its Response carries the rect and can carry a tooltip.
///
/// Example::
///
///     weak("unimportant")
///
///     # with a tooltip:
///     weak_response("hover me").on_hover_text("an explanation")
#[pyfunction]
unsafe fn weak_response(text: &str) -> PyResult<Response> {
    let ui = current_ui(&UI)?;

    Ok(Response {
        inner: ui.weak(text),
    })
}

/// Show some text.
///
/// Example::
///
///     label("some text")
#[pyfunction]
unsafe fn label(text: &str) -> PyResult<()> {
    label_response(text)?;
    Ok(())
}

/// Returns the Response of the text. A text widget does not react to clicks,
/// but its Response carries the rect and can carry a tooltip.
///
/// Example::
///
///     label("some text")
///
///     # with a tooltip:
///     label_response("hover me").on_hover_text("an explanation")
#[pyfunction]
unsafe fn label_response(text: &str) -> PyResult<Response> {
    let ui = current_ui(&UI)?;

    Ok(Response {
        inner: ui.label(text),
    })
}

/// Show text as monospace with a gray background.
///
/// Example::
///
///     code("print(42 + 27)")
#[pyfunction]
unsafe fn code(text: &str) -> PyResult<()> {
    code_response(text)?;
    Ok(())
}

/// Returns the Response of the text. A text widget does not react to clicks,
/// but its Response carries the rect and can carry a tooltip.
///
/// Example::
///
///     code("code")
///
///     # with a tooltip:
///     code_response("hover me").on_hover_text("an explanation")
#[pyfunction]
unsafe fn code_response(text: &str) -> PyResult<Response> {
    let ui = current_ui(&UI)?;

    Ok(Response {
        inner: ui.code(text),
    })
}

/// Show singleline text field and update the text
///
/// Example::
///
///     text = Str("print(42 + 27)")
///     # inside update func
///     code_editor(text)
#[pyfunction]
unsafe fn code_editor(text: &mut Str) -> PyResult<()> {
    code_editor_response(text)?;
    Ok(())
}

/// Returns the Response of the code editor. Use changed to know the text was
/// edited.
///
/// Example::
///
///     text = Str("print(42)")
///     if code_editor_response(text).changed:
///       print("now", text.value)
#[pyfunction]
unsafe fn code_editor_response(text: &mut Str) -> PyResult<Response> {
    let ui = current_ui(&UI)?;

    Ok(Response {
        inner: ui.code_editor(&mut text.value),
    })
}

/// Show singleline text field and update the text
///
/// Example::
///
///     text = Str("editable")
///     # inside update func
///     text_edit_singleline(text, hint_text="hint me bro")
///
/// Builder options go in **options: `hint_text`, `password`, `desired_width`,
/// `desired_rows`, `char_limit`, `interactive`, `clip_text`, `lock_focus`,
/// `frame`, `cursor_at_end`, `background_color`, `margin`, `horizontal_align`,
/// `vertical_align`. An unknown name is an error.
#[pyfunction]
#[pyo3(signature = (text, **kwargs))]
unsafe fn text_edit_singleline(text: &mut Str, kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<()> {
    text_edit_singleline_response(text, kwargs)?;
    Ok(())
}

/// Returns the Response of the single line text field. Use changed to know
/// the text was edited.
///
/// Example::
///
///     text = Str("editable")
///     response = text_edit_singleline_response(text, hint_text="type here")
///     if response.changed:
///       print("now", text.value)
///
/// Builder options go in **options: `hint_text`, `password`, `desired_width`,
/// `desired_rows`, `char_limit`, `interactive`, `clip_text`, `lock_focus`,
/// `frame`, `cursor_at_end`, `background_color`, `margin`, `horizontal_align`,
/// `vertical_align`. An unknown name is an error.
#[pyfunction]
#[pyo3(signature = (text, **kwargs))]
unsafe fn text_edit_singleline_response(
    text: &mut Str,
    kwargs: Option<&Bound<'_, PyDict>>,
) -> PyResult<Response> {
    let ui = current_ui(&UI)?;
    let mut used = OptNames::new();

    // The option list is shared with `text_edit_multiline_response` through
    // `apply_text_edit_options`, which owns `used` for both. A name cannot
    // arrive both as a keyword parameter and inside **kwargs (CPython raises
    // TypeError on the duplicate first), so nothing else needs recording.
    let w = apply_text_edit_options(
        egui::TextEdit::singleline(&mut text.value),
        kwargs,
        &mut used,
        "text_edit_singleline",
    )?;

    Ok(Response {
        inner: ui.add(w),
    })
}

/// Show multiline text field and update the text
///
/// Example::
///
///     text = Str("editable")
///     # inside update func
///     text_edit_multiline(text, hint_text="hint")
///
/// Builder options go in **options: `hint_text`, `password`, `desired_width`,
/// `desired_rows`, `char_limit`, `interactive`, `clip_text`, `lock_focus`,
/// `frame`, `cursor_at_end`, `background_color`, `margin`, `horizontal_align`,
/// `vertical_align`. An unknown name is an error.
#[pyfunction]
#[pyo3(signature = (text, **kwargs))]
unsafe fn text_edit_multiline(text: &mut Str, kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<()> {
    text_edit_multiline_response(text, kwargs)?;
    Ok(())
}

/// Returns the Response of the multiline text field. Use changed to know
/// the text was edited.
///
/// Example::
///
///     text = Str("editable")
///     if text_edit_multiline_response(text).changed:
///       print("now", text.value)
///
/// Builder options go in **options: `hint_text`, `password`, `desired_width`,
/// `desired_rows`, `char_limit`, `interactive`, `clip_text`, `lock_focus`,
/// `frame`, `cursor_at_end`, `background_color`, `margin`, `horizontal_align`,
/// `vertical_align`. An unknown name is an error.
#[pyfunction]
#[pyo3(signature = (text, **kwargs))]
unsafe fn text_edit_multiline_response(
    text: &mut Str,
    kwargs: Option<&Bound<'_, PyDict>>,
) -> PyResult<Response> {
    let ui = current_ui(&UI)?;
    let mut used = OptNames::new();

    // See `text_edit_singleline_response`.
    let w = apply_text_edit_options(
        egui::TextEdit::multiline(&mut text.value),
        kwargs,
        &mut used,
        "text_edit_multiline",
    )?;

    Ok(Response {
        inner: ui.add(w),
    })
}

/// Returns true if the button was clicked this frame
///
/// Example::
///
///     if button_clicked("click me"):
///       print("click me, my friend")
#[pyfunction]
unsafe fn button_clicked(text: &str) -> PyResult<bool> {
    Ok(button_response(text)?.clicked())
}

/// Returns the Response of the button, which carries the full interaction
/// state: hovered, clicked, dragged, focus and rect.
///
/// Example::
///
///     response = button_response("click me")
///     if response.clicked:
///       print("clicked")
///     if response.hovered:
///       response.on_hover_text("press it")
#[pyfunction]
unsafe fn button_response(text: &str) -> PyResult<Response> {
    let ui = current_ui(&UI)?;

    Ok(Response {
        inner: ui.button(text),
    })
}

/// Returns true if the small button was clicked this frame
///
/// Example::
///
///     if small_button_clicked("click me"):
///       print("click me, my friend")
#[pyfunction]
unsafe fn small_button_clicked(text: &str) -> PyResult<bool> {
    Ok(small_button_response(text)?.clicked())
}

/// Returns the Response of the small button. See button_response.
///
/// Example::
///
///     if small_button_response("x").clicked:
///       pass
#[pyfunction]
unsafe fn small_button_response(text: &str) -> PyResult<Response> {
    let ui = current_ui(&UI)?;

    Ok(Response {
        inner: ui.small_button(text),
    })
}

// ------------------------------------------------------- container options
// Builders in egui take typed setters (.min_size(Vec2), .resizable(bool)).
// Python passes them as a kwargs dict, so each container must read that dict
// and apply the ones it recognises.
//
// An unrecognised key raises rather than being ignored: `resizble=True`
// quietly producing a fixed-size window is the kind of mistake that costs an
// afternoon. Every unknown key is reported at once so one run finds them all.

/// Raise `PyValueError` if `opts` contains a key that is not in `known`.
unsafe fn validate_options(opts: &Bound<'_, PyDict>, known: &[&str]) -> PyResult<()> {
    let mut unknown: Vec<String> = Vec::new();

    for key in opts.keys().iter() {
        let name: String = key.extract()?;
        if !known.contains(&name.as_str()) {
            unknown.push(name);
        }
    }

    if !unknown.is_empty() {
        return Err(PyValueError::new_err(format!(
            "unknown container option(s): {}. Known options: {}",
            unknown.join(", "),
            known.join(", ")
        )));
    }

    Ok(())
}

/// The set of option names a widget has consumed.
///
/// The unknown-option check needs to know which keys were read. `opt_*` helpers
/// deliberately leave the dict untouched -- mutating it would break callers who
/// reuse one dict across several widgets -- so the record lives here instead.
type OptNames = std::collections::HashSet<String>;

/// Fail on any option key the widget did not consume.
///
/// Decided 2026-10-04: a misspelled option must be an error, not a silent
/// no-op. Any key in `opts` outside `used` is either a typo or an option this
/// binding does not expose, and silently doing nothing is the worst possible
/// answer.
///
/// Relies on this contract: every key supplied to a widget is consumed exactly
/// once by that widget's own option-processing path. A helper that reads
/// options without threading `used` through will make a supported option look
/// unknown, so delegated paths must pass it down.
///
/// Not called anywhere yet. The existing containers keep reporting unknown
/// options through `validate_options`, which switches a fixed name list off a
/// `&[&str]` const; adopting this check for them is a separate change that would
/// reject option names in code that ships today. The six widget groups that do
/// call this arrive in the tasks after this one, so `dead_code` fires until
/// then.
unsafe fn reject_unknown_options(
    opts: &Bound<'_, PyDict>,
    used: &OptNames,
    widget: &str,
) -> PyResult<()> {
    // Same shape as `validate_options` above: a key that is not a string is a
    // real error, propagated rather than dropped. A `filter_map` here would
    // silently succeed on a non-string key that `validate_options` rejects.
    let mut unknown: Vec<String> = Vec::new();
    for key in opts.keys().iter() {
        let name: String = key.extract()?;
        if !used.contains(&name) {
            unknown.push(name);
        }
    }
    if unknown.is_empty() {
        return Ok(());
    }
    unknown.sort();
    Err(PyValueError::new_err(format!(
        "{widget} got unknown option(s): {}. See the documentation for the \
         options this widget accepts.",
        unknown.join(", ")
    )))
}

/// Read an optional bool from `opts`, recording `name` as consumed.
///
/// `used.insert` runs before the read and unconditionally, so a declared but
/// absent option still counts as known: the check distinguishes names the
/// widget does not implement from values the caller did not supply.
unsafe fn opt_bool(
    opts: &Bound<'_, PyDict>,
    name: &str,
    used: &mut OptNames,
) -> PyResult<Option<bool>> {
    used.insert(name.to_string());
    match opts.get_item(name)? {
        Some(value) => Ok(Some(value.extract()?)),
        None => Ok(None),
    }
}

/// Read an optional f32 from `opts`, recording `name` as consumed.
unsafe fn opt_f32(
    opts: &Bound<'_, PyDict>,
    name: &str,
    used: &mut OptNames,
) -> PyResult<Option<f32>> {
    used.insert(name.to_string());
    match opts.get_item(name)? {
        Some(value) => Ok(Some(value.extract()?)),
        None => Ok(None),
    }
}

/// Read an optional usize from `opts`, recording `name` as consumed.
unsafe fn opt_usize(
    opts: &Bound<'_, PyDict>,
    name: &str,
    used: &mut OptNames,
) -> PyResult<Option<usize>> {
    used.insert(name.to_string());
    match opts.get_item(name)? {
        Some(value) => {
            // A negative count is a caller's mistake, and `usize` extraction
            // reports it as an OverflowError that names neither the option nor
            // the value. Say so here instead.
            let value: i64 = value.extract()?;
            if value < 0 {
                return Err(PyValueError::new_err(format!(
                    "{name} must be a non-negative integer, got {value}"
                )));
            }
            Ok(Some(value as usize))
        }
        None => Ok(None),
    }
}

/// Read an optional f64 from `opts`, recording `name` as consumed.
///
/// Separate from `opt_f32` because egui's `Slider::step_by` and
/// `drag_value_speed` take `f64`; a float slider's range is still `f32` in
/// Python, but routing a 64-bit builder argument through f32 would round.
unsafe fn opt_f64(
    opts: &Bound<'_, PyDict>,
    name: &str,
    used: &mut OptNames,
) -> PyResult<Option<f64>> {
    used.insert(name.to_string());
    match opts.get_item(name)? {
        Some(value) => Ok(Some(value.extract()?)),
        None => Ok(None),
    }
}

/// Check a word against the ones an egui enum accepts.
///
/// egui's `Slider` has enum-valued options that Python has no type for --
/// `HandleShape` and `SliderClamping` -- so the word is validated here and
/// mapped to a variant at the call site. The error lists the accepted words,
/// because "unknown variant" on its own tells a caller nothing about what to
/// type instead.
///
/// Returns `PyResult<()>` rather than the word: with three `&str` parameters
/// Rust's lifetime elision cannot pick which one a returned `&str` would
/// borrow, so returning the word would need an explicit lifetime for no gain.
///
/// Deliberately a plain function rather than a macro: the brief asked for one
/// call site, and Task 3 generalises this if the pattern repeats.
fn enum_word(word: &str, name: &str, accepted: &[&str]) -> PyResult<()> {
    if !accepted.contains(&word) {
        return Err(PyValueError::new_err(format!(
            "unknown {name} {word:?}; expected one of {}",
            accepted
                .iter()
                .map(|w| format!("{w:?}"))
                .collect::<Vec<_>>()
                .join(", ")
        )));
    }
    Ok(())
}

/// Extract a Python value as a string, then check it with [`enum_word`].
///
/// This is the dict-free form. A reading-from-dict variant was written first
/// (`opt_enum_str`, mirroring `opt_bool`) and then deleted: `handle_shape` is
/// the only enum-valued option in this group, and its value may be a
/// `(word, aspect_ratio)` pair that a string extractor rejects outright, so
/// the dict read has to happen in `handle_shape_opt` regardless. A helper
/// nothing calls would only be a `dead_code` warning. Task 3, which reaches
/// options that are plain words only, is where a reading variant belongs.
fn enum_word_py(
    value: &Bound<'_, PyAny>,
    name: &str,
    accepted: &[&str],
) -> PyResult<String> {
    let word: String = value.extract()?;
    enum_word(&word, name, accepted)?;
    Ok(word)
}

/// Read a radix-format option (`binary`, `octal`, `hexadecimal`) from `opts`,
/// recording `name` as consumed and returning `(min_width, twos_complement,
/// upper)`.
///
/// egui spells these `(min_width, twos_complement)`, plus a third `upper` for
/// hexadecimal, so the Python value is a sequence:
/// `binary=(8, False)`, `hexadecimal=(16, False, True)`. `upper` is defaulted
/// to False so `hexadecimal=(16,)` still works.
///
/// Values extract as `i64` because Python `bool` is an `int` subclass, so
/// `binary=(8, True)` is the documented way to ask for two's complement and
/// must not have to be spelled `1`.
unsafe fn opt_radix(
    opts: &Bound<'_, PyDict>,
    name: &str,
    used: &mut OptNames,
) -> PyResult<Option<Vec<i64>>> {
    used.insert(name.to_string());
    let value = match opts.get_item(name)? {
        Some(value) => value,
        None => return Ok(None),
    };

    // egui's three radix builders all take `(min_width, twos_complement)`;
    // hexadecimal takes a third `upper`. `upper` is optional, so
    // `hexadecimal=(16, False)` is accepted and defaults to False.
    let parts: Vec<i64> = value.extract().map_err(|_| {
        PyValueError::new_err(format!(
            "{name} must be a sequence like {name}=(8, False) -- egui's \
             binary/octal/hexadecimal take a minimum digit width and a \
             twos_complement flag"
        ))
    })?;
    validate_radix(&parts, name)?;
    Ok(Some(parts))
}

/// Check a radix option's arity and minimum width.
///
/// egui `assert!`s on a zero width, and inside a pyo3 function an `assert!` is
/// a panic across the FFI boundary rather than a Python exception, so the check
/// has to happen on this side.
fn validate_radix(parts: &[i64], name: &str) -> PyResult<()> {
    let required = if name == "hexadecimal" { 3 } else { 2 };
    let optional_upper = name == "hexadecimal";
    let widest = required + usize::from(optional_upper);
    if parts.len() < required || parts.len() > widest {
        return Err(PyValueError::new_err(format!(
            "{name} takes {required} values (minimum width, twos_complement{}) \
             but got {}",
            if optional_upper { ", upper" } else { "" },
            parts.len()
        )));
    }
    if parts[0] <= 0 {
        return Err(PyValueError::new_err(format!(
            "{name} minimum width must be greater than 0, got {}",
            parts[0]
        )));
    }
    Ok(())
}

/// Read an optional `String` from `opts`, recording `name` as consumed.
///
/// egui's `TextEdit::hint_text` takes `impl Into<WidgetText>`, so a plain
/// `&str` is the useful Python shape and a `WidgetText` class (TODO §4) would
/// be a later refinement rather than a prerequisite.
unsafe fn opt_string(
    opts: &Bound<'_, PyDict>,
    name: &str,
    used: &mut OptNames,
) -> PyResult<Option<String>> {
    used.insert(name.to_string());
    match opts.get_item(name)? {
        Some(value) => Ok(Some(value.extract()?)),
        None => Ok(None),
    }
}

/// Read the **options tail of a DragValue and return the rebuilt builder.
///
/// Takes and returns the builder by value for the same reason
/// `apply_slider_options` does: egui's setters consume `self`, so
/// `*drag = drag.suffix(v)` through a `&mut` would move out of a borrow.
///
/// The name says "tail" because `suffix`, `prefix`, `speed` and
/// `clamping`-alike common names are keyword parameters on the pyfunctions and
/// so never arrive in the dict. Only `range`-adjacent and display options do.
///
/// `unsafe` because the `opt_*` helpers are.
unsafe fn apply_drag_options<'a>(
    mut drag: egui::DragValue<'a>,
    options: Option<&Bound<'_, PyDict>>,
    used: &mut OptNames,
    widget: &str,
) -> PyResult<egui::DragValue<'a>> {
    let o = match options {
        Some(o) => o,
        None => return Ok(drag),
    };

    if let Some(v) = opt_bool(o, "update_while_editing", used)? {
        drag = drag.update_while_editing(v);
    }
    if let Some(v) = opt_bool(o, "clamp_existing_to_range", used)? {
        drag = drag.clamp_existing_to_range(v);
    }
    if let Some(v) = opt_usize(o, "fixed_decimals", used)? {
        drag = drag.fixed_decimals(v);
    }
    if let Some(v) = opt_usize(o, "min_decimals", used)? {
        drag = drag.min_decimals(v);
    }
    if let Some(v) = opt_usize(o, "max_decimals", used)? {
        drag = drag.max_decimals(v);
    }
    // egui's radix builders take `(min_width, twos_complement)`, hexadecimal a
    // third `upper`. Each replaces the custom formatter wholesale, so passing
    // two means only the last takes effect -- egui's own behaviour, kept here.
    if let Some(v) = opt_radix(o, "binary", used)? {
        drag = drag.binary(v[0] as usize, v[1] != 0);
    }
    if let Some(v) = opt_radix(o, "octal", used)? {
        drag = drag.octal(v[0] as usize, v[1] != 0);
    }
    if let Some(v) = opt_radix(o, "hexadecimal", used)? {
        let upper = v.get(2).copied().unwrap_or(0) != 0;
        drag = drag.hexadecimal(v[0] as usize, v[1] != 0, upper);
    }

    reject_unknown_options(o, used, widget)?;

    Ok(drag)
}

/// Map an `Align` word onto egui's `Align`.
///
/// egui's `TextEdit::horizontal_align` / `vertical_align` take an `emath::Align`,
/// which is re-exported as `egui::Align` (egui-0.31.1/src/lib.rs:461). The enum
/// has exactly three variants -- `Min`, `Center`, `Max` -- with `LEFT`/`RIGHT`/
/// `TOP`/`BOTTOM` as associated consts aliasing `Min`/`Max`. The accepted words
/// are therefore the variant names in snake_case, not the const names: `"min"`
/// and `"center"` and `"max"`.
fn align(word: &str, name: &str) -> PyResult<egui::Align> {
    enum_word(word, name, ALIGN_WORDS)?;
    Ok(match word {
        "min" => egui::Align::Min,
        "center" => egui::Align::Center,
        // `enum_word` already rejected anything else.
        _ => egui::Align::Max,
    })
}

/// The words `horizontal_align` and `vertical_align` accept.
///
/// egui's `Align` variants in declaration order. Note that `Align` also exposes
/// `LEFT`/`TOP` (both `Min`) and `RIGHT`/`BOTTOM` (both `Max`) as consts; those
/// spellings are NOT accepted here, because accepting them would give four words
/// for three variants with no way to tell a caller which is which.
const ALIGN_WORDS: &[&str] = &["min", "center", "max"];

/// Read an `Align`-valued option from `opts`, recording `name` as consumed.
///
/// The dict-reading counterpart `enum_word_py` was written for, and the one
/// `enum_word` itself could not serve: `enum_word` takes an already-extracted
/// `&str`, but the value is a `PyAny` here.
unsafe fn opt_align(
    opts: &Bound<'_, PyDict>,
    name: &str,
    used: &mut OptNames,
) -> PyResult<Option<egui::Align>> {
    used.insert(name.to_string());
    let value = match opts.get_item(name)? {
        Some(value) => value,
        None => return Ok(None),
    };
    let word = enum_word_py(&value, name, ALIGN_WORDS)?;
    // `enum_word_py` has already checked the word against `ALIGN_WORDS`, so
    // `align` cannot return its error here; the `unwrap_or` is unreachable and
    // exists only to satisfy the signature.
    let mapped = align(&word, name).unwrap_or(egui::Align::Min);
    Ok(Some(mapped))
}

/// Read the **options tail of a TextEdit and return the rebuilt builder.
///
/// Shared by `text_edit_singleline_response` and `text_edit_multiline_response`,
/// which differ only in `singleline()` vs `multiline()`. Delegating here is what
/// keeps `hint_text` -- the one option that shipped in Task 1 -- and the eleven
/// new ones in one list, so the two cannot drift.
///
/// `hint_text` stays in the dict rather than becoming a keyword parameter
/// deliberately: it is the option this API has always accepted by keyword, and
/// moving it into the signature would be a change to a shipped call rather than
/// an addition. It is read with `opt_string` now instead of a bare `get_item`,
/// which is behaviour-preserving: both extract a `String`, and both raise the
/// same `TypeError` for a non-string.
///
/// Takes and returns by value because egui's setters consume `self`, the same
/// reason `apply_slider_options` and `apply_drag_options` do.
unsafe fn apply_text_edit_options<'a>(
    mut w: egui::TextEdit<'a>,
    options: Option<&Bound<'_, PyDict>>,
    used: &mut OptNames,
    widget: &str,
) -> PyResult<egui::TextEdit<'a>> {
    let o = match options {
        Some(o) => o,
        None => return Ok(w),
    };

    if let Some(v) = opt_string(o, "hint_text", used)? {
        w = w.hint_text(v);
    }
    if let Some(v) = opt_bool(o, "password", used)? {
        w = w.password(v);
    }
    if let Some(v) = opt_f32(o, "desired_width", used)? {
        w = w.desired_width(v);
    }
    if let Some(v) = opt_usize(o, "desired_rows", used)? {
        w = w.desired_rows(v);
    }
    if let Some(v) = opt_usize(o, "char_limit", used)? {
        w = w.char_limit(v);
    }
    if let Some(v) = opt_bool(o, "interactive", used)? {
        w = w.interactive(v);
    }
    if let Some(v) = opt_bool(o, "clip_text", used)? {
        w = w.clip_text(v);
    }
    if let Some(v) = opt_bool(o, "lock_focus", used)? {
        w = w.lock_focus(v);
    }
    if let Some(v) = opt_bool(o, "frame", used)? {
        w = w.frame(v);
    }
    if let Some(v) = opt_bool(o, "cursor_at_end", used)? {
        w = w.cursor_at_end(v);
    }
    if let Some(v) = opt_color32(o, "background_color", used)? {
        w = w.background_color(v);
    }
    if let Some(v) = opt_margin(o, "margin", used)? {
        w = w.margin(v);
    }
    if let Some(v) = opt_align(o, "horizontal_align", used)? {
        w = w.horizontal_align(v);
    }
    if let Some(v) = opt_align(o, "vertical_align", used)? {
        w = w.vertical_align(v);
    }

    reject_unknown_options(o, used, widget)?;

    Ok(w)
}

/// Read an optional 2D size or position from `opts` as a 2-sequence of
/// numbers, e.g. `default_size=(400.0, 300.0)`.
///
/// egui's geometry types (`Vec2`, `Pos2`) have no Python equivalent yet --
/// they are TODO §4 -- so tuples are accepted for now and converted here.
/// When a `Vec2` class lands, these should switch to it.
unsafe fn opt_vec2(
    opts: &Bound<'_, PyDict>,
    name: &str,
    used: &mut OptNames,
) -> PyResult<Option<egui::Vec2>> {
    used.insert(name.to_string());
    let value = match opts.get_item(name)? {
        Some(value) => value,
        None => return Ok(None),
    };

    let pair: Vec<f32> = value.extract().map_err(|_| {
        PyValueError::new_err(format!(
            "{name} must be a 2-sequence of numbers, e.g. {name}=(400.0, 300.0)"
        ))
    })?;

    if pair.len() != 2 {
        return Err(PyValueError::new_err(format!(
            "{name} must have exactly 2 components, got {}",
            pair.len()
        )));
    }

    Ok(Some(egui::Vec2::new(pair[0], pair[1])))
}

/// Draw a secondary window.
///
/// egui requires windows to be added after any top-level panels, so call
/// this after `side_panel_left` and before or after `central_panel` -- a
/// window floats above the central panel either way.
///
/// Returns True when the window was visible this frame, False when it is
/// collapsed or fully closed.
///
/// `open` is an optional `Bool` that egui reads and writes: pass one and the
/// window's open/close button drives it.
///
/// Example::
///
///     def win_contents():
///       heading("a window")
///       if button_clicked("close"):
///           pass
///
///     def update_func(ctx):
///       window(ctx, "Settings", "settings", win_contents, open=win_open)
///       central_panel(ctx, main_contents)
///
/// Unknown keyword arguments raise `ValueError` naming them. Sizes and
/// positions are 2-tuples of numbers, e.g. `default_size=(400.0, 300.0)`.
#[pyfunction]
#[pyo3(signature = (ctx, title, id, contents, open=None, **options))]
unsafe fn window(
    ctx: &Context,
    title: &str,
    id: &str,
    contents: Bound<'_, PyAny>,
    mut open: Option<&mut Bool>,
    options: Option<&Bound<'_, PyDict>>,
) -> PyResult<bool> {
    let ctx = &ctx.0;

    // `Window<'open>` carries a lifetime tied to the `open` bool, and
    // `WidgetText` built from `&str` would borrow the Python string too. Own
    // the title so the builder is `'static` and can be assembled before the
    // `open` borrow is taken.
    let title = title.to_owned();
    let mut builder = egui::Window::new(title).id(egui::Id::new(id));

    if let Some(opts) = options {
        validate_options(opts, WINDOW_OPTIONS)?;
        // The tracker exists so `apply_window_options` can record what it
        // reads. `window` is not a §6 target yet, so nothing checks it here --
        // `validate_options` still rejects anything outside WINDOW_OPTIONS.
        let mut used = OptNames::new();
        builder = apply_window_options(builder, opts, &mut used)?;
    }

    // egui's `open` takes `&'open mut bool`. `Bool.value` is an owned field,
    // so borrowing it keeps the reference alive for exactly the `.show()`
    // call, which is all `'open` needs.
    let shown = match open.as_deref_mut() {
        Some(open) => builder
            .open(&mut open.value)
            .show(ctx, |ui| run_nested_update_func_lossy(ui, contents.clone()))
            .is_some(),
        None => builder
            .show(ctx, |ui| run_nested_update_func_lossy(ui, contents.clone()))
            .is_some(),
    };

    Ok(shown)
}

/// Every keyword `window` accepts. Anything else raises.
const WINDOW_OPTIONS: &[&str] = &[
    // geometry
    "default_size",
    "default_pos",
    "fixed_size",
    "fixed_pos",
    "min_size",
    "max_size",
    "min_width",
    "max_width",
    "min_height",
    "max_height",
    "default_width",
    "default_height",
    // behaviour
    "resizable",
    "collapsible",
    "title_bar",
    "movable",
    "scroll",
    "vscroll",
    "hscroll",
    "auto_sized",
    "enabled",
    "fade_in",
    "fade_out",
    "interactable",
    "constrain",
    "order",
];

unsafe fn apply_window_options(
    mut builder: egui::Window<'static>,
    opts: &Bound<'_, PyDict>,
    used: &mut OptNames,
) -> PyResult<egui::Window<'static>> {
    if let Some(v) = opt_vec2(opts, "default_size", used)? {
        builder = builder.default_size(v);
    }
    if let Some(v) = opt_vec2(opts, "fixed_size", used)? {
        builder = builder.fixed_size(v);
    }
    if let Some(v) = opt_vec2(opts, "min_size", used)? {
        builder = builder.min_size(v);
    }
    if let Some(v) = opt_vec2(opts, "max_size", used)? {
        builder = builder.max_size(v);
    }
    // `default_pos` and `fixed_pos` take a Pos2, which is the same Vec2 with
    // a different name in egui.
    if let Some(v) = opt_vec2(opts, "default_pos", used)? {
        builder = builder.default_pos(egui::Pos2::new(v.x, v.y));
    }
    if let Some(v) = opt_vec2(opts, "fixed_pos", used)? {
        builder = builder.fixed_pos(egui::Pos2::new(v.x, v.y));
    }

    for (name, setter) in [
        ("default_width", 0usize),
        ("max_width", 1),
        ("min_width", 2),
        ("default_height", 3),
        ("max_height", 4),
        ("min_height", 5),
    ] {
        if let Some(v) = opt_f32(opts, name, used)? {
            builder = match setter {
                0 => builder.default_width(v),
                1 => builder.max_width(v),
                2 => builder.min_width(v),
                3 => builder.default_height(v),
                4 => builder.max_height(v),
                _ => builder.min_height(v),
            };
        }
    }

    // egui takes `impl Into<Vec2b>`; a plain bool converts, and true means
    // both axes.
    for name in ["resizable", "scroll"] {
        if let Some(v) = opt_bool(opts, name, used)? {
            builder = if name == "resizable" {
                builder.resizable(v)
            } else {
                builder.scroll(v)
            };
        }
    }

    for name in [
        "collapsible",
        "title_bar",
        "movable",
        "vscroll",
        "hscroll",
        "enabled",
        "fade_in",
        "fade_out",
        "interactable",
        "constrain",
    ] {
        if let Some(v) = opt_bool(opts, name, used)? {
            builder = match name {
                "collapsible" => builder.collapsible(v),
                "title_bar" => builder.title_bar(v),
                "movable" => builder.movable(v),
                "vscroll" => builder.vscroll(v),
                "hscroll" => builder.hscroll(v),
                "enabled" => builder.enabled(v),
                "fade_in" => builder.fade_in(v),
                "fade_out" => builder.fade_out(v),
                "interactable" => builder.interactable(v),
                _ => builder.constrain(v),
            };
        }
    }

    // egui's `auto_sized` takes no argument -- it is a mode, not a toggle --
    // so passing True enables it and False leaves the window alone.
    if opt_bool(opts, "auto_sized", used)?.unwrap_or(false) {
        builder = builder.auto_sized();
    }

    // egui's `order` takes an `egui::Order`, not a number. Map the three
    // words rather than taking an f32 and discarding it -- silently ignoring
    // the value is the failure this whole option parser exists to prevent.
    if let Some(v) = opt_order(opts, used)? {
        builder = builder.order(v);
    }

    Ok(builder)
}

/// Draws the central panel. Must be called last in update_func, after any
/// side panels, top/bottom panels or modals.
///
/// egui requires `CentralPanel` to be added after every other top-level
/// panel (see the panel docs in egui 0.31.1), so Python composes the frame
/// itself rather than having `run_native` do it. This is what an egui app
/// does in Rust.
///
/// Example::
///
///     def main_contents():
///       heading("in the central panel")
///
///     def update_func(ctx):
///       side_panel_left(ctx, "nav", nav_contents)
///       central_panel(ctx, main_contents)
///
/// Widgets resolve their `Ui` from an internal stack, so they must be drawn
/// inside a callback such as this one, never directly in `update_func`.
/// Read an optional `(min, max)` float range from `opts`, egui's `Rangef`.
unsafe fn opt_range(
    opts: &Bound<'_, PyDict>,
    name: &str,
    used: &mut OptNames,
) -> PyResult<Option<egui::Rangef>> {
    used.insert(name.to_string());
    match opts.get_item(name)? {
        Some(value) => {
            let (min, max): (f32, f32) = value.extract().map_err(|_| {
                PyValueError::new_err(format!(
                    "{name} must be a (min, max) tuple of numbers"
                ))
            })?;
            Ok(Some(egui::Rangef::new(min, max)))
        }
        None => Ok(None),
    }
}

/// Read an optional `egui::Color32` from `opts`.
///
/// Accepts either a `Color32` or an `(r, g, b, a)` 4-tuple of ints in 0-255,
/// since a caller reaching for a fill should not have to construct a class just
/// to name a colour.
unsafe fn opt_color32(
    opts: &Bound<'_, PyDict>,
    name: &str,
    used: &mut OptNames,
) -> PyResult<Option<egui::Color32>> {
    used.insert(name.to_string());
    let value = match opts.get_item(name)? {
        Some(value) => value,
        None => return Ok(None),
    };

    let color = color32_from_any(&value).map_err(|_| {
        PyValueError::new_err(format!(
            "{name} must be a Color32 or an (r, g, b, a) tuple of ints in 0-255"
        ))
    })?;

    Ok(Some(color))
}

/// Read an optional `egui::CornerRadius` from `opts`.
///
/// egui's is four `u8` corners. A single number sets all four (which is what
/// `CornerRadius::same` does); a 2-sequence sets (nw, ne, sw, se), matching
/// how a CSS-style `a b / c d` reads.
unsafe fn opt_corner_radius(
    opts: &Bound<'_, PyDict>,
    name: &str,
    used: &mut OptNames,
) -> PyResult<Option<egui::CornerRadius>> {
    used.insert(name.to_string());
    let value = match opts.get_item(name)? {
        Some(value) => value,
        None => return Ok(None),
    };

    if let Ok(same) = value.extract::<u8>() {
        return Ok(Some(egui::CornerRadius::same(same)));
    }

    let pair: Vec<u8> = value.extract().map_err(|_| {
        PyValueError::new_err(format!(
            "{name} must be a number, or a 2-sequence of numbers, e.g. \
             {name}=(6, 2)"
        ))
    })?;

    match pair.len() {
        2 => Ok(Some(egui::CornerRadius {
            nw: pair[0],
            ne: pair[1],
            sw: pair[0],
            se: pair[1],
        })),
        4 => Ok(Some(egui::CornerRadius {
            nw: pair[0],
            ne: pair[1],
            sw: pair[2],
            se: pair[3],
        })),
        n => Err(PyValueError::new_err(format!(
            "{name} must have 2 or 4 components, got {n}"
        ))),
    }
}

/// Read an optional `egui::Margin` from `opts`.
///
/// egui's margins are `i8`, one per side. A single number sets all four (what
/// `Margin::same` does); a 2-sequence sets (horizontal, vertical).
unsafe fn opt_margin(
    opts: &Bound<'_, PyDict>,
    name: &str,
    used: &mut OptNames,
) -> PyResult<Option<egui::Margin>> {
    used.insert(name.to_string());
    let value = match opts.get_item(name)? {
        Some(value) => value,
        None => return Ok(None),
    };

    if let Ok(same) = value.extract::<i8>() {
        return Ok(Some(egui::Margin::same(same)));
    }

    let pair: Vec<i8> = value.extract().map_err(|_| {
        PyValueError::new_err(format!(
            "{name} must be a number, or a 2-sequence of numbers, e.g. \
             {name}=(8, 4)"
        ))
    })?;

    match pair.len() {
        2 => Ok(Some(egui::Margin::symmetric(pair[0], pair[1]))),
        4 => Ok(Some(egui::Margin {
            left: pair[0],
            right: pair[1],
            top: pair[2],
            bottom: pair[3],
        })),
        n => Err(PyValueError::new_err(format!(
            "{name} must have 2 or 4 components, got {n}"
        ))),
    }
}

/// Read an optional `egui::Stroke` from `opts`.
///
/// egui's is a width in points plus a colour. Accepts `(width, color)` where
/// color is anything `opt_color32` accepts, or a bare number, which means that
/// width in the style's current foreground colour -- the common case for a
/// divider line.
unsafe fn opt_stroke(
    opts: &Bound<'_, PyDict>,
    name: &str,
    used: &mut OptNames,
) -> PyResult<Option<egui::Stroke>> {
    used.insert(name.to_string());
    let value = match opts.get_item(name)? {
        Some(value) => value,
        None => return Ok(None),
    };

    if let Ok(width) = value.extract::<f32>() {
        return Ok(Some(egui::Stroke::new(
            width,
            current_ui(&UI)?.visuals().widgets.noninteractive.bg_stroke.color,
        )));
    }

    let pair: Vec<Bound<'_, PyAny>> = value.extract().map_err(|_| {
        PyValueError::new_err(format!(
            "{name} must be a width, or a (width, color) pair"
        ))
    })?;

    if pair.len() != 2 {
        return Err(PyValueError::new_err(format!(
            "{name} must have exactly 2 components, got {}",
            pair.len()
        )));
    }

    let width: f32 = pair[0].extract().map_err(|_| {
        PyValueError::new_err(format!("{name}: the first component must be a width"))
    })?;

    let color = color32_from_any(&pair[1]).map_err(|_| {
        PyValueError::new_err(format!(
            "{name}: the second component must be a Color32 or an \
             (r, g, b, a) tuple"
        ))
    })?;

    Ok(Some(egui::Stroke::new(width, color)))
}

/// Build a `Color32` from an already-extracted Python value.
///
/// Shared by `opt_color32` and `opt_stroke`, which both accept either a
/// `Color32` class instance or a bare `(r, g, b, a)` tuple.
///
/// `downcast`, not `extract`: `Color32` is a plain `#[pyclass]` with no
/// `extract` impl, so `Bound::extract::<Color32>` does not compile.
fn color32_from_any(value: &Bound<'_, PyAny>) -> PyResult<egui::Color32> {
    if let Ok(color) = value.downcast::<Color32>() {
        let color = color.borrow();
        return Ok(egui::Color32::from_rgba_unmultiplied(
            color.r, color.g, color.b, color.a,
        ));
    }

    let quad: Vec<u8> = value.extract().map_err(|_| {
        PyValueError::new_err("expected a Color32 or an (r, g, b, a) tuple of ints in 0-255")
    })?;

    if quad.len() != 4 {
        return Err(PyValueError::new_err(format!(
            "expected 4 components, got {}",
            quad.len()
        )));
    }

    Ok(egui::Color32::from_rgba_unmultiplied(
        quad[0], quad[1], quad[2], quad[3],
    ))
}

/// Read an optional `egui::Order` from `opts`, under the key "order".
///
/// egui 0.31.1 has five variants. Shared by `window` and `area` so the accepted
/// words are defined once. The key is fixed rather than passed in, so the
/// consumed name is recorded as a literal.
unsafe fn opt_order(opts: &Bound<'_, PyDict>, used: &mut OptNames) -> PyResult<Option<egui::Order>> {
    used.insert("order".to_string());
    match opts.get_item("order")? {
        Some(value) => {
            let name: String = value.extract()?;
            let order = match name.as_str() {
                "background" => egui::Order::Background,
                "middle" => egui::Order::Middle,
                "foreground" => egui::Order::Foreground,
                "tooltip" => egui::Order::Tooltip,
                "debug" => egui::Order::Debug,
                _ => {
                    return Err(PyValueError::new_err(format!(
                        "unknown order {name:?}; expected one of 'background', \
                         'middle', 'foreground', 'tooltip', 'debug'"
                    )))
                }
            };
            Ok(Some(order))
        }
        None => Ok(None),
    }
}

/// Builder options for `egui::Frame`, matching its setters exactly.
const FRAME_OPTIONS: &[&str] = &[
    "fill",
    "stroke",
    "corner_radius",
    "rounding",
    "inner_margin",
    "outer_margin",
    "multiply_with_opacity",
];

/// Apply egui's `Frame` setters from a Python kwargs dict.
///
/// `unsafe` because every `opt_*` helper is -- they call `PyDict::get_item`,
/// which pyo3 marks unsafe. Edition 2021, so this body is an implicit unsafe
/// block, same as `apply_window_options`.
unsafe fn apply_frame_options(
    frame: egui::Frame,
    opts: &Bound<'_, PyDict>,
    used: &mut OptNames,
) -> PyResult<egui::Frame> {
    let mut frame = frame;

    if let Some(v) = opt_color32(opts, "fill", used)? {
        frame = frame.fill(v);
    }
    if let Some(v) = opt_stroke(opts, "stroke", used)? {
        frame = frame.stroke(v);
    }
    // `corner_radius` and `rounding` are egui's own aliases for the same
    // setter, so both are accepted here too.
    for name in ["corner_radius", "rounding"] {
        if let Some(v) = opt_corner_radius(opts, name, used)? {
            frame = frame.corner_radius(v);
        }
    }
    if let Some(v) = opt_margin(opts, "inner_margin", used)? {
        frame = frame.inner_margin(v);
    }
    if let Some(v) = opt_margin(opts, "outer_margin", used)? {
        frame = frame.outer_margin(v);
    }
    if let Some(v) = opt_f32(opts, "multiply_with_opacity", used)? {
        frame = frame.multiply_with_opacity(v);
    }

    Ok(frame)
}

/// Draw `contents` inside an egui frame, mirroring `egui::Frame`.
///
/// A frame is egui's decoration: a background fill, a border, a corner radius
/// and padding. egui ships eight presets, each a `Frame` associated function
/// rather than a builder, so they are exposed here as the constructors
/// `frame_group`, `frame_popup`, `frame_menu`, `frame_window`,
/// `frame_canvas`, `frame_dark_canvas`, `frame_central_panel` and
/// `frame_side_top_panel`. Each takes the same keyword arguments, and
/// `frame_*` builds one from scratch with egui's defaults.
///
/// The presets read the *current* style, so a frame looks right in a dark
/// theme and a light one without the caller naming a colour. That is also why
/// these take no `Context`: they need a `Ui` to resolve the style from, which
/// `current_ui` provides.
///
/// Example::
///
///     def contents():
///       label("inside a popup-styled frame")
///
///     frame_popup(contents)
///
/// Unknown keyword arguments raise `ValueError` naming them.
#[pyfunction]
#[pyo3(signature = (contents, **options))]
unsafe fn frame(contents: Bound<'_, PyAny>, options: Option<&Bound<'_, PyDict>>) -> PyResult<()> {
    let ui = current_ui(&UI)?;
    let mut built = egui::Frame::none();

    if let Some(opts) = options {
        validate_options(opts, FRAME_OPTIONS)?;
        // Tracker only: `frame` is not a §6 target, and `validate_options`
        // already rejects anything outside FRAME_OPTIONS.
        let mut used = OptNames::new();
        built = apply_frame_options(built, opts, &mut used)?;
    }

    built.show(ui, |ui| run_nested_update_func_lossy(ui, contents.clone()));

    Ok(())
}

/// The eight `egui::Frame` presets, as constructors.
///
/// egui exposes these as associated functions on `Frame`, which Python cannot
/// call, so each becomes a module-level function. They all take the same
/// keyword arguments as `frame`; see that function for the option list.
macro_rules! frame_preset {
    ($pyfn:ident, $egui_fn:ident, $doc:expr) => {
        #[doc = $doc]
        #[pyfunction]
        #[pyo3(signature = (contents, **options))]
        unsafe fn $pyfn(
            contents: Bound<'_, PyAny>,
            options: Option<&Bound<'_, PyDict>>,
        ) -> PyResult<()> {
            let ui = current_ui(&UI)?;
            let mut built = egui::Frame::$egui_fn(&ui.style());

            if let Some(opts) = options {
                validate_options(opts, FRAME_OPTIONS)?;
                // Tracker only, as in `frame`.
                let mut used = OptNames::new();
                built = apply_frame_options(built, opts, &mut used)?;
            }

            built.show(ui, |ui| run_nested_update_func_lossy(ui, contents.clone()));

            Ok(())
        }
    };
}

frame_preset!(
    frame_group,
    group,
    "A `Frame` with egui's `group` preset: a rounded, filled background. This \
     is what the existing `group` helper draws."
);
frame_preset!(
    frame_popup,
    popup,
    "A `Frame` with egui's `popup` preset, used for popups and tooltips."
);
frame_preset!(
    frame_menu,
    menu,
    "A `Frame` with egui's `menu` preset, used for menu backgrounds."
);
frame_preset!(
    frame_window,
    window,
    "A `Frame` with egui's `window` preset, matching a `Window`'s background."
);
frame_preset!(
    frame_canvas,
    canvas,
    "A `Frame` with egui's `canvas` preset, the flat surface behind a plot."
);
frame_preset!(
    frame_dark_canvas,
    dark_canvas,
    "A `Frame` with egui's `dark_canvas` preset: a dark canvas regardless of \
     theme, so plotted colours keep their meaning."
);
frame_preset!(
    frame_central_panel,
    central_panel,
    "A `Frame` with egui's `central_panel` preset."
);
frame_preset!(
    frame_side_top_panel,
    side_top_panel,
    "A `Frame` with egui's `side_top_panel` preset."
);

/// Builder options for `egui::SidePanel`, matching its setters exactly.
const SIDE_PANEL_OPTIONS: &[&str] = &[
    "resizable",
    "show_separator_line",
    "default_width",
    "min_width",
    "max_width",
    "width_range",
];

/// Builder options for `egui::TopBottomPanel`, matching its setters exactly.
const TOP_BOTTOM_PANEL_OPTIONS: &[&str] = &[
    "resizable",
    "show_separator_line",
    "default_height",
    "min_height",
    "max_height",
    "height_range",
];

/// Apply the shared `egui::SidePanel` setters.
///
/// egui's panel setters consume `self` and return `Self`, so the builder is
/// threaded through rather than mutated in place.
unsafe fn apply_side_panel_options(
    mut builder: egui::SidePanel,
    opts: &Bound<'_, PyDict>,
    used: &mut OptNames,
) -> PyResult<egui::SidePanel> {
    if let Some(v) = opt_bool(opts, "resizable", used)? {
        builder = builder.resizable(v);
    }
    if let Some(v) = opt_bool(opts, "show_separator_line", used)? {
        builder = builder.show_separator_line(v);
    }
    if let Some(v) = opt_f32(opts, "default_width", used)? {
        builder = builder.default_width(v);
    }
    if let Some(v) = opt_f32(opts, "min_width", used)? {
        builder = builder.min_width(v);
    }
    if let Some(v) = opt_f32(opts, "max_width", used)? {
        builder = builder.max_width(v);
    }
    if let Some(v) = opt_range(opts, "width_range", used)? {
        builder = builder.width_range(v);
    }
    Ok(builder)
}

/// Apply the shared `egui::TopBottomPanel` setters.
unsafe fn apply_top_bottom_panel_options(
    mut builder: egui::TopBottomPanel,
    opts: &Bound<'_, PyDict>,
    used: &mut OptNames,
) -> PyResult<egui::TopBottomPanel> {
    if let Some(v) = opt_bool(opts, "resizable", used)? {
        builder = builder.resizable(v);
    }
    if let Some(v) = opt_bool(opts, "show_separator_line", used)? {
        builder = builder.show_separator_line(v);
    }
    if let Some(v) = opt_f32(opts, "default_height", used)? {
        builder = builder.default_height(v);
    }
    if let Some(v) = opt_f32(opts, "min_height", used)? {
        builder = builder.min_height(v);
    }
    if let Some(v) = opt_f32(opts, "max_height", used)? {
        builder = builder.max_height(v);
    }
    if let Some(v) = opt_range(opts, "height_range", used)? {
        builder = builder.height_range(v);
    }
    Ok(builder)
}

/// Show a left side panel, mirroring `egui::SidePanel::left`.
///
/// Panels are independent containers, so an app composes them explicitly.
/// Draw the side, top and bottom panels before `central_panel`, which then takes
/// whatever space is left.
///
/// ```python
/// def update_func(ctx):
///     side_panel_left(ctx, "nav", nav_contents, default_width=180.0)
///     central_panel(ctx, main_contents)
/// ```
#[pyfunction]
#[pyo3(signature = (ctx, id, contents, **options))]
unsafe fn side_panel_left(
    ctx: &Context,
    id: &str,
    contents: Bound<'_, PyAny>,
    options: Option<&Bound<'_, PyDict>>,
) -> PyResult<()> {
    // These take `impl Into<Id>`, and egui has `From<String> for Id` as well
    // as `From<&'static str>`. Passing the owned String selects the former:
    // a borrowed `&str` would resolve to the 'static impl and fail to compile,
    // and leaking the String to get a 'static str would grow the heap on every
    // frame.
    let id = id.to_owned();

    let builder = match options {
        Some(opts) => {
            validate_options(opts, SIDE_PANEL_OPTIONS)?;
            // Tracker only: the panels are not §6 targets, and
            // `validate_options` already rejects anything unrecognised.
            let mut used = OptNames::new();
            apply_side_panel_options(egui::SidePanel::left(id), opts, &mut used)?
        }
        None => egui::SidePanel::left(id),
    };

    builder.show(&ctx.0, |ui| run_nested_update_func_lossy(ui, contents.clone()));

    Ok(())
}

/// Show a right side panel, mirroring `egui::SidePanel::right`.
#[pyfunction]
#[pyo3(signature = (ctx, id, contents, **options))]
unsafe fn side_panel_right(
    ctx: &Context,
    id: &str,
    contents: Bound<'_, PyAny>,
    options: Option<&Bound<'_, PyDict>>,
) -> PyResult<()> {
    // These take `impl Into<Id>`, and egui has `From<String> for Id` as well
    // as `From<&'static str>`. Passing the owned String selects the former:
    // a borrowed `&str` would resolve to the 'static impl and fail to compile,
    // and leaking the String to get a 'static str would grow the heap on every
    // frame.
    let id = id.to_owned();

    let builder = match options {
        Some(opts) => {
            validate_options(opts, SIDE_PANEL_OPTIONS)?;
            // Tracker only, as in `side_panel_left`.
            let mut used = OptNames::new();
            apply_side_panel_options(egui::SidePanel::right(id), opts, &mut used)?
        }
        None => egui::SidePanel::right(id),
    };

    builder.show(&ctx.0, |ui| run_nested_update_func_lossy(ui, contents.clone()));

    Ok(())
}

/// Show a top panel, mirroring `egui::TopBottomPanel::top`.
#[pyfunction]
#[pyo3(signature = (ctx, id, contents, **options))]
unsafe fn top_panel(
    ctx: &Context,
    id: &str,
    contents: Bound<'_, PyAny>,
    options: Option<&Bound<'_, PyDict>>,
) -> PyResult<()> {
    // See side_panel_left: pass an owned String so `impl Into<Id>` resolves to
    // egui's `From<String>` rather than the `&'static str` impl.
    let id = id.to_owned();

    let builder = match options {
        Some(opts) => {
            validate_options(opts, TOP_BOTTOM_PANEL_OPTIONS)?;
            // Tracker only, as in `side_panel_left`.
            let mut used = OptNames::new();
            apply_top_bottom_panel_options(egui::TopBottomPanel::top(id), opts, &mut used)?
        }
        None => egui::TopBottomPanel::top(id),
    };

    builder.show(&ctx.0, |ui| run_nested_update_func_lossy(ui, contents.clone()));

    Ok(())
}

/// Show a bottom panel, mirroring `egui::TopBottomPanel::bottom`.
#[pyfunction]
#[pyo3(signature = (ctx, id, contents, **options))]
unsafe fn bottom_panel(
    ctx: &Context,
    id: &str,
    contents: Bound<'_, PyAny>,
    options: Option<&Bound<'_, PyDict>>,
) -> PyResult<()> {
    // See side_panel_left: pass an owned String so `impl Into<Id>` resolves to
    // egui's `From<String>` rather than the `&'static str` impl.
    let id = id.to_owned();

    let builder = match options {
        Some(opts) => {
            validate_options(opts, TOP_BOTTOM_PANEL_OPTIONS)?;
            // Tracker only, as in `side_panel_left`.
            let mut used = OptNames::new();
            apply_top_bottom_panel_options(egui::TopBottomPanel::bottom(id), opts, &mut used)?
        }
        None => egui::TopBottomPanel::bottom(id),
    };

    builder.show(&ctx.0, |ui| run_nested_update_func_lossy(ui, contents.clone()));

    Ok(())
}

/// Builder options for `egui::Modal`, matching its setters exactly.
const MODAL_OPTIONS: &[&str] = &["default_width", "default_height"];

/// Show a modal dialog, mirroring `egui::Modal`.
///
/// Returns whether the modal is still open, so a click on the backdrop (or the
/// close button) closes it. egui keeps only one modal open at a time, and this
/// one floats above every other container.
///
/// ```python
/// def update_func(ctx):
///     if button_clicked("Open"):
///         show_modal.value = True
///     if show_modal.value:
///         if modal(ctx, "confirm", modal_contents, default_width=320.0):
///             show_modal.value = False
///     central_panel(ctx, main_contents)
/// ```
///
/// Note that egui wants modals drawn before the central panel, so that the
/// central panel does not cover the backdrop.
#[pyfunction]
#[pyo3(signature = (ctx, id, contents, **options))]
unsafe fn modal(
    ctx: &Context,
    id: &str,
    contents: Bound<'_, PyAny>,
    options: Option<&Bound<'_, PyDict>>,
) -> PyResult<bool> {
    let ctx = &ctx.0;

    // `Modal::new` takes an `Id` concretely rather than `impl Into<Id>`, so
    // `Id::new` is used instead of the From<String> trick the panels need.
    let mut builder = egui::Modal::new(egui::Id::new(id));

    if let Some(opts) = options {
        validate_options(opts, MODAL_OPTIONS)?;
        // egui exposes the modal's size through its Area.
        let mut area =
            std::mem::replace(&mut builder, egui::Modal::new(egui::Id::new(""))).area;
        // Tracker only: `modal` is not a §6 target, and `validate_options`
        // already rejects anything outside MODAL_OPTIONS.
        let mut used = OptNames::new();
        if let Some(v) = opt_f32(opts, "default_width", &mut used)? {
            area = area.default_width(v);
        }
        if let Some(v) = opt_f32(opts, "default_height", &mut used)? {
            area = area.default_height(v);
        }
        builder = egui::Modal::new(egui::Id::new("")).area(area);
    }

    let response = builder.show(ctx, |ui| run_nested_update_func_lossy(ui, contents.clone()));

    // Returns whether the modal asked to be dismissed, i.e. whether the user
    // clicked the backdrop.
    //
    // NOT `is_top_modal`: that is "am I the topmost modal", which a lone modal
    // answers True on every frame. Returning it made the modal close itself
    // immediately, so the overlays screenshot showed no modal at all while the
    // render check still passed, since the rest of the page drew fine.
    Ok(response.backdrop_response.clicked())
}

/// Builder options for `egui::Resize`, matching its setters exactly.
const RESIZE_OPTIONS: &[&str] = &[
    "default_width",
    "default_height",
    "default_size",
    "min_size",
    "min_width",
    "min_height",
    "max_size",
    "max_width",
    "max_height",
    "resizable",
    "auto_sized",
    "fixed_size",
];

/// Show a resizable area, mirroring `egui::Resize`.
///
/// Unlike `central_panel` and `modal`, this is not a top-level container: egui's
/// `Resize::show` takes a `&mut Ui`, so it must be called from inside something
/// else -- a central panel, a window, or a nested update function.
///
/// ```python
/// def contents():
///     resize(lambda: heading("Drag my corner"), default_width=240.0)
/// ```
#[pyfunction]
#[pyo3(signature = (contents, **options))]
unsafe fn resize(
    contents: Bound<'_, PyAny>,
    options: Option<&Bound<'_, PyDict>>,
) -> PyResult<()> {
    // Resize needs an existing Ui; there is no Context path in egui for it.
    let ui = current_ui(&UI)?;

    let mut builder = egui::Resize::default();

    if let Some(opts) = options {
        validate_options(opts, RESIZE_OPTIONS)?;
        // Tracker only: `resize` is not a §6 target, and `validate_options`
        // already rejects anything outside RESIZE_OPTIONS.
        let mut used = OptNames::new();
        if let Some(v) = opt_f32(opts, "default_width", &mut used)? {
            builder = builder.default_width(v);
        }
        if let Some(v) = opt_f32(opts, "default_height", &mut used)? {
            builder = builder.default_height(v);
        }
        if let Some(v) = opt_vec2(opts, "default_size", &mut used)? {
            builder = builder.default_size(v);
        }
        if let Some(v) = opt_vec2(opts, "min_size", &mut used)? {
            builder = builder.min_size(v);
        }
        if let Some(v) = opt_f32(opts, "min_width", &mut used)? {
            builder = builder.min_width(v);
        }
        if let Some(v) = opt_f32(opts, "min_height", &mut used)? {
            builder = builder.min_height(v);
        }
        if let Some(v) = opt_vec2(opts, "max_size", &mut used)? {
            builder = builder.max_size(v);
        }
        if let Some(v) = opt_f32(opts, "max_width", &mut used)? {
            builder = builder.max_width(v);
        }
        if let Some(v) = opt_f32(opts, "max_height", &mut used)? {
            builder = builder.max_height(v);
        }
        if let Some(v) = opt_bool(opts, "resizable", &mut used)? {
            // egui 0.31.1's Vec2b has `new(x, y)` and no `splat`.
            builder = builder.resizable(egui::Vec2b::new(v, v));
        }
        if let Some(v) = opt_bool(opts, "auto_sized", &mut used)? {
            if v {
                builder = builder.auto_sized();
            }
        }
        if let Some(v) = opt_vec2(opts, "fixed_size", &mut used)? {
            builder = builder.fixed_size(v);
        }
    }

    builder.show(ui, |ui| run_nested_update_func_lossy(ui, contents.clone()));

    Ok(())
}

/// Builder options for `egui::Area`, matching its setters exactly.
const AREA_OPTIONS: &[&str] = &[
    "enabled",
    "movable",
    "interactable",
    "order",
    "default_pos",
    "default_size",
    "default_width",
    "default_height",
    "fixed_pos",
    "constrain",
    "fade_in",
];

/// Show a free-floating area, mirroring `egui::Area`.
///
/// An area is not tied to a panel and has no frame of its own by default, so it
/// is usually combined with a `Frame`. egui stores an area's position between
/// frames, so `movable=True` lets the user drag it.
///
/// ```python
/// def update_func(ctx):
///     area(ctx, lambda: label("Drag me"), default_pos=(40.0, 40.0), movable=True)
///     central_panel(ctx, main_contents)
/// ```
#[pyfunction]
#[pyo3(signature = (ctx, id, contents, **options))]
unsafe fn area(
    ctx: &Context,
    id: &str,
    contents: Bound<'_, PyAny>,
    options: Option<&Bound<'_, PyDict>>,
) -> PyResult<()> {
    let ctx = &ctx.0;
    // `Area::new` takes an `Id` concretely, unlike `SidePanel::left`
    // which takes `impl Into<Id>` -- so From<String> does not apply here.
    let mut builder = egui::Area::new(egui::Id::new(id));

    if let Some(opts) = options {
        validate_options(opts, AREA_OPTIONS)?;
        // Tracker only: `area` is not a §6 target, and `validate_options`
        // already rejects anything outside AREA_OPTIONS.
        let mut used = OptNames::new();
        if let Some(v) = opt_bool(opts, "enabled", &mut used)? {
            builder = builder.enabled(v);
        }
        if let Some(v) = opt_bool(opts, "movable", &mut used)? {
            builder = builder.movable(v);
        }
        if let Some(v) = opt_bool(opts, "interactable", &mut used)? {
            builder = builder.interactable(v);
        }
        if let Some(v) = opt_order(opts, &mut used)? {
            builder = builder.order(v);
        }
        if let Some(v) = opt_vec2(opts, "default_pos", &mut used)? {
            // `Pos2` is not `From<Vec2>`; window converts the same way.
            builder = builder.default_pos(egui::Pos2::new(v.x, v.y));
        }
        if let Some(v) = opt_vec2(opts, "default_size", &mut used)? {
            builder = builder.default_size(v);
        }
        if let Some(v) = opt_f32(opts, "default_width", &mut used)? {
            builder = builder.default_width(v);
        }
        if let Some(v) = opt_f32(opts, "default_height", &mut used)? {
            builder = builder.default_height(v);
        }
        if let Some(v) = opt_vec2(opts, "fixed_pos", &mut used)? {
            // `Pos2` is not `From<Vec2>`; window converts the same way.
            builder = builder.fixed_pos(egui::Pos2::new(v.x, v.y));
        }
        if let Some(v) = opt_bool(opts, "constrain", &mut used)? {
            builder = builder.constrain(v);
        }
        if let Some(v) = opt_bool(opts, "fade_in", &mut used)? {
            builder = builder.fade_in(v);
        }
    }

    builder.show(&ctx, |ui| run_nested_update_func_lossy(ui, contents.clone()));

    Ok(())
}

#[pyfunction]
unsafe fn central_panel(ctx: &Context, contents: Bound<'_, PyAny>) -> PyResult<()> {
    let ctx = &ctx.0;

    egui::CentralPanel::default().show(ctx, |ui| {
        run_nested_update_func_lossy(ui, contents.clone())
    });

    Ok(())
}

/// Start a ui with horizontal layout. After you have called this, the function registers the contents as any other widget.
///
/// Elements will be centered on the Y axis, i.e. adjusted up and down to lie in the center of the horizontal layout. The initial height is style.spacing.interact_size.y. Centering is almost always what you want if you are planning to mix widgets or use different types of text.
///
/// If you don’t want the contents to be centered, use horizontal_top instead.
///
/// Example::
///
///     def horizontal_update_func():
///       heading("I'm horizontal")
///     
///     horizontal(horizontal_update_func)
#[pyfunction]
unsafe fn horizontal(update_fun: Bound<'_, PyAny>) -> PyResult<()> {
    current_ui(&UI)?
        .horizontal(|ui| run_nested_update_func(ui, update_fun))
        .inner
}

/// Like horizontal, but allocates the full vertical height and then centers elements vertically.
#[pyfunction]
unsafe fn horizontal_centered(update_fun: Bound<'_, PyAny>) -> PyResult<()> {
    current_ui(&UI)?
        .horizontal_centered(|ui| run_nested_update_func(ui, update_fun))
        .inner
}
/// Like horizontal, but aligns content with top.
#[pyfunction]
unsafe fn horizontal_top(update_fun: Bound<'_, PyAny>) -> PyResult<()> {
    current_ui(&UI)?
        .horizontal_top(|ui| run_nested_update_func(ui, update_fun))
        .inner
}

/// Start a ui with horizontal layout that wraps to a new row when it reaches the right edge of the max_size. After you have called this, the function registers the contents as any other widget.
///
/// Elements will be centered on the Y axis, i.e. adjusted up and down to lie in the center of the horizontal layout. The initial height is style.spacing.interact_size.y. Centering is almost always what you want if you are planning to mix widgets or use different types of text.
#[pyfunction]
unsafe fn horizontal_wrapped(update_fun: Bound<'_, PyAny>) -> PyResult<()> {
    current_ui(&UI)?
        .horizontal_wrapped(|ui| run_nested_update_func(ui, update_fun))
        .inner
}

/// Start a ui with vertical layout. Widgets will be left-justified.
#[pyfunction]
unsafe fn vertical(update_fun: Bound<'_, PyAny>) -> PyResult<()> {
    current_ui(&UI)?
        .vertical(|ui| run_nested_update_func(ui, update_fun))
        .inner
}

/// Start a ui with vertical layout. Widgets will be horizontally centered.
#[pyfunction]
unsafe fn vertical_centered(update_fun: Bound<'_, PyAny>) -> PyResult<()> {
    current_ui(&UI)?
        .vertical_centered(|ui| run_nested_update_func(ui, update_fun))
        .inner
}

/// Start a ui with vertical layout. Widgets will be horizontally centered and justified (fill full width).
#[pyfunction]
unsafe fn vertical_centered_justified(update_fun: Bound<'_, PyAny>) -> PyResult<()> {
    current_ui(&UI)?
        .vertical_centered_justified(|ui| run_nested_update_func(ui, update_fun))
        .inner
}

/// This will make the next added widget centered and justified in the available space.
///
/// Only one widget may be added inside update_func!
#[pyfunction]
unsafe fn centered_and_justified(update_fun: Bound<'_, PyAny>) -> PyResult<()> {
    current_ui(&UI)?
        .centered_and_justified(|ui| run_nested_update_func(ui, update_fun))
        .inner
}

/// Builder options for `egui::CollapsingHeader`, matching its setters exactly.
const COLLAPSING_OPTIONS: &[&str] = &[
    "default_open",
    "open",
    "id_salt",
    "id_source",
    "enabled",
    "show_background",
];

/// A `CollapsingHeader`, mirroring `egui::CollapsingHeader`.
///
/// The existing `collapsing` helper always starts closed. This takes egui's
/// builder options, so a header can start open (`default_open=True`), be framed
/// (`show_background=True`), or be driven by a `Bool` the user toggles.
///
/// Returns the header's `Response` -- the clickable header itself. egui's own
/// `CollapsingResponse` is not a `Response` and is not exposed as a type; its
/// `openness` is 1.0 when fully open and 0.0 when fully closed, and is folded
/// into the `Bool` when one is passed as `open`.
///
/// `open` is a `Bool` that egui reads and writes: pass one and clicking the
/// header arrow drives it.
///
/// Example::
///
///     def body():
///       label("hidden until the header is opened")
///
///     collapsing_response("Details", body, default_open=True)
#[pyfunction]
#[pyo3(signature = (heading, update_fun, open=None, **options))]
unsafe fn collapsing_response(
    heading: &str,
    update_fun: Bound<'_, PyAny>,
    mut open: Option<&mut Bool>,
    options: Option<&Bound<'_, PyDict>>,
) -> PyResult<Response> {
    let ui = current_ui(&UI)?;

    // Own the heading text: the builder holds a `WidgetText` that would
    // otherwise borrow the Python string for as long as the builder lives.
    let heading = heading.to_owned();
    let mut builder = egui::CollapsingHeader::new(heading);

    if let Some(opts) = options {
        validate_options(opts, COLLAPSING_OPTIONS)?;
        // Tracker only: `collapsing` is not a §6 target, and
        // `validate_options` already rejects anything outside
        // COLLAPSING_OPTIONS.
        let mut used = OptNames::new();
        if let Some(v) = opt_bool(opts, "default_open", &mut used)? {
            builder = builder.default_open(v);
        }
        if let Some(v) = opt_bool(opts, "enabled", &mut used)? {
            builder = builder.enabled(v);
        }
        if let Some(v) = opt_bool(opts, "show_background", &mut used)? {
            builder = builder.show_background(v);
        }
        // egui's own alias pair; both hash a str directly. Recorded by hand:
        // `reject_unknown_options` would read the untouched `get_item` as an
        // unknown option, so this insert is required by that check.
        used.insert("id_salt".to_string());
        used.insert("id_source".to_string());
        for name in ["id_salt", "id_source"] {
            if let Some(value) = opts.get_item(name)? {
                let salt: String = value
                    .extract()
                    .map_err(|_| PyValueError::new_err(format!("{name} must be a string")))?;
                builder = builder.id_salt(salt);
            }
        }
    }

    // `CollapsingHeader::open` takes `Option<bool>`: Some drives the header
    // from the caller's value and reports the user's choice back through
    // `openness`. Passing Some always, when a Bool was supplied, is what makes
    // the arrow actually move the Bool.
    if let Some(open) = open.as_deref_mut() {
        builder = builder.open(Some(open.value));
    }

    let response = builder.show(ui, |ui| run_nested_update_func_lossy(ui, update_fun.clone()));

    // `openness` is 1.0 fully open, 0.0 fully closed, and in between while
    // animating. Treating anything above half-open as open is what the arrow
    // shows, and storing the raw float would put a number in a Bool.
    if let Some(open) = open.as_deref_mut() {
        open.value = response.openness > 0.5;
    }

    Ok(Response {
        inner: response.header_response,
    })
}

/// A `CollapsingHeader` that starts out collapsed.
///
/// Example::
///
///     def update_func():
///       heading("hi")
///       collapsing("collapsed", update_func)
#[pyfunction]
#[pyo3(signature = (heading, update_fun, **options))]
unsafe fn collapsing(
    heading: &str,
    update_fun: Bound<'_, PyAny>,
    options: Option<&Bound<'_, PyDict>>,
) -> PyResult<()> {
    collapsing_response(heading, update_fun, None, options)?;
    Ok(())
}

/// Keyword arguments `menu_button` accepts.
///
/// Empty by design: egui 0.31.1's `menu_button` takes no builder options. The
/// list exists so that an unknown keyword is reported by name, rather than
/// being silently accepted and dropped -- which is the failure this whole
/// option parser exists to prevent.
const MENU_BUTTON_OPTIONS: &[&str] = &[];

/// A menu button that opens a popup menu when clicked, mirroring
/// `Ui::menu_button`.
///
/// egui 0.31.1 has no menu-bar container; this is the whole menu surface. A
/// `menu_button` inside a `menu_button` is a submenu, so nesting these builds
/// one. `close_menu` closes the innermost menu, which is how a menu item acts
/// as a button.
///
/// Returns the button's `Response`, so a menu item that is itself a button can
/// be written as `menu_button(...)` wrapping `button_clicked(...)`.
///
/// Example::
///
///     def file_menu():
///       if button_clicked("New"):
///         print("new")
///       close_menu()
///
///     def main():
///       menu_button("File", file_menu)
#[pyfunction]
#[pyo3(signature = (text, contents, **options))]
unsafe fn menu_button(
    text: &str,
    contents: Bound<'_, PyAny>,
    options: Option<&Bound<'_, PyDict>>,
) -> PyResult<Response> {
    let ui = current_ui(&UI)?;

    if let Some(opts) = options {
        validate_options(opts, MENU_BUTTON_OPTIONS)?;
    }

    Ok(Response {
        inner: ui
            .menu_button(text.to_owned(), |ui| {
                run_nested_update_func_lossy(ui, contents.clone())
            })
            .response,
    })
}

/// A menu button with an image to the left of the text, mirroring
/// `Ui::menu_image_button`.
///
/// `source` is a URI, exactly as for `image`.
///
/// Example::
///
///     menu_image_button("file://icon.png", lambda: label("Open"))
#[pyfunction]
#[pyo3(signature = (source, contents))]
unsafe fn menu_image_button(source: &str, contents: Bound<'_, PyAny>) -> PyResult<Response> {
    let ui = current_ui(&UI)?;

    Ok(Response {
        inner: ui
            .menu_image_button(source, |ui| {
                run_nested_update_func_lossy(ui, contents.clone())
            })
            .response,
    })
}

/// A menu button with an image followed by text, mirroring
/// `Ui::menu_image_text_button`.
///
/// `source` is a URI, exactly as for `image`.
///
/// Example::
///
///     menu_image_text_button("file://icon.png", "Open", lambda: label("hi"))
#[pyfunction]
#[pyo3(signature = (source, text, contents))]
unsafe fn menu_image_text_button(
    source: &str,
    text: &str,
    contents: Bound<'_, PyAny>,
) -> PyResult<Response> {
    let ui = current_ui(&UI)?;

    Ok(Response {
        inner: ui
            .menu_image_text_button(source, text.to_owned(), |ui| {
                run_nested_update_func_lossy(ui, contents.clone())
            })
            .response,
    })
}

/// A rectangle, mirroring `egui::Rect`.
///
/// egui's is two corners (`min` and `max`), not a position and a size. This is
/// the value `scene` reads and writes between frames: egui mutates it as the
/// user pans and zooms, so it has to survive across frames.
///
/// Usage::
///
///     view = Rect((0.0, 0.0), (1000.0, 1000.0))
#[pyclass]
#[derive(Clone, Copy)]
struct Rect {
    #[pyo3(get, set)]
    min_x: f32,
    #[pyo3(get, set)]
    min_y: f32,
    #[pyo3(get, set)]
    max_x: f32,
    #[pyo3(get, set)]
    max_y: f32,
}

#[pymethods]
impl Rect {
    /// A rectangle from its minimum corner and its size.
    ///
    /// This is how a scene is usually set up: the region of content to show.
    #[new]
    #[pyo3(signature = (min=(0.0, 0.0), size=(0.0, 0.0)))]
    fn new(min: (f32, f32), size: (f32, f32)) -> Self {
        Rect {
            min_x: min.0,
            min_y: min.1,
            max_x: min.0 + size.0,
            max_y: min.1 + size.1,
        }
    }

    /// Build from a minimum corner and a maximum corner, egui's own `Rect`.
    #[staticmethod]
    fn from_corners(min: (f32, f32), max: (f32, f32)) -> Self {
        Rect {
            min_x: min.0,
            min_y: min.1,
            max_x: max.0,
            max_y: max.1,
        }
    }

    /// egui's `Rect::ZERO`, which `scene` treats as "no view yet" and resets.
    #[staticmethod]
    fn zero() -> Self {
        Rect {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 0.0,
            max_y: 0.0,
        }
    }

    #[getter]
    fn min(&self) -> (f32, f32) {
        (self.min_x, self.min_y)
    }

    #[getter]
    fn max(&self) -> (f32, f32) {
        (self.max_x, self.max_y)
    }

    #[getter]
    fn size(&self) -> (f32, f32) {
        (self.max_x - self.min_x, self.max_y - self.min_y)
    }

    #[getter]
    fn width(&self) -> f32 {
        self.max_x - self.min_x
    }

    #[getter]
    fn height(&self) -> f32 {
        self.max_y - self.min_y
    }

    #[getter]
    fn center(&self) -> (f32, f32) {
        (
            (self.min_x + self.max_x) / 2.0,
            (self.min_y + self.max_y) / 2.0,
        )
    }

    /// egui's `is_finite`. `scene` uses this to decide whether the stored view
    /// is usable or needs resetting.
    fn is_finite(&self) -> bool {
        self.min_x.is_finite()
            && self.min_y.is_finite()
            && self.max_x.is_finite()
            && self.max_y.is_finite()
    }

    fn __repr__(&self) -> String {
        format!(
            "Rect(min=({:.1}, {:.1}), max=({:.1}, {:.1}))",
            self.min_x, self.min_y, self.max_x, self.max_y
        )
    }
}

impl From<&Rect> for egui::Rect {
    fn from(rect: &Rect) -> Self {
        egui::Rect::from_min_max(
            egui::Pos2::new(rect.min_x, rect.min_y),
            egui::Pos2::new(rect.max_x, rect.max_y),
        )
    }
}

impl From<&egui::Rect> for Rect {
    /// egui's rects become pyegui ones by value, since the measurement
    /// functions hand back a fresh rect rather than borrowed storage.
    fn from(rect: &egui::Rect) -> Self {
        Rect {
            min_x: rect.min.x,
            min_y: rect.min.y,
            max_x: rect.max.x,
            max_y: rect.max.y,
        }
    }
}

/// Builder options for `egui::Scene`, matching its setters exactly.
const SCENE_OPTIONS: &[&str] = &["zoom_range", "max_inner_size"];

/// A pan-and-zoom canvas, mirroring `egui::Scene`.
///
/// A scene takes over the space its parent `Ui` has left, and lets the user drag
/// to pan and scroll to zoom. The visible region is a `Rect` that egui reads
/// and writes as the user interacts, so it has to be the same object every
/// frame -- pass the same one each time, not a fresh one.
///
/// Start it at `Rect.zero()` and egui will fit the contents on the first frame;
/// or give it a region up front with `Rect((0.0, 0.0), (w, h))`.
///
/// `zoom_range` is egui's `(min, max)`. The default, `(0.0, 1.0)`, allows
/// zooming out arbitrarily but not in past 1:1; pass something like
/// `(0.0, float("inf"))` to allow zooming in. Text gets blurry when zoomed in
/// past 1:1 -- egui issue 4813.
///
/// Example::
///
///     view = Rect.zero()
///     zoom = Float(1.0)
///
///     def contents():
///         label("drag to pan, scroll to zoom")
///         label(f"zoom {zoom.value:.2f}")
///
///     def main():
///         scene(contents, view, zoom_range=(0.1, 4.0))
#[pyfunction]
#[pyo3(signature = (contents, view, **options))]
unsafe fn scene(
    contents: Bound<'_, PyAny>,
    view: &mut Rect,
    options: Option<&Bound<'_, PyDict>>,
) -> PyResult<Response> {
    let ui = current_ui(&UI)?;

    let mut builder = egui::Scene::new();

    if let Some(opts) = options {
        validate_options(opts, SCENE_OPTIONS)?;
        // Tracker only: `scene` is not a §6 target, and `validate_options`
        // already rejects anything outside SCENE_OPTIONS.
        let mut used = OptNames::new();
        if let Some(v) = opt_range(opts, "zoom_range", &mut used)? {
            builder = builder.zoom_range(v);
        }
        if let Some(v) = opt_vec2(opts, "max_inner_size", &mut used)? {
            builder = builder.max_inner_size(v);
        }
    }

    // egui mutates the rect as the user pans and zooms, and reads it to decide
    // what to show. Converting in, then writing the result back out, keeps the
    // Python-side object as the single source of truth across frames.
    //
    // `egui::Rect::from(view)` does not compile: the impl below is for `&Rect`,
    // and trait resolution does not reborrow a `&mut Rect` to `&Rect` on its
    // own. Naming the type makes it unambiguous.
    let mut inner: egui::Rect = egui::Rect::from(&*view);

    let shown = builder.show(ui, &mut inner, |ui| {
        run_nested_update_func_lossy(ui, contents.clone())
    });

    view.min_x = inner.min.x;
    view.min_y = inner.min.y;
    view.max_x = inner.max.x;
    view.max_y = inner.max.y;

    Ok(Response {
        inner: shown.response,
    })
}

// ------------------------------------------------------- layout and sizing
//
// egui's sizing calls all take one value and return nothing: they set the size
// of the Ui they are called on, for the rest of that Ui's life. There is no
// builder to chain, so each is a separate function rather than keyword options
// on a container.
//
// They apply to the *current* Ui, which is whatever container most recently
// pushed onto the stack. Inside a callback that is the callback's own Ui.
//
// `set_width` fixes the width; `set_width_range` bounds it. Setting both is
// allowed and egui honours the tighter of the two, which is why the range
// variants exist rather than a min/max pair.

/// Fix the width of the current `Ui`, mirroring `Ui::set_width`.
#[pyfunction]
unsafe fn set_width(width: f32) -> PyResult<()> {
    current_ui(&UI)?.set_width(width);
    Ok(())
}

/// Fix the height of the current `Ui`, mirroring `Ui::set_height`.
#[pyfunction]
unsafe fn set_height(height: f32) -> PyResult<()> {
    current_ui(&UI)?.set_height(height);
    Ok(())
}

/// Set the minimum width of the current `Ui`, mirroring `Ui::set_min_width`.
#[pyfunction]
unsafe fn set_min_width(width: f32) -> PyResult<()> {
    current_ui(&UI)?.set_min_width(width);
    Ok(())
}

/// Set the maximum width of the current `Ui`, mirroring `Ui::set_max_width`.
#[pyfunction]
unsafe fn set_max_width(width: f32) -> PyResult<()> {
    current_ui(&UI)?.set_max_width(width);
    Ok(())
}

/// Set the minimum height of the current `Ui`, mirroring `Ui::set_min_height`.
#[pyfunction]
unsafe fn set_min_height(height: f32) -> PyResult<()> {
    current_ui(&UI)?.set_min_height(height);
    Ok(())
}

/// Set the maximum height of the current `Ui`, mirroring `Ui::set_max_height`.
#[pyfunction]
unsafe fn set_max_height(height: f32) -> PyResult<()> {
    current_ui(&UI)?.set_max_height(height);
    Ok(())
}

/// Set the minimum size of the current `Ui`, mirroring `Ui::set_min_size`.
///
/// Takes a 2-sequence, e.g. `set_min_size((100.0, 50.0))`.
#[pyfunction]
unsafe fn set_min_size(size: (f32, f32)) -> PyResult<()> {
    current_ui(&UI)?.set_min_size(egui::vec2(size.0, size.1));
    Ok(())
}

/// Set the maximum size of the current `Ui`, mirroring `Ui::set_max_size`.
///
/// Takes a 2-sequence, e.g. `set_max_size((400.0, 300.0))`.
#[pyfunction]
unsafe fn set_max_size(size: (f32, f32)) -> PyResult<()> {
    current_ui(&UI)?.set_max_size(egui::vec2(size.0, size.1));
    Ok(())
}

/// Bound the width of the current `Ui`, mirroring `Ui::set_width_range`.
///
/// Takes a `(min, max)` tuple, e.g. `set_width_range((100.0, 300.0))`.
#[pyfunction]
unsafe fn set_width_range(r: (f32, f32)) -> PyResult<()> {
    current_ui(&UI)?.set_width_range(egui::Rangef::new(r.0, r.1));
    Ok(())
}

/// Bound the height of the current `Ui`, mirroring `Ui::set_height_range`.
///
/// Takes a `(min, max)` tuple, e.g. `set_height_range((50.0, 200.0))`.
#[pyfunction]
unsafe fn set_height_range(r: (f32, f32)) -> PyResult<()> {
    current_ui(&UI)?.set_height_range(egui::Rangef::new(r.0, r.1));
    Ok(())
}

/// Shrink the current `Ui` to its content's width, mirroring
/// `Ui::shrink_width_to_current`.
///
/// The counterpart to `set_width`: rather than fixing the width, this lets the
/// Ui take exactly what it needs.
#[pyfunction]
unsafe fn shrink_width_to_current() -> PyResult<()> {
    current_ui(&UI)?.shrink_width_to_current();
    Ok(())
}

/// Shrink the current `Ui` to its content's height, mirroring
/// `Ui::shrink_height_to_current`.
#[pyfunction]
unsafe fn shrink_height_to_current() -> PyResult<()> {
    current_ui(&UI)?.shrink_height_to_current();
    Ok(())
}

// ---------------------------------------------------- multi-column layout
//
// egui's `columns` hands the callback a slice of Uis, one per column, rather
// than a single one. That is the shape that makes columns work: each column is
// its own Ui with its own cursor, so widgets in one do not affect the others.
// A Python callback cannot receive a slice of Uis as a slice, so `columns`
// takes a list of callables instead -- one per column -- and each is run with
// its own Ui. That is the shape pyegui uses everywhere else: a callable per
// container rather than a struct the caller has to learn.
//
//     columns(3, [col_a, col_b, col_c])
//
// Use `end_row` to close a row early and start the next, exactly as egui does.

/// Show `num_columns` side by side, mirroring `Ui::columns`.
///
/// `contents` is a list of callables, one per column, each run in its own Ui.
/// A column may be `None` to leave it empty.
///
/// Example::
///
///     columns(3, [
///         lambda: heading("left"),
///         lambda: heading("middle"),
///         lambda: heading("right"),
///     ])
#[pyfunction]
#[pyo3(signature = (num_columns, contents))]
unsafe fn columns(num_columns: usize, contents: Vec<Bound<'_, PyAny>>) -> PyResult<()> {
    let ui = current_ui(&UI)?;

    if contents.len() != num_columns {
        return Err(PyValueError::new_err(format!(
            "columns() was given {} columns but {} callables; \
             one callable per column, or None for an empty column",
            num_columns,
            contents.len()
        )));
    }

    // egui's callback wants `&mut [Ui]`. The Uis are created inside egui's own
    // call, so they cannot exist before it runs; a raw pointer per column is
    // borrowed through the Ui stack for the duration, which is how every other
    // nested container in this file already reaches a Ui it did not create.
    let mut slots: Vec<*mut egui::Ui> = vec![std::ptr::null_mut(); num_columns];

    ui.columns(num_columns, |column_uis| {
        for (slot, column) in slots.iter_mut().zip(column_uis.iter_mut()) {
            *slot = column as *mut egui::Ui;
        }

        for (index, content) in contents.iter().enumerate() {
            let column_ui = slots[index];
            if column_ui.is_null() {
                continue;
            }
            let column_ui = unsafe { &mut *column_ui };
            run_nested_update_func_lossy(column_ui, content.clone());
        }
    });

    Ok(())
}

/// Close the current column and move to the next row, mirroring `Ui::end_row`.
///
/// Only meaningful inside `columns`. egui wraps automatically when the last
/// column fills, so this is only needed to start a new row early.
#[pyfunction]
unsafe fn end_row() -> PyResult<()> {
    current_ui(&UI)?.end_row();
    Ok(())
}

/// Set the height of the current column, mirroring `Ui::set_row_height`.
///
/// egui sizes columns from their tallest sibling, so this is how a row of
/// columns is given a uniform height.
#[pyfunction]
unsafe fn set_row_height(height: f32) -> PyResult<()> {
    current_ui(&UI)?.set_row_height(height);
    Ok(())
}

/// The size still available in the current `Ui`, mirroring
/// `Ui::available_size`.
///
/// Returns a 2-tuple, `(width, height)`.
#[pyfunction]
unsafe fn available_size() -> PyResult<(f32, f32)> {
    let size = current_ui(&UI)?.available_size();
    Ok((size.x, size.y))
}

/// The width still available in the current `Ui`, mirroring
/// `Ui::available_width`.
#[pyfunction]
unsafe fn available_width() -> PyResult<f32> {
    Ok(current_ui(&UI)?.available_width())
}

/// The height still available in the current `Ui`, mirroring
/// `Ui::available_height`.
#[pyfunction]
unsafe fn available_height() -> PyResult<f32> {
    Ok(current_ui(&UI)?.available_height())
}

/// The size available before wrapping, mirroring
/// `Ui::available_size_before_wrap`.
///
/// Differs from `available_size` when the layout wraps: this is the width the
/// Ui had before text wrapping was applied.
#[pyfunction]
unsafe fn available_size_before_wrap() -> PyResult<(f32, f32)> {
    let size = current_ui(&UI)?.available_size_before_wrap();
    Ok((size.x, size.y))
}

/// The area available before wrapping, mirroring
/// `Ui::available_rect_before_wrap`.
#[pyfunction]
unsafe fn available_rect_before_wrap() -> PyResult<Rect> {
    let rect = current_ui(&UI)?.available_rect_before_wrap();
    Ok(Rect::from(&rect))
}

/// The cursor's rectangle, mirroring `Ui::cursor`.
///
/// This is where the next widget will be placed. Useful for custom painting and
/// for positioning something relative to the current line.
#[pyfunction]
unsafe fn cursor() -> PyResult<Rect> {
    let rect = current_ui(&UI)?.cursor();
    Ok(Rect::from(&rect))
}

/// The smallest rectangle containing everything drawn so far, mirroring
/// `Ui::min_rect`.
#[pyfunction]
unsafe fn min_rect() -> PyResult<Rect> {
    let rect = current_ui(&UI)?.min_rect();
    Ok(Rect::from(&rect))
}

/// The largest rectangle the current `Ui` may use, mirroring `Ui::max_rect`.
///
/// This is what `set_width`, `set_max_size` and the panels all bound
/// themselves by.
#[pyfunction]
unsafe fn max_rect() -> PyResult<Rect> {
    let rect = current_ui(&UI)?.max_rect();
    Ok(Rect::from(&rect))
}

/// The size of everything drawn so far, mirroring `Ui::min_size`.
#[pyfunction]
unsafe fn min_size() -> PyResult<(f32, f32)> {
    let size = current_ui(&UI)?.min_size();
    Ok((size.x, size.y))
}

/// The scale factor between egui points and physical pixels, mirroring
/// `Ui::pixels_per_point`.
///
/// Widget positions and sizes from the other measurement functions are in
/// points; multiply by this to get pixels.
#[pyfunction]
unsafe fn pixels_per_point() -> PyResult<f32> {
    Ok(current_ui(&UI)?.pixels_per_point())
}

/// Where the next widget will be placed, mirroring
/// `Ui::next_widget_position`.
///
/// Returns a 2-tuple, `(x, y)`.
#[pyfunction]
unsafe fn next_widget_position() -> PyResult<(f32, f32)> {
    let pos = current_ui(&UI)?.next_widget_position();
    Ok((pos.x, pos.y))
}

/// Whether a rectangle is inside the current `Ui` and not clipped away, mirroring
/// `Ui::is_rect_visible`.
///
/// Takes a 2-sequence of four numbers, `(min_x, min_y, max_x, max_y)`, or a
/// `Rect`.
#[pyfunction]
unsafe fn is_rect_visible(rect: Rect) -> PyResult<bool> {
    let inner: egui::Rect = (&rect).into();
    Ok(current_ui(&UI)?.is_rect_visible(inner))
}

/// Run `contents` with an id salt applied, mirroring `Ui::push_id`.
///
/// egui derives every widget's id from its position in the Ui tree. Two
/// identical widgets in a loop therefore share ids and their state leaks
/// between iterations -- the first `text_edit` gets the value, the second
/// renders it, and neither can be addressed. `push_id` gives a subtree its own
/// id space, which is what a repeated widget needs.
///
/// `salt` may be any hashable value; a string is the usual case.
///
/// Example::
///
///     for i, name in enumerate(names):
///         push_id(i, lambda i=i: text_edit_singleline(values[i]))
#[pyfunction]
#[pyo3(signature = (salt, contents))]
unsafe fn push_id(salt: String, contents: Bound<'_, PyAny>) -> PyResult<()> {
    let ui = current_ui(&UI)?;

    ui.push_id(salt, |ui| run_nested_update_func_lossy(ui, contents.clone()));

    Ok(())
}

/// Create a child ui which is indented to the right.
/// Example::
///
///     def update_func():
///       heading("I'm indented")
///     indent(update_func)
#[pyfunction]
unsafe fn indent(update_fun: Bound<'_, PyAny>) -> PyResult<()> {
    current_ui(&UI)?
        .indent("your mom", |ui| run_nested_update_func(ui, update_fun))
        .inner
}

/// Visually groups the contents together.
///
/// Example::
///
///     def update_func():
///       heading("hi")
///       heading("there")
///     
///     group(update_func)
#[pyfunction]
unsafe fn group(update_fun: Bound<'_, PyAny>) -> PyResult<()> {
    current_ui(&UI)?
        .group(|ui| run_nested_update_func(ui, update_fun))
        .inner
}

/// Builder options for `egui::ScrollArea`, matching its setters exactly.
const SCROLL_AREA_OPTIONS: &[&str] = &[
    "max_width",
    "max_height",
    "min_scrolled_width",
    "min_scrolled_height",
    "scroll_bar_visibility",
    "id_source",
    "id_salt",
    "auto_shrink",
    "animated",
    "drag_to_scroll",
    "stick_to_right",
    "stick_to_bottom",
];

/// Read an optional `ScrollBarVisibility` from `opts`.
///
/// egui 0.31.1 declares the enum in `containers::scroll_area` but does not
/// re-export it from the crate root -- `containers::mod` only re-exports the
/// `ScrollArea` itself -- so it has to be named by its full path. Reaching for
/// `egui::ScrollBarVisibility` does not compile.
unsafe fn opt_scroll_bar_visibility(
    opts: &Bound<'_, PyDict>,
    used: &mut OptNames,
) -> PyResult<Option<egui::containers::scroll_area::ScrollBarVisibility>> {
    use egui::containers::scroll_area::ScrollBarVisibility;

    used.insert("scroll_bar_visibility".to_string());
    match opts.get_item("scroll_bar_visibility")? {
        Some(value) => {
            let name: String = value.extract()?;
            let visibility = match name.as_str() {
                "always_hidden" => ScrollBarVisibility::AlwaysHidden,
                "visible_when_needed" => ScrollBarVisibility::VisibleWhenNeeded,
                "always_visible" => ScrollBarVisibility::AlwaysVisible,
                _ => {
                    return Err(PyValueError::new_err(format!(
                        "unknown scroll_bar_visibility {name:?}; expected one of \
                         'always_hidden', 'visible_when_needed', 'always_visible'"
                    )))
                }
            };
            Ok(Some(visibility))
        }
        None => Ok(None),
    }
}

/// Apply egui's `ScrollArea` setters from a Python kwargs dict.
///
/// Shared by `scroll_area_vertical`, `scroll_area_horizontal` and
/// `scroll_area_both` so the option list is defined once. The axis is chosen
/// by the caller before this runs, because egui's `vertical()`/`horizontal()`/
/// `both()` are constructors rather than setters.
///
/// `unsafe` because every `opt_*` helper is. Edition 2021, so this body is an
/// implicit unsafe block.
unsafe fn apply_scroll_area_options(
    area: egui::ScrollArea,
    opts: &Bound<'_, PyDict>,
    used: &mut OptNames,
) -> PyResult<egui::ScrollArea> {
    let mut area = area;

    for name in ["max_width", "max_height", "min_scrolled_width", "min_scrolled_height"] {
        if let Some(v) = opt_f32(opts, name, used)? {
            area = match name {
                "max_width" => area.max_width(v),
                "max_height" => area.max_height(v),
                "min_scrolled_width" => area.min_scrolled_width(v),
                _ => area.min_scrolled_height(v),
            };
        }
    }

    if let Some(v) = opt_scroll_bar_visibility(opts, used)? {
        area = area.scroll_bar_visibility(v);
    }

    // egui takes `impl Into<Vec2b>` here; a plain bool converts and means both
    // axes, which is what "shrink both ways" means.
    if let Some(v) = opt_bool(opts, "auto_shrink", used)? {
        area = area.auto_shrink(v);
    }

    for name in ["animated", "drag_to_scroll", "stick_to_right", "stick_to_bottom"] {
        if let Some(v) = opt_bool(opts, name, used)? {
            area = match name {
                "animated" => area.animated(v),
                "drag_to_scroll" => area.drag_to_scroll(v),
                "stick_to_right" => area.stick_to_right(v),
                _ => area.stick_to_bottom(v),
            };
        }
    }

    // `id_source` and `id_salt` are egui's own aliases for the same setter.
    // Both take `impl Hash`; a Python str hashes, so it is passed through
    // unchanged rather than converted. Recorded by hand, like the equivalent
    // loop in `collapsing_response`: `reject_unknown_options` would read the
    // untouched `get_item` as an unknown option, so this insert is required
    // by that check.
    used.insert("id_source".to_string());
    used.insert("id_salt".to_string());
    for name in ["id_source", "id_salt"] {
        if let Some(value) = opts.get_item(name)? {
            let salt: String = value.extract().map_err(|_| {
                PyValueError::new_err(format!("{name} must be a string"))
            })?;
            area = area.id_source(salt);
        }
    }

    Ok(area)
}

/// Create a vertical scroll area.
///
/// Example::
///
///     def update_func():
///       heading("hi")
///       heading("there")
///       # a lot of elements
///
///     scroll_area_vertical(update_func)
#[pyfunction]
#[pyo3(signature = (update_fun, **options))]
unsafe fn scroll_area_vertical(
    update_fun: Bound<'_, PyAny>,
    options: Option<&Bound<'_, PyDict>>,
) -> PyResult<()> {
    scroll_area(egui::ScrollArea::vertical(), update_fun, options)
}

/// Create a horizontal scroll area.
///
/// Example::
///
///     def update_func():
///       heading("hi")
///       heading("there")
///       # a lot of elements
///
///     scroll_area_horizontal(update_func)
#[pyfunction]
#[pyo3(signature = (update_fun, **options))]
unsafe fn scroll_area_horizontal(
    update_fun: Bound<'_, PyAny>,
    options: Option<&Bound<'_, PyDict>>,
) -> PyResult<()> {
    scroll_area(egui::ScrollArea::horizontal(), update_fun, options)
}

/// Create a scroll area scrollable on both axes.
///
/// `scroll_area_vertical` and `scroll_area_horizontal` each lock one axis.
/// This one scrolls on both, which is what a table, a log view or a canvas
/// larger than its viewport needs.
///
/// Example::
///
///     scroll_area_both(update_func, max_height=200.0)
#[pyfunction]
#[pyo3(signature = (update_fun, **options))]
unsafe fn scroll_area_both(
    update_fun: Bound<'_, PyAny>,
    options: Option<&Bound<'_, PyDict>>,
) -> PyResult<()> {
    scroll_area(egui::ScrollArea::both(), update_fun, options)
}

/// Shared body of the three scroll-area entry points.
///
/// egui's `vertical()`/`horizontal()`/`both()` are constructors, so the axis is
/// fixed by the caller and everything else arrives as keyword arguments.
unsafe fn scroll_area(
    builder: egui::ScrollArea,
    update_fun: Bound<'_, PyAny>,
    options: Option<&Bound<'_, PyDict>>,
) -> PyResult<()> {
    let ui = current_ui(&UI)?;

    let mut builder = builder;
    if let Some(opts) = options {
        validate_options(opts, SCROLL_AREA_OPTIONS)?;
        // Tracker only: the scroll areas are not §6 targets, and
        // `validate_options` already rejects anything outside
        // SCROLL_AREA_OPTIONS.
        let mut used = OptNames::new();
        builder = apply_scroll_area_options(builder, opts, &mut used)?;
    }

    builder
        .show(ui, |ui| run_nested_update_func(ui, update_fun))
        .inner?;

    Ok(())
}

/// Create a scoped child ui.
///
/// You can use this to temporarily change the Style of a sub-region.
///
/// Example::
///
///     def update_func():
///       heading("0.5 opacity")
///       set_opacity(0.5)
///     
///     heading("normal opacity")
///     scope(update_func)
#[pyfunction]
unsafe fn scope(update_fun: Bound<'_, PyAny>) -> PyResult<()> {
    current_ui(&UI)?
        .scope(|ui| run_nested_update_func(ui, update_fun))
        .inner
}

/// Control float with a slider.
///
/// Example::
///
///     data = Float(5)
///     # inside update_func
///     slider_float(data, 0, 50, "slide me")
///
/// Builder options go in **options: `drag_value_speed`, `vertical`,
/// `show_value`, `trailing_fill`, `text_color`, `fixed_decimals`,
/// `min_decimals`, `max_decimals`, `smallest_positive`, `largest_finite`,
/// `octal`, `hexadecimal`, `handle_shape`. An unknown name is an error.
#[pyfunction]
#[pyo3(signature = (value, min, max, text, suffix=None, prefix=None, step_by=None, logarithmic=None, clamping=None, binary=None, **options))]
unsafe fn slider_float(
    value: &mut Float,
    min: f32,
    max: f32,
    text: &str,
    suffix: Option<&str>,
    prefix: Option<&str>,
    step_by: Option<f32>,
    logarithmic: Option<bool>,
    clamping: Option<&Bound<'_, PyAny>>,
    binary: Option<&Bound<'_, PyAny>>,
    options: Option<&Bound<'_, PyDict>>,
) -> PyResult<()> {
    slider_float_response(
        value,
        min,
        max,
        text,
        suffix,
        prefix,
        step_by,
        logarithmic,
        clamping,
        binary,
        options,
    )?;
    Ok(())
}

/// Returns the Response of the slider. See button_response. Use changed to
/// know the value was edited.
///
/// Example::
///
///     data = Float(5)
///     if slider_float_response(data, 0, 50, "slide me").changed:
///       print("now", data.value)
///
/// Builder options go in **options: `drag_value_speed`, `vertical`,
/// `show_value`, `trailing_fill`, `text_color`, `fixed_decimals`,
/// `min_decimals`, `max_decimals`, `smallest_positive`, `largest_finite`,
/// `octal`, `hexadecimal`, `handle_shape`. An unknown name is an error.
#[pyfunction]
#[pyo3(signature = (value, min, max, text, suffix=None, prefix=None, step_by=None, logarithmic=None, clamping=None, binary=None, **options))]
unsafe fn slider_float_response(
    value: &mut Float,
    min: f32,
    max: f32,
    text: &str,
    suffix: Option<&str>,
    prefix: Option<&str>,
    step_by: Option<f32>,
    logarithmic: Option<bool>,
    clamping: Option<&Bound<'_, PyAny>>,
    binary: Option<&Bound<'_, PyAny>>,
    options: Option<&Bound<'_, PyDict>>,
) -> PyResult<Response> {
    let ui = current_ui(&UI)?;
    let mut used = OptNames::new();

    // The named parameters are deliberately NOT recorded in `used`. A name
    // cannot arrive both as a named parameter and inside **options: Python
    // raises TypeError on the duplicate before pyo3 is reached, which is what
    // makes `slider_float(d, 0, 1, "x", suffix="ms", **{"suffix": "s"})` an
    // error rather than a silent pick-one. So `used` only ever holds tail
    // names, and there is nothing here for `reject_unknown_options` to
    // misjudge.
    let mut slider = egui::Slider::new(&mut value.value, min..=max).text(text);

    if let Some(v) = suffix {
        slider = slider.suffix(v);
    }
    if let Some(v) = prefix {
        slider = slider.prefix(v);
    }
    if let Some(v) = step_by {
        slider = slider.step_by(v as f64);
    }
    if let Some(v) = logarithmic {
        slider = slider.logarithmic(v);
    }
    if let Some(v) = clamping {
        let word = enum_word_py(v, "clamping", CLAMPING_WORDS)?;
        slider = slider.clamping(slider_clamping(&word));
    }
    if let Some(v) = binary {
        // `binary` arrives as a named parameter rather than through **options,
        // so it skips `opt_radix` and needs the same validation by hand.
        let parts: Vec<i64> = v.extract().map_err(|_| {
            PyValueError::new_err(
                "binary must be a sequence like binary=(8, False) -- egui's \
                 binary takes a minimum digit width and a twos_complement flag",
            )
        })?;
        validate_radix(&parts, "binary")?;
        slider = slider.binary(parts[0] as usize, parts[1] != 0);
    }
    slider = apply_slider_options(slider, options, &mut used, "slider_float")?;

    Ok(Response {
        inner: ui.add(slider),
    })
}

/// Map the `clamping` word onto egui's `SliderClamping`.
///
/// egui's `clamping()` takes a `SliderClamping`, not a bool, so this cannot be
/// routed through `opt_bool`. The three words are egui's own variant names in
/// snake_case.
fn slider_clamping(word: &str) -> egui::SliderClamping {
    match word {
        "never" => egui::SliderClamping::Never,
        "edits" => egui::SliderClamping::Edits,
        _ => egui::SliderClamping::Always,
    }
}

/// The words `clamping` accepts, in egui's own order.
const CLAMPING_WORDS: &[&str] = &["never", "edits", "always"];

/// Map the `handle_shape` word onto egui's `HandleShape`.
///
/// `rect` carries an aspect ratio, so the Python value is either the bare
/// word or a `(word, aspect_ratio)` pair.
///
/// `egui::HandleShape` is not re-exported at the crate root in 0.31.1 -- only
/// `egui::Slider`, `egui::SliderClamping` and `egui::SliderOrientation` are
/// (egui-0.31.1/src/widgets/mod.rs) -- so this goes through the full path.
fn handle_shape(value: &Bound<'_, PyAny>) -> PyResult<egui::style::HandleShape> {
    // A pair carries the `aspect_ratio` that egui's `HandleShape::Rect` has.
    // Tried first because a bare word cannot extract as a pair.
    if let Ok((word, aspect_ratio)) = value.extract::<(String, f32)>() {
        enum_word(&word, "handle_shape", HANDLE_SHAPE_WORDS)?;
        return match word.as_str() {
            "rect" => Ok(egui::style::HandleShape::Rect { aspect_ratio }),
            // `enum_word` already rejected anything else.
            _ => unreachable!(),
        };
    }
    let word = enum_word_py(value, "handle_shape", HANDLE_SHAPE_WORDS)?;
    match word.as_str() {
        "circle" => Ok(egui::style::HandleShape::Circle),
        _ => Ok(egui::style::HandleShape::Rect { aspect_ratio: 1.0 }),
    }
}

/// The words `handle_shape` accepts, in egui's own order.
const HANDLE_SHAPE_WORDS: &[&str] = &["circle", "rect"];

/// Read the **options tail of a slider and return the rebuilt builder.
///
/// Takes and returns the builder by value rather than mutating through
/// `&mut`: egui's setters consume `self` and return a new `Slider`, so
/// `*slider = slider.suffix(v)` through a reference would move out of a
/// borrow. `Slider<'a>` borrows the value it edits, which is why the lifetime
/// is explicit.
///
/// Separate from the pyfunctions because `slider_float_response` and
/// `slider_int_response` both delegate here, and a helper that reads options
/// without threading `used` through makes a genuine option look unknown to
/// `reject_unknown_options`.
///
/// `unsafe` because the `opt_*` helpers are, and the `opt_*` family is unsafe
/// for the same reason `apply_window_options` is.
unsafe fn apply_slider_options<'a>(
    mut slider: egui::Slider<'a>,
    options: Option<&Bound<'_, PyDict>>,
    used: &mut OptNames,
    widget: &str,
) -> PyResult<egui::Slider<'a>> {
    let o = match options {
        Some(o) => o,
        None => return Ok(slider),
    };

    if let Some(v) = opt_f64(o, "drag_value_speed", used)? {
        slider = slider.drag_value_speed(v);
    }
    if let Some(v) = opt_bool(o, "vertical", used)? {
        // egui's `vertical()` takes no argument: it is a switch, not a
        // setter. `vertical=False` therefore means "leave it horizontal",
        // which is also the default, so only True reaches the builder.
        if v {
            slider = slider.vertical();
        }
    }
    if let Some(v) = opt_bool(o, "show_value", used)? {
        slider = slider.show_value(v);
    }
    if let Some(v) = opt_bool(o, "trailing_fill", used)? {
        slider = slider.trailing_fill(v);
    }
    if let Some(v) = opt_color32(o, "text_color", used)? {
        slider = slider.text_color(v);
    }
    if let Some(v) = opt_usize(o, "fixed_decimals", used)? {
        slider = slider.fixed_decimals(v);
    }
    if let Some(v) = opt_usize(o, "min_decimals", used)? {
        slider = slider.min_decimals(v);
    }
    if let Some(v) = opt_usize(o, "max_decimals", used)? {
        slider = slider.max_decimals(v);
    }
    if let Some(v) = opt_f64(o, "smallest_positive", used)? {
        slider = slider.smallest_positive(v);
    }
    if let Some(v) = opt_f64(o, "largest_finite", used)? {
        slider = slider.largest_finite(v);
    }
    // egui's radix builders take `(min_width, twos_complement)`, and
    // hexadecimal a third `upper`. `binary` is a named parameter on the float
    // slider so it is applied before this call; `octal`/`hexadecimal` are read
    // here. Order follows the read order, so the last setter the caller
    // supplied wins, matching egui's own behaviour. Note that each of these
    // replaces egui's custom formatter wholesale, so passing two of them means
    // only the last takes effect.
    if let Some(v) = opt_radix(o, "octal", used)? {
        slider = slider.octal(v[0] as usize, v[1] != 0);
    }
    if let Some(v) = opt_radix(o, "hexadecimal", used)? {
        let upper = v.get(2).copied().unwrap_or(0) != 0;
        slider = slider.hexadecimal(v[0] as usize, v[1] != 0, upper);
    }
    if let Some(v) = handle_shape_opt(o, used)? {
        slider = slider.handle_shape(v);
    }

    reject_unknown_options(o, used, widget)?;

    Ok(slider)
}

/// Read `handle_shape`, which may be a bare word or a `(word, ratio)` pair.
///
/// The pair form exists because egui's `HandleShape::Rect` carries an
/// `aspect_ratio`; a plain-string extractor would reject `("rect", 0.5)`
/// before any word could be checked. The name is inserted before the read, so
/// a declared-but-absent `handle_shape` still counts as known.
fn handle_shape_opt(
    o: &Bound<'_, PyDict>,
    used: &mut OptNames,
) -> PyResult<Option<egui::style::HandleShape>> {
    used.insert("handle_shape".to_string());
    match o.get_item("handle_shape")? {
        Some(value) => Ok(Some(handle_shape(&value)?)),
        None => Ok(None),
    }
}

/// Control int with a slider.
///
/// Example::
///
///     data = Int(5)
///     # inside update_func
///     slider_int(data, 0, 50, "slide me")
///
/// Builder options go in **options: `drag_value_speed`, `vertical`,
/// `show_value`, `trailing_fill`, `text_color`, `fixed_decimals`,
/// `min_decimals`, `max_decimals`, `smallest_positive`, `largest_finite`,
/// `octal`, `hexadecimal`, `handle_shape`. An unknown name is an error.
#[pyfunction]
#[pyo3(signature = (value, min, max, text, suffix=None, prefix=None, step_by=None, logarithmic=None, clamping=None, **options))]
unsafe fn slider_int(
    value: &mut Int,
    min: i32,
    max: i32,
    text: &str,
    suffix: Option<&str>,
    prefix: Option<&str>,
    step_by: Option<i32>,
    logarithmic: Option<bool>,
    clamping: Option<&Bound<'_, PyAny>>,
    options: Option<&Bound<'_, PyDict>>,
) -> PyResult<()> {
    slider_int_response(
        value,
        min,
        max,
        text,
        suffix,
        prefix,
        step_by,
        logarithmic,
        clamping,
        options,
    )?;
    Ok(())
}

/// Returns the Response of the int slider. See slider_float_response.
///
/// Example::
///
///     data = Int(5)
///     if slider_int_response(data, 0, 50, "slide me").changed:
///       print("now", data.value)
///
/// Builder options go in **options: `drag_value_speed`, `vertical`,
/// `show_value`, `trailing_fill`, `text_color`, `fixed_decimals`,
/// `min_decimals`, `max_decimals`, `smallest_positive`, `largest_finite`,
/// `octal`, `hexadecimal`, `handle_shape`. An unknown name is an error.
#[pyfunction]
#[pyo3(signature = (value, min, max, text, suffix=None, prefix=None, step_by=None, logarithmic=None, clamping=None, **options))]
unsafe fn slider_int_response(
    value: &mut Int,
    min: i32,
    max: i32,
    text: &str,
    suffix: Option<&str>,
    prefix: Option<&str>,
    step_by: Option<i32>,
    logarithmic: Option<bool>,
    clamping: Option<&Bound<'_, PyAny>>,
    options: Option<&Bound<'_, PyDict>>,
) -> PyResult<Response> {
    let ui = current_ui(&UI)?;
    let mut used = OptNames::new();

    // See `slider_float_response`: the named parameters cannot also appear in
    // **options, so only the tail needs recording.
    let mut slider = egui::Slider::new(&mut value.value, min..=max)
        .text(text)
        .integer();

    if let Some(v) = suffix {
        slider = slider.suffix(v);
    }
    if let Some(v) = prefix {
        slider = slider.prefix(v);
    }
    if let Some(v) = step_by {
        slider = slider.step_by(v as f64);
    }
    if let Some(v) = logarithmic {
        slider = slider.logarithmic(v);
    }
    if let Some(v) = clamping {
        let word = enum_word_py(v, "clamping", CLAMPING_WORDS)?;
        slider = slider.clamping(slider_clamping(&word));
    }
    slider = apply_slider_options(slider, options, &mut used, "slider_int")?;

    Ok(Response {
        inner: ui.add(slider),
    })
}

/// Control float by dragging the number.
///
/// Example::
///
///     data = Float(5)
///     # inside update_func
///     drag_float(data, 0, 50, 1.5)
///
/// Keyword parameters: `suffix`, `prefix`. Everything else goes in
/// **options: `update_while_editing`, `clamp_existing_to_range`,
/// `fixed_decimals`, `min_decimals`, `max_decimals`, `binary`, `octal`,
/// `hexadecimal`. An unknown name is an error.
#[pyfunction]
#[pyo3(signature = (value, min, max, speed, suffix=None, prefix=None, **options))]
unsafe fn drag_float(
    value: &mut Float,
    min: f32,
    max: f32,
    speed: f32,
    suffix: Option<&str>,
    prefix: Option<&str>,
    options: Option<&Bound<'_, PyDict>>,
) -> PyResult<()> {
    drag_float_response(value, min, max, speed, suffix, prefix, options)?;
    Ok(())
}

/// Returns the Response of the drag value. See slider_float_response.
///
/// Example::
///
///     data = Float(5)
///     if drag_float_response(data, 0, 50, 1.5).changed:
///       print("now", data.value)
///
/// Keyword parameters: `suffix`, `prefix`. Everything else goes in
/// **options: `update_while_editing`, `clamp_existing_to_range`,
/// `fixed_decimals`, `min_decimals`, `max_decimals`, `binary`, `octal`,
/// `hexadecimal`. An unknown name is an error.
#[pyfunction]
#[pyo3(signature = (value, min, max, speed, suffix=None, prefix=None, **options))]
unsafe fn drag_float_response(
    value: &mut Float,
    min: f32,
    max: f32,
    speed: f32,
    suffix: Option<&str>,
    prefix: Option<&str>,
    options: Option<&Bound<'_, PyDict>>,
) -> PyResult<Response> {
    let ui = current_ui(&UI)?;
    let mut used = OptNames::new();

    // See `slider_float_response`: the named parameters cannot also appear in
    // **options (CPython raises TypeError on the duplicate before pyo3 is
    // reached), so only the tail needs recording in `used`.
    let mut drag = egui::DragValue::new(&mut value.value)
        .speed(speed as f64)
        .range(min as f64..=max as f64);

    if let Some(v) = suffix {
        drag = drag.suffix(v);
    }
    if let Some(v) = prefix {
        drag = drag.prefix(v);
    }
    drag = apply_drag_options(drag, options, &mut used, "drag_float")?;

    Ok(Response {
        inner: ui.add(drag),
    })
}

/// Control int by dragging the number.
///
/// Example::
///
///     data = Int(5)
///     # inside update_func
///     drag_int(data, 0, 50, 1)
///
/// Keyword parameters: `suffix`, `prefix`. Everything else goes in
/// **options: `update_while_editing`, `clamp_existing_to_range`,
/// `fixed_decimals`, `min_decimals`, `max_decimals`, `binary`, `octal`,
/// `hexadecimal`. An unknown name is an error.
#[pyfunction]
#[pyo3(signature = (value, min, max, speed, suffix=None, prefix=None, **options))]
unsafe fn drag_int(
    value: &mut Int,
    min: i32,
    max: i32,
    speed: i32,
    suffix: Option<&str>,
    prefix: Option<&str>,
    options: Option<&Bound<'_, PyDict>>,
) -> PyResult<()> {
    drag_int_response(value, min, max, speed, suffix, prefix, options)?;
    Ok(())
}

/// Returns the Response of the int drag value. See slider_float_response.
///
/// Example::
///
///     data = Int(5)
///     if drag_int_response(data, 0, 50, 1).changed:
///       print("now", data.value)
///
/// Keyword parameters: `suffix`, `prefix`. Everything else goes in
/// **options: `update_while_editing`, `clamp_existing_to_range`,
/// `fixed_decimals`, `min_decimals`, `max_decimals`, `binary`, `octal`,
/// `hexadecimal`. An unknown name is an error.
#[pyfunction]
#[pyo3(signature = (value, min, max, speed, suffix=None, prefix=None, **options))]
unsafe fn drag_int_response(
    value: &mut Int,
    min: i32,
    max: i32,
    speed: i32,
    suffix: Option<&str>,
    prefix: Option<&str>,
    options: Option<&Bound<'_, PyDict>>,
) -> PyResult<Response> {
    let ui = current_ui(&UI)?;
    let mut used = OptNames::new();

    // See `slider_float_response`: the named parameters cannot also appear in
    // **options (CPython raises TypeError on the duplicate before pyo3 is
    // reached), so only the tail needs recording in `used`.
    let mut drag = egui::DragValue::new(&mut value.value)
        .speed(speed as f64)
        .range(min as f64..=max as f64);

    if let Some(v) = suffix {
        drag = drag.suffix(v);
    }
    if let Some(v) = prefix {
        drag = drag.prefix(v);
    }
    drag = apply_drag_options(drag, options, &mut used, "drag_int")?;

    Ok(Response {
        inner: ui.add(drag),
    })
}

/// Control an angle in radians (0 to 2*pi) by dragging around a dial.
/// The value wraps around, so dragging past 2*pi continues from 0.
///
/// Example::
///
///     data = Float(0)
///     # inside update_func
///     drag_angle(data)
///     heading(f"{data.value} rad")
#[pyfunction]
unsafe fn drag_angle(radians: &mut Float) -> PyResult<()> {
    drag_angle_response(radians)?;
    Ok(())
}

/// Returns the Response of the angle dial. See slider_float_response.
///
/// Example::
///
///     data = Float(0)
///     if drag_angle_response(data).changed:
///       print("now", data.value, "rad")
#[pyfunction]
unsafe fn drag_angle_response(radians: &mut Float) -> PyResult<Response> {
    let ui = current_ui(&UI)?;

    Ok(Response {
        inner: ui.drag_angle(&mut radians.value),
    })
}

/// Control an angle in radians by dragging around a dial that spans the full
/// circle (0 to 2*pi), instead of the half-circle dial of drag_angle.
///
/// Example::
///
///     data = Float(0)
///     # inside update_func
///     drag_angle_tau(data)
///     heading(f"{data.value} rad")
#[pyfunction]
unsafe fn drag_angle_tau(radians: &mut Float) -> PyResult<()> {
    drag_angle_tau_response(radians)?;
    Ok(())
}

/// Returns the Response of the full-circle angle dial. See drag_angle_response.
///
/// Example::
///
///     data = Float(0)
///     if drag_angle_tau_response(data).changed:
///       print("now", data.value, "rad")
#[pyfunction]
unsafe fn drag_angle_tau_response(radians: &mut Float) -> PyResult<Response> {
    let ui = current_ui(&UI)?;

    Ok(Response {
        inner: ui.drag_angle_tau(&mut radians.value),
    })
}

/// A clickable hyperlink
///
/// Example::
///
///     hyperlink("https://github.com/emilk/egui")
#[pyfunction]
unsafe fn hyperlink(url: &str) -> PyResult<()> {
    hyperlink_response(url)?;
    Ok(())
}

/// Returns the Response of the hyperlink.
///
/// Example::
///
///     hyperlink_response("https://github.com/emilk/egui")
#[pyfunction]
unsafe fn hyperlink_response(url: &str) -> PyResult<Response> {
    let ui = current_ui(&UI)?;

    Ok(Response {
        inner: ui.hyperlink(url),
    })
}

/// A clickable hyperlink with label
///
/// Example::
///
///     hyperlink_to("egui on GitHub", "https://www.github.com/emilk/egui/")
#[pyfunction]
unsafe fn hyperlink_to(label: &str, url: &str) -> PyResult<()> {
    hyperlink_to_response(label, url)?;
    Ok(())
}

/// Returns the Response of the labelled hyperlink.
///
/// Example::
///
///     hyperlink_to_response("egui", "https://github.com/emilk/egui/")
#[pyfunction]
unsafe fn hyperlink_to_response(label: &str, url: &str) -> PyResult<Response> {
    let ui = current_ui(&UI)?;

    Ok(Response {
        inner: ui.hyperlink_to(label, url),
    })
}

/// Clickable text, that looks like a hyperlink.
/// To link to a web page, use hyperlink or hyperlink_to.
///
/// Example::
///
///     if link_clicked("egui on GitHub"):
///       print("clicked on a fake link")
#[pyfunction]
unsafe fn link_clicked(label: &str) -> PyResult<bool> {
    Ok(link_response(label)?.clicked())
}

/// Returns the Response of the link. See button_response.
///
/// Example::
///
///     link_response("egui on GitHub").on_hover_text("opens in a browser")
#[pyfunction]
unsafe fn link_response(label: &str) -> PyResult<Response> {
    let ui = current_ui(&UI)?;

    Ok(Response {
        inner: ui.link(label),
    })
}

/// Show a checkbox.
///
/// Example::
///
///     data = Bool(false)
///     # inside update_func
///     checkbox(data, "check me")
#[pyfunction]
unsafe fn checkbox(checked: &mut Bool, text: &str) -> PyResult<()> {
    checkbox_response(checked, text)?;
    Ok(())
}

/// Returns the Response of the checkbox. See button_response.
///
/// Example::
///
///     data = Bool(False)
///     response = checkbox_response(data, "check me")
///     if response.changed:
///       print("toggled to", data.value)
#[pyfunction]
unsafe fn checkbox_response(checked: &mut Bool, text: &str) -> PyResult<Response> {
    let ui = current_ui(&UI)?;

    Ok(Response {
        inner: ui.checkbox(&mut checked.value, text),
    })
}

/// Acts like a checkbox, but looks like a selectable label.
///
/// Example::
///
///     data = Bool(false)
///     # inside update_func
///     toggle_value(data, "check me")
#[pyfunction]
unsafe fn toggle_value(selected: &mut Bool, text: &str) -> PyResult<()> {
    toggle_value_response(selected, text)?;
    Ok(())
}

/// Returns the Response of the toggle. See button_response.
///
/// Example::
///
///     data = Bool(False)
///     if toggle_value_response(data, "check me").clicked:
///       print("toggled")
#[pyfunction]
unsafe fn toggle_value_response(selected: &mut Bool, text: &str) -> PyResult<Response> {
    let ui = current_ui(&UI)?;

    Ok(Response {
        inner: ui.toggle_value(&mut selected.value, text),
    })
}

/// Show a radio button. It is selected if current_value == selected_value. If clicked, selected_value is assigned to current_value.
///
/// Example::
///
///     RED = 0
///     GREEN = 1
///     BLUE = 2
///     
///     c = Int(RED)
///     
///     radio_value(c, RED, "red")
///     radio_value(c, GREEN, "green")
///     radio_value(c, BLUE, "blue")
#[pyfunction]
unsafe fn radio_value(current_value: &mut Int, alternative: i32, text: &str) -> PyResult<()> {
    radio_value_response(current_value, alternative, text)?;
    Ok(())
}

/// Returns the Response of the radio button. See button_response.
///
/// Example::
///
///     c = Int(0)
///     if radio_value_response(c, 1, "green").clicked:
///       print("selected green")
#[pyfunction]
unsafe fn radio_value_response(
    current_value: &mut Int,
    alternative: i32,
    text: &str,
) -> PyResult<Response> {
    let ui = current_ui(&UI)?;

    Ok(Response {
        inner: ui.radio_value(&mut current_value.value, alternative, text),
    })
}

/// Show selectable text. It is selected if current_value == selected_value. If clicked, selected_value is assigned to current_value.
///
/// Example::
///
///     RED = 0
///     GREEN = 1
///     BLUE = 2
///     
///     c = Int(RED)
///     
///     selectable_value(c, RED, "red")
///     selectable_value(c, GREEN, "green")
///     selectable_value(c, BLUE, "blue")
#[pyfunction]
unsafe fn selectable_value(current_value: &mut Int, alternative: i32, text: &str) -> PyResult<()> {
    selectable_value_response(current_value, alternative, text)?;
    Ok(())
}

/// Returns the Response of the selectable label. See button_response.
///
/// Example::
///
///     c = Int(0)
///     if selectable_value_response(c, 2, "blue").clicked:
///       print("selected blue")
#[pyfunction]
unsafe fn selectable_value_response(
    current_value: &mut Int,
    alternative: i32,
    text: &str,
) -> PyResult<Response> {
    let ui = current_ui(&UI)?;

    Ok(Response {
        inner: ui.selectable_value(&mut current_value.value, alternative, text),
    })
}

/// Show a selectable label. It is highlighted while "selected" is True, and
/// toggles to True when clicked.
///
/// This is the counterpart of selectable_value: same widget, but the state is
/// a single bool rather than a value chosen from a set.
///
/// Example::
///
///     data = Bool(False)
///     # inside update_func
///     selectable_label(data, "select me")
#[pyfunction]
unsafe fn selectable_label(selected: &mut Bool, text: &str) -> PyResult<()> {
    selectable_label_response(selected, text)?;
    Ok(())
}

/// Returns the Response of the selectable label. See button_response.
///
/// Example::
///
///     data = Bool(False)
///     if selectable_label_response(data, "select me").clicked:
///       print("selected")
#[pyfunction]
unsafe fn selectable_label_response(selected: &mut Bool, text: &str) -> PyResult<Response> {
    let ui = current_ui(&UI)?;

    Ok(Response {
        inner: ui.selectable_label(selected.value, text),
    })
}

/// Show a radio button. It is selected while "selected" is True, and toggles to
/// True when clicked.
///
/// This is the counterpart of radio_value: the state is a single bool, so the
/// caller decides what "selected" means.
///
/// Example::
///
///     wifi = Bool(True)
///     # inside update_func
///     radio(wifi, "wifi")
#[pyfunction]
unsafe fn radio(selected: &mut Bool, text: &str) -> PyResult<()> {
    radio_response(selected, text)?;
    Ok(())
}

/// Returns the Response of the radio button. See button_response.
///
/// Example::
///
///     wifi = Bool(True)
///     if radio_response(wifi, "wifi").clicked:
///       print("clicked the radio")
#[pyfunction]
unsafe fn radio_response(selected: &mut Bool, text: &str) -> PyResult<Response> {
    let ui = current_ui(&UI)?;

    Ok(Response {
        inner: ui.radio(selected.value, text),
    })
}

/// Shows a combo box with values defined in "alternatives" and their corresponding names
/// defined in "names"
///
/// Example::
///
///     RED = 0
///     GREEN = 1
///     BLUE = 2
///
///     data = Int(RED)
///
///     def update_func(a):
///         combo_box(data, [RED, GREEN, BLUE], ["red", "green", "blue"], "choose your fate")
#[pyfunction]
unsafe fn combo_box(
    current_value: &mut Int,
    alternatives: Vec<i32>,
    names: Vec<String>,
    label: &str,
) -> PyResult<()> {
    let ui = current_ui(&UI)?;

    egui::ComboBox::from_label(label)
        .selected_text(
            names
                .get(current_value.value.try_into().unwrap_or(0))
                .unwrap_or(&"Unknown".to_string()),
        )
        .show_ui(ui, |ui| {
            for i in 0..alternatives.len() {
                ui.selectable_value(
                    &mut current_value.value,
                    alternatives[i],
                    names.get(i).unwrap_or(&"Unknown".to_string()),
                );
            }
        });
    Ok(())
}

/// Closes the currently open menu. Useful to dismiss a combo box or a context
/// menu programmatically from within the update function.
///
/// Example::
///
///     if button_clicked("close"):
///         close_menu()
#[pyfunction]
unsafe fn close_menu() -> PyResult<()> {
    let ui = current_ui(&UI)?;

    ui.close_menu();
    Ok(())
}

/// A simple progress bar.
/// value in the [0, 1] range, where 1 means “completed”.
///
/// Example::
///
///     progress(0.5)
#[pyfunction]
unsafe fn progress(value: f32) -> PyResult<()> {
    progress_response(value)?;
    Ok(())
}

/// Returns the Response of the progress bar. A progress bar is not interactive,
/// so this is only useful for its rect.
///
/// Example::
///
///     progress_response(0.5)
#[pyfunction]
unsafe fn progress_response(value: f32) -> PyResult<Response> {
    let ui = current_ui(&UI)?;

    Ok(Response {
        inner: ui.add(egui::widgets::ProgressBar::new(value).show_percentage()),
    })
}

/// A spinner widget used to indicate loading.
///
/// Example::
///
///     spinner()
#[pyfunction]
unsafe fn spinner() -> PyResult<()> {
    spinner_response()?;
    Ok(())
}

/// Returns the Response of the spinner. A spinner is not interactive, so this
/// is only useful for its rect.
///
/// Example::
///
///     spinner_response()
#[pyfunction]
unsafe fn spinner_response() -> PyResult<Response> {
    let ui = current_ui(&UI)?;

    Ok(Response {
        inner: ui.spinner(),
    })
}

/// Shows a button with the given color. If the user clicks the button, a full color picker is shown.
///
/// Example::
///
///     color = RGB(69, 69, 69)
///     # inside udpate_func
///     color_edit_button_rgb(color)
///     heading(f"r:{color.r} g:{color.g} b:{color.b}")
#[pyfunction]
unsafe fn color_edit_button_rgb(rgb: &mut RGB) -> PyResult<()> {
    color_edit_button_rgb_response(rgb)?;
    Ok(())
}

/// Returns the Response of the RGB color picker button. Use changed to know
/// the color was edited.
///
/// Example::
///
///     color = RGB(69, 69, 69)
///     if color_edit_button_rgb_response(color).changed:
///       print("now", color.r, color.g, color.b)
#[pyfunction]
unsafe fn color_edit_button_rgb_response(rgb: &mut RGB) -> PyResult<Response> {
    let ui = current_ui(&UI)?;

    let mut tmp: [f32; 3] = [rgb.r, rgb.g, rgb.b];

    let response = ui.color_edit_button_rgb(&mut tmp);

    rgb.r = tmp[0];
    rgb.g = tmp[1];
    rgb.b = tmp[2];

    Ok(Response { inner: response })
}

/// Shows a button with the given color in hue/saturation/value space. If the
/// user clicks the button, a full color picker is shown.
///
/// Example::
///
///     color = HSVA(0.0, 1.0, 1.0, 1.0)
///     # inside update_func
///     color_edit_button_hsva(color)
///     heading(f"h:{color.h} s:{color.s} v:{color.v}")
#[pyfunction]
unsafe fn color_edit_button_hsva(hsva: &mut HSVA) -> PyResult<()> {
    color_edit_button_hsva_response(hsva)?;
    Ok(())
}

/// Returns the Response of the HSVA color picker. See color_edit_button_rgb
/// for the equivalent of the boolean-returning style.
///
/// Example::
///
///     color = HSVA(0.0, 1.0, 1.0, 1.0)
///     if color_edit_button_hsva_response(color).changed:
///       print("now", color)
#[pyfunction]
unsafe fn color_edit_button_hsva_response(hsva: &mut HSVA) -> PyResult<Response> {
    let ui = current_ui(&UI)?;

    Ok(Response {
        inner: ui.color_edit_button_hsva(&mut egui::ecolor::Hsva {
            h: hsva.h,
            s: hsva.s,
            v: hsva.v,
            a: hsva.a,
        }),
    })
}

/// Shows a button with the given 8-bit sRGBA color. If the user clicks the
/// button, a full color picker is shown.
///
/// Example::
///
///     color = Color32(255, 0, 0, 255)
///     # inside update_func
///     color_edit_button_srgba(color)
#[pyfunction]
unsafe fn color_edit_button_srgba(srgba: &mut Color32) -> PyResult<()> {
    color_edit_button_srgba_response(srgba)?;
    Ok(())
}

/// Returns the Response of the sRGBA color picker.
///
/// Example::
///
///     color = Color32(255, 0, 0, 255)
///     if color_edit_button_srgba_response(color).changed:
///       print("now", color)
#[pyfunction]
unsafe fn color_edit_button_srgba_response(srgba: &mut Color32) -> PyResult<Response> {
    let ui = current_ui(&UI)?;

    // egui takes a Color32, which is a newtype over [u8; 4] with a private
    // field, so it is built and read back through the conversion helpers.
    let mut color =
        egui::Color32::from_rgba_unmultiplied(srgba.r, srgba.g, srgba.b, srgba.a);

    let response = ui.color_edit_button_srgba(&mut color);

    let [r, g, b, a] = color.to_srgba_unmultiplied();
    srgba.r = r;
    srgba.g = g;
    srgba.b = b;
    srgba.a = a;

    Ok(Response { inner: response })
}

/// Shows a button with the given sRGB color. If the user clicks the button, a
/// full color picker is shown.
///
/// Example::
///
///     color = SRGB(255, 128, 0)
///     # inside update_func
///     color_edit_button_srgb(color)
#[pyfunction]
unsafe fn color_edit_button_srgb(srgb: &mut SRGB) -> PyResult<()> {
    color_edit_button_srgb_response(srgb)?;
    Ok(())
}

/// Returns the Response of the sRGB color picker.
///
/// Example::
///
///     color = SRGB(255, 128, 0)
///     if color_edit_button_srgb_response(color).changed:
///       print("now", color)
#[pyfunction]
unsafe fn color_edit_button_srgb_response(srgb: &mut SRGB) -> PyResult<Response> {
    let ui = current_ui(&UI)?;

    let mut tmp: [u8; 3] = [srgb.r, srgb.g, srgb.b];

    let response = ui.color_edit_button_srgb(&mut tmp);

    srgb.r = tmp[0];
    srgb.g = tmp[1];
    srgb.b = tmp[2];

    Ok(Response { inner: response })
}

/// Shows a button with the given linear RGBA color with premultiplied alpha.
/// If the user clicks the button, a full color picker is shown.
///
/// Premultiplied alpha means the color channels are already scaled by alpha,
/// so a half-transparent red has r = 0.5 rather than r = 1.0. You rarely want
/// this; prefer color_edit_button_rgba_unmultiplied.
///
/// Example::
///
///     color = RGBA(0.5, 0.0, 0.0, 0.5)
///     # inside update_func
///     color_edit_button_rgba_premultiplied(color)
#[pyfunction]
unsafe fn color_edit_button_rgba_premultiplied(rgba: &mut RGBA) -> PyResult<()> {
    color_edit_button_rgba_premultiplied_response(rgba)?;
    Ok(())
}

/// Returns the Response of the premultiplied linear RGBA color picker.
///
/// Example::
///
///     color = RGBA(0.5, 0.0, 0.0, 0.5)
///     if color_edit_button_rgba_premultiplied_response(color).changed:
///       print("now", color)
#[pyfunction]
unsafe fn color_edit_button_rgba_premultiplied_response(rgba: &mut RGBA) -> PyResult<Response> {
    let ui = current_ui(&UI)?;

    let mut tmp: [f32; 4] = [rgba.r, rgba.g, rgba.b, rgba.a];

    let response = ui.color_edit_button_rgba_premultiplied(&mut tmp);

    rgba.r = tmp[0];
    rgba.g = tmp[1];
    rgba.b = tmp[2];
    rgba.a = tmp[3];

    Ok(Response { inner: response })
}

/// Shows a button with the given linear RGBA color, alpha not premultiplied.
/// If the user clicks the button, a full color picker is shown.
///
/// This is the linear-space counterpart to color_edit_button_rgba, and the
/// one to reach for if you need alpha at all. If unsure what "premultiplied
/// alpha" is, this is the function you want.
///
/// Example::
///
///     color = RGBA(1.0, 0.0, 0.0, 0.5)
///     # inside update_func
///     color_edit_button_rgba_unmultiplied(color)
#[pyfunction]
unsafe fn color_edit_button_rgba_unmultiplied(rgba: &mut RGBA) -> PyResult<()> {
    color_edit_button_rgba_unmultiplied_response(rgba)?;
    Ok(())
}

/// Returns the Response of the unmultiplied linear RGBA color picker.
///
/// Example::
///
///     color = RGBA(1.0, 0.0, 0.0, 0.5)
///     if color_edit_button_rgba_unmultiplied_response(color).changed:
///       print("now", color)
#[pyfunction]
unsafe fn color_edit_button_rgba_unmultiplied_response(rgba: &mut RGBA) -> PyResult<Response> {
    let ui = current_ui(&UI)?;

    let mut tmp: [f32; 4] = [rgba.r, rgba.g, rgba.b, rgba.a];

    let response = ui.color_edit_button_rgba_unmultiplied(&mut tmp);

    rgba.r = tmp[0];
    rgba.g = tmp[1];
    rgba.b = tmp[2];
    rgba.a = tmp[3];

    Ok(Response { inner: response })
}

/// Shows a button with the given 8-bit sRGBA color with premultiplied alpha.
/// If the user clicks the button, a full color picker is shown.
///
/// Example::
///
///     color = Color32(128, 0, 0, 128)
///     # inside update_func
///     color_edit_button_srgba_premultiplied(color)
#[pyfunction]
unsafe fn color_edit_button_srgba_premultiplied(srgba: &mut Color32) -> PyResult<()> {
    color_edit_button_srgba_premultiplied_response(srgba)?;
    Ok(())
}

/// Returns the Response of the premultiplied sRGBA color picker.
///
/// Example::
///
///     color = Color32(128, 0, 0, 128)
///     if color_edit_button_srgba_premultiplied_response(color).changed:
///       print("now", color)
#[pyfunction]
unsafe fn color_edit_button_srgba_premultiplied_response(srgba: &mut Color32) -> PyResult<Response> {
    let ui = current_ui(&UI)?;

    let mut tmp: [u8; 4] = [srgba.r, srgba.g, srgba.b, srgba.a];

    let response = ui.color_edit_button_srgba_premultiplied(&mut tmp);

    srgba.r = tmp[0];
    srgba.g = tmp[1];
    srgba.b = tmp[2];
    srgba.a = tmp[3];

    Ok(Response { inner: response })
}

/// Shows a button with the given 8-bit sRGBA color, alpha not premultiplied.
/// If the user clicks the button, a full color picker is shown.
///
/// This is the format egui stores textures in, and the right choice when you
/// need the exact 8-bit value. Prefer this over the premultiplied variant
/// unless you specifically need premultiplication.
///
/// Example::
///
///     color = Color32(255, 0, 0, 128)
///     # inside update_func
///     color_edit_button_srgba_unmultiplied(color)
#[pyfunction]
unsafe fn color_edit_button_srgba_unmultiplied(srgba: &mut Color32) -> PyResult<()> {
    color_edit_button_srgba_unmultiplied_response(srgba)?;
    Ok(())
}

/// Returns the Response of the unmultiplied sRGBA color picker.
///
/// Example::
///
///     color = Color32(255, 0, 0, 128)
///     if color_edit_button_srgba_unmultiplied_response(color).changed:
///       print("now", color)
#[pyfunction]
unsafe fn color_edit_button_srgba_unmultiplied_response(srgba: &mut Color32) -> PyResult<Response> {
    let ui = current_ui(&UI)?;

    let mut tmp: [u8; 4] = [srgba.r, srgba.g, srgba.b, srgba.a];

    let response = ui.color_edit_button_srgba_unmultiplied(&mut tmp);

    srgba.r = tmp[0];
    srgba.g = tmp[1];
    srgba.b = tmp[2];
    srgba.a = tmp[3];

    Ok(Response { inner: response })
}

/// Show an image available at the given uri.
///
/// Example::
///
///     image("https://picsum.photos/480")
///     image("file://assets/ferris.png", max_height = 50, max_width = 50)
#[pyfunction]
#[pyo3(signature = (source, **kwargs))]
unsafe fn image(source: &str, kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<()> {
    image_response(source, kwargs)?;
    Ok(())
}

/// Returns the Response of the image. An image is not interactive by default,
/// so this is mainly useful for its rect and for enabling a sense via
/// interact.
///
/// Example::
///
///     image_response("file://assets/ferris.png", max_height = 50)
#[pyfunction]
#[pyo3(signature = (source, **kwargs))]
unsafe fn image_response(source: &str, kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<Response> {
    let ui = current_ui(&UI)?;

    let mut img = egui::Image::new(source);

    // Tracker only: `image` is not a §6 group, but the records are made anyway
    // so the `get_item` audit below stays complete.
    let mut used = OptNames::new();

    if let Some(kwargs) = kwargs {
        // Recorded by hand: `reject_unknown_options` would read the untouched
        // `get_item` as an unknown option, so this insert is required by that
        // check.
        used.insert("max_height".to_string());
        if let Some(height) = kwargs.get_item("max_height")? {
            img = img.max_height(height.downcast::<PyInt>()?.extract()?);
        }
        used.insert("max_width".to_string());
        if let Some(width) = kwargs.get_item("max_width")? {
            img = img.max_width(width.downcast::<PyInt>()?.extract()?);
        }
    }

    Ok(Response {
        inner: ui.add(img),
    })
}

/// Creates a button with an image to the left of the text
///
/// Example::
///
///     if image_and_text_clicked("https://picsum.photos/480", "click me"):
///       print("clicked")
#[pyfunction]
unsafe fn image_and_text_clicked(source: &str, text: &str) -> PyResult<bool> {
    Ok(image_and_text_response(source, text)?.clicked())
}

/// Returns the Response of the image-and-text button. See button_response.
///
/// Example::
///
///     response = image_and_text_response("https://picsum.photos/480", "click me")
///     if response.clicked:
///       print("clicked")
#[pyfunction]
unsafe fn image_and_text_response(source: &str, text: &str) -> PyResult<Response> {
    let ui = current_ui(&UI)?;

    Ok(Response {
        inner: ui.add(egui::Button::image_and_text(source, text)),
    })
}

/// A visual separator. A horizontal or vertical line on layout.
///
/// Example::
///
///     separator()
#[pyfunction]
unsafe fn separator() -> PyResult<()> {
    separator_response()?;
    Ok(())
}

/// Returns the Response of the separator. A separator is not interactive, so
/// this is only useful for its rect.
///
/// Example::
///
///     separator_response()
#[pyfunction]
unsafe fn separator_response() -> PyResult<Response> {
    let ui = current_ui(&UI)?;

    Ok(Response {
        inner: ui.separator(),
    })
}

/// Calling set_invisible() will cause all further widgets to be invisible, yet still allocate space.
///
/// The widgets will not be interactive (set_invisible() implies disable()).
///
/// Once invisible, there is no way to make the Ui visible again.
///
/// Example::
///
///     set_invisible()
///     heading("this will not be visible")
#[pyfunction]
unsafe fn set_invisible() -> PyResult<()> {
    let ui = current_ui(&UI)?;

    ui.set_invisible();
    Ok(())
}

/// Calling disable() will cause the Ui to deny all future interaction and all the widgets will draw with a gray look.
///
/// Usually it is more convenient to use add_enabled.
///
/// Note that once disabled, there is no way to re-enable the Ui.
///
/// Example::
///
///     disable()
///     if button_clicked("you can't click me"):
///       pass
#[pyfunction]
unsafe fn disable() -> PyResult<()> {
    let ui = current_ui(&UI)?;

    ui.disable();
    Ok(())
}

/// Add a section that is possibly disabled, i.e. greyed out and non-interactive.
///
/// If you call add_enabled from within an already disabled Ui, the result will always be disabled, even if the enabled argument is true.
///
/// Example::
///
///     add_enabled(False, lambda: button_clicked("you can't click me"))
///     button_clicked("but you can click me")
#[pyfunction]
unsafe fn add_enabled(enabled: bool, update_fun: Bound<'_, PyAny>) -> PyResult<()> {
    current_ui(&UI)?
        .add_enabled_ui(enabled, |ui| run_nested_update_func(ui, update_fun))
        .inner
}

/// Make the widget in this Ui semi-transparent.
///
/// opacity must be between 0.0 and 1.0, where 0.0 means fully transparent (i.e., invisible) and 1.0 means fully opaque.
/// Example::
///
///     set_opacity(0.5)
#[pyfunction]
unsafe fn set_opacity(opacity: f32) -> PyResult<()> {
    let ui = current_ui(&UI)?;

    ui.set_opacity(opacity);
    Ok(())
}

/// Shows a date, and will open a date picker popup when clicked.
///
/// Example::
///
///     date = Date(datetime.datetime.now())
///     # inside update_func
///     date_picker_button(date)
#[pyfunction]
unsafe fn date_picker_button(selection: &mut Date) -> PyResult<()> {
    date_picker_button_response(selection)?;
    Ok(())
}

/// Returns the Response of the date picker button. Use changed to know a new
/// date was picked.
///
/// Example::
///
///     date = Date(datetime.datetime.now())
///     if date_picker_button_response(date).changed:
///       print("picked", date.value)
#[pyfunction]
unsafe fn date_picker_button_response(selection: &mut Date) -> PyResult<Response> {
    let ui = current_ui(&UI)?;

    Ok(Response {
        inner: ui.add(egui_extras::DatePickerButton::new(&mut selection.value)),
    })
}

/// Add extra space before the next widget.
///
/// The direction is dependent on the layout.
/// Example::
///
///     add_space(5)
///     heading("I'm so spaced now")
#[pyfunction]
unsafe fn add_space(amount: f32) -> PyResult<()> {
    let ui = current_ui(&UI)?;

    ui.add_space(amount);
    Ok(())
}

#[pymodule]
fn pyegui(m: &Bound<'_, PyModule>) -> PyResult<()> {
    pyo3_log::init();
    // classes
    m.add_class::<Str>()?;
    m.add_class::<Bool>()?;
    m.add_class::<Int>()?;
    m.add_class::<Float>()?;
    m.add_class::<RGB>()?;
    m.add_class::<Date>()?;
    m.add_class::<Context>()?;
    m.add_class::<Layout>()?;
    m.add_class::<LayoutType>()?;
    m.add_class::<Scope>()?;
    m.add_class::<Group>()?;
    m.add_class::<Response>()?;
    m.add_class::<Rect>()?;
    m.add_class::<PointerButton>()?;
    m.add_class::<RGBA>()?;
    m.add_class::<HSVA>()?;
    m.add_class::<Color32>()?;
    m.add_class::<SRGB>()?;
    // functions
    m.add_function(wrap_pyfunction!(run_native, m)?)?;
    m.add_function(wrap_pyfunction!(heading, m)?)?;
    m.add_function(wrap_pyfunction!(monospace, m)?)?;
    m.add_function(wrap_pyfunction!(small, m)?)?;
    m.add_function(wrap_pyfunction!(strong, m)?)?;
    m.add_function(wrap_pyfunction!(weak, m)?)?;
    m.add_function(wrap_pyfunction!(label, m)?)?;
    m.add_function(wrap_pyfunction!(code, m)?)?;
    m.add_function(wrap_pyfunction!(code_editor, m)?)?;
    m.add_function(wrap_pyfunction!(text_edit_singleline, m)?)?;
    m.add_function(wrap_pyfunction!(text_edit_multiline, m)?)?;
    m.add_function(wrap_pyfunction!(button_clicked, m)?)?;
    m.add_function(wrap_pyfunction!(small_button_clicked, m)?)?;
    m.add_function(wrap_pyfunction!(horizontal, m)?)?;
    m.add_function(wrap_pyfunction!(horizontal_centered, m)?)?;
    m.add_function(wrap_pyfunction!(horizontal_top, m)?)?;
    m.add_function(wrap_pyfunction!(horizontal_wrapped, m)?)?;
    m.add_function(wrap_pyfunction!(vertical, m)?)?;
    m.add_function(wrap_pyfunction!(vertical_centered, m)?)?;
    m.add_function(wrap_pyfunction!(vertical_centered_justified, m)?)?;
    m.add_function(wrap_pyfunction!(centered_and_justified, m)?)?;
    m.add_function(wrap_pyfunction!(collapsing, m)?)?;
    m.add_function(wrap_pyfunction!(collapsing_response, m)?)?;
    m.add_function(wrap_pyfunction!(menu_button, m)?)?;
    m.add_function(wrap_pyfunction!(menu_image_button, m)?)?;
    m.add_function(wrap_pyfunction!(menu_image_text_button, m)?)?;
    m.add_function(wrap_pyfunction!(indent, m)?)?;
    m.add_function(wrap_pyfunction!(group, m)?)?;
    m.add_function(wrap_pyfunction!(frame, m)?)?;
    m.add_function(wrap_pyfunction!(frame_group, m)?)?;
    m.add_function(wrap_pyfunction!(frame_popup, m)?)?;
    m.add_function(wrap_pyfunction!(frame_menu, m)?)?;
    m.add_function(wrap_pyfunction!(frame_window, m)?)?;
    m.add_function(wrap_pyfunction!(frame_canvas, m)?)?;
    m.add_function(wrap_pyfunction!(frame_dark_canvas, m)?)?;
    m.add_function(wrap_pyfunction!(frame_central_panel, m)?)?;
    m.add_function(wrap_pyfunction!(frame_side_top_panel, m)?)?;
    m.add_function(wrap_pyfunction!(scroll_area_vertical, m)?)?;
    m.add_function(wrap_pyfunction!(scroll_area_horizontal, m)?)?;
    m.add_function(wrap_pyfunction!(scroll_area_both, m)?)?;
    m.add_function(wrap_pyfunction!(scene, m)?)?;
    m.add_function(wrap_pyfunction!(set_width, m)?)?;
    m.add_function(wrap_pyfunction!(set_height, m)?)?;
    m.add_function(wrap_pyfunction!(set_min_width, m)?)?;
    m.add_function(wrap_pyfunction!(set_max_width, m)?)?;
    m.add_function(wrap_pyfunction!(set_min_height, m)?)?;
    m.add_function(wrap_pyfunction!(set_max_height, m)?)?;
    m.add_function(wrap_pyfunction!(set_min_size, m)?)?;
    m.add_function(wrap_pyfunction!(set_max_size, m)?)?;
    m.add_function(wrap_pyfunction!(set_width_range, m)?)?;
    m.add_function(wrap_pyfunction!(set_height_range, m)?)?;
    m.add_function(wrap_pyfunction!(shrink_width_to_current, m)?)?;
    m.add_function(wrap_pyfunction!(shrink_height_to_current, m)?)?;
    m.add_function(wrap_pyfunction!(columns, m)?)?;
    m.add_function(wrap_pyfunction!(end_row, m)?)?;
    m.add_function(wrap_pyfunction!(set_row_height, m)?)?;
    m.add_function(wrap_pyfunction!(available_size, m)?)?;
    m.add_function(wrap_pyfunction!(available_width, m)?)?;
    m.add_function(wrap_pyfunction!(available_height, m)?)?;
    m.add_function(wrap_pyfunction!(available_size_before_wrap, m)?)?;
    m.add_function(wrap_pyfunction!(available_rect_before_wrap, m)?)?;
    m.add_function(wrap_pyfunction!(cursor, m)?)?;
    m.add_function(wrap_pyfunction!(min_rect, m)?)?;
    m.add_function(wrap_pyfunction!(max_rect, m)?)?;
    m.add_function(wrap_pyfunction!(min_size, m)?)?;
    m.add_function(wrap_pyfunction!(pixels_per_point, m)?)?;
    m.add_function(wrap_pyfunction!(next_widget_position, m)?)?;
    m.add_function(wrap_pyfunction!(is_rect_visible, m)?)?;
    m.add_function(wrap_pyfunction!(push_id, m)?)?;
    m.add_function(wrap_pyfunction!(scope, m)?)?;
    m.add_function(wrap_pyfunction!(slider_float, m)?)?;
    m.add_function(wrap_pyfunction!(slider_int, m)?)?;
    m.add_function(wrap_pyfunction!(drag_int, m)?)?;
    m.add_function(wrap_pyfunction!(drag_float, m)?)?;
    m.add_function(wrap_pyfunction!(drag_angle, m)?)?;
    m.add_function(wrap_pyfunction!(drag_angle_tau, m)?)?;
    m.add_function(wrap_pyfunction!(hyperlink, m)?)?;
    m.add_function(wrap_pyfunction!(hyperlink_to, m)?)?;
    m.add_function(wrap_pyfunction!(link_clicked, m)?)?;
    m.add_function(wrap_pyfunction!(side_panel_left, m)?)?;
    m.add_function(wrap_pyfunction!(side_panel_right, m)?)?;
    m.add_function(wrap_pyfunction!(top_panel, m)?)?;
    m.add_function(wrap_pyfunction!(bottom_panel, m)?)?;
    m.add_function(wrap_pyfunction!(modal, m)?)?;
    m.add_function(wrap_pyfunction!(resize, m)?)?;
    m.add_function(wrap_pyfunction!(area, m)?)?;
    m.add_function(wrap_pyfunction!(central_panel, m)?)?;
    m.add_function(wrap_pyfunction!(window, m)?)?;
    m.add_function(wrap_pyfunction!(checkbox, m)?)?;
    m.add_function(wrap_pyfunction!(radio_value, m)?)?;
    m.add_function(wrap_pyfunction!(toggle_value, m)?)?;
    m.add_function(wrap_pyfunction!(selectable_value, m)?)?;
    m.add_function(wrap_pyfunction!(selectable_label, m)?)?;
    m.add_function(wrap_pyfunction!(radio, m)?)?;
    m.add_function(wrap_pyfunction!(combo_box, m)?)?;
    m.add_function(wrap_pyfunction!(close_menu, m)?)?;
    m.add_function(wrap_pyfunction!(progress, m)?)?;
    m.add_function(wrap_pyfunction!(spinner, m)?)?;
    m.add_function(wrap_pyfunction!(color_edit_button_rgb, m)?)?;
    m.add_function(wrap_pyfunction!(color_edit_button_hsva, m)?)?;
    m.add_function(wrap_pyfunction!(color_edit_button_srgb, m)?)?;
    m.add_function(wrap_pyfunction!(color_edit_button_srgba, m)?)?;
    m.add_function(wrap_pyfunction!(color_edit_button_rgba_unmultiplied, m)?)?;
    m.add_function(wrap_pyfunction!(color_edit_button_rgba_premultiplied, m)?)?;
    m.add_function(wrap_pyfunction!(color_edit_button_srgba_unmultiplied, m)?)?;
    m.add_function(wrap_pyfunction!(color_edit_button_srgba_premultiplied, m)?)?;
    m.add_function(wrap_pyfunction!(crate::image, m)?)?;
    m.add_function(wrap_pyfunction!(image_and_text_clicked, m)?)?;
    m.add_function(wrap_pyfunction!(separator, m)?)?;
    m.add_function(wrap_pyfunction!(set_invisible, m)?)?;
    m.add_function(wrap_pyfunction!(disable, m)?)?;
    m.add_function(wrap_pyfunction!(add_enabled, m)?)?;
    m.add_function(wrap_pyfunction!(set_opacity, m)?)?;
    m.add_function(wrap_pyfunction!(date_picker_button, m)?)?;
    m.add_function(wrap_pyfunction!(add_space, m)?)?;
    // *_response variants (egui Response wrappers)
    m.add_function(wrap_pyfunction!(button_response, m)?)?;
    m.add_function(wrap_pyfunction!(small_button_response, m)?)?;
    m.add_function(wrap_pyfunction!(link_response, m)?)?;
    m.add_function(wrap_pyfunction!(image_and_text_response, m)?)?;
    m.add_function(wrap_pyfunction!(heading_response, m)?)?;
    m.add_function(wrap_pyfunction!(label_response, m)?)?;
    m.add_function(wrap_pyfunction!(monospace_response, m)?)?;
    m.add_function(wrap_pyfunction!(small_response, m)?)?;
    m.add_function(wrap_pyfunction!(strong_response, m)?)?;
    m.add_function(wrap_pyfunction!(weak_response, m)?)?;
    m.add_function(wrap_pyfunction!(code_response, m)?)?;
    m.add_function(wrap_pyfunction!(hyperlink_response, m)?)?;
    m.add_function(wrap_pyfunction!(hyperlink_to_response, m)?)?;
    m.add_function(wrap_pyfunction!(checkbox_response, m)?)?;
    m.add_function(wrap_pyfunction!(toggle_value_response, m)?)?;
    m.add_function(wrap_pyfunction!(radio_value_response, m)?)?;
    m.add_function(wrap_pyfunction!(radio_response, m)?)?;
    m.add_function(wrap_pyfunction!(selectable_value_response, m)?)?;
    m.add_function(wrap_pyfunction!(selectable_label_response, m)?)?;
    m.add_function(wrap_pyfunction!(slider_float_response, m)?)?;
    m.add_function(wrap_pyfunction!(slider_int_response, m)?)?;
    m.add_function(wrap_pyfunction!(drag_float_response, m)?)?;
    m.add_function(wrap_pyfunction!(drag_int_response, m)?)?;
    m.add_function(wrap_pyfunction!(drag_angle_response, m)?)?;
    m.add_function(wrap_pyfunction!(drag_angle_tau_response, m)?)?;
    m.add_function(wrap_pyfunction!(text_edit_singleline_response, m)?)?;
    m.add_function(wrap_pyfunction!(text_edit_multiline_response, m)?)?;
    m.add_function(wrap_pyfunction!(code_editor_response, m)?)?;
    m.add_function(wrap_pyfunction!(color_edit_button_rgb_response, m)?)?;
    m.add_function(wrap_pyfunction!(color_edit_button_hsva_response, m)?)?;
    m.add_function(wrap_pyfunction!(color_edit_button_srgb_response, m)?)?;
    m.add_function(wrap_pyfunction!(color_edit_button_srgba_response, m)?)?;
    m.add_function(wrap_pyfunction!(color_edit_button_rgba_unmultiplied_response, m)?)?;
    m.add_function(wrap_pyfunction!(color_edit_button_rgba_premultiplied_response, m)?)?;
    m.add_function(wrap_pyfunction!(color_edit_button_srgba_unmultiplied_response, m)?)?;
    m.add_function(wrap_pyfunction!(color_edit_button_srgba_premultiplied_response, m)?)?;
    m.add_function(wrap_pyfunction!(date_picker_button_response, m)?)?;
    m.add_function(wrap_pyfunction!(crate::image_response, m)?)?;
    m.add_function(wrap_pyfunction!(progress_response, m)?)?;
    m.add_function(wrap_pyfunction!(spinner_response, m)?)?;
    m.add_function(wrap_pyfunction!(separator_response, m)?)?;
    Ok(())
}
