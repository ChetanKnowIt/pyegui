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
#[pyfunction]
#[pyo3(signature = (text, **kwargs))]
unsafe fn text_edit_singleline_response(
    text: &mut Str,
    kwargs: Option<&Bound<'_, PyDict>>,
) -> PyResult<Response> {
    let ui = current_ui(&UI)?;

    let mut w = egui::TextEdit::singleline(&mut text.value);

    if let Some(kwargs) = kwargs {
        if let Some(hint_text) = kwargs.get_item("hint_text")? {
            w = w.hint_text(hint_text.downcast::<PyString>()?.extract::<String>()?);
        }
    }

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
#[pyfunction]
#[pyo3(signature = (text, **kwargs))]
unsafe fn text_edit_multiline(text: &mut Str, kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<()> {
    text_edit_multiline_response(text, kwargs)?;
    Ok(())
}

/// Returns the Response of the multiline text field. Use changed to know the
/// text was edited.
///
/// Example::
///
///     text = Str("editable")
///     if text_edit_multiline_response(text).changed:
///       print("now", text.value)
#[pyfunction]
#[pyo3(signature = (text, **kwargs))]
unsafe fn text_edit_multiline_response(
    text: &mut Str,
    kwargs: Option<&Bound<'_, PyDict>>,
) -> PyResult<Response> {
    let ui = current_ui(&UI)?;

    let mut w = egui::TextEdit::multiline(&mut text.value);

    if let Some(kwargs) = kwargs {
        if let Some(hint_text) = kwargs.get_item("hint_text")? {
            w = w.hint_text(hint_text.downcast::<PyString>()?.extract::<String>()?);
        }
    }

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

/// Read an optional bool from `opts`.
unsafe fn opt_bool(opts: &Bound<'_, PyDict>, name: &str) -> PyResult<Option<bool>> {
    match opts.get_item(name)? {
        Some(value) => Ok(Some(value.extract()?)),
        None => Ok(None),
    }
}

/// Read an optional f32 from `opts`.
unsafe fn opt_f32(opts: &Bound<'_, PyDict>, name: &str) -> PyResult<Option<f32>> {
    match opts.get_item(name)? {
        Some(value) => Ok(Some(value.extract()?)),
        None => Ok(None),
    }
}

/// Read an optional 2D size or position from `opts` as a 2-sequence of
/// numbers, e.g. `default_size=(400.0, 300.0)`.
///
/// egui's geometry types (`Vec2`, `Pos2`) have no Python equivalent yet --
/// they are TODO §4 -- so tuples are accepted for now and converted here.
/// When a `Vec2` class lands, these should switch to it.
unsafe fn opt_vec2(opts: &Bound<'_, PyDict>, name: &str) -> PyResult<Option<egui::Vec2>> {
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
        builder = apply_window_options(builder, opts)?;
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
) -> PyResult<egui::Window<'static>> {
    if let Some(v) = opt_vec2(opts, "default_size")? {
        builder = builder.default_size(v);
    }
    if let Some(v) = opt_vec2(opts, "fixed_size")? {
        builder = builder.fixed_size(v);
    }
    if let Some(v) = opt_vec2(opts, "min_size")? {
        builder = builder.min_size(v);
    }
    if let Some(v) = opt_vec2(opts, "max_size")? {
        builder = builder.max_size(v);
    }
    // `default_pos` and `fixed_pos` take a Pos2, which is the same Vec2 with
    // a different name in egui.
    if let Some(v) = opt_vec2(opts, "default_pos")? {
        builder = builder.default_pos(egui::Pos2::new(v.x, v.y));
    }
    if let Some(v) = opt_vec2(opts, "fixed_pos")? {
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
        if let Some(v) = opt_f32(opts, name)? {
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
        if let Some(v) = opt_bool(opts, name)? {
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
        if let Some(v) = opt_bool(opts, name)? {
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
    if opt_bool(opts, "auto_sized")?.unwrap_or(false) {
        builder = builder.auto_sized();
    }

    // egui's `order` takes an `egui::Order`, not a number. Map the three
    // words rather than taking an f32 and discarding it -- silently ignoring
    // the value is the failure this whole option parser exists to prevent.
    if let Some(value) = opts.get_item("order")? {
        let name: String = value.extract().map_err(|_| {
            PyValueError::new_err("order must be one of 'background', 'middle', 'foreground', 'tooltip', 'debug'")
        })?;
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
        builder = builder.order(order);
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
unsafe fn opt_range(opts: &Bound<'_, PyDict>, name: &str) -> PyResult<Option<egui::Rangef>> {
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
) -> PyResult<egui::SidePanel> {
    if let Some(v) = opt_bool(opts, "resizable")? {
        builder = builder.resizable(v);
    }
    if let Some(v) = opt_bool(opts, "show_separator_line")? {
        builder = builder.show_separator_line(v);
    }
    if let Some(v) = opt_f32(opts, "default_width")? {
        builder = builder.default_width(v);
    }
    if let Some(v) = opt_f32(opts, "min_width")? {
        builder = builder.min_width(v);
    }
    if let Some(v) = opt_f32(opts, "max_width")? {
        builder = builder.max_width(v);
    }
    if let Some(v) = opt_range(opts, "width_range")? {
        builder = builder.width_range(v);
    }
    Ok(builder)
}

/// Apply the shared `egui::TopBottomPanel` setters.
unsafe fn apply_top_bottom_panel_options(
    mut builder: egui::TopBottomPanel,
    opts: &Bound<'_, PyDict>,
) -> PyResult<egui::TopBottomPanel> {
    if let Some(v) = opt_bool(opts, "resizable")? {
        builder = builder.resizable(v);
    }
    if let Some(v) = opt_bool(opts, "show_separator_line")? {
        builder = builder.show_separator_line(v);
    }
    if let Some(v) = opt_f32(opts, "default_height")? {
        builder = builder.default_height(v);
    }
    if let Some(v) = opt_f32(opts, "min_height")? {
        builder = builder.min_height(v);
    }
    if let Some(v) = opt_f32(opts, "max_height")? {
        builder = builder.max_height(v);
    }
    if let Some(v) = opt_range(opts, "height_range")? {
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
            apply_side_panel_options(egui::SidePanel::left(id), opts)?
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
            apply_side_panel_options(egui::SidePanel::right(id), opts)?
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
            apply_top_bottom_panel_options(egui::TopBottomPanel::top(id), opts)?
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
            apply_top_bottom_panel_options(egui::TopBottomPanel::bottom(id), opts)?
        }
        None => egui::TopBottomPanel::bottom(id),
    };

    builder.show(&ctx.0, |ui| run_nested_update_func_lossy(ui, contents.clone()));

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

/// A CollapsingHeader that starts out collapsed.
///
/// Example::
///
///     def update_func():
///       heading("hi")
///     collapsing("collapsed", update_func)
#[pyfunction]
unsafe fn collapsing(heading: &str, update_fun: Bound<'_, PyAny>) -> PyResult<()> {
    current_ui(&UI)?.collapsing(heading, |ui| run_nested_update_func(ui, update_fun));
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
unsafe fn scroll_area_vertical(update_fun: Bound<'_, PyAny>) -> PyResult<()> {
    egui::ScrollArea::vertical().show(current_ui(&UI)?, |ui| run_nested_update_func(ui, update_fun)).inner
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
unsafe fn scroll_area_horizontal(update_fun: Bound<'_, PyAny>) -> PyResult<()> {
    egui::ScrollArea::horizontal().show(current_ui(&UI)?, |ui| run_nested_update_func(ui, update_fun)).inner
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
#[pyfunction]
unsafe fn slider_float(value: &mut Float, min: f32, max: f32, text: &str) -> PyResult<()> {
    slider_float_response(value, min, max, text)?;
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
#[pyfunction]
unsafe fn slider_float_response(
    value: &mut Float,
    min: f32,
    max: f32,
    text: &str,
) -> PyResult<Response> {
    let ui = current_ui(&UI)?;

    Ok(Response {
        inner: ui.add(egui::Slider::new(&mut value.value, min..=max).text(text)),
    })
}

/// Control int with a slider.
///
/// Example::
///
///     data = Int(5)
///     # inside update_func
///     slider_int(data, 0, 50, "slide me")
#[pyfunction]
unsafe fn slider_int(value: &mut Int, min: i32, max: i32, text: &str) -> PyResult<()> {
    slider_int_response(value, min, max, text)?;
    Ok(())
}

/// Returns the Response of the int slider. See slider_float_response.
///
/// Example::
///
///     data = Int(5)
///     if slider_int_response(data, 0, 50, "slide me").changed:
///       print("now", data.value)
#[pyfunction]
unsafe fn slider_int_response(
    value: &mut Int,
    min: i32,
    max: i32,
    text: &str,
) -> PyResult<Response> {
    let ui = current_ui(&UI)?;

    Ok(Response {
        inner: ui.add(
            egui::Slider::new(&mut value.value, min..=max)
                .text(text)
                .integer(),
        ),
    })
}

/// Control float by dragging the number.
///
/// Example::
///
///     data = Float(5)
///     # inside update_func
///     drag_float(data, 0, 50, 1.5)
#[pyfunction]
unsafe fn drag_float(value: &mut Float, min: f32, max: f32, speed: f32) -> PyResult<()> {
    drag_float_response(value, min, max, speed)?;
    Ok(())
}

/// Returns the Response of the drag value. See slider_float_response.
///
/// Example::
///
///     data = Float(5)
///     if drag_float_response(data, 0, 50, 1.5).changed:
///       print("now", data.value)
#[pyfunction]
unsafe fn drag_float_response(
    value: &mut Float,
    min: f32,
    max: f32,
    speed: f32,
) -> PyResult<Response> {
    let ui = current_ui(&UI)?;

    Ok(Response {
        inner: ui.add(
            egui::DragValue::new(&mut value.value)
                .speed(speed)
                .range(min..=max),
        ),
    })
}

/// Control int by dragging the number.
///
/// Example::
///
///     data = Int(5)
///     # inside update_func
///     drag_int(data, 0, 50, 1)
#[pyfunction]
unsafe fn drag_int(value: &mut Int, min: i32, max: i32, speed: i32) -> PyResult<()> {
    drag_int_response(value, min, max, speed)?;
    Ok(())
}

/// Returns the Response of the int drag value. See slider_float_response.
///
/// Example::
///
///     data = Int(5)
///     if drag_int_response(data, 0, 50, 1).changed:
///       print("now", data.value)
#[pyfunction]
unsafe fn drag_int_response(
    value: &mut Int,
    min: i32,
    max: i32,
    speed: i32,
) -> PyResult<Response> {
    let ui = current_ui(&UI)?;

    Ok(Response {
        inner: ui.add(
            egui::DragValue::new(&mut value.value)
                .speed(speed)
                .range(min..=max),
        ),
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

    if let Some(kwargs) = kwargs {
        if let Some(height) = kwargs.get_item("max_height")? {
            img = img.max_height(height.downcast::<PyInt>()?.extract()?);
        }
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
    m.add_function(wrap_pyfunction!(indent, m)?)?;
    m.add_function(wrap_pyfunction!(group, m)?)?;
    m.add_function(wrap_pyfunction!(scroll_area_vertical, m)?)?;
    m.add_function(wrap_pyfunction!(scroll_area_horizontal, m)?)?;
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
