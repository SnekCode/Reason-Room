mod examples;
mod workspace_session;
use infinite_note_canvas_canvas::RelationshipRouteCache;

use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet, HashMap},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};
#[cfg(not(target_arch = "wasm32"))]
use std::{
    env,
    fs::OpenOptions,
    io::Write,
    sync::atomic::{AtomicBool, Ordering},
};

use eframe::egui::{
    self, Align, Align2, Color32, FontId, Layout, Painter, PointerButton, Pos2, Rect, Sense,
    Stroke, StrokeKind, Vec2,
};
use infinite_note_canvas_canvas::{
    Camera, CanvasCommand, CanvasDataset, CanvasInteraction, CanvasItemBounds, CanvasItemId,
    ConceptCard, ConceptKind, MAX_SCALE, MIN_SCALE, Point, Rect as CanvasRect, Relationship,
    RelationshipKind, SemanticZoomCardLayerMode, SemanticZoomCardLayerVisibility,
    SemanticZoomFrame, SemanticZoomLayer, SemanticZoomLayerPercentSettings,
    SemanticZoomLayerProjection, SemanticZoomLevel, VisualBoundary, WHEEL_ZOOM_SENSITIVITY,
    WorkspaceDocument, ZoomPercentRange, canvas_item_move_ids_with_contained_descendants,
    card_move_ids_with_contained_descendants, card_parent_ids, cards_are_in_containment_chain,
    relationship_path_around_obstacles, semantic_zoom_level,
};
#[cfg(test)]
use infinite_note_canvas_canvas::{
    RelationshipOverlayPlacementOverrides, SelectedConceptRelationshipOverlayPath,
};
use infinite_note_canvas_workspace_session::{
    ConceptDraft, SemanticZoomLayerChoice, StagedWorkspaceMutation,
    evaluation_actions::ConceptInspectorEditDraft,
};
use serde::{Deserialize, Serialize};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use workspace_session::{WorkspaceSession, load_from_environment};

// Operational Atlas presentation tokens. These are product-chrome roles, not
// workspace entity categories. A workspace keeps ownership of category color.
const ATLAS_GROUND: Color32 = Color32::from_rgb(22, 34, 45);
const ATLAS_SURVEY_SLATE: Color32 = Color32::from_rgb(36, 53, 67);
const ATLAS_PAPER_SIGNAL: Color32 = Color32::from_rgb(232, 240, 245);
const ATLAS_QUIET_GRAPHITE: Color32 = Color32::from_rgb(159, 178, 191);
const ATLAS_ROUTE_BLUE: Color32 = Color32::from_rgb(104, 197, 233);
const ATLAS_CONTROL_AMBER: Color32 = Color32::from_rgb(243, 184, 91);
const ATLAS_RECOVERY_RED: Color32 = Color32::from_rgb(239, 139, 139);
const ATLAS_INSPECTOR_TITLE_SIZE: f32 = 20.0;
const ATLAS_CARD_TITLE_SIZE: f32 = 16.0;
const ATLAS_RELATIONSHIP_LABEL_SIZE: f32 = 13.0;
const ATLAS_SUPPORTING_TEXT_SIZE: f32 = 12.0;
const ATLAS_DETAIL_BODY_SIZE: f32 = 14.0;
const INSPECTOR_OUTER_MARGIN: f32 = 8.0;
const INSPECTOR_INNER_MARGIN: f32 = 14.0;
const CONTEXT_ENTITY_OPACITY_AT_OVERVIEW: f32 = 0.25;
const CONTEXT_ENTITY_OPACITY_AT_DETAIL: f32 = 0.05;
const CONTEXT_FADE_START_VIEWPORT_COVERAGE: f32 = 0.25;
const CONTEXT_FADE_COMPLETE_VIEWPORT_COVERAGE: f32 = 0.75;
const MAX_RENDERED_CARDS: usize = 750;
const MINIMUM_VISIBLE_CARD_SIZE: f32 = 3.0;
const INSPECTOR_PREFERRED_WIDTH: f32 = 380.0;
const INSPECTOR_MAXIMUM_WIDTH_FRACTION: f32 = 0.45;
const CARD_STROKE_WIDTH: f32 = 1.0;
const SELECTED_CARD_STROKE_WIDTH: f32 = 2.5;
const NEW_CONCEPT_SIZE: Point = Point::new(240.0, 120.0);
const MINIMUM_CARD_SIZE: Point = Point::new(80.0, 48.0);
// The approved compact detail-card footprint. It stays usable for selection,
// ports, and resize without collapsing into a visual marker.
const MINIMUM_CARD_INTERACTION_SCREEN_SIZE: Point = Point::new(130.5, 120.6875);
// This is a numerical floor only. Deep detail levels need sub-unit card
// geometry so a new card can retain a normal on-screen size at 409,600%.
const MINIMUM_CARD_PERSISTED_SIZE: Point = Point::new(0.001, 0.001);
const MINIMUM_POINTER_TARGET_WORLD_SIZE: f32 = 0.001;
const NEW_BOUNDARY_SIZE: Point = Point::new(560.0, 360.0);
const MINIMUM_BOUNDARY_SIZE: Point = Point::new(160.0, 96.0);
const BOUNDARY_RESIZE_HANDLE_SCREEN_SIZE: f32 = 14.0;
const CARD_RESIZE_HANDLE_SCREEN_SIZE: f32 = 12.0;
const RELATIONSHIP_LINE_JUMP_RADIUS: f32 = 8.0;
const RELATIONSHIP_LINE_JUMP_HEIGHT: f32 = 5.0;
const RELATIONSHIP_LINE_JUMP_ENDPOINT_CLEARANCE: f32 = 11.0;
const TYPE_CHANGE_TOAST_VISIBLE_SECONDS: f64 = 10.0;
const TYPE_CHANGE_TOAST_FADE_SECONDS: f64 = 0.5;
const ORDINARY_NOTICE_VISIBLE_SECONDS: f64 = TYPE_CHANGE_TOAST_VISIBLE_SECONDS;
const ORDINARY_NOTICE_FADE_SECONDS: f64 = TYPE_CHANGE_TOAST_FADE_SECONDS;
const CONCEPT_INSPECTOR_SAVE_LABEL: &str = "Save";
const CONCEPT_INSPECTOR_HEADER_GAP: f32 = 12.0;
const MAX_VISIBLE_ORDINARY_TOASTS: usize = 4;
const TOAST_STACK_SPACING: f32 = 8.0;
const TYPE_CHANGE_DIAGNOSTIC_FILE_NAME: &str = "infinite-note-canvas-type-change-failures.log";
#[cfg(not(target_arch = "wasm32"))]
const EXTERNAL_WORKSPACE_REFRESH_INTERVAL: Duration = Duration::from_millis(500);
#[cfg(not(target_arch = "wasm32"))]
const STARTUP_DIAGNOSTIC_FILE_NAME: &str = "infinite-note-canvas-startup.log";
#[cfg(not(target_arch = "wasm32"))]
const WORKSPACE_ENVIRONMENT_VARIABLE: &str = "INFINITE_NOTE_CANVAS_WORKSPACE";
#[cfg(not(target_arch = "wasm32"))]
const FALLBACK_WORKSPACE_ENVIRONMENT_VARIABLE: &str = "INFINITE_NOTE_CANVAS_FALLBACK_WORKSPACE";
#[cfg(not(target_arch = "wasm32"))]
const INITIAL_WINDOW_INNER_SIZE: [f32; 2] = [1_000.0, 720.0];
const SEMANTIC_SCALE_RAIL_MARGIN: f32 = 14.0;
const SEMANTIC_SCALE_RAIL_PREFERRED_WIDTH: f32 = 360.0;
const SEMANTIC_SCALE_RAIL_MINIMUM_WIDTH: f32 = 180.0;
const SEMANTIC_SCALE_RAIL_BAND_HEIGHT: f32 = 26.0;
const SEMANTIC_SCALE_RAIL_HEADER_LINE_HEIGHT: f32 = 14.0;
const SEMANTIC_SCALE_RAIL_PADDING: f32 = 8.0;
const SEMANTIC_SCALE_RAIL_GAP: f32 = 4.0;
const SEMANTIC_SCALE_RAIL_ZOOM_ROW_HEIGHT: f32 = 24.0;
const SEMANTIC_SCALE_RAIL_ZOOM_LABEL_WIDTH: f32 = 72.0;
const CONTEXT_TRAIL_GAP: f32 = 8.0;
const CONTEXT_TRAIL_LINE_HEIGHT: f32 = 14.0;
const SETTINGS_VIEWPORT_ID: &str = "infinite-note-canvas-settings";
const PREFERENCES_STORAGE_KEY: &str = "infinite-note-canvas-preferences-v1";
const SETTINGS_WINDOW_INNER_SIZE: [f32; 2] = [480.0, 380.0];
const SETTINGS_WINDOW_MINIMUM_SIZE: [f32; 2] = [400.0, 320.0];
const NEW_CANVAS_DEFAULT_TITLE: &str = "Untitled canvas";
const NEW_CANVAS_FILE_SUFFIX: &str = ".infinite-note.json";
const CARD_PASTE_OFFSET: f32 = 36.0;

#[cfg(any(target_os = "macos", target_os = "windows"))]
const NATIVE_SETTINGS_MENU_ITEM_ID: &str = "open-settings";
#[cfg(any(target_os = "macos", target_os = "windows"))]
const NATIVE_NEW_CANVAS_MENU_ITEM_ID: &str = "new-canvas";
#[cfg(any(target_os = "macos", target_os = "windows"))]
const NATIVE_OPEN_CANVAS_MENU_ITEM_ID: &str = "open-canvas";
#[cfg(any(target_os = "macos", target_os = "windows"))]
const NATIVE_SAVE_CANVAS_MENU_ITEM_ID: &str = "save-canvas";
#[cfg(any(target_os = "macos", target_os = "windows"))]
const NATIVE_SAVE_AS_CANVAS_MENU_ITEM_ID: &str = "save-as-canvas";

// The chrome and canvas use a fixed dark palette. Match egui's text and
// editable controls to those surfaces even when the operating system is light.
fn atlas_visuals() -> egui::Visuals {
    let mut visuals = egui::Visuals::dark();
    visuals.override_text_color = Some(ATLAS_PAPER_SIGNAL);
    visuals.panel_fill = ATLAS_GROUND;
    visuals.window_fill = ATLAS_SURVEY_SLATE;
    visuals.extreme_bg_color = ATLAS_GROUND;
    visuals
}

fn settings_window_title() -> &'static str {
    if cfg!(target_os = "macos") {
        "Preferences"
    } else {
        "Settings"
    }
}

/// Prevent macOS from turning the canvas and Preferences into tabs in one
/// window. Preferences is an independent application window by design.
#[cfg(target_os = "macos")]
fn disable_macos_automatic_window_tabbing() {
    use objc2::MainThreadMarker;
    use objc2_app_kit::NSWindow;

    if let Some(main_thread) = MainThreadMarker::new() {
        NSWindow::setAllowsAutomaticWindowTabbing(false, main_thread);
    }
}

/// Starts the supported native application shell.
#[cfg(not(target_arch = "wasm32"))]
pub fn run_native() -> eframe::Result<()> {
    #[cfg(target_os = "macos")]
    disable_macos_automatic_window_tabbing();

    let native_options = eframe::NativeOptions {
        // Root-window geometry belongs to application chrome, not workspace
        // data. eframe restores it from its local application storage.
        persist_window: true,
        viewport: egui::ViewportBuilder::default()
            .with_title("Reason Room")
            .with_inner_size(INITIAL_WINDOW_INNER_SIZE)
            .with_min_inner_size([800.0, 600.0]),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };

    let startup_diagnostic_path = startup_diagnostic_path(
        env::var_os(WORKSPACE_ENVIRONMENT_VARIABLE)
            .or_else(|| env::var_os(FALLBACK_WORKSPACE_ENVIRONMENT_VARIABLE))
            .map(PathBuf::from),
    );
    // Keep the existing eframe storage identity while the product name changes.
    let result = eframe::run_native(
        "Infinite Note Canvas",
        native_options,
        Box::new(|creation_context| Ok(Box::new(InfiniteNoteCanvasApp::new(creation_context)))),
    );

    if let Err(error) = &result {
        let diagnostic =
            format_startup_failure_diagnostic(&startup_diagnostic_path, &error.to_string());
        match append_diagnostic(&startup_diagnostic_path, &diagnostic) {
            Ok(()) => eprintln!(
                "Reason Room could not initialize. Startup diagnostic log: {}",
                startup_diagnostic_path.display()
            ),
            Err(write_error) => eprintln!(
                "Reason Room could not initialize, and its startup diagnostic log could not be written at {}: {write_error}",
                startup_diagnostic_path.display()
            ),
        }
    }

    result
}

/// A JavaScript-owned handle for the browser application.
///
/// The canvas and the entire UI model run in WebAssembly. Browser access to a
/// local document requires an explicit person-selected file handle.
#[cfg(target_arch = "wasm32")]
#[derive(Clone)]
#[wasm_bindgen]
pub struct WebHandle {
    runner: eframe::WebRunner,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window, js_name = infiniteNoteCanvasDownload)]
    fn download_browser_workspace(source: &str, file_name: &str);

    #[wasm_bindgen(js_namespace = window, js_name = infiniteNoteCanvasOpenWorkspace)]
    fn open_browser_workspace_picker();

    #[wasm_bindgen(js_namespace = window, js_name = infiniteNoteCanvasOpenWatchedWorkspace)]
    fn open_watched_browser_workspace_picker();

    #[wasm_bindgen(js_namespace = window, js_name = infiniteNoteCanvasPersistWorkspace)]
    fn persist_browser_workspace(source: &str);

    #[wasm_bindgen(js_namespace = window, js_name = infiniteNoteCanvasStopWatchingWorkspace)]
    fn stop_watching_browser_workspace();
}

#[cfg(target_arch = "wasm32")]
impl Default for WebHandle {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn remembered_workspace_path(
    last_opened_canvas_path: Option<&Path>,
    has_explicit_workspace_override: bool,
) -> Option<PathBuf> {
    (!has_explicit_workspace_override)
        .then(|| last_opened_canvas_path.map(Path::to_path_buf))
        .flatten()
}

#[cfg(not(target_arch = "wasm32"))]
fn load_initial_workspace_session(
    last_opened_canvas_path: Option<&Path>,
) -> Result<WorkspaceSession, String> {
    if let Some(path) = remembered_workspace_path(
        last_opened_canvas_path,
        workspace_session::has_explicit_workspace_override(),
    ) && let Ok(session) = WorkspaceSession::load_default(path)
    {
        return Ok(session);
    }
    load_from_environment()
}

#[cfg(target_arch = "wasm32")]
fn load_initial_workspace_session(
    _last_opened_canvas_path: Option<&Path>,
) -> Result<WorkspaceSession, String> {
    load_from_environment()
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WebHandle {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            runner: eframe::WebRunner::new(),
        }
    }

    /// Starts the WebAssembly application in the supplied canvas.
    pub async fn start(&self, canvas: web_sys::HtmlCanvasElement) -> Result<(), JsValue> {
        self.runner
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(|creation_context| {
                    Ok(Box::new(InfiniteNoteCanvasApp::new(creation_context)))
                }),
            )
            .await
    }

    /// Stops the app and removes its browser event handlers.
    pub fn destroy(&self) {
        self.runner.destroy();
    }

    /// Lets JavaScript report a trapped application panic.
    pub fn panic_message(&self) -> Option<String> {
        self.runner.panic_summary().map(|summary| summary.message())
    }

    /// Imports a user-selected local workspace into browser-local storage.
    ///
    /// The browser file picker provides the source. This method never uses an
    /// ambient desktop path.
    pub fn import_local_workspace(&self, source: String, file_name: String) -> Result<(), JsValue> {
        let mut app = self
            .runner
            .app_mut::<InfiniteNoteCanvasApp>()
            .ok_or_else(|| JsValue::from_str("the browser canvas is not running"))?;
        app.import_local_browser_workspace(&source, &file_name)
            .map_err(|error| JsValue::from_str(&error))
    }

    /// Restores the last browser-local canvas after this page starts.
    pub fn restore_browser_workspace(&self, source: String) -> Result<(), JsValue> {
        let mut app = self
            .runner
            .app_mut::<InfiniteNoteCanvasApp>()
            .ok_or_else(|| JsValue::from_str("the browser canvas is not running"))?;
        app.restore_browser_workspace(&source)
            .map_err(|error| JsValue::from_str(&error))
    }

    /// Refreshes a user-selected local canvas after its file changes outside
    /// the browser, for example through the local MCP server.
    ///
    /// Returns false while local authoring is active. JavaScript retries the
    /// same source after the canvas becomes idle.
    pub fn refresh_local_workspace(
        &self,
        source: String,
        file_name: String,
    ) -> Result<bool, JsValue> {
        let mut app = self
            .runner
            .app_mut::<InfiniteNoteCanvasApp>()
            .ok_or_else(|| JsValue::from_str("the browser canvas is not running"))?;
        app.refresh_local_browser_workspace(&source, &file_name)
            .map_err(|error| JsValue::from_str(&error))
    }
}

/// Holds the platform menu for the full life of the application.
///
/// The native menu items own operating-system resources, so dropping this
/// record would remove the menu even though the eframe window remains open.
#[cfg(any(target_os = "macos", target_os = "windows"))]
struct NativeApplicationMenu {
    _menu: muda::Menu,
}

/// Persistent application choices. They are stored locally and never change
/// workspace facts, cards, layers, or relationships.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
struct StoredPreferences {
    show_canvas_grid: bool,
    show_relationship_labels: bool,
    last_opened_canvas_path: Option<PathBuf>,
    default_card_width: f32,
    default_card_height: f32,
    default_card_type_id: Option<String>,
    default_boundary_width: f32,
    default_boundary_height: f32,
    default_layer_start_percent: f32,
    default_layer_end_percent: f32,
    default_layer_fade_percent: f32,
}

impl Default for StoredPreferences {
    fn default() -> Self {
        let layer_settings = semantic_zoom_layer_creation_percent_settings(0);
        Self {
            show_canvas_grid: true,
            show_relationship_labels: true,
            last_opened_canvas_path: None,
            default_card_width: NEW_CONCEPT_SIZE.x,
            default_card_height: NEW_CONCEPT_SIZE.y,
            default_card_type_id: None,
            default_boundary_width: NEW_BOUNDARY_SIZE.x,
            default_boundary_height: NEW_BOUNDARY_SIZE.y,
            default_layer_start_percent: layer_settings.active_zoom_range.start_percent,
            default_layer_end_percent: layer_settings.active_zoom_range.end_percent,
            default_layer_fade_percent: layer_settings.fade_percent,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum PreferencesTab {
    #[default]
    Canvas,
    NewCards,
    NewBoundaries,
    ZoomLayers,
}

impl PreferencesTab {
    const ALL: [Self; 4] = [
        Self::Canvas,
        Self::NewCards,
        Self::NewBoundaries,
        Self::ZoomLayers,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Canvas => "Canvas",
            Self::NewCards => "New cards",
            Self::NewBoundaries => "New boundaries",
            Self::ZoomLayers => "Zoom layers",
        }
    }
}

/// Session-only Preferences window state. The values in `stored` survive
/// restarts; opening and focus state deliberately do not.
#[derive(Default)]
struct PreferencesWindowState {
    open: bool,
    focus_requested: bool,
    selected_tab: PreferencesTab,
    stored: StoredPreferences,
}

impl PreferencesWindowState {
    fn from_stored(stored: StoredPreferences) -> Self {
        Self {
            stored,
            ..Self::default()
        }
    }
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
fn install_native_application_menu(
    creation_context: &eframe::CreationContext<'_>,
    settings_window_requested: Arc<AtomicBool>,
    new_canvas_requested: Arc<AtomicBool>,
    open_canvas_requested: Arc<AtomicBool>,
    save_canvas_requested: Arc<AtomicBool>,
    save_as_canvas_requested: Arc<AtomicBool>,
    example_requested: Arc<Mutex<Option<examples::Example>>>,
) -> Result<NativeApplicationMenu, String> {
    #[cfg(target_os = "macos")]
    let _ = creation_context;

    let settings_item = muda::MenuItem::with_id(
        NATIVE_SETTINGS_MENU_ITEM_ID,
        if cfg!(target_os = "macos") {
            "Preferences..."
        } else {
            "Settings"
        },
        true,
        None,
    );
    let new_canvas_item =
        muda::MenuItem::with_id(NATIVE_NEW_CANVAS_MENU_ITEM_ID, "New Canvas...", true, None);
    let open_canvas_item =
        muda::MenuItem::with_id(NATIVE_OPEN_CANVAS_MENU_ITEM_ID, "Open...", true, None);
    let save_canvas_item =
        muda::MenuItem::with_id(NATIVE_SAVE_CANVAS_MENU_ITEM_ID, "Save", true, None);
    let save_as_canvas_item =
        muda::MenuItem::with_id(NATIVE_SAVE_AS_CANVAS_MENU_ITEM_ID, "Save As...", true, None);
    let example_items: Vec<_> = examples::ALL
        .iter()
        .map(|example| muda::MenuItem::with_id(example.menu_id, example.title, true, None))
        .collect();
    let example_refs: Vec<&dyn muda::IsMenuItem> = example_items
        .iter()
        .map(|item| item as &dyn muda::IsMenuItem)
        .collect();
    let examples_menu = muda::Submenu::with_items("Examples", true, &example_refs)
        .map_err(|error| error.to_string())?;
    let menu = muda::Menu::new();
    #[cfg(target_os = "macos")]
    {
        let about = muda::PredefinedMenuItem::about(Some("About Reason Room"), None);
        let first_separator = muda::PredefinedMenuItem::separator();
        let second_separator = muda::PredefinedMenuItem::separator();
        let services = muda::PredefinedMenuItem::services(None);
        let third_separator = muda::PredefinedMenuItem::separator();
        let hide = muda::PredefinedMenuItem::hide(None);
        let hide_others = muda::PredefinedMenuItem::hide_others(None);
        let show_all = muda::PredefinedMenuItem::show_all(None);
        let fourth_separator = muda::PredefinedMenuItem::separator();
        let quit = muda::PredefinedMenuItem::quit(None);
        let application_menu = muda::Submenu::with_items(
            "Reason Room",
            true,
            &[
                &about,
                &first_separator,
                &settings_item,
                &second_separator,
                &services,
                &third_separator,
                &hide,
                &hide_others,
                &show_all,
                &fourth_separator,
                &quit,
            ],
        )
        .map_err(|error| error.to_string())?;
        menu.append(&application_menu)
            .map_err(|error| error.to_string())?;
        let new_open_separator = muda::PredefinedMenuItem::separator();
        let open_save_separator = muda::PredefinedMenuItem::separator();
        let file_menu = muda::Submenu::with_items(
            "File",
            true,
            &[
                &new_canvas_item,
                &new_open_separator,
                &open_canvas_item,
                &open_save_separator,
                &save_canvas_item,
                &save_as_canvas_item,
                &examples_menu,
            ],
        )
        .map_err(|error| error.to_string())?;
        menu.append(&file_menu).map_err(|error| error.to_string())?;
        menu.init_for_nsapp();
    }

    #[cfg(target_os = "windows")]
    {
        let new_open_separator = muda::PredefinedMenuItem::separator();
        let open_save_separator = muda::PredefinedMenuItem::separator();
        let file_menu = muda::Submenu::with_items(
            "File",
            true,
            &[
                &new_canvas_item,
                &new_open_separator,
                &open_canvas_item,
                &open_save_separator,
                &save_canvas_item,
                &save_as_canvas_item,
                &examples_menu,
            ],
        )
        .map_err(|error| error.to_string())?;
        menu.append(&file_menu).map_err(|error| error.to_string())?;
        let settings_menu = muda::Submenu::with_items("Settings", true, &[&settings_item])
            .map_err(|error| error.to_string())?;
        menu.append(&settings_menu)
            .map_err(|error| error.to_string())?;
        use raw_window_handle::{HasWindowHandle as _, RawWindowHandle};

        let window_handle = creation_context
            .window_handle()
            .map_err(|error| format!("could not read the native window handle: {error}"))?;
        let RawWindowHandle::Win32(handle) = window_handle.as_raw() else {
            return Err("the native window does not provide a Windows handle".to_string());
        };
        // The eframe creation context guarantees this handle is the live root
        // window for the full duration of the application menu.
        unsafe {
            menu.init_for_hwnd(handle.hwnd.get())
                .map_err(|error| error.to_string())?;
        }
    }

    let menu_context = creation_context.egui_ctx.clone();
    muda::MenuEvent::set_event_handler(Some(move |event: muda::MenuEvent| {
        if let Some(example) = examples::ALL
            .iter()
            .find(|example| event.id.as_ref() == example.menu_id)
        {
            *example_requested.lock().expect("example menu lock") = Some(*example);
            menu_context.request_repaint();
        }
        if event.id.as_ref() == NATIVE_SETTINGS_MENU_ITEM_ID {
            settings_window_requested.store(true, Ordering::Release);
            menu_context.request_repaint();
        } else if event.id.as_ref() == NATIVE_NEW_CANVAS_MENU_ITEM_ID {
            new_canvas_requested.store(true, Ordering::Release);
            menu_context.request_repaint();
        } else if event.id.as_ref() == NATIVE_OPEN_CANVAS_MENU_ITEM_ID {
            open_canvas_requested.store(true, Ordering::Release);
            menu_context.request_repaint();
        } else if event.id.as_ref() == NATIVE_SAVE_CANVAS_MENU_ITEM_ID {
            save_canvas_requested.store(true, Ordering::Release);
            menu_context.request_repaint();
        } else if event.id.as_ref() == NATIVE_SAVE_AS_CANVAS_MENU_ITEM_ID {
            save_as_canvas_requested.store(true, Ordering::Release);
            menu_context.request_repaint();
        }
    }));

    Ok(NativeApplicationMenu { _menu: menu })
}

struct InfiniteNoteCanvasApp {
    dataset: CanvasDataset,
    interaction: CanvasInteraction,
    workspace_state: WorkspaceState,
    gesture: Option<PointerGesture>,
    previous_shift_hover: Option<Point>,
    dialog: Option<EditorDialog>,
    inspector_draft: Option<ConceptInspectorDraft>,
    card_clipboard: Option<CardClipboard>,
    publish_card_clipboard: bool,
    last_canvas_pointer: Option<Point>,
    last_canvas_viewport: Option<Rect>,
    pending_concept_name_focus_id: Option<u32>,
    pending_creation_name_focus: Option<CreationNameFocus>,
    inspector_collapsed: bool,
    side_pane_mode: SidePaneMode,
    semantic_zoom_layer_id: Option<String>,
    semantic_zoom_layer_draft: Option<SemanticZoomLayerEditorDraft>,
    preferences: Arc<Mutex<PreferencesWindowState>>,
    #[cfg(not(target_arch = "wasm32"))]
    settings_window_requested: Arc<AtomicBool>,
    #[cfg(not(target_arch = "wasm32"))]
    new_canvas_requested: Arc<AtomicBool>,
    #[cfg(not(target_arch = "wasm32"))]
    open_canvas_requested: Arc<AtomicBool>,
    #[cfg(not(target_arch = "wasm32"))]
    save_canvas_requested: Arc<AtomicBool>,
    #[cfg(not(target_arch = "wasm32"))]
    save_as_canvas_requested: Arc<AtomicBool>,
    #[cfg(not(target_arch = "wasm32"))]
    example_requested: Arc<Mutex<Option<examples::Example>>>,
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    _native_application_menu: Option<NativeApplicationMenu>,
    show_all_cards: bool,
    roll_up_relationships: bool,
    card_creation_draft: ConceptEditorDraft,
    boundary_creation_draft: BoundaryEditorDraft,
    layer_creation_title: String,
    layer_creation_active_zoom_start_percent: f32,
    layer_creation_active_zoom_end_percent: f32,
    layer_creation_fade_percent: f32,
    layer_cross_section_drafts: Vec<LayerRangeDraft>,
    layer_cross_section_creating: bool,
    layer_creation_index: usize,
    /// Temporary canvas controls. These never change a saved Lens filter.
    canvas_search_query: String,
    canvas_search_match_index: usize,
    canvas_type_filter_ids: BTreeSet<String>,
    canvas_type_id_by_concept_id: BTreeMap<u32, String>,
    canvas_context_menu: Option<CanvasContextMenuState>,
    pending_canvas_creation_position: Option<Point>,
    version_creation_draft: WorkspaceTitleDraft,
    lens_creation_draft: LensCreationDraft,
    selected_boundary_id: Option<String>,
    selected_relationship_id: Option<String>,
    primary_relationship_concept_id: Option<u32>,
    boundary_inspector_draft: Option<BoundaryInspectorDraft>,
    hover_target: Option<CanvasHoverTarget>,
    lens_cameras: HashMap<LensViewportKey, Camera>,
    lens_canvas_controls: HashMap<LensViewportKey, LensCanvasControls>,
    canvas_viewport_size: Point,
    fit_example_on_next_frame: bool,
    relationship_route_cache: RefCell<RelationshipRouteCache>,
    notices: Vec<Notice>,
    type_change_failure_toast: Option<TypeChangeFailureToast>,
    ui_time_seconds: f64,
    #[cfg(not(target_arch = "wasm32"))]
    next_external_workspace_refresh_at: f64,
    #[cfg(not(target_arch = "wasm32"))]
    external_workspace_refresh_error: Option<String>,
}

enum WorkspaceState {
    Loaded(Box<WorkspaceSession>),
    ReadOnlyFailure(String),
}

#[derive(Debug, PartialEq)]
enum PointerGesture {
    Pan,
    Selection,
    ZoomSlider,
    Move {
        item_ids: Vec<CanvasItemId>,
        anchor_world_position: Point,
        preview_world_delta: Point,
    },
    ResizeBoundary {
        boundary_id: String,
        initial_bounds: CanvasRect,
        anchor_world_position: Point,
        preview_bounds: CanvasRect,
    },
    ResizeCard {
        concept_id: u32,
        initial_bounds: CanvasRect,
        anchor_world_position: Point,
        centered: bool,
        preview_bounds: CanvasRect,
    },
    CreateRelationship {
        source_id: u32,
        source_port: CardEdge,
        preview_world_position: Point,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum SuccessNotice {
    Default(String),
    Specific { title: String, detail: String },
    None,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum CardContainmentChange {
    Entered {
        card_title: String,
        parent_title: String,
    },
    Exited {
        card_title: String,
        parent_title: String,
    },
    Moved {
        card_title: String,
        previous_parent_title: String,
        parent_title: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum CanvasHoverTarget {
    Card(u32),
    CardResize(u32),
    RelationshipPort { concept_id: u32, edge: CardEdge },
    Boundary(String),
    Relationship(String),
    ScaleLayer(String),
    ZoomSlider,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum CanvasDeleteTarget {
    Cards(Vec<u32>),
    Boundary(String),
    Relationship(String),
}

/// The canvas item that owns the current right-click menu. The target is
/// recorded at click time so a menu stays stable even if the pointer moves.
#[derive(Clone, Debug, PartialEq, Eq)]
enum CanvasContextMenuTarget {
    Canvas,
    Card(u32),
    Boundary(String),
    Relationship(String),
}

#[derive(Clone, Debug, PartialEq)]
struct CanvasContextMenuState {
    target: CanvasContextMenuTarget,
    world_position: Point,
}

#[derive(Clone, Debug, PartialEq)]
enum CanvasContextMenuAction {
    InspectCard(u32),
    ShiftCardLayer(u32, bool),
    CopyCard(u32),
    DeleteCard(u32),
    InspectBoundary(String),
    DeleteBoundary(String),
    InspectRelationship(String),
    DeleteRelationship(String),
    PasteCards,
    CreateCard(Point),
    CreateBoundary(Point),
    CreateLayer,
    CenterView,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CardEdge {
    Top,
    Right,
    Bottom,
    Left,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SidePaneMode {
    Inspect,
    CreateCard,
    CreateBoundary,
    CreateVersion,
    CreateLens,
    EditLayers,
    LayerCrossSection,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CreationNameFocus {
    Card,
    Boundary,
    Version,
    View,
    Layer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MoveCommitRoute {
    Single(u32),
    Group,
}

#[derive(Clone, Debug, PartialEq)]
struct SemanticZoomLayerEditorDraft {
    layer_id: String,
    title: String,
    active_zoom_start_percent: f32,
    active_zoom_end_percent: f32,
    fade_percent: f32,
    card_scale: f32,
}

/// The editable range controls rendered for one row in the layer cross-section.
/// The persisted layer remains the source of membership and card ordering.
#[derive(Clone, Debug, PartialEq)]
struct LayerRangeDraft {
    layer_id: String,
    title: String,
    active_zoom_start_percent: f32,
    active_zoom_end_percent: f32,
    fade_percent: f32,
    card_scale: f32,
}

/// A presentational ordinal for a Lens projection or an unlayered camera view.
/// It has no persisted state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SemanticScaleBand {
    Broad,
    Working,
    Detail,
}

impl SemanticScaleBand {
    const ALL: [Self; 3] = [Self::Broad, Self::Working, Self::Detail];

    const fn label(self) -> &'static str {
        match self {
            Self::Broad => "Broad",
            Self::Working => "Working",
            Self::Detail => "Detail",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SemanticScaleRailSource {
    Camera,
    LensLayer,
    LensLayerGap,
}

#[derive(Clone, Debug, PartialEq)]
struct SemanticScaleRailBand {
    layer_id: Option<String>,
    label: String,
    bounds: Rect,
    opacity: f32,
    active: bool,
}

#[derive(Clone, Debug, PartialEq)]
struct SemanticScaleRail {
    bounds: Rect,
    source: SemanticScaleRailSource,
    active_band: Option<SemanticScaleBand>,
    active_layer_title: Option<String>,
    header_lines: Vec<String>,
    bands: Vec<SemanticScaleRailBand>,
    zoom_slider_bounds: Rect,
    zoom_slider_fraction: f32,
    zoom_percent: f32,
}

impl SemanticScaleRail {
    #[cfg(test)]
    fn description(&self) -> String {
        match (self.source, self.active_band) {
            (SemanticScaleRailSource::Camera, Some(band)) => {
                format!("Camera scale: {}", band.label())
            }
            (SemanticScaleRailSource::LensLayer, _) => {
                format!(
                    "Current layer: {}",
                    self.active_layer_title
                        .as_deref()
                        .unwrap_or("no active layer")
                )
            }
            (SemanticScaleRailSource::LensLayerGap, None) => {
                "Current layer: no active layer".to_string()
            }
            _ => "Current layer: unavailable".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
struct Notice {
    title: String,
    detail: String,
    failed: bool,
    shown_at_seconds: f64,
    hover_started_at_seconds: Option<f64>,
    paused_seconds: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum OrdinaryNoticeVisualState {
    Opaque,
    Fading { opacity: f32 },
    Dismissed,
}

impl Notice {
    fn new(
        title: impl Into<String>,
        detail: impl Into<String>,
        failed: bool,
        shown_at_seconds: f64,
    ) -> Self {
        Self {
            title: title.into(),
            detail: detail.into(),
            failed,
            shown_at_seconds,
            hover_started_at_seconds: None,
            paused_seconds: 0.0,
        }
    }

    fn set_hovered(&mut self, hovered: bool, now_seconds: f64) {
        match (hovered, self.hover_started_at_seconds) {
            (true, None) => self.hover_started_at_seconds = Some(now_seconds),
            (false, Some(hover_started_at_seconds)) => {
                self.paused_seconds += (now_seconds - hover_started_at_seconds).max(0.0);
                self.hover_started_at_seconds = None;
            }
            _ => {}
        }
    }

    fn active_elapsed_seconds(&self, now_seconds: f64) -> f64 {
        let active_hover_seconds = self
            .hover_started_at_seconds
            .map(|hover_started_at_seconds| (now_seconds - hover_started_at_seconds).max(0.0))
            .unwrap_or_default();
        (now_seconds - self.shown_at_seconds - self.paused_seconds - active_hover_seconds).max(0.0)
    }

    fn visual_state(&self, now_seconds: f64) -> OrdinaryNoticeVisualState {
        let elapsed_seconds = self.active_elapsed_seconds(now_seconds);
        if elapsed_seconds < ORDINARY_NOTICE_VISIBLE_SECONDS {
            return OrdinaryNoticeVisualState::Opaque;
        }

        let fade_elapsed_seconds = elapsed_seconds - ORDINARY_NOTICE_VISIBLE_SECONDS;
        if fade_elapsed_seconds < ORDINARY_NOTICE_FADE_SECONDS {
            return OrdinaryNoticeVisualState::Fading {
                opacity: (1.0 - fade_elapsed_seconds / ORDINARY_NOTICE_FADE_SECONDS) as f32,
            };
        }

        OrdinaryNoticeVisualState::Dismissed
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ToastStackRoute {
    Ordinary(usize),
    TypeChangeFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InspectorSaveFailureRoute {
    OrdinaryNotice,
    TypeChangeToast,
}

#[derive(Clone, Debug)]
struct TypeChangeFailureToast {
    error: String,
    diagnostic_path: PathBuf,
    diagnostic_write_error: Option<String>,
    shown_at_seconds: f64,
    hover_started_at_seconds: Option<f64>,
    paused_seconds: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum TypeChangeToastVisualState {
    Opaque,
    Fading { opacity: f32 },
    Dismissed,
}

impl TypeChangeFailureToast {
    fn new(
        error: String,
        diagnostic_path: PathBuf,
        diagnostic_write_error: Option<String>,
        shown_at_seconds: f64,
    ) -> Self {
        Self {
            error,
            diagnostic_path,
            diagnostic_write_error,
            shown_at_seconds,
            hover_started_at_seconds: None,
            paused_seconds: 0.0,
        }
    }

    fn set_hovered(&mut self, hovered: bool, now_seconds: f64) {
        match (hovered, self.hover_started_at_seconds) {
            (true, None) => self.hover_started_at_seconds = Some(now_seconds),
            (false, Some(hover_started_at_seconds)) => {
                self.paused_seconds += (now_seconds - hover_started_at_seconds).max(0.0);
                self.hover_started_at_seconds = None;
            }
            _ => {}
        }
    }

    fn active_elapsed_seconds(&self, now_seconds: f64) -> f64 {
        let active_hover_seconds = self
            .hover_started_at_seconds
            .map(|hover_started_at_seconds| (now_seconds - hover_started_at_seconds).max(0.0))
            .unwrap_or_default();
        (now_seconds - self.shown_at_seconds - self.paused_seconds - active_hover_seconds).max(0.0)
    }

    fn visual_state(&self, now_seconds: f64) -> TypeChangeToastVisualState {
        let elapsed_seconds = self.active_elapsed_seconds(now_seconds);
        if elapsed_seconds < TYPE_CHANGE_TOAST_VISIBLE_SECONDS {
            return TypeChangeToastVisualState::Opaque;
        }

        let fade_elapsed_seconds = elapsed_seconds - TYPE_CHANGE_TOAST_VISIBLE_SECONDS;
        if fade_elapsed_seconds < TYPE_CHANGE_TOAST_FADE_SECONDS {
            return TypeChangeToastVisualState::Fading {
                opacity: (1.0 - fade_elapsed_seconds / TYPE_CHANGE_TOAST_FADE_SECONDS) as f32,
            };
        }

        TypeChangeToastVisualState::Dismissed
    }
}

#[derive(Debug)]
enum EditorDialog {
    NewCanvas {
        draft: NewCanvasDraft,
    },
    #[cfg(target_arch = "wasm32")]
    SaveAsBrowserCanvas {
        draft: NewCanvasDraft,
    },
    ConfirmTypeChange {
        draft: ConceptInspectorEditDraft,
        staged: Box<StagedWorkspaceMutation>,
        lens_title: String,
    },
    RenameVersion {
        version_id: String,
        draft: WorkspaceTitleDraft,
    },
    RenameLens {
        lens_id: String,
        draft: WorkspaceTitleDraft,
    },
    ChooseRelationshipKind {
        source_id: u32,
        target_id: u32,
        position: Pos2,
    },
    ConfirmRelationshipRemoval {
        relationship: RelationshipView,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HistoryCommand {
    Undo,
    Redo,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HistoryCommandRoute {
    Apply(HistoryCommand),
    Consume,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum WorkspaceStructureAction {
    CreateVersion { source_title: String },
    RenameVersion { version_id: String, title: String },
    DeleteVersion { version_id: String },
    CreateLens { source_title: String },
    RenameLens { lens_id: String, title: String },
    DeleteLens { lens_id: String },
}

#[derive(Clone, Debug)]
struct ConceptEditorDraft {
    title: String,
    type_id: Option<String>,
    summary: String,
}

#[derive(Clone, Debug)]
struct BoundaryEditorDraft {
    title: String,
    summary: String,
}

#[derive(Clone, Debug)]
struct WorkspaceTitleDraft {
    title: String,
}

#[derive(Clone, Debug)]
struct LensCreationDraft {
    title: String,
    clone_current: bool,
}

#[derive(Clone, Debug)]
struct CardClipboard {
    version_id: String,
    lens_id: String,
    concept_ids: Vec<u32>,
    source: WorkspaceDocument,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CardClipboardCommand {
    Copy,
    Paste,
}

#[derive(Clone, Debug)]
struct NewCanvasDraft {
    title: String,
    #[cfg(not(target_arch = "wasm32"))]
    save_path: String,
    #[cfg(not(target_arch = "wasm32"))]
    save_path_tracks_title: bool,
}

impl NewCanvasDraft {
    fn suggested() -> Self {
        let title = NEW_CANVAS_DEFAULT_TITLE.to_string();
        #[cfg(target_arch = "wasm32")]
        {
            Self { title }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self {
                save_path: suggested_new_canvas_path(&title).display().to_string(),
                title,
                save_path_tracks_title: true,
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn browser_save_as(title: String) -> Self {
        Self { title }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn update_suggested_path(&mut self) {
        if self.save_path_tracks_title {
            self.save_path = suggested_new_canvas_path(&self.title).display().to_string();
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct BoundaryInspectorDraft {
    boundary_id: String,
    title: String,
    summary: String,
}

#[derive(Clone, Debug)]
struct ConceptInspectorDraft {
    concept_id: u32,
    title: String,
    type_id: String,
    summary: String,
    details: String,
    assertion_attributes: BTreeMap<String, String>,
    assertion_key: String,
    assertion_value: String,
    assertion_error: Option<String>,
}

#[derive(Clone, Debug, Default)]
struct LensCanvasControls {
    search_query: String,
    search_match_index: usize,
    type_filter_ids: BTreeSet<String>,
    show_all_cards: bool,
    roll_up_relationships: bool,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct LensViewportKey {
    version_id: String,
    lens_id: String,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug)]
struct RelationshipOverlayViewport<'a> {
    visible_world: CanvasRect,
    displayed_card_bounds_by_concept_id: &'a BTreeMap<u32, CanvasRect>,
}

impl InfiniteNoteCanvasApp {
    fn new(creation_context: &eframe::CreationContext<'_>) -> Self {
        // The settings viewport is a separate operating-system window on the
        // supported desktop targets. Do not allow egui to embed it in canvas.
        #[cfg(not(target_arch = "wasm32"))]
        creation_context.egui_ctx.set_embed_viewports(false);
        creation_context.egui_ctx.set_theme(egui::Theme::Dark);
        creation_context.egui_ctx.set_visuals(atlas_visuals());

        let stored_preferences: StoredPreferences = creation_context
            .storage
            .and_then(|storage| eframe::get_value(storage, PREFERENCES_STORAGE_KEY))
            .unwrap_or_default();

        let (dataset, workspace_state) = match load_initial_workspace_session(
            stored_preferences.last_opened_canvas_path.as_deref(),
        ) {
            Ok(session) => (
                session.dataset().clone(),
                WorkspaceState::Loaded(Box::new(session)),
            ),
            Err(error) => (
                CanvasDataset::from_records(Vec::new(), Vec::new()),
                WorkspaceState::ReadOnlyFailure(error),
            ),
        };
        #[cfg(not(target_arch = "wasm32"))]
        let settings_window_requested = Arc::new(AtomicBool::new(false));
        #[cfg(not(target_arch = "wasm32"))]
        let new_canvas_requested = Arc::new(AtomicBool::new(false));
        #[cfg(not(target_arch = "wasm32"))]
        let open_canvas_requested = Arc::new(AtomicBool::new(false));
        #[cfg(not(target_arch = "wasm32"))]
        let save_canvas_requested = Arc::new(AtomicBool::new(false));
        #[cfg(not(target_arch = "wasm32"))]
        let save_as_canvas_requested = Arc::new(AtomicBool::new(false));
        #[cfg(not(target_arch = "wasm32"))]
        let example_requested = Arc::new(Mutex::new(None));
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        let native_application_menu = install_native_application_menu(
            creation_context,
            Arc::clone(&settings_window_requested),
            Arc::clone(&new_canvas_requested),
            Arc::clone(&open_canvas_requested),
            Arc::clone(&save_canvas_requested),
            Arc::clone(&save_as_canvas_requested),
            Arc::clone(&example_requested),
        )
        .map_or_else(
            |error| {
                eprintln!("Reason Room could not install its native settings menu: {error}");
                None
            },
            Some,
        );

        let mut app = Self {
            dataset,
            interaction: CanvasInteraction::new(Camera::default()),
            workspace_state,
            gesture: None,
            previous_shift_hover: None,
            dialog: None,
            inspector_draft: None,
            card_clipboard: None,
            publish_card_clipboard: false,
            last_canvas_pointer: None,
            last_canvas_viewport: None,
            pending_concept_name_focus_id: None,
            pending_creation_name_focus: None,
            inspector_collapsed: false,
            side_pane_mode: SidePaneMode::Inspect,
            semantic_zoom_layer_id: None,
            semantic_zoom_layer_draft: None,
            preferences: Arc::new(Mutex::new(PreferencesWindowState::from_stored(
                stored_preferences,
            ))),
            #[cfg(not(target_arch = "wasm32"))]
            settings_window_requested,
            #[cfg(not(target_arch = "wasm32"))]
            new_canvas_requested,
            #[cfg(not(target_arch = "wasm32"))]
            open_canvas_requested,
            #[cfg(not(target_arch = "wasm32"))]
            save_canvas_requested,
            #[cfg(not(target_arch = "wasm32"))]
            save_as_canvas_requested,
            #[cfg(not(target_arch = "wasm32"))]
            example_requested,
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            _native_application_menu: native_application_menu,
            show_all_cards: false,
            roll_up_relationships: false,
            card_creation_draft: ConceptEditorDraft {
                title: String::new(),
                type_id: None,
                summary: String::new(),
            },
            boundary_creation_draft: BoundaryEditorDraft {
                title: String::new(),
                summary: String::new(),
            },
            layer_creation_title: String::new(),
            layer_creation_active_zoom_start_percent: 1.0,
            layer_creation_active_zoom_end_percent: 65.0,
            layer_creation_fade_percent: 10.0,
            layer_cross_section_drafts: Vec::new(),
            layer_cross_section_creating: false,
            layer_creation_index: 0,
            canvas_search_query: String::new(),
            canvas_search_match_index: 0,
            canvas_type_filter_ids: BTreeSet::new(),
            canvas_type_id_by_concept_id: BTreeMap::new(),
            canvas_context_menu: None,
            pending_canvas_creation_position: None,
            version_creation_draft: WorkspaceTitleDraft {
                title: String::new(),
            },
            lens_creation_draft: LensCreationDraft {
                title: String::new(),
                clone_current: false,
            },
            selected_boundary_id: None,
            selected_relationship_id: None,
            primary_relationship_concept_id: None,
            boundary_inspector_draft: None,
            hover_target: None,
            lens_cameras: HashMap::new(),
            lens_canvas_controls: HashMap::new(),
            canvas_viewport_size: Point::default(),
            fit_example_on_next_frame: false,
            relationship_route_cache: RefCell::default(),
            notices: Vec::new(),
            type_change_failure_toast: None,
            ui_time_seconds: 0.0,
            #[cfg(not(target_arch = "wasm32"))]
            next_external_workspace_refresh_at: 0.0,
            #[cfg(not(target_arch = "wasm32"))]
            external_workspace_refresh_error: None,
        };
        if app
            .session()
            .is_some_and(|session| examples::is_example(session.document()))
        {
            app.fit_example_on_next_frame = true;
            app.inspector_collapsed = true;
        }
        app.refresh_canvas_type_id_index();
        #[cfg(not(target_arch = "wasm32"))]
        app.remember_open_canvas_path();
        app
    }

    fn selected_concept_id(&self) -> Option<u32> {
        primary_selected_concept_id(
            self.primary_relationship_concept_id,
            self.interaction.selected_card_ids(),
        )
    }

    fn inspector_has_no_selection(&self) -> bool {
        self.side_pane_mode == SidePaneMode::Inspect
            && self.selected_concept_id().is_none()
            && self.selected_boundary_id.is_none()
            && self.selected_relationship_id.is_none()
    }

    fn set_selection(&mut self, concept_id: Option<u32>) {
        if self.selected_concept_id() != concept_id {
            self.inspector_draft = None;
        }
        self.selected_boundary_id = None;
        self.selected_relationship_id = None;
        self.boundary_inspector_draft = None;
        let selected_ids: Vec<_> = self.interaction.selected_card_ids().collect();
        for selected_id in selected_ids {
            self.interaction.remove_selected_card_id(selected_id);
        }
        if let Some(concept_id) = concept_id {
            self.interaction.select_only_card_id(concept_id);
            self.inspector_collapsed = false;
        } else {
            // The inspector is an item workspace. When a canvas action clears
            // the last item selection, return that canvas space immediately.
            self.inspector_collapsed = true;
        }
        self.primary_relationship_concept_id = concept_id;
        self.side_pane_mode = SidePaneMode::Inspect;
    }

    fn extend_selection(&mut self, concept_id: u32) {
        if !self.interaction.is_selected(concept_id) {
            self.interaction.add_selected_card_id(concept_id);
        }
        if self.primary_relationship_concept_id.is_none() {
            self.primary_relationship_concept_id = Some(concept_id);
        }
        self.inspector_collapsed = false;
        self.selected_boundary_id = None;
        self.selected_relationship_id = None;
        self.boundary_inspector_draft = None;
        self.side_pane_mode = SidePaneMode::Inspect;
    }

    fn set_boundary_selection(&mut self, boundary_id: Option<String>) {
        let selected_ids: Vec<_> = self.interaction.selected_card_ids().collect();
        for selected_id in selected_ids {
            self.interaction.remove_selected_card_id(selected_id);
        }
        if self.selected_boundary_id != boundary_id {
            self.boundary_inspector_draft = None;
        }
        self.selected_boundary_id = boundary_id;
        self.inspector_collapsed = self.selected_boundary_id.is_none();
        self.selected_relationship_id = None;
        self.inspector_draft = None;
        self.primary_relationship_concept_id = None;
        self.side_pane_mode = SidePaneMode::Inspect;
    }

    fn set_relationship_selection(&mut self, relationship_id: Option<String>) {
        let selected_ids: Vec<_> = self.interaction.selected_card_ids().collect();
        for selected_id in selected_ids {
            self.interaction.remove_selected_card_id(selected_id);
        }
        self.selected_boundary_id = None;
        self.boundary_inspector_draft = None;
        self.inspector_draft = None;
        self.primary_relationship_concept_id = None;
        self.selected_relationship_id = relationship_id;
        self.inspector_collapsed = self.selected_relationship_id.is_none();
        self.side_pane_mode = SidePaneMode::Inspect;
    }

    fn open_create_pane(&mut self, mode: SidePaneMode) {
        self.pending_canvas_creation_position = None;
        self.side_pane_mode = mode;
        self.inspector_collapsed = false;
        self.pending_creation_name_focus = match mode {
            SidePaneMode::CreateCard => Some(CreationNameFocus::Card),
            SidePaneMode::CreateBoundary => Some(CreationNameFocus::Boundary),
            SidePaneMode::CreateVersion => Some(CreationNameFocus::Version),
            SidePaneMode::CreateLens => Some(CreationNameFocus::View),
            SidePaneMode::Inspect | SidePaneMode::EditLayers | SidePaneMode::LayerCrossSection => {
                None
            }
        };
        if mode == SidePaneMode::CreateCard
            && self.card_creation_draft.title.is_empty()
            && self.card_creation_draft.summary.is_empty()
        {
            self.card_creation_draft.type_id = self.default_card_type_id();
        }
    }

    fn open_create_pane_at(&mut self, mode: SidePaneMode, world_position: Point) {
        self.open_create_pane(mode);
        self.pending_canvas_creation_position = Some(world_position);
    }

    fn stored_preferences(&self) -> StoredPreferences {
        self.preferences
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .stored
            .clone()
    }

    fn default_card_type_id(&self) -> Option<String> {
        self.stored_preferences().default_card_type_id
    }

    fn stage_action(
        &self,
        action: impl FnOnce(&WorkspaceSession) -> Result<StagedWorkspaceMutation, String>,
    ) -> Result<StagedWorkspaceMutation, String> {
        match &self.workspace_state {
            WorkspaceState::Loaded(session) => action(session),
            WorkspaceState::ReadOnlyFailure(error) => Err(format!(
                "the workspace is unavailable and no action was saved: {error}"
            )),
        }
    }

    fn commit_staged_action(
        &mut self,
        staged: Result<StagedWorkspaceMutation, String>,
        selection: Option<u32>,
        preserve_selection: bool,
        detail: impl Into<String>,
    ) {
        self.commit_staged_action_with_notice(
            staged,
            selection,
            preserve_selection,
            SuccessNotice::Default(detail.into()),
        );
    }

    fn commit_staged_action_with_notice(
        &mut self,
        staged: Result<StagedWorkspaceMutation, String>,
        selection: Option<u32>,
        preserve_selection: bool,
        notice: SuccessNotice,
    ) {
        let staged = match staged {
            Ok(staged) => staged,
            Err(error) => {
                self.set_failure("ACTION NOT SAVED", error);
                return;
            }
        };
        let dataset = staged.dataset().clone();
        let selection = (!preserve_selection)
            .then(|| selection.or_else(|| staged.selection_ids().first().copied()))
            .flatten();
        let persisted = matches!(
            &self.workspace_state,
            WorkspaceState::Loaded(session) if staged.document() != session.document()
        );
        let result = match &mut self.workspace_state {
            WorkspaceState::Loaded(session) => {
                session.commit(staged).map_err(|error| error.to_string())
            }
            WorkspaceState::ReadOnlyFailure(error) => Err(error.clone()),
        };
        match result {
            Ok(()) => {
                self.dataset = dataset;
                self.refresh_canvas_type_id_index();
                if !preserve_selection {
                    self.set_selection(selection);
                }
                self.refresh_display_dataset();
                #[cfg(target_arch = "wasm32")]
                self.persist_current_browser_workspace();
                match notice {
                    SuccessNotice::Default(detail) => {
                        self.show_notice(if persisted { "SAVED" } else { "APPLIED" }, detail, false)
                    }
                    SuccessNotice::Specific { title, detail } => {
                        self.show_notice(title, detail, false);
                    }
                    SuccessNotice::None => {}
                }
            }
            Err(error) => self.set_failure(
                "SAVE FAILED",
                format!("Workspace data was not refreshed. {error}"),
            ),
        }
    }

    fn apply_history_command(&mut self, command: HistoryCommand) {
        let selected_concept_id = self.selected_concept_id();
        let staged = self.stage_action(|session| match command {
            HistoryCommand::Undo => session.stage_undo().map_err(|error| error.to_string()),
            HistoryCommand::Redo => session.stage_redo().map_err(|error| error.to_string()),
        });
        let staged = match staged {
            Ok(staged) => staged,
            Err(error) => {
                self.set_failure("HISTORY NOT APPLIED", error);
                return;
            }
        };
        let dataset = staged.dataset().clone();
        let selection =
            history_restore_selection(staged.selection_ids(), selected_concept_id, &dataset);
        let result = match &mut self.workspace_state {
            WorkspaceState::Loaded(session) => {
                session.commit(staged).map_err(|error| error.to_string())
            }
            WorkspaceState::ReadOnlyFailure(error) => Err(error.clone()),
        };
        match result {
            Ok(()) => {
                self.dataset = dataset;
                self.refresh_canvas_type_id_index();
                self.inspector_draft = None;
                self.set_selection(selection);
                self.refresh_display_dataset();
                #[cfg(target_arch = "wasm32")]
                self.persist_current_browser_workspace();
                self.show_notice(
                    match command {
                        HistoryCommand::Undo => "UNDONE",
                        HistoryCommand::Redo => "REDONE",
                    },
                    "The saved workspace history was restored.",
                    false,
                );
            }
            Err(error) => self.set_failure(
                "HISTORY NOT APPLIED",
                format!("Workspace data was not refreshed. {error}"),
            ),
        }
    }

    fn handle_history_shortcuts(&mut self, ctx: &egui::Context) {
        let Some(command) = consume_history_shortcut(ctx) else {
            return;
        };
        match history_command_route(
            command,
            self.has_unsaved_inspector_draft(),
            self.dialog.is_some(),
        ) {
            HistoryCommandRoute::Apply(command) => self.apply_history_command(command),
            HistoryCommandRoute::Consume => {}
        }
    }

    fn handle_card_clipboard_shortcuts(&mut self, ctx: &egui::Context) {
        let has_selected_cards = self.interaction.selected_card_ids().next().is_some();
        if !card_clipboard_shortcuts_take_priority(
            self.dialog.is_some(),
            ctx.text_edit_focused(),
            has_selected_cards,
        ) {
            return;
        }
        match consume_card_clipboard_command(ctx) {
            Some(CardClipboardCommand::Copy) => self.copy_selected_cards(),
            Some(CardClipboardCommand::Paste) => {
                let position = self.last_canvas_viewport.and_then(|viewport| {
                    ctx.input(|input| input.pointer.hover_pos())
                        .filter(|position| viewport.contains(*position))
                        .map(|position| {
                            self.interaction
                                .camera()
                                .screen_to_world(point_in_viewport(position, viewport))
                        })
                });
                self.paste_cards_at(position.or(self.last_canvas_pointer));
            }
            None => {}
        }
    }

    fn publish_pending_card_clipboard(&mut self, ctx: &egui::Context) {
        if self.publish_card_clipboard {
            // Nonempty clipboard content enables semantic Paste events in platform integrations.
            ctx.copy_text("Reason Room cards".to_string());
            self.publish_card_clipboard = false;
        }
    }

    fn has_unsaved_inspector_draft(&self) -> bool {
        let Some(draft) = self.inspector_draft.as_ref() else {
            return false;
        };
        let Some(concept_id) = self.selected_concept_id() else {
            return false;
        };
        let Some(card) = self.dataset.card_by_id(concept_id) else {
            return false;
        };
        let selected_concept = self.session().and_then(|session| {
            session
                .document()
                .concept(session.active_version_id(), concept_id)
                .ok()
        });
        let type_id = selected_concept
            .map(|concept| concept.type_id.as_str())
            .unwrap_or_else(|| card.kind.type_id());
        let summary = selected_concept
            .map(|concept| concept.summary.as_str())
            .unwrap_or(&card.summary);
        let details = selected_concept
            .map(|concept| concept.details.as_str())
            .unwrap_or_default();
        let assertion_attributes = selected_concept
            .map(|concept| &concept.attributes)
            .cloned()
            .unwrap_or_default();

        inspector_draft_has_unsaved_changes(
            draft,
            concept_id,
            &card.title,
            type_id,
            summary,
            details,
            &assertion_attributes,
        )
    }

    fn external_workspace_refresh_is_safe(&self) -> bool {
        self.dialog.is_none()
            && self.gesture.is_none()
            && !self.has_unsaved_inspector_draft()
            && self.side_pane_mode == SidePaneMode::Inspect
            && self.boundary_inspector_draft.is_none()
            && self.semantic_zoom_layer_draft.is_none()
            && !self.layer_cross_section_creating
    }

    /// Reloads a locally changed workspace after an MCP or another local
    /// author has persisted a verified source revision. Active editor state
    /// defers this work so a background refresh cannot discard a draft.
    #[cfg(not(target_arch = "wasm32"))]
    fn refresh_external_workspace(&mut self, ctx: &egui::Context) {
        ctx.request_repaint_after(EXTERNAL_WORKSPACE_REFRESH_INTERVAL);
        let now_seconds = ctx.input(|input| input.time);
        if now_seconds < self.next_external_workspace_refresh_at {
            return;
        }
        self.next_external_workspace_refresh_at =
            now_seconds + EXTERNAL_WORKSPACE_REFRESH_INTERVAL.as_secs_f64();
        if !self.external_workspace_refresh_is_safe() {
            return;
        }

        let refresh = match &self.workspace_state {
            WorkspaceState::Loaded(session) => session.reload_if_local_source_changed(),
            WorkspaceState::ReadOnlyFailure(_) => return,
        };
        match refresh {
            Ok(Some(session)) => {
                let title = session.document().title.clone();
                self.replace_workspace_session(session);
                self.external_workspace_refresh_error = None;
                self.show_notice(
                    "CANVAS REFRESHED",
                    format!("{title} was updated by MCP or another local author."),
                    false,
                );
            }
            Ok(None) => self.external_workspace_refresh_error = None,
            Err(error) => {
                let detail = error.to_string();
                if self.external_workspace_refresh_error.as_deref() != Some(detail.as_str()) {
                    self.external_workspace_refresh_error = Some(detail.clone());
                    self.set_failure("CANVAS NOT REFRESHED", detail);
                }
            }
        }
    }

    fn set_failure(&mut self, title: impl Into<String>, detail: impl Into<String>) {
        self.show_notice(title, detail, true);
    }

    fn show_notice(&mut self, title: impl Into<String>, detail: impl Into<String>, failed: bool) {
        remove_dismissed_notices(&mut self.notices, self.ui_time_seconds);
        if self.notices.len() == MAX_VISIBLE_ORDINARY_TOASTS {
            self.notices.remove(0);
        }
        self.notices
            .push(Notice::new(title, detail, failed, self.ui_time_seconds));
    }

    fn set_type_change_failure(&mut self, concept_id: u32, type_id: &str, error: String) {
        let diagnostic_path = self
            .session()
            .and_then(|session| session.workspace_path())
            .map(type_change_diagnostic_path)
            .unwrap_or_else(|| PathBuf::from(TYPE_CHANGE_DIAGNOSTIC_FILE_NAME));
        let diagnostic =
            format_type_change_failure_diagnostic(&diagnostic_path, concept_id, type_id, &error);
        let diagnostic_write_error = append_diagnostic(&diagnostic_path, &diagnostic)
            .err()
            .map(|write_error| {
                eprintln!(
                    "Type change failed for concept {concept_id}. The local diagnostic log could not be written at {}: {write_error}. Check that the workspace directory is writable, then try the type change again.",
                    diagnostic_path.display()
                );
                write_error.to_string()
            });
        self.type_change_failure_toast = Some(TypeChangeFailureToast::new(
            error,
            diagnostic_path,
            diagnostic_write_error,
            self.ui_time_seconds,
        ));
    }

    fn commit_inspector_save(
        &mut self,
        staged: Result<StagedWorkspaceMutation, String>,
        selection: Option<u32>,
        detail: Option<String>,
        concept_id: u32,
        type_id: String,
        failure_route: InspectorSaveFailureRoute,
    ) {
        let staged = match staged {
            Ok(staged) => staged,
            Err(error) => {
                self.route_inspector_save_failure(concept_id, &type_id, failure_route, error);
                return;
            }
        };
        let dataset = staged.dataset().clone();
        let selection = selection.or_else(|| staged.selection_ids().first().copied());
        let persisted = matches!(
            &self.workspace_state,
            WorkspaceState::Loaded(session) if staged.document() != session.document()
        );
        let result = match &mut self.workspace_state {
            WorkspaceState::Loaded(session) => {
                session.commit(staged).map_err(|error| error.to_string())
            }
            WorkspaceState::ReadOnlyFailure(error) => Err(error.clone()),
        };
        match result {
            Ok(()) => {
                self.dataset = dataset;
                self.inspector_draft = None;
                self.set_selection(selection);
                self.refresh_display_dataset();
                #[cfg(target_arch = "wasm32")]
                self.persist_current_browser_workspace();
                if let Some(detail) = detail {
                    self.show_notice(if persisted { "SAVED" } else { "APPLIED" }, detail, false);
                }
            }
            Err(error) => self.route_inspector_save_failure(
                concept_id,
                &type_id,
                failure_route,
                format!("Workspace data was not refreshed. {error}"),
            ),
        }
    }

    fn route_inspector_save_failure(
        &mut self,
        concept_id: u32,
        type_id: &str,
        failure_route: InspectorSaveFailureRoute,
        error: String,
    ) {
        match failure_route {
            InspectorSaveFailureRoute::OrdinaryNotice => {
                self.set_failure("CONCEPT NOT SAVED", error);
            }
            InspectorSaveFailureRoute::TypeChangeToast => {
                self.set_type_change_failure(concept_id, type_id, error);
            }
        }
    }

    fn canvas_item_move_ids(
        &self,
        selected_item_ids: impl IntoIterator<Item = CanvasItemId>,
    ) -> Vec<CanvasItemId> {
        let items =
            self.dataset
                .cards()
                .map(|card| CanvasItemBounds {
                    id: CanvasItemId::Card(card.id),
                    bounds: card.bounds,
                })
                .chain(self.active_visual_boundaries().into_iter().map(|boundary| {
                    CanvasItemBounds {
                        id: CanvasItemId::Boundary(boundary.id),
                        bounds: boundary.bounds,
                    }
                }));
        canvas_item_move_ids_with_contained_descendants(items, selected_item_ids)
    }

    fn move_canvas_items(&mut self, item_ids: &[CanvasItemId], world_delta: Point) {
        if item_ids.is_empty() {
            self.set_failure(
                "SELECT AN ITEM",
                "Select one card or boundary before moving it.",
            );
            return;
        }
        let concept_ids = item_ids
            .iter()
            .filter_map(|item_id| match item_id {
                CanvasItemId::Card(concept_id) => Some(*concept_id),
                CanvasItemId::Boundary(_) => None,
            })
            .collect::<Vec<_>>();
        let route = move_commit_route(&concept_ids);
        let preserve_selection = matches!(route, Some(MoveCommitRoute::Group));
        let notice = if concept_ids.len() == 1 {
            match card_containment_notice_after_move(&self.dataset, &concept_ids, world_delta) {
                SuccessNotice::None => visual_boundary_containment_notice_after_move(
                    &self.dataset,
                    &self.active_visual_boundaries(),
                    &concept_ids,
                    world_delta,
                ),
                notice => notice,
            }
        } else {
            SuccessNotice::None
        };
        self.commit_staged_action_with_notice(
            self.stage_action(|session| {
                session
                    .stage_move_active_canvas_items(item_ids, world_delta)
                    .map_err(|error| error.to_string())
            }),
            match route {
                Some(MoveCommitRoute::Single(concept_id)) => Some(concept_id),
                Some(MoveCommitRoute::Group) | None => None,
            },
            preserve_selection,
            notice,
        );
    }

    fn resize_concept(&mut self, concept_id: u32, bounds: CanvasRect) {
        let before = self.card_layer_visibility(concept_id, None);
        let after = self.card_layer_visibility(concept_id, Some(bounds));
        let before_label = self.card_base_layer_label(before.as_ref());
        let after_label = self.card_base_layer_label(after.as_ref());
        let card_title = self
            .dataset
            .card_by_id(concept_id)
            .map(|card| card.title.as_str())
            .unwrap_or("Card");
        let notice = resize_layer_change_notice(card_title, &before_label, &after_label);
        self.commit_staged_action_with_notice(
            self.stage_action(|session| {
                session
                    .stage_resize_active_concept(concept_id, bounds)
                    .map_err(|error| error.to_string())
            }),
            Some(concept_id),
            false,
            notice,
        );
    }

    fn create_concept(&mut self, draft: ConceptEditorDraft) {
        let camera = self.interaction.camera();
        let preferences = self.stored_preferences();
        let active_layer = self.active_layer_for_card_creation();
        let size = new_concept_size_for_active_layer(
            &preferences,
            camera.scale,
            active_layer
                .as_ref()
                .map(|(_, card_scale)| *card_scale)
                .unwrap_or(1.0),
        );
        let origin = self
            .pending_canvas_creation_position
            .take()
            .map(|position| position - size * 0.5)
            .unwrap_or_else(|| camera.screen_to_world(Point::new(360.0, 260.0)));
        let bounds = CanvasRect::new(origin, size);
        let type_id = created_card_type_id(
            draft.type_id.as_deref(),
            preferences.default_card_type_id.as_deref(),
        );
        self.commit_staged_action(
            self.stage_action(|session| {
                session
                    .stage_create_active_concept_with_layer_mode(
                        ConceptDraft {
                            type_id,
                            title: draft.title,
                            summary: draft.summary,
                            bounds,
                        },
                        active_layer.as_ref().map(|(layer_id, _)| {
                            SemanticZoomCardLayerMode::Manual {
                                layer_ids: vec![layer_id.clone()],
                            }
                        }),
                    )
                    .map_err(|error| error.to_string())
            }),
            None,
            false,
            "Created a card in the active Lens.",
        );
    }

    fn create_visual_boundary(&mut self, draft: BoundaryEditorDraft) {
        let camera = self.interaction.camera();
        let boundary_size = new_boundary_size_from_preferences(&self.stored_preferences());
        let origin = self
            .pending_canvas_creation_position
            .take()
            .map(|position| position - boundary_size * 0.5)
            .unwrap_or_else(|| {
                camera.screen_to_world(self.canvas_viewport_size * 0.5 - boundary_size * 0.5)
            });
        let bounds = CanvasRect::new(origin, boundary_size);
        self.commit_staged_action(
            self.stage_action(|session| {
                session
                    .stage_create_visual_boundary(draft.title, draft.summary, bounds)
                    .map_err(|error| error.to_string())
            }),
            None,
            true,
            "Created a visual boundary in the active Lens.",
        );
    }

    fn update_visual_boundary(&mut self, draft: BoundaryInspectorDraft) {
        let boundary_id = draft.boundary_id.clone();
        self.commit_staged_action(
            self.stage_action(|session| {
                session
                    .stage_update_visual_boundary(&boundary_id, draft.title, draft.summary)
                    .map_err(|error| error.to_string())
            }),
            None,
            true,
            "Saved boundary details.",
        );
        self.boundary_inspector_draft = None;
    }

    fn resize_visual_boundary(&mut self, boundary_id: &str, bounds: CanvasRect) {
        self.commit_staged_action(
            self.stage_action(|session| {
                session
                    .stage_resize_visual_boundary(boundary_id, bounds)
                    .map_err(|error| error.to_string())
            }),
            None,
            true,
            "Resized a visual boundary. Cards and relationships did not change.",
        );
    }

    fn remove_visual_boundary(&mut self, boundary_id: &str) {
        self.commit_staged_action(
            self.stage_action(|session| {
                session
                    .stage_remove_visual_boundary(boundary_id)
                    .map_err(|error| error.to_string())
            }),
            None,
            true,
            "Removed the visual boundary. Cards and relationships did not change.",
        );
        self.set_boundary_selection(None);
    }

    fn active_visual_boundaries(&self) -> Vec<VisualBoundary> {
        let visible_layers: BTreeSet<_> = self
            .semantic_zoom_frame()
            .map(|frame| {
                frame
                    .layers
                    .into_iter()
                    .filter(|layer| layer.opacity > 0.0)
                    .map(|layer| layer.id)
                    .collect()
            })
            .unwrap_or_default();
        self.session()
            .and_then(|session| {
                session
                    .document()
                    .version(session.active_version_id())
                    .ok()
                    .and_then(|version| {
                        version
                            .lenses
                            .iter()
                            .find(|lens| lens.id == session.active_lens_id())
                    })
            })
            .map(|lens| {
                lens.visual_boundaries
                    .iter()
                    .filter(|boundary| {
                        !self.semantic_layer_filtering_is_active()
                            || boundary
                                .layer_id
                                .as_ref()
                                .is_none_or(|id| visible_layers.contains(id))
                    })
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// A boundary surrounding the whole viewport is navigation context. It
    /// stays visible in the context trail, but direct manipulation is reserved
    /// for the nested map currently being viewed.
    fn interactive_visual_boundaries(&self, viewport_size: Point) -> Vec<VisualBoundary> {
        let visible_world = self.interaction.camera().visible_world_rect(viewport_size);
        interactive_visual_boundaries(&self.active_visual_boundaries(), visible_world)
            .into_iter()
            .cloned()
            .collect()
    }

    fn cycle_lens(&mut self) {
        let Some(session) = self.session() else {
            self.set_failure("LENS NOT CHANGED", "The workspace is unavailable.");
            return;
        };
        let lenses = match session.available_lenses(session.active_version_id()) {
            Ok(lenses) => lenses,
            Err(error) => {
                self.set_failure("LENS NOT CHANGED", error.to_string());
                return;
            }
        };
        let Some(next_lens_id) = next_choice(
            &lenses
                .iter()
                .map(|lens| lens.id.clone())
                .collect::<Vec<_>>(),
            session.active_lens_id(),
        ) else {
            return;
        };
        self.select_lens(next_lens_id);
    }

    fn select_lens(&mut self, lens_id: String) {
        self.save_active_lens_camera();
        let selected_concept_id = self.selected_concept_id();
        let staged = self.stage_action(|session| {
            session
                .stage_select_lens(&lens_id)
                .map_err(|error| error.to_string())
        });
        let selection = selected_concept_id.filter(|concept_id| {
            staged
                .as_ref()
                .ok()
                .is_some_and(|staged| staged.dataset().card_by_id(*concept_id).is_some())
        });
        self.commit_staged_action(staged, selection, false, "Selected Lens.");
        self.restore_active_lens_camera();
    }

    fn select_version(&mut self, version_id: String) {
        self.save_active_lens_camera();
        let selected_concept_id = self.selected_concept_id();
        let staged = self.stage_action(|session| {
            session
                .stage_select_version(&version_id)
                .map_err(|error| error.to_string())
        });
        let selection = selected_concept_id.filter(|concept_id| {
            staged
                .as_ref()
                .ok()
                .is_some_and(|staged| staged.dataset().card_by_id(*concept_id).is_some())
        });
        self.commit_staged_action(staged, selection, false, "Selected workspace version.");
        self.restore_active_lens_camera();
    }

    fn copy_selected_cards(&mut self) {
        let concept_ids: Vec<_> = self.interaction.selected_card_ids().collect();
        if concept_ids.is_empty() {
            self.set_failure("CARDS NOT COPIED", "Select one or more cards first.");
            return;
        }
        let Some(session) = self.session() else {
            self.set_failure("CARDS NOT COPIED", "The workspace is unavailable.");
            return;
        };
        self.card_clipboard = Some(CardClipboard {
            version_id: session.active_version_id().to_string(),
            lens_id: session.active_lens_id().to_string(),
            concept_ids: concept_ids.clone(),
            source: session.document().clone(),
        });
        self.publish_card_clipboard = true;
        self.show_notice(
            "CARDS COPIED",
            format!(
                "{} card{} copied. Relationships are not included.",
                concept_ids.len(),
                if concept_ids.len() == 1 { "" } else { "s" }
            ),
            false,
        );
    }

    fn can_paste_cards(&self) -> bool {
        self.card_clipboard.is_some() && self.session().is_some()
    }

    fn paste_cards(&mut self) {
        self.paste_cards_at(self.last_canvas_pointer);
    }

    fn paste_cards_at(&mut self, world_position: Option<Point>) {
        let Some(clipboard) = self.card_clipboard.clone() else {
            self.set_failure("CARDS NOT PASTED", "Copy one or more cards first.");
            return;
        };
        let preview = self.stage_action(|session| {
            session
                .stage_paste_concepts(
                    &clipboard.source,
                    &clipboard.version_id,
                    &clipboard.lens_id,
                    &clipboard.concept_ids,
                    Point::default(),
                )
                .map_err(|error| error.to_string())
        });
        let preview = match preview {
            Ok(preview) => preview,
            Err(error) => {
                self.commit_staged_action(Err(error), None, false, "Cards not pasted.");
                return;
            }
        };
        let camera_scale = self.interaction.camera().scale;
        let projected = paste_card_visual_bounds(
            preview.document(),
            preview.active_version_id(),
            preview.active_lens_id(),
            camera_scale,
            self.show_all_cards,
        );
        let source_bounds = preview
            .selection_ids()
            .iter()
            .filter_map(|id| projected.get(id).copied())
            .collect::<Vec<_>>();
        let destination_bounds = self
            .session()
            .map(|session| {
                paste_card_visual_bounds(
                    session.document(),
                    session.active_version_id(),
                    session.active_lens_id(),
                    camera_scale,
                    self.show_all_cards,
                )
                .into_values()
                .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let anchor = world_position.unwrap_or_else(|| {
            self.interaction
                .camera()
                .screen_to_world(self.canvas_viewport_size * 0.5)
        });
        let offset = card_paste_delta(&source_bounds, &destination_bounds, anchor);
        let staged = self.stage_action(|session| {
            session
                .stage_paste_concepts(
                    &clipboard.source,
                    &clipboard.version_id,
                    &clipboard.lens_id,
                    &clipboard.concept_ids,
                    offset,
                )
                .map_err(|error| error.to_string())
        });
        let shared_references = if self.session().is_some_and(|session| {
            session.active_version_id() == clipboard.version_id
                && session.active_lens_id() != clipboard.lens_id
        }) {
            staged
                .as_ref()
                .map(|mutation| {
                    mutation
                        .selection_ids()
                        .iter()
                        .filter(|id| clipboard.concept_ids.contains(id))
                        .count()
                })
                .unwrap_or(0)
        } else {
            0
        };
        let message = if shared_references == clipboard.concept_ids.len() {
            format!(
                "Added {shared_references} shared card {} without relationships.",
                if shared_references == 1 {
                    "reference"
                } else {
                    "references"
                }
            )
        } else if shared_references == 0 {
            format!(
                "Pasted {} independent card {} without relationships.",
                clipboard.concept_ids.len(),
                if clipboard.concept_ids.len() == 1 {
                    "copy"
                } else {
                    "copies"
                }
            )
        } else {
            format!(
                "Pasted {} cards ({shared_references} shared references) without relationships.",
                clipboard.concept_ids.len()
            )
        };
        self.commit_staged_action(staged, None, false, message);
    }

    fn open_create_version_pane(&mut self, source_title: String) {
        self.version_creation_draft = WorkspaceTitleDraft {
            title: format!("{source_title} copy"),
        };
        self.side_pane_mode = SidePaneMode::CreateVersion;
        self.inspector_collapsed = false;
        self.pending_creation_name_focus = Some(CreationNameFocus::Version);
    }

    fn open_rename_version_dialog(&mut self, version_id: String, title: String) {
        self.dialog = Some(EditorDialog::RenameVersion {
            version_id,
            draft: WorkspaceTitleDraft { title },
        });
    }

    fn open_create_lens_pane(&mut self, source_title: String) {
        self.lens_creation_draft = LensCreationDraft {
            title: format!("{source_title} view"),
            clone_current: false,
        };
        self.side_pane_mode = SidePaneMode::CreateLens;
        self.inspector_collapsed = false;
        self.pending_creation_name_focus = Some(CreationNameFocus::View);
    }

    fn open_rename_lens_dialog(&mut self, lens_id: String, title: String) {
        self.dialog = Some(EditorDialog::RenameLens {
            lens_id,
            draft: WorkspaceTitleDraft { title },
        });
    }

    fn create_version_from_active(&mut self, title: String) {
        self.save_active_lens_camera();
        self.commit_staged_action(
            self.stage_action(|session| {
                session
                    .stage_create_version_from_active(title)
                    .map_err(|error| error.to_string())
            }),
            None,
            false,
            "Created a version from the selected version.",
        );
        self.restore_active_lens_camera();
    }

    fn rename_version(&mut self, version_id: String, title: String) {
        self.commit_staged_action(
            self.stage_action(|session| {
                session
                    .stage_rename_version(&version_id, title)
                    .map_err(|error| error.to_string())
            }),
            None,
            true,
            "Saved version name.",
        );
    }

    fn delete_version(&mut self, version_id: String) {
        self.save_active_lens_camera();
        self.commit_staged_action(
            self.stage_action(|session| {
                session
                    .stage_delete_version(&version_id)
                    .map_err(|error| error.to_string())
            }),
            None,
            false,
            "Deleted version.",
        );
        self.restore_active_lens_camera();
    }

    fn create_lens_from_active(&mut self, title: String, clone_current: bool) {
        self.save_active_lens_camera();
        self.commit_staged_action(
            self.stage_action(|session| {
                if clone_current {
                    session
                        .stage_create_lens_from_active(title)
                        .map_err(|error| error.to_string())
                } else {
                    session
                        .stage_create_empty_lens(title)
                        .map_err(|error| error.to_string())
                }
            }),
            None,
            false,
            if clone_current {
                "Created a view from the selected view."
            } else {
                "Created a blank view."
            },
        );
        self.restore_active_lens_camera();
    }

    fn rename_lens(&mut self, lens_id: String, title: String) {
        self.commit_staged_action(
            self.stage_action(|session| {
                session
                    .stage_rename_lens(&lens_id, title)
                    .map_err(|error| error.to_string())
            }),
            None,
            true,
            "Saved view name.",
        );
    }

    fn delete_lens(&mut self, lens_id: String) {
        self.save_active_lens_camera();
        self.commit_staged_action(
            self.stage_action(|session| {
                session
                    .stage_delete_lens(&lens_id)
                    .map_err(|error| error.to_string())
            }),
            None,
            false,
            "Deleted view.",
        );
        self.restore_active_lens_camera();
    }

    fn request_relationship_kind(&mut self, source_id: u32, target_id: u32, position: Pos2) {
        self.dialog = Some(EditorDialog::ChooseRelationshipKind {
            source_id,
            target_id,
            position,
        });
    }

    fn create_relationship(&mut self, source_id: u32, target_id: u32, kind: RelationshipKind) {
        if source_id == target_id {
            self.set_failure(
                "RELATIONSHIP NOT SAVED",
                "Choose a different target concept.",
            );
            return;
        }
        if self.relationship_endpoints_share_containment_chain(source_id, target_id) {
            self.set_failure(
                "RELATIONSHIP NOT SAVED",
                "Contained cards already have spatial context. Move a card outside its parent before you connect it.",
            );
            return;
        }
        self.commit_staged_action(
            self.stage_action(|session| {
                session
                    .stage_create_relationship(source_id, target_id, kind)
                    .map_err(|error| error.to_string())
            }),
            None,
            true,
            format!("Created {} relationship.", kind.label()),
        );
    }

    fn request_relationship_removal(&mut self, relationship: RelationshipView) {
        self.dialog = Some(EditorDialog::ConfirmRelationshipRemoval { relationship });
    }

    fn change_relationship_kind(&mut self, relationship_id: String, kind: RelationshipKind) {
        let keep_relationship_selected =
            self.selected_relationship_id.as_deref() == Some(relationship_id.as_str());
        self.commit_staged_action(
            self.stage_action(|session| {
                session
                    .stage_change_relationship_kind(&relationship_id, kind)
                    .map_err(|error| error.to_string())
            }),
            self.selected_concept_id(),
            false,
            format!("Changed relationship to {}.", kind.label()),
        );
        if keep_relationship_selected
            && self
                .relationship_by_id(&relationship_id)
                .is_some_and(|relationship| relationship.kind == kind)
        {
            self.set_relationship_selection(Some(relationship_id));
        }
    }

    fn remove_relationship(&mut self, relationship_id: String) {
        self.commit_staged_action(
            self.stage_action(|session| {
                session
                    .stage_remove_relationship(&relationship_id)
                    .map_err(|error| error.to_string())
            }),
            self.selected_concept_id(),
            false,
            "Removed relationship.",
        );
    }

    fn delete_canvas_selection(&mut self) {
        match canvas_delete_target(
            self.interaction.selected_card_ids(),
            self.selected_boundary_id.as_deref(),
            self.selected_relationship_id.as_deref(),
        ) {
            Some(CanvasDeleteTarget::Relationship(relationship_id)) => {
                self.remove_relationship(relationship_id);
            }
            Some(CanvasDeleteTarget::Boundary(boundary_id)) => {
                self.remove_visual_boundary(&boundary_id);
            }
            Some(CanvasDeleteTarget::Cards(concept_ids)) => {
                let concept_count = concept_ids.len();
                self.commit_staged_action(
                    self.stage_action(|session| {
                        session
                            .stage_remove_active_concept_references(&concept_ids)
                            .map_err(|error| error.to_string())
                    }),
                    None,
                    false,
                    format!(
                        "Removed {concept_count} selected card{} and its direct relationships.",
                        if concept_count == 1 { "" } else { "s" }
                    ),
                );
            }
            None => {}
        }
    }

    fn request_inspector_save(&mut self, draft: ConceptInspectorEditDraft) {
        self.request_inspector_save_with_notice(draft, true);
    }

    fn request_inspector_autosave(&mut self, draft: ConceptInspectorEditDraft) {
        self.request_inspector_save_with_notice(draft, false);
    }

    fn request_inspector_save_with_notice(
        &mut self,
        draft: ConceptInspectorEditDraft,
        show_notice: bool,
    ) {
        if let Some(error) = inspector_save_validation_error(&draft) {
            self.set_failure("NAME NOT SAVED", error);
            return;
        }

        let concept_id = draft.concept_id;
        let type_id = draft.type_id.clone();
        let failure_route = self
            .session()
            .and_then(|session| {
                session
                    .document()
                    .concept(session.active_version_id(), concept_id)
                    .ok()
                    .map(|concept| inspector_save_failure_route(&concept.type_id, &type_id))
            })
            .or_else(|| {
                self.dataset
                    .card_by_id(concept_id)
                    .map(|card| inspector_save_failure_route(card.kind.type_id(), &type_id))
            })
            .unwrap_or(InspectorSaveFailureRoute::OrdinaryNotice);
        let staged = self.stage_action(|session| {
            session
                .stage_edit_active_concept_inspector(draft.clone())
                .map_err(|error| error.to_string())
        });
        match staged {
            Ok(staged)
                if type_change_requires_lens_confirmation(
                    staged.dataset().card_by_id(concept_id).is_some(),
                ) =>
            {
                let lens_title = self
                    .session()
                    .and_then(|session| {
                        session
                            .document()
                            .lens_title(session.active_version_id(), session.active_lens_id())
                            .ok()
                    })
                    .unwrap_or("the active Lens")
                    .to_string();
                self.dialog = Some(EditorDialog::ConfirmTypeChange {
                    draft,
                    staged: Box::new(staged),
                    lens_title,
                });
            }
            staged => self.commit_inspector_save(
                staged,
                Some(concept_id),
                show_notice.then(|| "Saved concept inspector changes.".to_string()),
                concept_id,
                type_id,
                failure_route,
            ),
        }
    }

    fn confirm_inspector_save(
        &mut self,
        draft: ConceptInspectorEditDraft,
        staged: Box<StagedWorkspaceMutation>,
    ) {
        let concept_id = draft.concept_id;
        let type_id = draft.type_id.clone();
        self.commit_inspector_save(
            Ok(*staged),
            None,
            Some(
                "Saved concept inspector changes. The concept is hidden by the active Lens filter."
                    .to_string(),
            ),
            concept_id,
            type_id,
            InspectorSaveFailureRoute::TypeChangeToast,
        );
    }

    fn toasts(&mut self, ctx: &egui::Context) {
        let now_seconds = ctx.input(|input| input.time);
        remove_dismissed_notices(&mut self.notices, now_seconds);
        let ordinary_notice_opacities = self
            .notices
            .iter()
            .map(|notice| match notice.visual_state(now_seconds) {
                OrdinaryNoticeVisualState::Opaque => 1.0,
                OrdinaryNoticeVisualState::Fading { opacity } => opacity,
                OrdinaryNoticeVisualState::Dismissed => {
                    unreachable!("dismissed notices are removed before rendering")
                }
            })
            .collect::<Vec<_>>();
        let type_change_failure = self.type_change_failure_toast.as_ref().and_then(|toast| {
            let opacity = match toast.visual_state(now_seconds) {
                TypeChangeToastVisualState::Opaque => 1.0,
                TypeChangeToastVisualState::Fading { opacity } => opacity,
                TypeChangeToastVisualState::Dismissed => return None,
            };
            Some((
                toast.error.clone(),
                toast.diagnostic_path.clone(),
                toast.diagnostic_write_error.clone(),
                opacity,
            ))
        });
        if self.type_change_failure_toast.is_some() && type_change_failure.is_none() {
            self.type_change_failure_toast = None;
        }
        let routes = toast_stack_routes(self.notices.len(), type_change_failure.is_some());
        if routes.is_empty() {
            return;
        }

        let mut ordinary_notice_hovered = vec![false; self.notices.len()];
        let mut type_change_failure_hovered = false;
        egui::Area::new(egui::Id::unique("notice-toast-stack"))
            .anchor(Align2::LEFT_BOTTOM, Vec2::new(16.0, -16.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                ui.set_max_width(460.0);
                ui.spacing_mut().item_spacing.y = TOAST_STACK_SPACING;
                ui.vertical(|ui| {
                    for route in routes {
                        match route {
                            ToastStackRoute::Ordinary(index) => {
                                ordinary_notice_hovered[index] = ordinary_notice_toast(
                                    ui,
                                    &self.notices[index],
                                    ordinary_notice_opacities[index],
                                )
                                .hovered();
                            }
                            ToastStackRoute::TypeChangeFailure => {
                                let (error, diagnostic_path, diagnostic_write_error, opacity) =
                                    type_change_failure
                                        .as_ref()
                                        .expect("type-change toast route requires toast data");
                                type_change_failure_hovered = type_change_failure_toast(
                                    ui,
                                    error,
                                    diagnostic_path,
                                    diagnostic_write_error.as_deref(),
                                    *opacity,
                                )
                                .hovered();
                            }
                        }
                    }
                });
            });
        for (notice, hovered) in self.notices.iter_mut().zip(ordinary_notice_hovered) {
            notice.set_hovered(hovered, now_seconds);
        }
        if let Some(toast) = &mut self.type_change_failure_toast {
            toast.set_hovered(type_change_failure_hovered, now_seconds);
        }
        if !self.notices.is_empty()
            || (type_change_failure.is_some() && !type_change_failure_hovered)
        {
            ctx.request_repaint_after(Duration::from_millis(16));
        }
    }

    fn session(&self) -> Option<&WorkspaceSession> {
        match &self.workspace_state {
            WorkspaceState::Loaded(session) => Some(session),
            WorkspaceState::ReadOnlyFailure(_) => None,
        }
    }

    fn open_settings_window(&self) {
        let mut preferences = self
            .preferences
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        preferences.open = true;
        preferences.focus_requested = true;
    }

    fn open_new_canvas_dialog(&mut self) {
        if !self.can_switch_canvas("create a new canvas") {
            return;
        }
        self.dialog = Some(EditorDialog::NewCanvas {
            draft: NewCanvasDraft::suggested(),
        });
    }

    fn can_switch_canvas(&mut self, action: &str) -> bool {
        if self.dialog.is_some() {
            self.set_failure(
                "FILE ACTION NOT APPLIED",
                "Finish or cancel the open dialog before you change the active canvas.",
            );
            return false;
        }
        if self.has_unsaved_inspector_draft() {
            self.set_failure(
                "SAVE OR DISCARD CARD EDIT",
                format!("Save or discard the current card edit before you {action}."),
            );
            return false;
        }
        true
    }

    #[cfg(target_arch = "wasm32")]
    fn request_open_local_browser_workspace(&mut self) {
        if !self.can_switch_canvas("open a local canvas") {
            return;
        }
        open_browser_workspace_picker();
    }

    #[cfg(target_arch = "wasm32")]
    fn request_open_watched_browser_workspace(&mut self) {
        if !self.can_switch_canvas("open and refresh a local canvas") {
            return;
        }
        open_watched_browser_workspace_picker();
    }

    #[cfg(target_arch = "wasm32")]
    fn open_save_as_browser_canvas_dialog(&mut self) {
        if !self.can_switch_canvas("save a local copy of this canvas") {
            return;
        }
        let Some(session) = self.session() else {
            self.set_failure("CANVAS NOT SAVED", "No workspace is available to save.");
            return;
        };
        self.dialog = Some(EditorDialog::SaveAsBrowserCanvas {
            draft: NewCanvasDraft::browser_save_as(session.document().title.clone()),
        });
    }

    fn process_native_menu_request(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let requested = self
                .example_requested
                .lock()
                .expect("example menu lock")
                .take();
            if let Some(example) = requested {
                self.load_example(example);
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            if self
                .settings_window_requested
                .swap(false, Ordering::Acquire)
            {
                self.open_settings_window();
            }
            if self.new_canvas_requested.swap(false, Ordering::Acquire) {
                self.open_new_canvas_dialog();
            }
            if self.open_canvas_requested.swap(false, Ordering::Acquire) {
                self.request_open_canvas();
            }
            if self.save_canvas_requested.swap(false, Ordering::Acquire) {
                self.save_active_canvas();
            }
            if self.save_as_canvas_requested.swap(false, Ordering::Acquire) {
                self.request_save_as_canvas();
            }
        }
    }

    fn load_example(&mut self, example: examples::Example) {
        if !self.can_switch_canvas("load an example") {
            return;
        }
        let boundary_changed = self.boundary_inspector_draft.as_ref().is_some_and(|draft| {
            self.active_visual_boundaries()
                .iter()
                .find(|boundary| boundary.id == draft.boundary_id)
                .is_some_and(|boundary| boundary_inspector_draft(boundary) != *draft)
        });
        let layer_changed = self
            .semantic_zoom_layer_draft
            .as_ref()
            .is_some_and(|draft| {
                self.session()
                    .and_then(|session| {
                        session
                            .document()
                            .version(session.active_version_id())
                            .ok()
                            .and_then(|version| {
                                version
                                    .lenses
                                    .iter()
                                    .find(|lens| lens.id == session.active_lens_id())
                            })
                    })
                    .into_iter()
                    .flat_map(|lens| lens.semantic_zoom_layers.iter())
                    .find(|layer| layer.id == draft.layer_id)
                    .is_some_and(|layer| semantic_zoom_layer_editor_draft(layer) != *draft)
            });
        if boundary_changed || layer_changed {
            self.set_failure("SAVE OR DISCARD EDIT", "Save or discard the current boundary or zoom layer edit before loading an example.");
            return;
        }
        let result = example.document().and_then(|document| {
            #[cfg(not(target_arch = "wasm32"))]
            {
                WorkspaceSession::create_new(suggested_new_canvas_path(example.title), document)
                    .map_err(|error| error.to_string())
            }
            #[cfg(target_arch = "wasm32")]
            {
                WorkspaceSession::create_new_browser(document).map_err(|error| error.to_string())
            }
        });
        match result {
            Ok(session) => {
                #[cfg(target_arch = "wasm32")]
                stop_watching_browser_workspace();
                self.replace_workspace_session(session);
                self.fit_example_on_next_frame = true;
                self.inspector_collapsed = true;
                self.show_notice("EXAMPLE LOADED", format!("{}. Zoom into a large card to reveal process and working detail. Changes are saved to your own copy.", example.title), false);
            }
            Err(error) => self.set_failure("EXAMPLE NOT LOADED", error),
        }
    }

    fn create_new_canvas(&mut self, draft: &NewCanvasDraft) -> Result<(), String> {
        let title = draft.title.trim();
        if title.is_empty() {
            return Err("Enter a canvas name.".to_string());
        }
        #[cfg(not(target_arch = "wasm32"))]
        let path = {
            let save_path = draft.save_path.trim();
            if save_path.is_empty() {
                return Err("Enter a save location.".to_string());
            }
            PathBuf::from(save_path)
        };
        let document =
            WorkspaceDocument::blank(workspace_id_from_canvas_title(title), title.to_string())
                .map_err(|error| error.to_string())?;
        #[cfg(not(target_arch = "wasm32"))]
        let session = WorkspaceSession::create_new(path.clone(), document)
            .map_err(|error| error.to_string())?;
        #[cfg(target_arch = "wasm32")]
        let session =
            WorkspaceSession::create_new_browser(document).map_err(|error| error.to_string())?;
        let saved_title = session.document().title.clone();
        #[cfg(not(target_arch = "wasm32"))]
        let save_detail = format!("{saved_title} is saved at {}.", path.display());
        #[cfg(target_arch = "wasm32")]
        let save_detail = format!("{saved_title} is open in the current browser session.");
        self.replace_workspace_session(session);
        self.show_notice("CANVAS CREATED", save_detail, false);
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn request_open_canvas(&mut self) {
        if !self.can_switch_canvas("open another canvas") {
            return;
        }
        let starting_directory = self.canvas_file_dialog_directory();
        let dialog = rfd::FileDialog::new()
            .set_title("Open canvas")
            .add_filter("Reason Room", &["json"])
            .set_directory(starting_directory);
        let Some(path) = dialog.pick_file() else {
            return;
        };
        match WorkspaceSession::load_default(path.clone()) {
            Ok(session) => self.activate_canvas_session(
                session,
                "CANVAS OPENED",
                format!("Opened {}.", path.display()),
            ),
            Err(error) => self.set_failure("CANVAS NOT OPENED", error.to_string()),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn save_active_canvas(&mut self) {
        let result = match &mut self.workspace_state {
            WorkspaceState::Loaded(session) => (|| {
                let title = session.document().title.clone();
                let path = session
                    .workspace_path()
                    .map(Path::to_path_buf)
                    .ok_or_else(|| "The active workspace has no local file path.".to_string())?;
                session
                    .save()
                    .map(|()| (title, path))
                    .map_err(|error| error.to_string())
            })(),
            WorkspaceState::ReadOnlyFailure(error) => Err(error.clone()),
        };
        match result {
            Ok((title, path)) => self.show_notice(
                "SAVED",
                format!("{title} is saved at {}.", path.display()),
                false,
            ),
            Err(error) => self.set_failure("CANVAS NOT SAVED", error),
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn save_browser_canvas(&mut self) {
        let export = self.session().map_or_else(
            || Err("No workspace is available to save.".to_string()),
            |session| {
                session
                    .document()
                    .to_pretty_json()
                    .map(|source| {
                        (
                            source,
                            format!(
                                "{}{}",
                                session.document().workspace_id,
                                NEW_CANVAS_FILE_SUFFIX
                            ),
                        )
                    })
                    .map_err(|error| error.to_string())
            },
        );
        let result = export.map(|(source, file_name)| {
            download_browser_workspace(&source, &file_name);
        });
        match result {
            Ok(()) => self.show_notice(
                "CANVAS SAVED",
                "The canvas was downloaded as local JSON.",
                false,
            ),
            Err(error) => self.set_failure("CANVAS NOT SAVED", error),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn request_save_as_canvas(&mut self) {
        if !self.can_switch_canvas("save this canvas as a new document") {
            return;
        }
        let Some((document, directory, file_name)) = self.session().and_then(|session| {
            let workspace_path = session.workspace_path()?;
            let directory = workspace_path
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(default_new_canvas_directory);
            let file_name = workspace_path
                .file_name()
                .and_then(|name| name.to_str())
                .map(str::to_owned)
                .unwrap_or_else(|| {
                    format!(
                        "{}{}",
                        workspace_id_from_canvas_title(&session.document().title),
                        NEW_CANVAS_FILE_SUFFIX
                    )
                });
            Some((session.document().clone(), directory, file_name))
        }) else {
            self.set_failure("CANVAS NOT SAVED", "The active workspace is unavailable.");
            return;
        };
        let dialog = rfd::FileDialog::new()
            .set_title("Save canvas as")
            .add_filter("Reason Room", &["json"])
            .set_directory(directory)
            .set_file_name(file_name);
        let Some(path) = dialog.save_file() else {
            return;
        };
        match WorkspaceSession::create_new(path.clone(), document) {
            Ok(session) => self.activate_canvas_session(
                session,
                "CANVAS SAVED AS",
                format!("The new copy is saved at {}.", path.display()),
            ),
            Err(error) => self.set_failure("CANVAS NOT SAVED", error.to_string()),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn canvas_file_dialog_directory(&self) -> PathBuf {
        self.session()
            .and_then(|session| {
                session
                    .workspace_path()
                    .and_then(|path| path.parent().map(Path::to_path_buf))
            })
            .unwrap_or_else(default_new_canvas_directory)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn activate_canvas_session(
        &mut self,
        session: WorkspaceSession,
        notice_title: &str,
        notice_detail: String,
    ) {
        self.replace_workspace_session(session);
        self.show_notice(notice_title, notice_detail, false);
    }

    #[cfg(target_arch = "wasm32")]
    fn save_as_browser_canvas(&mut self, draft: &NewCanvasDraft) -> Result<(), String> {
        let title = draft.title.trim();
        if title.is_empty() {
            return Err("Enter a canvas name.".to_string());
        }
        let Some(session) = self.session() else {
            return Err("No workspace is available to save.".to_string());
        };
        let source = session
            .document()
            .to_pretty_json()
            .map_err(|error| error.to_string())?;
        let file_name = format!(
            "{}{}",
            workspace_id_from_canvas_title(title),
            NEW_CANVAS_FILE_SUFFIX
        );
        download_browser_workspace(&source, &file_name);
        self.show_notice(
            "CANVAS SAVED AS",
            format!("The canvas was downloaded as {file_name}."),
            false,
        );
        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    fn import_local_browser_workspace(
        &mut self,
        source: &str,
        file_name: &str,
    ) -> Result<(), String> {
        let document = WorkspaceDocument::parse_json(source)
            .map_err(|error| format!("could not read {file_name} as a canvas: {error}"))?;
        let session =
            WorkspaceSession::create_new_browser(document).map_err(|error| error.to_string())?;
        let title = session.document().title.clone();
        self.replace_workspace_session(session);
        self.show_notice(
            "CANVAS IMPORTED",
            format!("{title} was opened from {file_name}."),
            false,
        );
        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    fn restore_browser_workspace(&mut self, source: &str) -> Result<(), String> {
        let document = WorkspaceDocument::parse_json(source)
            .map_err(|error| format!("could not restore the last browser canvas: {error}"))?;
        let session =
            WorkspaceSession::open_browser(document).map_err(|error| error.to_string())?;
        self.replace_workspace_session(session);
        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    fn refresh_local_browser_workspace(
        &mut self,
        source: &str,
        file_name: &str,
    ) -> Result<bool, String> {
        if !self.external_workspace_refresh_is_safe() {
            return Ok(false);
        }
        let document = WorkspaceDocument::parse_json(source)
            .map_err(|error| format!("could not read {file_name} as a canvas: {error}"))?;
        let Some(current_document) = self.session().map(WorkspaceSession::document) else {
            return Err("No workspace is available to refresh.".to_string());
        };
        if current_document == &document {
            return Ok(true);
        }
        let session =
            WorkspaceSession::open_browser(document).map_err(|error| error.to_string())?;
        let title = session.document().title.clone();
        self.replace_workspace_session(session);
        self.show_notice(
            "CANVAS REFRESHED",
            format!("{title} was updated by MCP or another local author."),
            false,
        );
        Ok(true)
    }

    #[cfg(target_arch = "wasm32")]
    fn persist_current_browser_workspace(&self) {
        let Some(session) = self.session() else {
            return;
        };
        match session.document().to_pretty_json() {
            Ok(source) => persist_browser_workspace(&source),
            Err(error) => eprintln!("could not persist the browser canvas: {error}"),
        }
    }

    fn replace_workspace_session(&mut self, session: WorkspaceSession) {
        let is_example = examples::is_example(session.document());
        self.dataset = session.dataset().clone();
        self.workspace_state = WorkspaceState::Loaded(Box::new(session));
        #[cfg(target_arch = "wasm32")]
        self.persist_current_browser_workspace();
        #[cfg(not(target_arch = "wasm32"))]
        self.remember_open_canvas_path();
        self.interaction = CanvasInteraction::new(Camera::default());
        self.fit_example_on_next_frame = is_example;
        if is_example {
            self.inspector_collapsed = true;
        }
        self.gesture = None;
        self.previous_shift_hover = None;
        self.inspector_draft = None;
        self.side_pane_mode = SidePaneMode::Inspect;
        self.semantic_zoom_layer_id = None;
        self.semantic_zoom_layer_draft = None;
        self.show_all_cards = false;
        self.card_creation_draft = ConceptEditorDraft {
            title: String::new(),
            type_id: None,
            summary: String::new(),
        };
        self.boundary_creation_draft = BoundaryEditorDraft {
            title: String::new(),
            summary: String::new(),
        };
        self.layer_creation_title.clear();
        self.layer_cross_section_drafts.clear();
        self.layer_cross_section_creating = false;
        self.canvas_search_query.clear();
        self.canvas_search_match_index = 0;
        self.canvas_type_filter_ids.clear();
        self.refresh_canvas_type_id_index();
        self.version_creation_draft.title.clear();
        self.lens_creation_draft = LensCreationDraft {
            title: String::new(),
            clone_current: false,
        };
        self.selected_boundary_id = None;
        self.selected_relationship_id = None;
        self.primary_relationship_concept_id = None;
        self.boundary_inspector_draft = None;
        self.hover_target = None;
        self.lens_cameras.clear();
        self.lens_canvas_controls.clear();
        self.notices.clear();
        self.type_change_failure_toast = None;
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn remember_open_canvas_path(&mut self) {
        let Some(path) = self
            .session()
            .and_then(|session| session.workspace_path().map(Path::to_path_buf))
        else {
            return;
        };
        self.preferences
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .stored
            .last_opened_canvas_path = Some(path);
    }

    fn settings_window(&self, ctx: &egui::Context) {
        let open = self
            .preferences
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .open;
        if !open {
            return;
        }

        let preferences = Arc::clone(&self.preferences);
        let card_types = self.session().map(supported_type_ids).unwrap_or_else(|| {
            ConceptKind::ALL
                .into_iter()
                .map(|kind| kind.type_id().to_string())
                .collect()
        });
        ctx.show_viewport_deferred(
            egui::ViewportId::from_hash_of(SETTINGS_VIEWPORT_ID),
            egui::ViewportBuilder::default()
                .with_title(settings_window_title())
                .with_inner_size(SETTINGS_WINDOW_INNER_SIZE)
                .with_min_inner_size(SETTINGS_WINDOW_MINIMUM_SIZE)
                .with_resizable(true),
            move |ui, viewport_class| {
                ui.visuals_mut().clone_from(&atlas_visuals());
                let mut preferences = preferences
                    .lock()
                    .unwrap_or_else(|error| error.into_inner());
                if !preferences.open {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    return;
                }
                if preferences.focus_requested {
                    if viewport_class != egui::ViewportClass::EmbeddedWindow {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Focus);
                    }
                    preferences.focus_requested = false;
                }
                let mut close_window = ui.input(|input| input.viewport().close_requested());

                ui.painter().rect_filled(ui.max_rect(), 0.0, ATLAS_GROUND);
                ui.add_space(18.0);
                ui.horizontal(|ui| {
                    ui.add_space(18.0);
                    ui.vertical(|ui| {
                        ui.heading(
                            egui::RichText::new(settings_window_title())
                                .size(ATLAS_INSPECTOR_TITLE_SIZE)
                                .strong()
                                .color(ATLAS_PAPER_SIGNAL),
                        );
                        ui.horizontal_wrapped(|ui| {
                            for tab in PreferencesTab::ALL {
                                let selected = preferences.selected_tab == tab;
                                if ui.selectable_label(selected, tab.label()).clicked() {
                                    preferences.selected_tab = tab;
                                }
                            }
                        });
                        ui.separator();
                        ui.add_space(8.0);
                        egui::Frame::new()
                            .fill(ATLAS_SURVEY_SLATE)
                            .stroke(Stroke::new(
                                1.0,
                                color_with_opacity(ATLAS_QUIET_GRAPHITE, 0.6),
                            ))
                            .inner_margin(egui::Margin::same(12))
                            .show(ui, |ui| match preferences.selected_tab {
                                PreferencesTab::Canvas => {
                                    ui.label(
                                        egui::RichText::new("Canvas display")
                                            .strong()
                                            .color(ATLAS_PAPER_SIGNAL),
                                    );
                                    ui.small("These choices affect this application window only.");
                                    ui.add_space(8.0);
                                    ui.checkbox(
                                        &mut preferences.stored.show_canvas_grid,
                                        "Show canvas grid",
                                    );
                                    ui.add_space(6.0);
                                    ui.checkbox(
                                        &mut preferences.stored.show_relationship_labels,
                                        "Show relationship labels",
                                    );
                                }
                                PreferencesTab::NewCards => {
                                    ui.label(
                                        egui::RichText::new("Defaults for new cards")
                                            .strong()
                                            .color(ATLAS_PAPER_SIGNAL),
                                    );
                                    ui.small(
                                        "They apply when you create a card. Existing cards do not change.",
                                    );
                                    ui.add_space(8.0);
                                    ui.horizontal(|ui| {
                                        ui.label("Width");
                                        ui.add(
                                            egui::DragValue::new(
                                                &mut preferences.stored.default_card_width,
                                            )
                                            .range(MINIMUM_CARD_SIZE.x..=100_000.0)
                                            .speed(1.0),
                                        );
                                        ui.label("Height");
                                        ui.add(
                                            egui::DragValue::new(
                                                &mut preferences.stored.default_card_height,
                                            )
                                            .range(MINIMUM_CARD_SIZE.y..=100_000.0)
                                            .speed(1.0),
                                        );
                                    });
                                    ui.add_space(8.0);
                                    ui.label("Preferred type");
                                    let selected_type = preferences
                                        .stored
                                        .default_card_type_id
                                        .as_deref()
                                        .unwrap_or("No preferred type");
                                    egui::ComboBox::from_id_salt("preference-default-card-type")
                                        .selected_text(selected_type)
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(
                                                &mut preferences.stored.default_card_type_id,
                                                None,
                                                "No preferred type",
                                            );
                                            for type_id in &card_types {
                                                ui.selectable_value(
                                                    &mut preferences.stored.default_card_type_id,
                                                    Some(type_id.clone()),
                                                    type_id,
                                                );
                                            }
                                        });
                                }
                                PreferencesTab::NewBoundaries => {
                                    ui.label(
                                        egui::RichText::new("Defaults for new boundaries")
                                            .strong()
                                            .color(ATLAS_PAPER_SIGNAL),
                                    );
                                    ui.small(
                                        "They apply when you create a boundary. Existing boundaries do not change.",
                                    );
                                    ui.add_space(8.0);
                                    ui.horizontal(|ui| {
                                        ui.label("Width");
                                        ui.add(
                                            egui::DragValue::new(
                                                &mut preferences.stored.default_boundary_width,
                                            )
                                            .range(MINIMUM_BOUNDARY_SIZE.x..=100_000.0)
                                            .speed(1.0),
                                        );
                                        ui.label("Height");
                                        ui.add(
                                            egui::DragValue::new(
                                                &mut preferences.stored.default_boundary_height,
                                            )
                                            .range(MINIMUM_BOUNDARY_SIZE.y..=100_000.0)
                                            .speed(1.0),
                                        );
                                    });
                                }
                                PreferencesTab::ZoomLayers => {
                                    ui.label(
                                        egui::RichText::new("Defaults for new zoom layers")
                                            .strong()
                                            .color(ATLAS_PAPER_SIGNAL),
                                    );
                                    ui.small(
                                        "They apply when you create a layer. Existing layers do not change.",
                                    );
                                    ui.add_space(8.0);
                                    ui.horizontal(|ui| {
                                        ui.label("Start");
                                        ui.add(
                                            egui::DragValue::new(
                                                &mut preferences
                                                    .stored
                                                    .default_layer_start_percent,
                                            )
                                            .range(1.0..=409_600.0)
                                            .speed(1.0)
                                            .suffix("%"),
                                        );
                                        ui.label("End");
                                        ui.add(
                                            egui::DragValue::new(
                                                &mut preferences
                                                    .stored
                                                    .default_layer_end_percent,
                                            )
                                            .range(1.0..=409_600.0)
                                            .speed(1.0)
                                            .suffix("%"),
                                        );
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Fade");
                                        ui.add(
                                            egui::DragValue::new(
                                                &mut preferences
                                                    .stored
                                                    .default_layer_fade_percent,
                                            )
                                            .range(0.0..=409_599.0)
                                            .speed(1.0)
                                            .suffix(" percentage points"),
                                        );
                                    });
                                    ui.small("Fade extends before and after the active range.");
                                }
                            });
                        ui.add_space(12.0);
                        ui.label(
                            egui::RichText::new(
                                "Preferences are saved locally. Workspace facts are not changed here.",
                            )
                            .small()
                            .color(ATLAS_QUIET_GRAPHITE),
                        );
                        ui.add_space(16.0);
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui.button("Done").clicked() {
                                close_window = true;
                            }
                        });
                    });
                    ui.add_space(18.0);
                });

                if close_window {
                    preferences.open = false;
                    preferences.focus_requested = false;
                    if viewport_class != egui::ViewportClass::EmbeddedWindow {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                }
            },
        );
    }

    fn display_preferences(&self) -> (bool, bool) {
        let preferences = self
            .preferences
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        (
            preferences.stored.show_canvas_grid,
            preferences.stored.show_relationship_labels,
        )
    }

    fn semantic_zoom_frame(&self) -> Option<SemanticZoomFrame> {
        let session = self.session()?;
        session
            .document()
            .semantic_zoom_frame_with_relationship_rollup(
                session.active_version_id(),
                session.active_lens_id(),
                self.interaction.camera().scale,
                self.roll_up_relationships,
            )
            .ok()
    }

    /// Returns the strongest active authored layer and its display scale. A
    /// card created while a layer is active belongs to that layer explicitly,
    /// so its initial size and visibility match the user's current view.
    fn active_layer_for_card_creation(&self) -> Option<(String, f32)> {
        let frame = self.semantic_zoom_frame()?;
        let layer_index = strongest_visible_layer_indices(&frame.layers)
            .into_iter()
            .last()?;
        let layer_id = frame.layers.get(layer_index)?.id.clone();
        let session = self.session()?;
        let card_scale = session
            .document()
            .version(session.active_version_id())
            .ok()?
            .lenses
            .iter()
            .find(|lens| lens.id == session.active_lens_id())?
            .semantic_zoom_layers
            .iter()
            .find(|layer| layer.id == layer_id)
            .map(|layer| layer.card_scale)?;
        Some((layer_id, card_scale))
    }

    fn semantic_zoom_is_configured(&self) -> bool {
        self.session().is_some_and(|session| {
            session
                .document()
                .version(session.active_version_id())
                .ok()
                .and_then(|version| {
                    version
                        .lenses
                        .iter()
                        .find(|lens| lens.id == session.active_lens_id())
                })
                .is_some_and(|lens| !lens.semantic_zoom_layers.is_empty())
        })
    }

    /// View-all is a temporary canvas mode. It never changes a saved layer,
    /// card policy, or the active Lens.
    fn semantic_layer_filtering_is_active(&self) -> bool {
        semantic_layer_filtering_is_active(self.semantic_zoom_is_configured(), self.show_all_cards)
    }

    fn toggle_show_all_cards(&mut self) {
        if !self.semantic_zoom_is_configured() {
            return;
        }
        self.show_all_cards = !self.show_all_cards;
        self.refresh_display_dataset();
        if self.show_all_cards {
            self.show_notice(
                "VIEWING ALL CARDS",
                "Layer filtering is off. Saved layers are unchanged.",
                false,
            );
        } else {
            self.show_notice(
                "LAYER VIEW RESTORED",
                "Cards now follow the current zoom layer.",
                false,
            );
        }
    }

    fn refresh_display_dataset(&mut self) {
        if !self.semantic_layer_filtering_is_active() {
            if let Some(session) = self.session() {
                self.dataset = session.dataset().clone();
            }
            return;
        }
        if let Some(frame) = self.semantic_zoom_frame() {
            self.dataset = semantic_zoom_dataset(&frame);
        }
    }

    fn select_semantic_zoom_layer(&mut self, layer_id: String) {
        let draft = self.session().and_then(|session| {
            session
                .document()
                .version(session.active_version_id())
                .ok()
                .and_then(|version| {
                    version
                        .lenses
                        .iter()
                        .find(|lens| lens.id == session.active_lens_id())
                })
                .and_then(|lens| {
                    lens.semantic_zoom_layers
                        .iter()
                        .find(|layer| layer.id == layer_id)
                })
                .map(semantic_zoom_layer_editor_draft)
        });
        self.semantic_zoom_layer_id = draft.as_ref().map(|draft| draft.layer_id.clone());
        self.semantic_zoom_layer_draft = draft;
    }

    fn open_layer_cross_section(&mut self) {
        self.layer_cross_section_drafts = self
            .session()
            .and_then(|session| {
                session
                    .document()
                    .version(session.active_version_id())
                    .ok()
                    .and_then(|version| {
                        version
                            .lenses
                            .iter()
                            .find(|lens| lens.id == session.active_lens_id())
                    })
                    .map(|lens| {
                        lens.semantic_zoom_layers
                            .iter()
                            .map(semantic_zoom_layer_range_draft)
                            .collect()
                    })
            })
            .unwrap_or_default();
        self.side_pane_mode = SidePaneMode::LayerCrossSection;
    }

    /// Opens the cross-section with a fresh, inline layer draft. Layer
    /// creation belongs next to the ranges it will join, rather than in a
    /// disconnected inspector pane.
    fn begin_inline_layer_creation(&mut self) {
        self.open_layer_cross_section();
        let layer_count = self.layer_cross_section_drafts.len();
        self.layer_creation_index = layer_count;
        let defaults = self.stored_preferences();
        self.layer_creation_title = semantic_zoom_layer_creation_title(layer_count);
        self.layer_creation_active_zoom_start_percent = defaults.default_layer_start_percent;
        self.layer_creation_active_zoom_end_percent = defaults.default_layer_end_percent;
        self.layer_creation_fade_percent = defaults.default_layer_fade_percent;
        self.layer_cross_section_creating = true;
        self.pending_creation_name_focus = Some(CreationNameFocus::Layer);
    }

    /// Keeps custom type filtering a direct lookup while the canvas paints or
    /// searches. The document is the source of truth; the screen card only
    /// retains a legacy core kind.
    fn refresh_canvas_type_id_index(&mut self) {
        self.canvas_type_id_by_concept_id = self
            .session()
            .and_then(|session| {
                session
                    .document()
                    .version(session.active_version_id())
                    .ok()
                    .map(|version| {
                        version
                            .concepts
                            .iter()
                            .map(|concept| (concept.id, concept.type_id.clone()))
                            .collect()
                    })
            })
            .unwrap_or_default();
    }

    fn canvas_card_matches_type_filter(&self, card: &ConceptCard) -> bool {
        self.canvas_type_filter_ids.is_empty()
            || self.canvas_type_filter_ids.contains(
                self.canvas_type_id_by_concept_id
                    .get(&card.id)
                    .map(String::as_str)
                    .unwrap_or(card.kind.type_id()),
            )
    }

    fn canvas_concept_id_matches_type_filter(&self, concept_id: u32) -> bool {
        self.dataset
            .card_by_id(concept_id)
            .is_some_and(|card| self.canvas_card_matches_type_filter(card))
    }

    fn canvas_filter_type_ids(&self) -> Vec<String> {
        self.session().map(supported_type_ids).unwrap_or_else(|| {
            let mut type_ids: Vec<_> = self
                .dataset
                .cards()
                .map(|card| card.kind.type_id().to_string())
                .collect();
            type_ids.sort();
            type_ids.dedup();
            type_ids
        })
    }

    fn set_canvas_type_filter_enabled(
        &mut self,
        type_id: &str,
        enabled: bool,
        all_type_ids: &[String],
    ) {
        if enabled {
            if !self.canvas_type_filter_ids.is_empty() {
                self.canvas_type_filter_ids.insert(type_id.to_string());
                if all_type_ids
                    .iter()
                    .all(|candidate| self.canvas_type_filter_ids.contains(candidate))
                {
                    self.canvas_type_filter_ids.clear();
                }
            }
        } else {
            if self.canvas_type_filter_ids.is_empty() {
                self.canvas_type_filter_ids = all_type_ids.iter().cloned().collect();
            }
            self.canvas_type_filter_ids.remove(type_id);
        }

        self.canvas_search_match_index = 0;
        self.clear_noninteractive_canvas_selection();
    }

    fn clear_noninteractive_canvas_selection(&mut self) {
        let context_card_ids = self.context_trail_card_ids();
        let hidden_selected_ids: Vec<_> = self
            .interaction
            .selected_card_ids()
            .filter(|concept_id| {
                self.dataset.card_by_id(*concept_id).is_none_or(|card| {
                    !self.canvas_card_matches_type_filter(card)
                        || context_card_ids.contains(concept_id)
                })
            })
            .collect();
        for concept_id in hidden_selected_ids {
            self.interaction.remove_selected_card_id(concept_id);
        }
        if self
            .primary_relationship_concept_id
            .is_some_and(|concept_id| !self.interaction.is_selected(concept_id))
        {
            self.primary_relationship_concept_id = self.interaction.selected_card_ids().next();
        }
        if self.inspector_has_no_selection() {
            self.inspector_collapsed = true;
        }
    }

    fn canvas_search_match_ids(&self) -> Vec<u32> {
        let query = self.canvas_search_query.trim();
        if query.is_empty() {
            return Vec::new();
        }
        let query = query.to_lowercase();
        let mut matches: Vec<_> = self
            .dataset
            .cards()
            .filter(|card| {
                self.canvas_card_matches_type_filter(card)
                    && canvas_card_matches_search(card, &query)
            })
            .map(|card| (card.id, card.title.to_lowercase()))
            .collect();
        matches.sort_by(|(left_id, left_title), (right_id, right_title)| {
            left_title.cmp(right_title).then(left_id.cmp(right_id))
        });
        matches.into_iter().map(|(id, _)| id).collect()
    }

    fn focus_canvas_search_match(&mut self, concept_id: u32) {
        let Some(card) = self.dataset.card_by_id(concept_id) else {
            return;
        };
        let camera = camera_focused_on_card(
            self.interaction.camera(),
            card.bounds,
            self.canvas_viewport_size,
        );
        self.interaction.set_camera(camera);
        self.set_selection(Some(concept_id));
        self.refresh_display_dataset();
    }

    fn filtered_canvas_hit_test(&self, world_position: Point) -> Option<u32> {
        let context_card_ids = self.context_trail_card_ids();
        filtered_card_hit_test(&self.dataset, world_position, |card| {
            self.canvas_card_matches_type_filter(card) && !context_card_ids.contains(&card.id)
        })
        .map(|card| card.id)
    }

    fn context_trail_card_ids(&self) -> BTreeSet<u32> {
        let visible_world = self
            .interaction
            .camera()
            .visible_world_rect(self.canvas_viewport_size);
        canvas_context_entries(
            &[],
            self.dataset
                .cards_intersecting(visible_world)
                .into_iter()
                .filter(|card| self.canvas_card_matches_type_filter(card)),
            &BTreeMap::new(),
            self.gesture.as_ref(),
            visible_world,
        )
        .into_iter()
        .filter_map(|entry| match entry.entity {
            CanvasContextEntity::Card(concept_id) => Some(concept_id),
            CanvasContextEntity::Boundary(_) => None,
        })
        .collect()
    }

    fn canvas_card_is_interactive(&self, concept_id: u32) -> bool {
        self.dataset
            .card_by_id(concept_id)
            .is_some_and(|card| self.canvas_card_matches_type_filter(card))
            && !self.context_trail_card_ids().contains(&concept_id)
    }

    fn card_layer_visibility(
        &self,
        concept_id: u32,
        proposed_bounds: Option<CanvasRect>,
    ) -> Option<SemanticZoomCardLayerVisibility> {
        self.session().and_then(|session| {
            let document = session.document();
            let result = proposed_bounds.map_or_else(
                || {
                    document.semantic_zoom_card_layer_visibility(
                        session.active_version_id(),
                        session.active_lens_id(),
                        concept_id,
                    )
                },
                |bounds| {
                    document.semantic_zoom_card_layer_visibility_at_bounds(
                        session.active_version_id(),
                        session.active_lens_id(),
                        concept_id,
                        bounds,
                    )
                },
            );
            result.ok()
        })
    }

    fn layer_title_by_id(&self, layer_id: &str) -> Option<String> {
        self.session().and_then(|session| {
            session
                .document()
                .version(session.active_version_id())
                .ok()
                .and_then(|version| {
                    version
                        .lenses
                        .iter()
                        .find(|lens| lens.id == session.active_lens_id())
                })
                .and_then(|lens| {
                    lens.semantic_zoom_layers
                        .iter()
                        .find(|layer| layer.id == layer_id)
                })
                .map(|layer| layer.title.clone())
        })
    }

    fn card_base_layer_label(
        &self,
        visibility: Option<&SemanticZoomCardLayerVisibility>,
    ) -> String {
        let Some(visibility) = visibility else {
            return "no assigned layer".to_string();
        };
        if matches!(visibility.mode, SemanticZoomCardLayerMode::Manual { .. })
            && visibility.visible_layer_ids.len() > 1
        {
            return "multiple selected layers".to_string();
        }
        visibility
            .base_layer_id
            .as_deref()
            .and_then(|layer_id| self.layer_title_by_id(layer_id))
            .unwrap_or_else(|| "no assigned layer".to_string())
    }

    fn context_card_layer_titles(&self) -> BTreeMap<u32, String> {
        self.dataset
            .cards()
            .filter_map(|card| {
                let visibility = self.card_layer_visibility(card.id, None)?;
                let layer_id = visibility.base_layer_id?;
                self.layer_title_by_id(&layer_id)
                    .map(|title| (card.id, title))
            })
            .collect()
    }

    fn resize_layer_preview_text(&self) -> Option<String> {
        let PointerGesture::ResizeCard {
            concept_id,
            preview_bounds,
            centered,
            ..
        } = self.gesture.as_ref()?
        else {
            return None;
        };
        let current = self.card_layer_visibility(*concept_id, None);
        let preview = self.card_layer_visibility(*concept_id, Some(*preview_bounds));
        if current.is_none() && preview.is_none() {
            return None;
        }
        let current_label = self.card_base_layer_label(current.as_ref());
        let preview_label = self.card_base_layer_label(preview.as_ref());
        let anchor_label = if *centered {
            "center anchor"
        } else {
            "top-left anchor"
        };
        let automatic = current.as_ref().is_none_or(|visibility| {
            matches!(visibility.mode, SemanticZoomCardLayerMode::Automatic)
        });
        let layer_change = if current_label == preview_label && automatic {
            format!("automatic layer stays {preview_label}")
        } else if current_label == preview_label {
            format!("layer policy keeps the card on {preview_label}")
        } else {
            format!("Automatic layer: {current_label} to {preview_label}")
        };
        Some(format!("Resize preview: {layer_change}. {anchor_label}"))
    }

    fn active_lens_viewport_key(&self) -> Option<LensViewportKey> {
        self.session().map(|session| LensViewportKey {
            version_id: session.active_version_id().to_string(),
            lens_id: session.active_lens_id().to_string(),
        })
    }

    fn save_active_lens_camera(&mut self) {
        if let Some(key) = self.active_lens_viewport_key() {
            self.lens_canvas_controls.insert(
                key.clone(),
                LensCanvasControls {
                    search_query: self.canvas_search_query.clone(),
                    search_match_index: self.canvas_search_match_index,
                    type_filter_ids: self.canvas_type_filter_ids.clone(),
                    show_all_cards: self.show_all_cards,
                    roll_up_relationships: self.roll_up_relationships,
                },
            );
            self.lens_cameras.insert(key, self.interaction.camera());
        }
    }

    fn restore_active_lens_camera(&mut self) {
        let Some(key) = self.active_lens_viewport_key() else {
            return;
        };
        let controls = self
            .lens_canvas_controls
            .get(&key)
            .cloned()
            .unwrap_or_default();
        self.canvas_search_query = controls.search_query;
        self.canvas_search_match_index = controls.search_match_index;
        self.canvas_type_filter_ids = controls.type_filter_ids;
        self.show_all_cards = controls.show_all_cards;
        self.roll_up_relationships = controls.roll_up_relationships;
        self.last_canvas_pointer = None;
        let selected_card = self
            .selected_concept_id()
            .and_then(|concept_id| self.dataset.card_by_id(concept_id));
        let camera = camera_for_lens_activation(
            self.lens_cameras.get(&key).copied(),
            selected_card,
            self.canvas_viewport_size,
        );
        self.interaction.set_camera(camera);
        self.refresh_display_dataset();
    }

    fn set_canvas_zoom_scale(&mut self, scale: f32, viewport_size: Point) {
        self.interaction
            .set_camera(camera_with_scale_at_viewport_center(
                self.interaction.camera(),
                scale,
                viewport_size,
            ));
        self.refresh_display_dataset();
    }

    fn center_active_view(&mut self) {
        let card_bounds = self
            .session()
            .map(|session| session.dataset().cards().map(|card| card.bounds))
            .into_iter()
            .flatten();
        let bounds = card_bounds
            .chain(
                self.active_visual_boundaries()
                    .into_iter()
                    .map(|boundary| boundary.bounds),
            )
            .reduce(canvas_rect_union);
        let Some(bounds) = bounds else {
            self.set_failure(
                "VIEW NOT CENTERED",
                "Add a card or boundary before centering the view.",
            );
            return;
        };
        self.interaction.set_camera(camera_centered_to_fit_bounds(
            self.interaction.camera(),
            bounds,
            self.canvas_viewport_size,
        ));
        self.refresh_display_dataset();
        self.show_notice(
            "VIEW CENTERED",
            "All cards and boundaries in this view now fit in the canvas.",
            false,
        );
    }

    fn top_bar(&mut self, ui: &mut egui::Ui) {
        let built = concat!("Built ", env!("REASON_ROOM_BUILD_UTC"));
        let font = FontId::proportional(ATLAS_SUPPORTING_TEXT_SIZE);
        let width = ui
            .painter()
            .layout_no_wrap(built.to_string(), font.clone(), ATLAS_QUIET_GRAPHITE)
            .size()
            .x;
        ui.horizontal_top(|ui| {
            let controls_width = (ui.available_width() - width - ui.spacing().item_spacing.x).max(0.0);
            ui.allocate_ui_with_layout(Vec2::new(controls_width, 0.0), Layout::top_down(Align::Min), |ui| {
                ui.set_min_width(controls_width);
                self.top_bar_controls(ui);
            });
            ui.label(egui::RichText::new(built).font(font).color(ATLAS_QUIET_GRAPHITE))
                .on_hover_text("When this application artifact was compiled, in UTC. Reloading or opening it does not change this time.");
        });
    }

    fn top_bar_controls(&mut self, ui: &mut egui::Ui) {
        let (can_undo, can_redo) = self
            .session()
            .map(|session| (session.can_undo(), session.can_redo()))
            .unwrap_or((false, false));
        let workspace_controls_enabled = true;
        let history_controls_enabled = workspace_controls_enabled && self.dialog.is_none();
        ui.horizontal_wrapped(|ui| {
            let mut requested_example = None;
            let mut requested_new_canvas = false;
            let mut requested_open_canvas = false;
            #[cfg(target_arch = "wasm32")]
            let mut requested_open_watched_canvas = false;
            let mut requested_save_canvas = false;
            let mut requested_save_as_canvas = false;
            ui.menu_button("File", |ui| {
                ui.menu_button("Examples", |ui| {
                    for example in examples::ALL {
                        if ui.button(example.title).clicked() {
                            requested_example = Some(*example);
                            ui.close();
                        }
                    }
                });
                ui.separator();
                if ui.button("New canvas...").clicked() {
                    requested_new_canvas = true;
                    ui.close();
                }
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if ui.button("Open...").clicked() {
                        requested_open_canvas = true;
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Save").clicked() {
                        requested_save_canvas = true;
                        ui.close();
                    }
                    if ui.button("Save As...").clicked() {
                        requested_save_as_canvas = true;
                        ui.close();
                    }
                }
                #[cfg(target_arch = "wasm32")]
                {
                    if ui.button("Open canvas...").clicked() {
                        requested_open_canvas = true;
                        ui.close();
                    }
                    if ui.button("Open and refresh from file...").clicked() {
                        requested_open_watched_canvas = true;
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Save").clicked() {
                        requested_save_canvas = true;
                        ui.close();
                    }
                    if ui.button("Save As...").clicked() {
                        requested_save_as_canvas = true;
                        ui.close();
                    }
                }
            });
            if let Some(example) = requested_example {
                self.load_example(example);
            }
            if requested_new_canvas {
                self.open_new_canvas_dialog();
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                if requested_open_canvas {
                    self.request_open_canvas();
                }
                if requested_save_canvas {
                    self.save_active_canvas();
                }
                if requested_save_as_canvas {
                    self.request_save_as_canvas();
                }
            }
            #[cfg(target_arch = "wasm32")]
            if requested_open_canvas {
                self.request_open_local_browser_workspace();
            }
            #[cfg(target_arch = "wasm32")]
            if requested_open_watched_canvas {
                self.request_open_watched_browser_workspace();
            }
            #[cfg(target_arch = "wasm32")]
            if requested_save_canvas {
                self.save_browser_canvas();
            }
            #[cfg(target_arch = "wasm32")]
            if requested_save_as_canvas {
                self.open_save_as_browser_canvas_dialog();
            }
            let mut requested_copy_cards = false;
            let mut requested_paste_cards = false;
            ui.menu_button("Edit", |ui| {
                let has_selected_cards = self.interaction.selected_card_ids().next().is_some();
                if ui
                    .add_enabled(has_selected_cards, egui::Button::new("Copy cards"))
                    .clicked()
                {
                    requested_copy_cards = true;
                    ui.close();
                }
                let paste = ui
                    .add_enabled(self.can_paste_cards(), egui::Button::new("Paste cards"))
                    .on_hover_text(
                        "Paste into the same version and view where the cards were copied.",
                    );
                if paste.clicked() {
                    requested_paste_cards = true;
                    ui.close();
                }
            });
            if requested_copy_cards {
                self.copy_selected_cards();
            }
            if requested_paste_cards {
                self.paste_cards();
            }
            let mut requested_create_mode = None;
            ui.add_enabled_ui(workspace_controls_enabled, |ui| {
                ui.menu_button("Create", |ui| {
                    if ui.button("Card").clicked() {
                        requested_create_mode = Some(SidePaneMode::CreateCard);
                        ui.close();
                    }
                    if ui.button("Boundary").clicked() {
                        requested_create_mode = Some(SidePaneMode::CreateBoundary);
                        ui.close();
                    }
                    if ui.button("Zoom layer").clicked() {
                        requested_create_mode = Some(SidePaneMode::LayerCrossSection);
                        ui.close();
                    }
                });
            });
            if let Some(mode) = requested_create_mode {
                if mode == SidePaneMode::LayerCrossSection {
                    self.begin_inline_layer_creation();
                } else {
                    self.open_create_pane(mode);
                }
            }
            if ui.button("Preferences").clicked() {
                self.open_settings_window();
            }
            if ui
                .add_enabled(workspace_controls_enabled, egui::Button::new("Layers"))
                .clicked()
            {
                self.open_layer_cross_section();
            }
            if ui
                .add_enabled(
                    history_controls_enabled && can_undo,
                    egui::Button::new(history_command_label(HistoryCommand::Undo)),
                )
                .clicked()
            {
                self.apply_history_command(HistoryCommand::Undo);
            }
            if ui
                .add_enabled(
                    history_controls_enabled && can_redo,
                    egui::Button::new(history_command_label(HistoryCommand::Redo)),
                )
                .clicked()
            {
                self.apply_history_command(HistoryCommand::Redo);
            }
            if ui
                .add_enabled(workspace_controls_enabled, egui::Button::new("Cycle view"))
                .clicked()
            {
                self.cycle_lens();
            }
        });

        let Some(session) = self.session() else {
            if let WorkspaceState::ReadOnlyFailure(error) = &self.workspace_state {
                ui.colored_label(
                    ATLAS_RECOVERY_RED,
                    format!("WORKSPACE LOAD FAILED: {error}"),
                );
            }
            return;
        };
        let versions = session.available_versions();
        let active_version_id = session.active_version_id().to_string();
        let active_lens_id = session.active_lens_id().to_string();
        let lenses = session
            .available_lenses(&active_version_id)
            .unwrap_or_default();
        let mut requested_version = None;
        let mut requested_lens = None;
        let mut requested_structure_action = None;
        ui.add_enabled_ui(workspace_controls_enabled, |ui| {
            ui.horizontal(|ui| {
                ui.label("Version");
                egui::ComboBox::from_id_salt("workspace-version")
                    .selected_text(
                        versions
                            .iter()
                            .find(|version| version.id == active_version_id)
                            .map_or(active_version_id.as_str(), |version| version.title.as_str()),
                    )
                    .show_ui(ui, |ui| {
                        for version in &versions {
                            if ui
                                .selectable_label(version.id == active_version_id, &version.title)
                                .clicked()
                            {
                                requested_version = Some(version.id.clone());
                            }
                        }
                    });
                ui.label("View");
                egui::ComboBox::from_id_salt("workspace-lens")
                    .selected_text(
                        lenses
                            .iter()
                            .find(|lens| lens.id == active_lens_id)
                            .map_or(active_lens_id.as_str(), |lens| lens.title.as_str()),
                    )
                    .show_ui(ui, |ui| {
                        for lens in &lenses {
                            if ui
                                .selectable_label(lens.id == active_lens_id, &lens.title)
                                .clicked()
                            {
                                requested_lens = Some(lens.id.clone());
                            }
                        }
                    });
                ui.menu_button("Manage", |ui| {
                    ui.menu_button("Versions", |ui| {
                        let active_version_title = versions
                            .iter()
                            .find(|version| version.id == active_version_id)
                            .map(|version| version.title.clone())
                            .unwrap_or_else(|| active_version_id.clone());
                        if ui.button("New version from selected").clicked() {
                            requested_structure_action =
                                Some(WorkspaceStructureAction::CreateVersion {
                                    source_title: active_version_title.clone(),
                                });
                            ui.close();
                        }
                        if ui.button("Rename selected version").clicked() {
                            requested_structure_action =
                                Some(WorkspaceStructureAction::RenameVersion {
                                    version_id: active_version_id.clone(),
                                    title: active_version_title,
                                });
                            ui.close();
                        }
                        let can_delete_version =
                            active_version_id != session.document().current_state_id();
                        if ui
                            .add_enabled(
                                can_delete_version,
                                egui::Button::new("Delete selected version"),
                            )
                            .on_disabled_hover_text(
                                "The current version is the permanent document root.",
                            )
                            .clicked()
                        {
                            requested_structure_action =
                                Some(WorkspaceStructureAction::DeleteVersion {
                                    version_id: active_version_id.clone(),
                                });
                            ui.close();
                        }
                    });
                    ui.menu_button("Views", |ui| {
                        let active_lens_title = lenses
                            .iter()
                            .find(|lens| lens.id == active_lens_id)
                            .map(|lens| lens.title.clone())
                            .unwrap_or_else(|| active_lens_id.clone());
                        if ui.button("New view from selected").clicked() {
                            requested_structure_action =
                                Some(WorkspaceStructureAction::CreateLens {
                                    source_title: active_lens_title.clone(),
                                });
                            ui.close();
                        }
                        if ui.button("Rename selected view").clicked() {
                            requested_structure_action =
                                Some(WorkspaceStructureAction::RenameLens {
                                    lens_id: active_lens_id.clone(),
                                    title: active_lens_title,
                                });
                            ui.close();
                        }
                        if ui
                            .add_enabled(
                                lenses.len() > 1,
                                egui::Button::new("Delete selected view"),
                            )
                            .on_disabled_hover_text("Each version must keep at least one view.")
                            .clicked()
                        {
                            requested_structure_action =
                                Some(WorkspaceStructureAction::DeleteLens {
                                    lens_id: active_lens_id.clone(),
                                });
                            ui.close();
                        }
                    });
                });
                ui.label(format!("{} cards", self.dataset.card_count()));
                ui.label(format!(
                    "{} relationships",
                    self.dataset.relationship_count()
                ));
            });
        });
        if let Some(version_id) = requested_version {
            self.select_version(version_id);
        }
        if let Some(lens_id) = requested_lens {
            self.select_lens(lens_id);
        }
        match requested_structure_action {
            Some(WorkspaceStructureAction::CreateVersion { source_title }) => {
                self.open_create_version_pane(source_title)
            }
            Some(WorkspaceStructureAction::RenameVersion { version_id, title }) => {
                self.open_rename_version_dialog(version_id, title)
            }
            Some(WorkspaceStructureAction::DeleteVersion { version_id }) => {
                self.delete_version(version_id)
            }
            Some(WorkspaceStructureAction::CreateLens { source_title }) => {
                self.open_create_lens_pane(source_title)
            }
            Some(WorkspaceStructureAction::RenameLens { lens_id, title }) => {
                self.open_rename_lens_dialog(lens_id, title)
            }
            Some(WorkspaceStructureAction::DeleteLens { lens_id }) => self.delete_lens(lens_id),
            None => {}
        }
    }

    fn canvas_search_and_filter_controls(&mut self, ctx: &egui::Context, viewport: Rect) -> Rect {
        const WIDTH: f32 = 300.0;

        let position = Pos2::new(viewport.right() - WIDTH - 14.0, viewport.top() + 14.0);
        let type_ids = self.canvas_filter_type_ids();
        let match_ids_before_change = self.canvas_search_match_ids();
        let mut query_changed = false;
        let mut previous_requested = false;
        let mut next_requested = false;
        let mut search_focus_id = None;
        let mut clear_requested = false;
        let mut type_changes = Vec::new();

        let search_panel = egui::Area::new(egui::Id::unique("canvas-search-and-filter"))
            .order(egui::Order::Foreground)
            .fixed_pos(position)
            .show(ctx, |ui| {
                ui.set_width(WIDTH);
                egui::Frame::new()
                    .fill(color_with_opacity(ATLAS_SURVEY_SLATE, 0.96))
                    .stroke(Stroke::new(
                        1.0,
                        color_with_opacity(ATLAS_QUIET_GRAPHITE, 0.78),
                    ))
                    .inner_margin(egui::Margin::same(8))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let search = ui.add_sized(
                                Vec2::new(202.0, ui.spacing().interact_size.y),
                                egui::TextEdit::singleline(&mut self.canvas_search_query)
                                    .hint_text("Search cards"),
                            );
                            query_changed = search.changed();
                            if search.has_focus()
                                && ui.input(|input| input.key_pressed(egui::Key::Enter))
                            {
                                if ui.input(|input| input.modifiers.shift) {
                                    previous_requested = true;
                                } else {
                                    next_requested = true;
                                }
                                search_focus_id = Some(search.id);
                            }
                            if ui
                                .add_enabled(
                                    !self.canvas_search_query.is_empty(),
                                    egui::Button::new("Clear"),
                                )
                                .clicked()
                            {
                                clear_requested = true;
                            }
                        });
                        ui.horizontal(|ui| {
                            if ui.checkbox(&mut self.roll_up_relationships, "O-I").on_hover_text("Roll relationships from hidden child cards up to their visible parent.").changed() {
                                self.refresh_display_dataset();
                            }
                            let mut show_all = self.show_all_cards;
                            if ui.add_enabled(self.semantic_zoom_is_configured(), egui::Checkbox::new(&mut show_all, "View all")).changed() {
                                self.toggle_show_all_cards();
                            }
                        });
                        ui.horizontal(|ui| {
                            egui::menu::MenuButton::new("Filter types")
                                .config(
                                    egui::menu::MenuConfig::new().close_behavior(
                                        egui::PopupCloseBehavior::CloseOnClickOutside,
                                    ),
                                )
                                .ui(ui, |ui| {
                                    if ui.button("All card types").clicked() {
                                        type_changes.extend(
                                            type_ids.iter().cloned().map(|type_id| (type_id, true)),
                                        );
                                        ui.close();
                                    }
                                    ui.separator();
                                    for type_id in &type_ids {
                                        let mut enabled = self.canvas_type_filter_ids.is_empty()
                                            || self.canvas_type_filter_ids.contains(type_id);
                                        if ui.checkbox(&mut enabled, type_id).changed() {
                                            type_changes.push((type_id.clone(), enabled));
                                        }
                                    }
                                });
                            if self.canvas_search_query.trim().is_empty() {
                                ui.small("Find a card by name or summary.");
                            } else {
                                let current = if match_ids_before_change.is_empty() {
                                    0
                                } else {
                                    self.canvas_search_match_index % match_ids_before_change.len()
                                        + 1
                                };
                                ui.small(format!("{current} of {}", match_ids_before_change.len()));
                                previous_requested = ui
                                    .add_enabled(
                                        !match_ids_before_change.is_empty(),
                                        egui::Button::new("< Previous"),
                                    )
                                    .clicked();
                                next_requested = ui
                                    .add_enabled(
                                        !match_ids_before_change.is_empty(),
                                        egui::Button::new("Next >"),
                                    )
                                    .clicked();
                            }
                        });
                    });
            });

        if clear_requested {
            self.canvas_search_query.clear();
            query_changed = true;
        }
        let filter_changed = !type_changes.is_empty();
        for (type_id, enabled) in type_changes {
            self.set_canvas_type_filter_enabled(&type_id, enabled, &type_ids);
        }
        if query_changed || filter_changed {
            self.canvas_search_match_index = 0;
        }

        let match_ids = self.canvas_search_match_ids();
        if match_ids.is_empty() {
            self.canvas_search_match_index = 0;
        } else {
            self.canvas_search_match_index = search_match_index_after_navigation(
                self.canvas_search_match_index,
                match_ids.len(),
                previous_requested,
                next_requested,
            );
            if query_changed || previous_requested || next_requested {
                self.focus_canvas_search_match(match_ids[self.canvas_search_match_index]);
            }
        }
        if let Some(search_id) = search_focus_id {
            // Selecting a search result opens the inspector. Keep typing focus
            // in the search field so Enter and Shift+Enter can keep cycling.
            ctx.memory_mut(|memory| memory.request_focus(search_id));
        }

        // Return the actual frame, not a fixed capture rectangle. A fixed
        // height used to make an invisible area below this compact panel
        // capture canvas actions.
        search_panel.response.rect
    }

    fn canvas(&mut self, ui: &mut egui::Ui) {
        self.refresh_display_dataset();
        let available_size = ui.available_size();
        let (response, painter) = ui.allocate_painter(available_size, Sense::click_and_drag());
        let viewport = response.rect;
        let viewport_size = Point::new(viewport.width(), viewport.height());
        self.canvas_viewport_size = viewport_size;
        self.last_canvas_viewport = Some(viewport);
        if let Some(position) = ui
            .input(|input| input.pointer.hover_pos())
            .filter(|position| viewport.contains(*position))
        {
            self.last_canvas_pointer = Some(
                self.interaction
                    .camera()
                    .screen_to_world(point_in_viewport(position, viewport)),
            );
        }
        if self.fit_example_on_next_frame && viewport_size.x > 0.0 && viewport_size.y > 0.0 {
            self.fit_example_on_next_frame = false;
            let bounds = self
                .session()
                .into_iter()
                .flat_map(|session| session.dataset().cards().map(|card| card.bounds))
                .chain(
                    self.active_visual_boundaries()
                        .into_iter()
                        .map(|boundary| boundary.bounds),
                )
                .reduce(canvas_rect_union);
            if let Some(bounds) = bounds {
                self.interaction
                    .set_camera(example_overview_camera(bounds, viewport_size));
            }
            self.refresh_display_dataset();
            self.notices.clear();
            self.show_notice("EXPLORE THIS EXAMPLE", "Zoom into a large card: process steps appear at 24%, working detail at 85%. Select a card to read its notes. File > Examples opens a fresh copy.", false);
        }
        self.clear_noninteractive_canvas_selection();
        let controls_bounds = self.canvas_search_and_filter_controls(ui.ctx(), viewport);
        self.handle_canvas_input(ui, &response, viewport, viewport_size, controls_bounds);
        self.paint_canvas(&painter, viewport, viewport_size);
        self.canvas_context_menu(&response, viewport, viewport_size, controls_bounds);
    }

    fn canvas_context_menu(
        &mut self,
        response: &egui::Response,
        viewport: Rect,
        viewport_size: Point,
        controls_bounds: Rect,
    ) {
        let pointer_position = response
            .ctx
            .input(|input| input.pointer.hover_pos())
            .filter(|position| viewport.contains(*position));
        if response.secondary_clicked()
            && let Some(screen_position) = pointer_position
            && !controls_bounds.contains(screen_position)
        {
            let world_position = self
                .interaction
                .camera()
                .screen_to_world(point_in_viewport(screen_position, viewport));
            self.canvas_context_menu = Some(CanvasContextMenuState {
                target: self.canvas_context_menu_target_at(
                    world_position,
                    screen_position,
                    viewport,
                    viewport_size,
                ),
                world_position,
            });
        }

        response.context_menu(|ui| self.canvas_context_menu_contents(ui));
    }

    fn canvas_context_menu_target_at(
        &self,
        world_position: Point,
        screen_position: Pos2,
        viewport: Rect,
        viewport_size: Point,
    ) -> CanvasContextMenuTarget {
        if let Some(relationship_id) = self.canvas_relationship_at(screen_position, viewport) {
            return CanvasContextMenuTarget::Relationship(relationship_id);
        }
        if let Some(concept_id) = self.filtered_canvas_hit_test(world_position) {
            return CanvasContextMenuTarget::Card(concept_id);
        }
        if let Some(target) = boundary_pointer_target(
            &self.interactive_visual_boundaries(viewport_size),
            world_position,
            boundary_resize_handle_world_size(self.interaction.camera()),
        ) {
            let boundary_id = match target {
                BoundaryPointerTarget::Move { boundary_id }
                | BoundaryPointerTarget::Resize { boundary_id, .. } => boundary_id,
            };
            return CanvasContextMenuTarget::Boundary(boundary_id);
        }
        CanvasContextMenuTarget::Canvas
    }

    fn canvas_context_menu_contents(&mut self, ui: &mut egui::Ui) {
        let Some(context_menu) = self.canvas_context_menu.clone() else {
            return;
        };
        let mut action = None;
        match &context_menu.target {
            CanvasContextMenuTarget::Canvas => {
                ui.label("Canvas");
                if ui
                    .add_enabled(self.can_paste_cards(), egui::Button::new("Paste cards"))
                    .clicked()
                {
                    action = Some(CanvasContextMenuAction::PasteCards);
                }
            }
            CanvasContextMenuTarget::Card(concept_id) => {
                ui.label(format!("Card: {}", self.concept_label(*concept_id)));
                if ui.button("Inspect card").clicked() {
                    action = Some(CanvasContextMenuAction::InspectCard(*concept_id));
                }
                let layers = self
                    .session()
                    .and_then(|session| session.available_semantic_zoom_layers().ok())
                    .unwrap_or_default();
                let base = self
                    .card_layer_visibility(*concept_id, None)
                    .and_then(|visibility| visibility.base_layer_id);
                let index = layers
                    .iter()
                    .position(|layer| Some(&layer.id) == base.as_ref());
                if ui
                    .add_enabled(
                        index.is_some_and(|index| index > 0),
                        egui::Button::new("Move up a layer and resize"),
                    )
                    .clicked()
                {
                    action = Some(CanvasContextMenuAction::ShiftCardLayer(*concept_id, false));
                }
                if ui
                    .add_enabled(
                        index.is_some_and(|index| index + 1 < layers.len()),
                        egui::Button::new("Move down a layer and resize"),
                    )
                    .clicked()
                {
                    action = Some(CanvasContextMenuAction::ShiftCardLayer(*concept_id, true));
                }
                if ui.button("Copy card").clicked() {
                    action = Some(CanvasContextMenuAction::CopyCard(*concept_id));
                }
                if ui
                    .add(egui::Button::new("Delete card").fill(ATLAS_RECOVERY_RED))
                    .clicked()
                {
                    action = Some(CanvasContextMenuAction::DeleteCard(*concept_id));
                }
            }
            CanvasContextMenuTarget::Boundary(boundary_id) => {
                let title = self
                    .active_visual_boundaries()
                    .into_iter()
                    .find(|boundary| &boundary.id == boundary_id)
                    .map(|boundary| boundary.title)
                    .unwrap_or_else(|| "Unavailable boundary".to_string());
                ui.label(format!("Boundary: {title}"));
                if ui.button("Inspect boundary").clicked() {
                    action = Some(CanvasContextMenuAction::InspectBoundary(
                        boundary_id.clone(),
                    ));
                }
                if ui
                    .add(egui::Button::new("Delete boundary").fill(ATLAS_RECOVERY_RED))
                    .clicked()
                {
                    action = Some(CanvasContextMenuAction::DeleteBoundary(boundary_id.clone()));
                }
            }
            CanvasContextMenuTarget::Relationship(relationship_id) => {
                let label = self
                    .relationship_by_id(relationship_id)
                    .map(|relationship| relationship_description(&relationship))
                    .unwrap_or_else(|| "Unavailable relationship".to_string());
                ui.label(label);
                if ui.button("Inspect relationship").clicked() {
                    action = Some(CanvasContextMenuAction::InspectRelationship(
                        relationship_id.clone(),
                    ));
                }
                if ui
                    .add(egui::Button::new("Delete relationship").fill(ATLAS_RECOVERY_RED))
                    .clicked()
                {
                    action = Some(CanvasContextMenuAction::DeleteRelationship(
                        relationship_id.clone(),
                    ));
                }
            }
        }

        ui.separator();
        if ui.button("Create card here").clicked() {
            action = Some(CanvasContextMenuAction::CreateCard(
                context_menu.world_position,
            ));
        }
        if ui.button("Create boundary here").clicked() {
            action = Some(CanvasContextMenuAction::CreateBoundary(
                context_menu.world_position,
            ));
        }
        if ui.button("Create zoom layer").clicked() {
            action = Some(CanvasContextMenuAction::CreateLayer);
        }
        ui.separator();
        if ui.button("Center view").clicked() {
            action = Some(CanvasContextMenuAction::CenterView);
        }

        if let Some(action) = action {
            ui.close();
            self.apply_canvas_context_menu_action(action);
            self.canvas_context_menu = None;
        }
    }

    fn apply_canvas_context_menu_action(&mut self, action: CanvasContextMenuAction) {
        match action {
            CanvasContextMenuAction::InspectCard(concept_id) => {
                self.set_selection(Some(concept_id))
            }
            CanvasContextMenuAction::ShiftCardLayer(concept_id, deeper) => {
                self.commit_staged_action(
                    self.stage_action(|session| {
                        session
                            .stage_shift_concept_layer(concept_id, deeper)
                            .map_err(|error| error.to_string())
                    }),
                    Some(concept_id),
                    true,
                    "Moved and resized card to adjacent layer.",
                );
            }
            CanvasContextMenuAction::CopyCard(concept_id) => {
                self.set_selection(Some(concept_id));
                self.copy_selected_cards();
            }
            CanvasContextMenuAction::DeleteCard(concept_id) => {
                self.set_selection(Some(concept_id));
                self.delete_canvas_selection();
            }
            CanvasContextMenuAction::InspectBoundary(boundary_id) => {
                self.set_boundary_selection(Some(boundary_id));
            }
            CanvasContextMenuAction::DeleteBoundary(boundary_id) => {
                self.remove_visual_boundary(&boundary_id);
            }
            CanvasContextMenuAction::InspectRelationship(relationship_id) => {
                self.set_relationship_selection(Some(relationship_id));
            }
            CanvasContextMenuAction::DeleteRelationship(relationship_id) => {
                self.remove_relationship(relationship_id);
            }
            CanvasContextMenuAction::PasteCards => {
                let position = self
                    .canvas_context_menu
                    .as_ref()
                    .map(|menu| menu.world_position);
                self.paste_cards_at(position);
            }
            CanvasContextMenuAction::CreateCard(world_position) => {
                self.open_create_pane_at(SidePaneMode::CreateCard, world_position);
            }
            CanvasContextMenuAction::CreateBoundary(world_position) => {
                self.open_create_pane_at(SidePaneMode::CreateBoundary, world_position);
            }
            CanvasContextMenuAction::CreateLayer => self.begin_inline_layer_creation(),
            CanvasContextMenuAction::CenterView => self.center_active_view(),
        }
    }

    fn active_relationships(&self) -> Vec<RelationshipView> {
        let relationships: Vec<_> = self
            .session()
            .and_then(|session| {
                session
                    .document()
                    .relationships_in_lens(session.active_version_id(), session.active_lens_id())
                    .ok()
                    .map(|relationships| {
                        relationships
                            .iter()
                            .map(|relationship| RelationshipView {
                                id: relationship.id.clone(),
                                source_id: relationship.source_id,
                                target_id: relationship.target_id,
                                kind: relationship.kind,
                                source_title: session
                                    .document()
                                    .concept(session.active_version_id(), relationship.source_id)
                                    .map(|concept| concept.title.clone())
                                    .unwrap_or_else(|_| {
                                        format!("Concept #{}", relationship.source_id)
                                    }),
                                target_title: session
                                    .document()
                                    .concept(session.active_version_id(), relationship.target_id)
                                    .map(|concept| concept.title.clone())
                                    .unwrap_or_else(|_| {
                                        format!("Concept #{}", relationship.target_id)
                                    }),
                            })
                            .collect()
                    })
            })
            .unwrap_or_default();
        relationships
            .into_iter()
            .filter(|relationship| {
                let dataset = self
                    .session()
                    .map(|session| session.dataset())
                    .unwrap_or(&self.dataset);
                [relationship.source_id, relationship.target_id]
                    .into_iter()
                    .all(|id| {
                        dataset
                            .card_by_id(id)
                            .is_some_and(|card| self.canvas_card_matches_type_filter(card))
                    })
            })
            .collect()
    }

    fn visible_relationship_paths(&self, viewport: Rect) -> Vec<RelationshipPaintPath> {
        let relationships = self.active_relationships();
        let relationships = if self.semantic_layer_filtering_is_active() {
            let Some(frame) = self.semantic_zoom_frame() else {
                return Vec::new();
            };
            relationship_views_in_semantic_frame(&relationships, &frame)
        } else {
            relationships
        };
        let relationships: Vec<_> = relationships
            .into_iter()
            .filter(|relationship| {
                self.canvas_concept_id_matches_type_filter(relationship.source_id)
                    && self.canvas_concept_id_matches_type_filter(relationship.target_id)
            })
            .collect();
        let cards = self
            .session()
            .map(|session| session.dataset())
            .unwrap_or(&self.dataset);
        let world_bounds = cards
            .cards()
            .map(|card| {
                (
                    card.id,
                    move_preview_bounds(card.bounds, card.id, self.gesture.as_ref()),
                )
            })
            .collect();
        let boundaries: Vec<_> = self
            .active_visual_boundaries()
            .iter()
            .map(|boundary| boundary_preview_bounds(boundary, self.gesture.as_ref()))
            .collect();
        let camera = self.interaction.camera();
        let screen_boundaries: Vec<_> = boundaries
            .iter()
            .map(|bounds| canvas_rect_to_screen(*bounds, camera, viewport))
            .collect();
        let endpoint_bounds = self.relationship_endpoint_screen_bounds(&relationships, viewport);
        let mut cache = self.relationship_route_cache.borrow_mut();
        cache.update_layout(world_bounds, boundaries);
        relationships
            .iter()
            .filter_map(|relationship| {
                let path = cache.route(relationship.source_id, relationship.target_id)?;
                let mut path: Vec<_> = path
                    .iter()
                    .map(|point| canvas_point_to_screen(*point, camera, viewport))
                    .collect();
                // Keep the cached bends and attachment side. Only the terminal
                // anchor follows a painted card's minimum size/layer scale.
                for (index, id) in [
                    (0, relationship.source_id),
                    (path.len() - 1, relationship.target_id),
                ] {
                    if let Some(bounds) = endpoint_bounds.get(&id) {
                        let anchor = Point::new(path[index].x, path[index].y);
                        let port = relationship_port_toward(*bounds, anchor).1;
                        path[index] = Pos2::new(port.x, port.y);
                    }
                }
                relationship_paint_path(relationship, &path, &screen_boundaries, viewport)
            })
            .collect()
    }

    fn relationship_endpoint_screen_bounds(
        &self,
        relationships: &[RelationshipView],
        viewport: Rect,
    ) -> BTreeMap<u32, CanvasRect> {
        let endpoint_ids: BTreeSet<_> = relationships
            .iter()
            .flat_map(|relationship| [relationship.source_id, relationship.target_id])
            .collect();
        let selected_card_ids: BTreeSet<_> = self.interaction.selected_card_ids().collect();
        let camera = self.interaction.camera();
        self.dataset
            .cards()
            .filter(|card| endpoint_ids.contains(&card.id))
            .map(|card| {
                let screen_bounds = canvas_rect_to_screen(
                    move_preview_bounds(card.bounds, card.id, self.gesture.as_ref()),
                    camera,
                    viewport,
                );
                let painted_bounds =
                    card_paint_bounds(screen_bounds, selected_card_ids.contains(&card.id));
                (
                    card.id,
                    CanvasRect::new(
                        Point::new(painted_bounds.left(), painted_bounds.top()),
                        Point::new(painted_bounds.width(), painted_bounds.height()),
                    ),
                )
            })
            .collect()
    }

    fn visual_boundary_screen_rects(&self, viewport: Rect) -> Vec<Rect> {
        let camera = self.interaction.camera();
        self.active_visual_boundaries()
            .iter()
            .map(|boundary| {
                canvas_rect_to_screen(
                    boundary_preview_bounds(boundary, self.gesture.as_ref()),
                    camera,
                    viewport,
                )
            })
            .collect()
    }

    fn canvas_relationship_at(&self, screen_position: Pos2, viewport: Rect) -> Option<String> {
        // Relationships paint over card surfaces, including enclosing cards.
        // Match that order for hover, selection, and context menus.
        relationship_path_at(
            &self.visible_relationship_paths(viewport),
            screen_position,
            7.0,
        )
    }

    fn canvas_hover_target(
        &self,
        world_position: Point,
        screen_position: Pos2,
        viewport: Rect,
        viewport_size: Point,
    ) -> Option<CanvasHoverTarget> {
        let selected_card_ids: BTreeSet<_> = self.interaction.selected_card_ids().collect();
        if let Some((concept_id, _)) = card_resize_target(
            &self.dataset,
            &selected_card_ids,
            world_position,
            card_resize_handle_world_size(self.interaction.camera()),
        ) && self.canvas_card_is_interactive(concept_id)
        {
            return Some(CanvasHoverTarget::CardResize(concept_id));
        }
        if let Some((concept_id, edge)) = relationship_source_port_at(
            &self.dataset,
            world_position,
            relationship_port_hit_radius(self.interaction.camera()),
        ) && self.canvas_card_is_interactive(concept_id)
        {
            return Some(CanvasHoverTarget::RelationshipPort { concept_id, edge });
        }
        if let Some(relationship_id) = self.canvas_relationship_at(screen_position, viewport) {
            return Some(CanvasHoverTarget::Relationship(relationship_id));
        }
        if let Some(concept_id) = self.filtered_canvas_hit_test(world_position) {
            return Some(CanvasHoverTarget::Card(concept_id));
        }
        boundary_pointer_target(
            &self.interactive_visual_boundaries(viewport_size),
            world_position,
            boundary_resize_handle_world_size(self.interaction.camera()),
        )
        .map(|target| match target {
            BoundaryPointerTarget::Move { boundary_id }
            | BoundaryPointerTarget::Resize { boundary_id, .. } => {
                CanvasHoverTarget::Boundary(boundary_id)
            }
        })
    }

    fn handle_canvas_input(
        &mut self,
        ui: &egui::Ui,
        response: &egui::Response,
        viewport: Rect,
        viewport_size: Point,
        controls_bounds: Rect,
    ) {
        if matches!(
            self.dialog,
            Some(EditorDialog::ChooseRelationshipKind { .. })
        ) {
            return;
        }
        let input = ui.input(|input| {
            (
                input.pointer.hover_pos(),
                input.smooth_scroll_delta,
                input.modifiers.shift,
                input.pointer.primary_down(),
                input.pointer.middle_down(),
                input.pointer.primary_pressed(),
                input.pointer.primary_released(),
                input.modifiers.command || input.modifiers.ctrl,
                input.modifiers.alt,
            )
        });
        let pointer = input.0.filter(|position| viewport.contains(*position));
        let viewport_position = pointer.map(|position| point_in_viewport(position, viewport));

        // The search and type controls are painted over the canvas. They must
        // receive their click, scroll, and drag without also selecting or
        // panning a card beneath them. An existing canvas drag still finishes.
        if self.gesture.is_none()
            && pointer.is_some_and(|position| controls_bounds.contains(position))
        {
            self.previous_shift_hover = None;
            self.hover_target = None;
            return;
        }

        if let Some(viewport_position) = viewport_position {
            if input.1 != Vec2::ZERO {
                self.interaction.apply(
                    &self.dataset,
                    wheel_canvas_command(input.2, viewport_position, input.1),
                );
                self.refresh_display_dataset();
            }
            if input.2 {
                if let Some(command) =
                    shift_hover_pan_command(self.previous_shift_hover, viewport_position, true)
                {
                    self.interaction.apply(&self.dataset, command);
                }
                self.previous_shift_hover = Some(viewport_position);
            } else {
                self.previous_shift_hover = None;
            }
            let semantic_zoom_frame = self.semantic_zoom_frame();
            let scale_rail = semantic_scale_rail(
                viewport,
                self.interaction.camera().scale,
                semantic_zoom_frame
                    .as_ref()
                    .map(|frame| frame.layers.as_slice())
                    .unwrap_or_default(),
            );
            self.hover_target = semantic_scale_rail_zoom_scale_at(
                &scale_rail,
                pointer.expect("a viewport position has a pointer position"),
            )
            .map(|_| CanvasHoverTarget::ZoomSlider)
            .or_else(|| {
                semantic_scale_rail_layer_id_at(
                    &scale_rail,
                    pointer.expect("a viewport position has a pointer position"),
                )
                .map(CanvasHoverTarget::ScaleLayer)
            })
            .or_else(|| {
                self.canvas_hover_target(
                    self.interaction.camera().screen_to_world(viewport_position),
                    pointer.expect("a viewport position has a pointer position"),
                    viewport,
                    viewport_size,
                )
            });
            if let Some(target) = &self.hover_target {
                ui.ctx().set_cursor_icon(match target {
                    CanvasHoverTarget::Card(_) | CanvasHoverTarget::Boundary(_) => {
                        egui::CursorIcon::Grab
                    }
                    CanvasHoverTarget::CardResize(_) => egui::CursorIcon::ResizeNwSe,
                    CanvasHoverTarget::RelationshipPort { .. } => egui::CursorIcon::Crosshair,
                    CanvasHoverTarget::Relationship(_) | CanvasHoverTarget::ScaleLayer(_) => {
                        egui::CursorIcon::PointingHand
                    }
                    CanvasHoverTarget::ZoomSlider => egui::CursorIcon::ResizeHorizontal,
                });
            }

            if response.double_clicked_by(PointerButton::Primary)
                && let Some(concept_id) = self.filtered_canvas_hit_test(
                    self.interaction.camera().screen_to_world(viewport_position),
                )
            {
                self.set_selection(Some(concept_id));
                self.pending_concept_name_focus_id = Some(concept_id);
                self.gesture = None;
                self.refresh_display_dataset();
                return;
            }

            let pressed_zoom_slider = input.5
                && semantic_scale_rail_zoom_scale_at(
                    &scale_rail,
                    pointer.expect("a pointer exists"),
                )
                .is_some();
            if pressed_zoom_slider {
                let scale = semantic_scale_rail_zoom_scale_at(
                    &scale_rail,
                    pointer.expect("a pointer exists"),
                )
                .expect("the zoom slider was checked above");
                self.set_canvas_zoom_scale(scale, viewport_size);
                self.gesture = Some(PointerGesture::ZoomSlider);
            } else if input.5
                && let Some(layer_id) =
                    semantic_scale_rail_layer_id_at(&scale_rail, pointer.expect("a pointer exists"))
            {
                self.select_semantic_zoom_layer(layer_id);
                self.side_pane_mode = SidePaneMode::EditLayers;
                self.gesture = None;
                self.refresh_display_dataset();
                return;
            } else if response.drag_started_by(PointerButton::Middle) {
                self.gesture = Some(PointerGesture::Pan);
                self.interaction
                    .apply(&self.dataset, CanvasCommand::BeginPan { viewport_position });
            } else if input.5 {
                // Canvas input must take keyboard focus away from an earlier
                // inspector field. Otherwise Cmd+C / Cmd+V remains captured
                // by that field after a user has returned to the canvas.
                ui.ctx().memory_mut(|memory| {
                    if let Some(focused_id) = memory.focused() {
                        memory.surrender_focus(focused_id);
                    }
                });
                let world_position = self.interaction.camera().screen_to_world(viewport_position);
                let selected_card_ids: BTreeSet<_> = self.interaction.selected_card_ids().collect();
                if let Some((concept_id, initial_bounds)) = card_resize_target(
                    &self.dataset,
                    &selected_card_ids,
                    world_position,
                    card_resize_handle_world_size(self.interaction.camera()),
                ) && self.canvas_card_is_interactive(concept_id)
                {
                    self.set_selection(Some(concept_id));
                    self.gesture = Some(PointerGesture::ResizeCard {
                        concept_id,
                        initial_bounds,
                        anchor_world_position: world_position,
                        centered: input.8,
                        preview_bounds: initial_bounds,
                    });
                } else if let Some((source_id, source_port)) = relationship_source_port_at(
                    &self.dataset,
                    world_position,
                    relationship_port_hit_radius(self.interaction.camera()),
                ) && self.canvas_card_is_interactive(source_id)
                {
                    // Connecting an edge port is a canvas gesture, not inspection.
                    self.gesture = Some(PointerGesture::CreateRelationship {
                        source_id,
                        source_port,
                        preview_world_position: world_position,
                    });
                } else if let Some(relationship_id) = self.canvas_relationship_at(
                    pointer.expect("a viewport position has a pointer position"),
                    viewport,
                ) {
                    self.set_relationship_selection(Some(relationship_id));
                } else if let Some(concept_id) = self.filtered_canvas_hit_test(world_position) {
                    let selected_concept_ids: Vec<_> = selected_card_ids.into_iter().collect();
                    let concept_ids = if input.7 {
                        self.extend_selection(concept_id);
                        vec![concept_id]
                    } else {
                        move_gesture_concept_ids(&self.dataset, concept_id, &selected_concept_ids)
                    };
                    if !input.7 && !selected_concept_ids.contains(&concept_id) {
                        self.set_selection(Some(concept_id));
                    }
                    self.gesture = Some(PointerGesture::Move {
                        item_ids: self
                            .canvas_item_move_ids(concept_ids.into_iter().map(CanvasItemId::Card)),
                        anchor_world_position: self
                            .interaction
                            .camera()
                            .screen_to_world(viewport_position),
                        preview_world_delta: Point::default(),
                    });
                } else if let Some(target) = boundary_pointer_target(
                    &self.interactive_visual_boundaries(viewport_size),
                    self.interaction.camera().screen_to_world(viewport_position),
                    boundary_resize_handle_world_size(self.interaction.camera()),
                ) {
                    match target {
                        BoundaryPointerTarget::Move { boundary_id } => {
                            self.set_boundary_selection(Some(boundary_id.clone()));
                            self.gesture = Some(PointerGesture::Move {
                                item_ids: self
                                    .canvas_item_move_ids([CanvasItemId::Boundary(boundary_id)]),
                                anchor_world_position: self
                                    .interaction
                                    .camera()
                                    .screen_to_world(viewport_position),
                                preview_world_delta: Point::default(),
                            });
                        }
                        BoundaryPointerTarget::Resize {
                            boundary_id,
                            initial_bounds,
                        } => {
                            self.set_boundary_selection(Some(boundary_id.clone()));
                            self.gesture = Some(PointerGesture::ResizeBoundary {
                                boundary_id,
                                initial_bounds,
                                anchor_world_position: self
                                    .interaction
                                    .camera()
                                    .screen_to_world(viewport_position),
                                preview_bounds: initial_bounds,
                            });
                        }
                    }
                } else {
                    if !input.7 {
                        self.set_selection(None);
                    }
                    self.gesture = Some(PointerGesture::Selection);
                    self.interaction.apply(
                        &self.dataset,
                        CanvasCommand::BeginSelection { viewport_position },
                    );
                }
            }

            match self.gesture.take() {
                Some(PointerGesture::ZoomSlider) if input.3 => {
                    self.set_canvas_zoom_scale(
                        semantic_scale_rail_zoom_scale_from_x(
                            &scale_rail,
                            pointer
                                .expect("a pointer exists while dragging the zoom slider")
                                .x,
                        ),
                        viewport_size,
                    );
                    self.gesture = Some(PointerGesture::ZoomSlider);
                }
                Some(PointerGesture::ZoomSlider) if !input.3 => {
                    self.gesture = None;
                }
                Some(PointerGesture::Pan) if input.4 => {
                    self.interaction.apply(
                        &self.dataset,
                        CanvasCommand::ContinuePan { viewport_position },
                    );
                    self.gesture = Some(PointerGesture::Pan);
                }
                Some(PointerGesture::Selection) if input.3 => {
                    self.interaction.apply(
                        &self.dataset,
                        CanvasCommand::ContinueSelection { viewport_position },
                    );
                    self.gesture = Some(PointerGesture::Selection);
                }
                Some(PointerGesture::Move {
                    item_ids,
                    anchor_world_position,
                    ..
                }) if input.3 => {
                    self.gesture = Some(PointerGesture::Move {
                        item_ids,
                        anchor_world_position,
                        preview_world_delta: move_world_delta(
                            self.interaction.camera(),
                            anchor_world_position,
                            viewport_position,
                        ),
                    });
                }
                Some(PointerGesture::ResizeBoundary {
                    boundary_id,
                    initial_bounds,
                    anchor_world_position,
                    ..
                }) if input.3 => {
                    self.gesture = Some(PointerGesture::ResizeBoundary {
                        boundary_id,
                        initial_bounds,
                        anchor_world_position,
                        preview_bounds: resized_boundary_bounds(
                            initial_bounds,
                            move_world_delta(
                                self.interaction.camera(),
                                anchor_world_position,
                                viewport_position,
                            ),
                        ),
                    });
                }
                Some(PointerGesture::ResizeCard {
                    concept_id,
                    initial_bounds,
                    anchor_world_position,
                    ..
                }) if input.3 => {
                    self.gesture = Some(PointerGesture::ResizeCard {
                        concept_id,
                        initial_bounds,
                        anchor_world_position,
                        // Option changes the anchor while resizing. Reading it
                        // on every frame makes the modifier useful even when a
                        // person presses it after they begin the drag.
                        centered: input.8,
                        preview_bounds: resized_card_bounds(
                            initial_bounds,
                            move_world_delta(
                                self.interaction.camera(),
                                anchor_world_position,
                                viewport_position,
                            ),
                            input.8,
                            minimum_card_size_at_screen_scale(self.interaction.camera().scale),
                        ),
                    });
                }
                Some(PointerGesture::CreateRelationship {
                    source_id,
                    source_port,
                    ..
                }) if input.3 => {
                    self.gesture = Some(PointerGesture::CreateRelationship {
                        source_id,
                        source_port,
                        preview_world_position: self
                            .interaction
                            .camera()
                            .screen_to_world(viewport_position),
                    });
                }
                Some(PointerGesture::Pan) if !input.4 => {
                    self.interaction.apply(&self.dataset, CanvasCommand::EndPan);
                    self.gesture = None;
                }
                Some(PointerGesture::Selection) if !input.3 => {
                    self.interaction.apply(
                        &self.dataset,
                        CanvasCommand::EndSelection {
                            viewport_position,
                            extend_selection: input.7,
                        },
                    );
                    self.clear_noninteractive_canvas_selection();
                    self.gesture = None;
                    self.primary_relationship_concept_id =
                        self.interaction.selected_card_ids().next();
                }
                Some(PointerGesture::Move {
                    item_ids,
                    anchor_world_position,
                    ..
                }) if !input.3 => {
                    let world_delta = move_world_delta(
                        self.interaction.camera(),
                        anchor_world_position,
                        viewport_position,
                    );
                    self.gesture = None;
                    if world_delta != Point::default() {
                        self.move_canvas_items(&item_ids, world_delta);
                    }
                }
                Some(PointerGesture::ResizeBoundary {
                    boundary_id,
                    initial_bounds,
                    anchor_world_position,
                    ..
                }) if !input.3 => {
                    let bounds = resized_boundary_bounds(
                        initial_bounds,
                        move_world_delta(
                            self.interaction.camera(),
                            anchor_world_position,
                            viewport_position,
                        ),
                    );
                    self.gesture = None;
                    if bounds != initial_bounds {
                        self.resize_visual_boundary(&boundary_id, bounds);
                    }
                }
                Some(PointerGesture::ResizeCard {
                    concept_id,
                    initial_bounds,
                    anchor_world_position,
                    ..
                }) if !input.3 => {
                    let bounds = resized_card_bounds(
                        initial_bounds,
                        move_world_delta(
                            self.interaction.camera(),
                            anchor_world_position,
                            viewport_position,
                        ),
                        input.8,
                        minimum_card_size_at_screen_scale(self.interaction.camera().scale),
                    );
                    self.gesture = None;
                    if bounds != initial_bounds {
                        self.resize_concept(concept_id, bounds);
                    }
                }
                Some(PointerGesture::CreateRelationship { source_id, .. }) if !input.3 => {
                    let target_id = relationship_drop_target(
                        &self.dataset,
                        source_id,
                        self.interaction.camera().screen_to_world(viewport_position),
                        relationship_port_hit_radius(self.interaction.camera()),
                    );
                    self.gesture = None;
                    if let Some(target_id) = target_id.filter(|target_id| {
                        self.canvas_concept_id_matches_type_filter(source_id)
                            && self.canvas_concept_id_matches_type_filter(*target_id)
                    }) {
                        self.request_relationship_kind(source_id, target_id, pointer.unwrap());
                    }
                }
                gesture => self.gesture = gesture,
            }
        } else {
            self.previous_shift_hover = None;
            self.hover_target = None;
            if input.6
                && matches!(
                    self.gesture,
                    Some(PointerGesture::CreateRelationship { .. })
                        | Some(PointerGesture::ZoomSlider)
                )
            {
                self.gesture = None;
            }
        }
        self.refresh_display_dataset();
        let _ = viewport_size;
    }

    fn paint_canvas(&self, painter: &Painter, viewport: Rect, viewport_size: Point) {
        let (show_canvas_grid, show_relationship_labels) = self.display_preferences();
        painter.rect_filled(viewport, 0.0, ATLAS_GROUND);
        if show_canvas_grid {
            paint_grid(painter, viewport, self.interaction.camera());
        }
        let camera = self.interaction.camera();
        let visible_world = camera.visible_world_rect(viewport_size);
        let context_card_layer_titles = self.context_card_layer_titles();
        let active_visual_boundaries = self.active_visual_boundaries();
        let context_candidate_cards = self
            .dataset
            .cards_intersecting(visible_world)
            .into_iter()
            .filter(|card| self.canvas_card_matches_type_filter(card))
            .collect::<Vec<_>>();
        let context_entries = canvas_context_entries(
            &active_visual_boundaries,
            context_candidate_cards.iter().copied(),
            &context_card_layer_titles,
            self.gesture.as_ref(),
            visible_world,
        );
        let fade_boundary_bounds_by_id = active_visual_boundaries
            .iter()
            .map(|boundary| {
                (
                    boundary.id.clone(),
                    boundary_preview_bounds(boundary, self.gesture.as_ref()),
                )
            })
            .collect::<BTreeMap<_, _>>();
        let fade_card_bounds_by_id = context_candidate_cards
            .iter()
            .map(|card| {
                (
                    card.id,
                    move_preview_bounds(card.bounds, card.id, self.gesture.as_ref()),
                )
            })
            .collect::<BTreeMap<_, _>>();
        let boundary_layer_opacities: BTreeMap<_, _> = self
            .semantic_zoom_frame()
            .map(|frame| {
                frame
                    .layers
                    .into_iter()
                    .map(|layer| (layer.id, layer.opacity))
                    .collect()
            })
            .unwrap_or_default();
        paint_visual_boundaries(
            painter,
            &active_visual_boundaries,
            &VisualBoundaryPaintContext {
                fade_bounds_by_id: &fade_boundary_bounds_by_id,
                layer_opacities: &boundary_layer_opacities,
                filter_layers: self.semantic_layer_filtering_is_active(),
                selected_boundary_id: self.selected_boundary_id.as_deref(),
                hover_target: self.hover_target.as_ref(),
                gesture: self.gesture.as_ref(),
                camera,
                viewport,
                visible_world,
            },
        );
        let visual_boundary_screen_rects = self.visual_boundary_screen_rects(viewport);
        let visible_card_count = self
            .dataset
            .visible_cards(visible_world)
            .filter(|card| self.canvas_card_matches_type_filter(card))
            .count();
        let mut visible_cards: Vec<_> = self
            .dataset
            .visible_cards(visible_world)
            .filter(|card| self.canvas_card_matches_type_filter(card))
            .take(rendered_card_count(visible_card_count, MAX_RENDERED_CARDS))
            .collect();
        let displayed_card_world_bounds_by_concept_id =
            displayed_card_world_bounds_by_concept_id(&visible_cards, self.gesture.as_ref());
        let card_parent_by_child = card_parent_ids(
            displayed_card_world_bounds_by_concept_id
                .iter()
                .map(|(card_id, bounds)| (*card_id, *bounds)),
        );
        // Paint visual containers first so a fully enclosed card stays legible.
        // Pointer interaction uses the same child-first containment order.
        sort_cards_for_painting(&mut visible_cards, &card_parent_by_child);
        let mut relationship_boundary_screen_rects = visual_boundary_screen_rects.clone();
        relationship_boundary_screen_rects.extend(
            card_parent_by_child
                .values()
                .copied()
                .collect::<BTreeSet<_>>()
                .into_iter()
                .filter_map(|card_id| displayed_card_world_bounds_by_concept_id.get(&card_id))
                .map(|bounds| canvas_rect_to_screen(*bounds, camera, viewport)),
        );
        let semantic_zoom_frame = self.semantic_zoom_frame();
        let semantic_layer_filtering_is_active = self.semantic_layer_filtering_is_active();
        let scale_rail = semantic_scale_rail(
            viewport,
            camera.scale,
            semantic_zoom_frame
                .as_ref()
                .map(|frame| frame.layers.as_slice())
                .unwrap_or_default(),
        );
        let card_opacity_by_id: BTreeMap<_, _> = if semantic_layer_filtering_is_active {
            semantic_zoom_frame
                .as_ref()
                .map(|frame| {
                    frame
                        .cards
                        .iter()
                        .map(|card| (card.card.id, card.opacity))
                        .collect()
                })
                .unwrap_or_default()
        } else {
            BTreeMap::new()
        };
        let relationship_paths = self.visible_relationship_paths(viewport);
        let hidden_card_count = visible_card_count.saturating_sub(visible_cards.len());
        let zoom_level = card_detail_level(camera.scale);
        // Paint cards before relationship paths. Container cards are surfaces
        // that establish context, not occluders for the relationships between
        // their nested cards. Routed paths stop at card edges, so endpoint
        // cards remain unobscured while their in-container segments stay
        // readable.
        for card in &visible_cards {
            paint_card(
                painter,
                canvas_rect_to_screen(
                    move_preview_bounds(card.bounds, card.id, self.gesture.as_ref()),
                    camera,
                    viewport,
                ),
                card,
                CardPaintState {
                    selected: self.interaction.is_selected(card.id),
                    hovered: matches!(self.hover_target.as_ref(), Some(CanvasHoverTarget::Card(id)) if *id == card.id),
                    relationship_port_hovered: match self.hover_target.as_ref() {
                        Some(CanvasHoverTarget::RelationshipPort { concept_id, edge })
                            if *concept_id == card.id =>
                        {
                            Some(*edge)
                        }
                        _ => None,
                    },
                    resize_from_center: resize_anchor_for_card(card.id, self.gesture.as_ref()),
                    layer_shifting: match self.gesture.as_ref() {
                        Some(PointerGesture::ResizeCard {
                            concept_id,
                            preview_bounds,
                            ..
                        }) if *concept_id == card.id => {
                            self.card_layer_visibility(card.id, None)
                                .and_then(|visibility| visibility.base_layer_id)
                                != self
                                    .card_layer_visibility(card.id, Some(*preview_bounds))
                                    .and_then(|visibility| visibility.base_layer_id)
                        }
                        _ => false,
                    },
                    detail_level: zoom_level,
                    opacity: context_entity_opacity(
                        card_opacity_by_id.get(&card.id).copied().unwrap_or(1.0),
                        fade_card_bounds_by_id.get(&card.id).copied(),
                        visible_world,
                    ),
                },
            );
        }
        if semantic_layer_filtering_is_active && let Some(frame) = &semantic_zoom_frame {
            let relationship_context = SemanticRelationshipPaintContext {
                relationship_paths: &relationship_paths,
                boundary_rects: &relationship_boundary_screen_rects,
                selected_relationship_id: self.selected_relationship_id.as_deref(),
                hover_target: self.hover_target.as_ref(),
                show_relationship_labels,
            };
            paint_semantic_zoom_relationships(painter, frame, &relationship_context);
        } else {
            paint_interactive_relationship_paths(
                painter,
                &relationship_paths,
                &relationship_boundary_screen_rects,
                self.selected_relationship_id.as_deref(),
                self.hover_target.as_ref(),
                show_relationship_labels,
            );
        }
        paint_relationship_drag_preview(
            painter,
            &self.dataset,
            self.gesture.as_ref(),
            camera,
            viewport,
            &relationship_boundary_screen_rects,
        );
        paint_relationship_direction_markers(
            painter,
            &relationship_paths,
            &relationship_boundary_screen_rects,
            self.selected_relationship_id.as_deref(),
            semantic_zoom_frame.as_ref(),
        );
        if let Some(marquee) = self.interaction.marquee_viewport_rect() {
            let marquee_rect = Rect::from_min_size(
                viewport.min + Vec2::new(marquee.origin.x, marquee.origin.y),
                Vec2::new(marquee.size.x, marquee.size.y),
            );
            painter.rect_stroke(
                marquee_rect,
                2.0,
                Stroke::new(1.0, ATLAS_ROUTE_BLUE),
                StrokeKind::Inside,
            );
        }
        let hover_hint = self.hover_target.as_ref().map(canvas_hover_hint);
        let resize_preview = self.resize_layer_preview_text();
        if hidden_card_count > 0 {
            painter.text(
                viewport.left_bottom() + Vec2::new(14.0, -14.0),
                Align2::LEFT_BOTTOM,
                format!(
                    "{} more visible cards are not painted in this viewport.",
                    hidden_card_count
                ),
                FontId::proportional(ATLAS_SUPPORTING_TEXT_SIZE),
                ATLAS_CONTROL_AMBER,
            );
        }
        painter.text(
            semantic_scale_instruction_position(viewport),
            Align2::RIGHT_BOTTOM,
            "Wheel: zoom at pointer. Shift + wheel or hover: pan. Middle drag: pan.",
            FontId::proportional(ATLAS_SUPPORTING_TEXT_SIZE),
            ATLAS_QUIET_GRAPHITE,
        );
        let hovered_layer_id = match self.hover_target.as_ref() {
            Some(CanvasHoverTarget::ScaleLayer(layer_id)) => Some(layer_id.as_str()),
            _ => None,
        };
        paint_semantic_scale_rail(
            painter,
            &scale_rail,
            hovered_layer_id,
            matches!(self.hover_target, Some(CanvasHoverTarget::ZoomSlider)),
            viewport,
        );
        paint_context_trail(
            painter,
            viewport,
            &scale_rail,
            &context_entries,
            hover_hint,
            resize_preview.as_deref(),
        );
    }

    fn inspector(&mut self, ui: &mut egui::Ui) {
        // Every authoring and inspection workflow shares one continuous
        // control surface. Individual fields stay quiet; the surface, title,
        // and section dividers provide the hierarchy.
        let pane_rect = ui.max_rect().shrink2(Vec2::splat(INSPECTOR_OUTER_MARGIN));
        let content_rect = pane_rect.shrink2(Vec2::splat(INSPECTOR_INNER_MARGIN));
        ui.painter()
            .rect_filled(pane_rect, 8.0, color_with_opacity(ATLAS_GROUND, 0.94));
        ui.painter().rect_stroke(
            pane_rect,
            8.0,
            Stroke::new(1.0, color_with_opacity(ATLAS_QUIET_GRAPHITE, 0.68)),
            StrokeKind::Inside,
        );
        ui.scope_builder(
            egui::UiBuilder::new()
                .max_rect(content_rect)
                .layout(Layout::top_down(Align::Min)),
            |ui| {
                configure_inspector_ui(ui);
                egui::ScrollArea::vertical()
                    .id_salt("concept-inspector-scroll")
                    .max_height(content_rect.height())
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.set_width(content_rect.width());
                        self.inspector_contents(ui);
                    });
            },
        );
    }

    fn inspector_contents(&mut self, ui: &mut egui::Ui) {
        match self.side_pane_mode {
            SidePaneMode::CreateCard => {
                self.create_card_pane(ui);
                return;
            }
            SidePaneMode::CreateBoundary => {
                self.create_boundary_pane(ui);
                return;
            }
            SidePaneMode::CreateVersion => {
                self.create_version_pane(ui);
                return;
            }
            SidePaneMode::CreateLens => {
                self.create_lens_pane(ui);
                return;
            }
            SidePaneMode::EditLayers => {
                self.semantic_zoom_layer_section(ui);
                return;
            }
            SidePaneMode::LayerCrossSection => {
                self.semantic_zoom_layer_cross_section(ui);
                return;
            }
            SidePaneMode::Inspect => {}
        }
        if self.selected_boundary_id.is_some() {
            self.boundary_inspector(ui);
            return;
        }
        if self.selected_relationship_id.is_some() {
            self.relationship_inspector(ui);
            return;
        }
        let Some(concept_id) = self.selected_concept_id() else {
            inspector_pane_header(
                ui,
                "Concept inspector",
                "Select a card to edit its name, type, display, and connections.",
            );
            if ui.button("Collapse panel").clicked() {
                self.inspector_collapsed = true;
            }
            return;
        };
        let Some(card) = self.dataset.card_by_id(concept_id).cloned() else {
            inspector_pane_header(
                ui,
                "Concept inspector",
                "The selected card is outside the current view.",
            );
            return;
        };
        let selected_concept = self.session().and_then(|session| {
            session
                .document()
                .concept(session.active_version_id(), concept_id)
                .ok()
                .cloned()
        });
        let current_type_id = selected_concept
            .as_ref()
            .map(|concept| concept.type_id.clone())
            .unwrap_or_else(|| card.kind.type_id().to_string());
        let current_summary = selected_concept
            .as_ref()
            .map(|concept| concept.summary.clone())
            .unwrap_or_else(|| card.summary.clone());
        let current_details = selected_concept
            .as_ref()
            .map(|concept| concept.details.clone())
            .unwrap_or_default();
        let assertion_attributes = selected_concept
            .as_ref()
            .map(|concept| concept.attributes.clone())
            .unwrap_or_default();
        let types = self.session().map(supported_type_ids).unwrap_or_else(|| {
            ConceptKind::ALL
                .into_iter()
                .map(|kind| kind.type_id().to_string())
                .collect()
        });
        let focus_name = self.pending_concept_name_focus_id == Some(concept_id);
        if focus_name {
            self.pending_concept_name_focus_id = None;
        }
        let (mut manual_save_requested, mut autosave_requested) = {
            let mut manual_save_requested = false;
            let mut autosave_requested = false;
            let draft = inspector_draft_for(
                &mut self.inspector_draft,
                concept_id,
                &card.title,
                &current_type_id,
                &current_summary,
                &current_details,
                &assertion_attributes,
            );

            let (_, save_button) = concept_inspector_header(ui);
            if save_button.clicked() {
                manual_save_requested = true;
            }
            ui.small(format!("Card #{}", card.id));
            let card_parent_by_child = self.dataset.card_parent_ids();
            if let Some(parent_id) = card_parent_by_child.get(&concept_id) {
                let parent_title = self
                    .dataset
                    .card_by_id(*parent_id)
                    .map(|parent| parent.title.as_str())
                    .unwrap_or("an unavailable card");
                ui.small(format!("Inside card boundary: {parent_title}"));
            }
            let contained_card_count = card_parent_by_child
                .values()
                .filter(|parent_id| **parent_id == concept_id)
                .count();
            if contained_card_count > 0 {
                ui.small(format!(
                    "Card boundary: contains {contained_card_count} fully enclosed card{}.",
                    if contained_card_count == 1 { "" } else { "s" }
                ));
            }
            if card_parent_by_child.contains_key(&concept_id) || contained_card_count > 0 {
                ui.small("Spatial containment only. No relationship was created.");
            }
            inspector_section_heading(ui, "Identity");
            inspector_field_label(ui, "Name", None);
            let title_id = ui.make_persistent_id(("concept-inspector-name", concept_id));
            if focus_name {
                ui.memory_mut(|memory| memory.request_focus(title_id));
            }
            let title_response = ui.add_sized(
                Vec2::new(ui.available_width(), ui.spacing().interact_size.y),
                egui::TextEdit::singleline(&mut draft.title).id(title_id),
            );
            let title_has_focus = title_response.has_focus();
            let title_lost_focus = title_response.lost_focus();
            let save_on_title_enter =
                title_lost_focus && ui.input(|input| input.key_pressed(egui::Key::Enter));
            autosave_requested |= title_lost_focus;
            ui.add_space(4.0);
            inspector_field_label(
                ui,
                "Type",
                Some("Optional. It controls compatible relationship kinds."),
            );
            let type_before = draft.type_id.clone();
            egui::ComboBox::from_id_salt(("concept-type", concept_id))
                .selected_text(&draft.type_id)
                .width(ui.available_width())
                .show_ui(ui, |ui| {
                    for type_id in &types {
                        ui.selectable_value(&mut draft.type_id, type_id.clone(), type_id);
                    }
                });
            autosave_requested |= draft.type_id != type_before;

            inspector_section_heading(ui, "Description");
            inspector_field_label(ui, "Summary", None);
            let summary_response = ui.add_sized(
                Vec2::new(ui.available_width(), 72.0),
                egui::TextEdit::multiline(&mut draft.summary).desired_rows(3),
            );
            let summary_has_focus = summary_response.has_focus();
            autosave_requested |= summary_response.lost_focus();
            ui.horizontal(|ui| {
                if ui.small_button("Revert summary").clicked() {
                    draft.summary = current_summary.clone();
                }
            });

            ui.add_space(4.0);
            inspector_field_label(ui, "Details", Some("Free-form notes for this card."));
            let details_response = ui.add_sized(
                Vec2::new(ui.available_width(), 96.0),
                egui::TextEdit::multiline(&mut draft.details).desired_rows(4),
            );
            let details_has_focus = details_response.has_focus();
            autosave_requested |= details_response.lost_focus();
            ui.horizontal(|ui| {
                if ui.small_button("Revert details").clicked() {
                    draft.details = current_details.clone();
                }
            });

            let command_enter_target = consume_command_enter_for_focused_input(
                ui.ctx(),
                focused_input_draft_save_target(
                    title_has_focus,
                    summary_has_focus,
                    details_has_focus,
                    false,
                ),
            );
            if save_on_title_enter || command_enter_target {
                manual_save_requested = true;
            }
            (manual_save_requested, autosave_requested)
        };
        inspector_section_heading(ui, "Canvas display");
        ui.small(
            "Drag an edge node to connect. Drag the blue lower-right square to resize from the top-left. Hold Option for a center anchor.",
        );
        self.card_layer_policy_controls(ui, concept_id);
        self.relationship_creation_controls(ui, concept_id);
        inspector_section_heading(ui, "Relationships");
        let relationships = self.relationships_for_concept(concept_id);
        if relationships.is_empty() {
            ui.label("No relationships in the active version.");
        }
        for relationship in relationships {
            let mut kind_change = None;
            let mut remove_relationship = false;
            ui.group(|ui| {
                ui.label(format!("Source: {}", relationship.source_label()));
                ui.label("Kind");
                let mut kind = relationship.kind;
                egui::ComboBox::from_id_salt(("concept-relationship-kind", &relationship.id))
                    .selected_text(kind.label())
                    .show_ui(ui, |ui| {
                        for candidate in self.relationship_kinds_for_endpoints(
                            relationship.source_id,
                            relationship.target_id,
                        ) {
                            ui.selectable_value(&mut kind, candidate, candidate.label());
                        }
                    });
                if kind != relationship.kind {
                    kind_change = Some(kind);
                }
                ui.label(format!("Target: {}", relationship.target_label()));
                ui.label(format!("Relationship ID: {}", relationship.id));
                remove_relationship = ui.button("Remove").clicked();
            });
            if let Some(kind) = kind_change {
                self.change_relationship_kind(relationship.id.clone(), kind);
            }
            if remove_relationship {
                self.request_relationship_removal(relationship);
            }
        }

        ui.add_space(8.0);
        ui.separator();
        egui::CollapsingHeader::new("Assertions")
            .id_salt(("concept-assertions", concept_id))
            .default_open(false)
            .show(ui, |ui| {
                ui.small("User-stated attributes. They are not verified facts.");
                let draft = inspector_draft_for(
                    &mut self.inspector_draft,
                    concept_id,
                    &card.title,
                    &current_type_id,
                    &current_summary,
                    &current_details,
                    &assertion_attributes,
                );
                if draft.assertion_attributes.is_empty() {
                    ui.label("No assertions.");
                }
                let drafted_assertions: Vec<_> = draft
                    .assertion_attributes
                    .iter()
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect();
                for (key, value) in drafted_assertions {
                    ui.horizontal(|ui| {
                        ui.label(&key);
                        ui.label(&value);
                        if ui.button("Edit").clicked() {
                            load_assertion_into_draft(draft, &key, &value);
                        }
                        if ui.button("Remove").clicked() {
                            remove_draft_assertion(draft, &key);
                            autosave_requested = true;
                        }
                    });
                }
                ui.label("Assertion key");
                let assertion_key_has_focus = ui
                    .text_edit_singleline(&mut draft.assertion_key)
                    .has_focus();
                ui.label("Assertion value");
                let assertion_value_has_focus = ui
                    .text_edit_singleline(&mut draft.assertion_value)
                    .has_focus();
                let save_label =
                    assertion_save_label(&draft.assertion_attributes, &draft.assertion_key);
                ui.horizontal(|ui| {
                    if ui.button(save_label).clicked() {
                        match save_pending_draft_assertion(draft) {
                            Ok(()) => autosave_requested = true,
                            Err(error) => draft.assertion_error = Some(error.to_string()),
                        }
                    }
                    if ui.button("Cancel assertion").clicked() {
                        draft.assertion_key.clear();
                        draft.assertion_value.clear();
                        draft.assertion_error = None;
                    }
                });
                if let Some(error) = &draft.assertion_error {
                    ui.colored_label(ATLAS_RECOVERY_RED, error);
                }
                if ui.button("Revert assertions").clicked() {
                    revert_draft_assertions(draft, &assertion_attributes);
                    autosave_requested = true;
                }
                if consume_command_enter_for_focused_input(
                    ui.ctx(),
                    assertion_key_has_focus || assertion_value_has_focus,
                ) {
                    manual_save_requested = true;
                }
            });

        let save_request = {
            let draft = inspector_draft_for(
                &mut self.inspector_draft,
                concept_id,
                &card.title,
                &current_type_id,
                &current_summary,
                &current_details,
                &assertion_attributes,
            );
            if manual_save_requested {
                match save_pending_draft_assertion(draft) {
                    Ok(()) => Some((concept_inspector_edit_draft(draft), true)),
                    Err(error) => {
                        draft.assertion_error = Some(error.to_string());
                        None
                    }
                }
            } else if autosave_requested
                && inspector_autosave_is_safe(draft)
                && inspector_draft_has_unsaved_changes(
                    draft,
                    concept_id,
                    &card.title,
                    &current_type_id,
                    &current_summary,
                    &current_details,
                    &assertion_attributes,
                )
            {
                Some((concept_inspector_edit_draft(draft), false))
            } else {
                None
            }
        };
        if let Some((draft, show_notice)) = save_request {
            if show_notice {
                self.request_inspector_save(draft);
            } else {
                self.request_inspector_autosave(draft);
            }
        }
    }

    fn boundary_inspector(&mut self, ui: &mut egui::Ui) {
        let Some(boundary_id) = self.selected_boundary_id.clone() else {
            return;
        };
        let Some(boundary) = self
            .active_visual_boundaries()
            .into_iter()
            .find(|boundary| boundary.id == boundary_id)
        else {
            self.set_boundary_selection(None);
            return;
        };
        let should_replace_draft = self
            .boundary_inspector_draft
            .as_ref()
            .is_none_or(|draft| draft.boundary_id != boundary.id);
        if should_replace_draft {
            self.boundary_inspector_draft = Some(boundary_inspector_draft(&boundary));
        }

        inspector_pane_header(
            ui,
            "Boundary inspector",
            "A visual boundary groups the canvas. It does not create a graph fact.",
        );
        let layers = self
            .session()
            .and_then(|session| session.available_semantic_zoom_layers().ok())
            .unwrap_or_default();
        let mut layer_id = boundary.layer_id.clone();
        egui::ComboBox::from_label("Zoom layer")
            .selected_text(
                layer_id
                    .as_ref()
                    .and_then(|id| layers.iter().find(|layer| &layer.id == id))
                    .map(|layer| layer.title.as_str())
                    .unwrap_or("Master context"),
            )
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut layer_id, None, "Master context");
                for layer in layers {
                    ui.selectable_value(&mut layer_id, Some(layer.id), layer.title);
                }
            });
        if layer_id != boundary.layer_id {
            self.commit_staged_action(
                self.stage_action(|session| {
                    session
                        .stage_set_visual_boundary_layer(&boundary_id, layer_id)
                        .map_err(|error| error.to_string())
                }),
                None,
                true,
                "Moved boundary to zoom layer.",
            );
        }
        let mut save = None;
        let mut delete = None;
        let mut invalid_title = false;
        if let Some(draft) = self.boundary_inspector_draft.as_mut() {
            inspector_section_heading(ui, "Boundary details");
            inspector_field_label(ui, "Name", None);
            let title_has_focus = ui
                .add_sized(
                    Vec2::new(ui.available_width(), ui.spacing().interact_size.y),
                    egui::TextEdit::singleline(&mut draft.title),
                )
                .has_focus();
            ui.add_space(4.0);
            inspector_field_label(ui, "Short context", None);
            let summary_has_focus = ui
                .add_sized(
                    Vec2::new(ui.available_width(), 72.0),
                    egui::TextEdit::multiline(&mut draft.summary).desired_rows(3),
                )
                .has_focus();
            let save_on_command_enter = consume_command_enter_for_focused_input(
                ui.ctx(),
                title_has_focus || summary_has_focus,
            );
            ui.add_space(8.0);
            if save_on_command_enter || ui.add(primary_button("Save boundary")).clicked() {
                if draft.title.trim().is_empty() {
                    invalid_title = true;
                } else {
                    save = Some(draft.clone());
                }
            }

            inspector_section_heading(ui, "On canvas");
            ui.label("Drag the boundary to move it. Drag the lower-right handle to resize it.");
            ui.add_space(8.0);
            if ui
                .add(egui::Button::new("Remove boundary").fill(ATLAS_RECOVERY_RED))
                .clicked()
            {
                delete = Some(draft.boundary_id.clone());
            }
        }
        if invalid_title {
            self.set_failure("BOUNDARY NOT SAVED", "A boundary name cannot be empty.");
        }
        if let Some(draft) = save {
            self.update_visual_boundary(draft);
        }
        if let Some(boundary_id) = delete {
            self.remove_visual_boundary(&boundary_id);
        }
    }

    fn create_card_pane(&mut self, ui: &mut egui::Ui) {
        let types = self.session().map(supported_type_ids).unwrap_or_else(|| {
            ConceptKind::ALL
                .into_iter()
                .map(|kind| kind.type_id().to_string())
                .collect()
        });
        if self
            .card_creation_draft
            .type_id
            .as_ref()
            .is_some_and(|type_id| !types.contains(type_id))
        {
            self.card_creation_draft.type_id = None;
        }

        let mut create = false;
        let mut cancel = false;
        authoring_pane(
            ui,
            "Create card",
            "Add one named item to the current view.",
            |ui| {
                inspector_field_label(ui, "Name", None);
                let title_id = ui.make_persistent_id("create-card-name");
                let title_has_focus = ui
                    .add_sized(
                        Vec2::new(ui.available_width(), ui.spacing().interact_size.y),
                        egui::TextEdit::singleline(&mut self.card_creation_draft.title)
                            .id(title_id),
                    )
                    .has_focus();
                if self.pending_creation_name_focus == Some(CreationNameFocus::Card) {
                    ui.memory_mut(|memory| memory.request_focus(title_id));
                    self.pending_creation_name_focus = None;
                }
                ui.add_space(4.0);
                inspector_field_label(ui, "Short summary", None);
                let summary_has_focus = ui
                    .add_sized(
                        Vec2::new(ui.available_width(), 64.0),
                        egui::TextEdit::multiline(&mut self.card_creation_draft.summary)
                            .desired_rows(3),
                    )
                    .has_focus();
                ui.add_space(4.0);
                inspector_field_label(
                    ui,
                    "Type",
                    Some("Optional. Select one type when it helps identify the card."),
                );
                ui.horizontal_wrapped(|ui| {
                    for type_id in &types {
                        let selected = self.card_creation_draft.type_id.as_deref() == Some(type_id);
                        if ui.add(egui::RadioButton::new(selected, type_id)).clicked() {
                            self.card_creation_draft.type_id = toggle_created_card_type(
                                self.card_creation_draft.type_id.clone(),
                                type_id,
                            );
                        }
                    }
                });
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    create = ui.add(primary_button("Create card")).clicked()
                        || consume_command_enter_for_focused_input(
                            ui.ctx(),
                            focused_input_draft_save_target(
                                title_has_focus,
                                summary_has_focus,
                                false,
                                false,
                            ),
                        );
                    cancel = ui.button("Cancel").clicked();
                });
            },
        );
        if cancel {
            self.side_pane_mode = SidePaneMode::Inspect;
        } else if create {
            if self.card_creation_draft.title.trim().is_empty() {
                self.set_failure("CARD NOT SAVED", "A card name cannot be empty.");
            } else {
                let draft = self.card_creation_draft.clone();
                self.create_concept(draft);
                self.card_creation_draft.title.clear();
                self.card_creation_draft.summary.clear();
            }
        }
    }

    fn create_boundary_pane(&mut self, ui: &mut egui::Ui) {
        let mut create = false;
        let mut cancel = false;
        authoring_pane(
            ui,
            "Create boundary",
            "Add a named visual area to the current view.",
            |ui| {
                inspector_field_label(ui, "Name", None);
                let title_id = ui.make_persistent_id("create-boundary-name");
                let title_has_focus = ui
                    .add_sized(
                        Vec2::new(ui.available_width(), ui.spacing().interact_size.y),
                        egui::TextEdit::singleline(&mut self.boundary_creation_draft.title)
                            .id(title_id),
                    )
                    .has_focus();
                if self.pending_creation_name_focus == Some(CreationNameFocus::Boundary) {
                    ui.memory_mut(|memory| memory.request_focus(title_id));
                    self.pending_creation_name_focus = None;
                }
                ui.add_space(4.0);
                inspector_field_label(ui, "Short context", None);
                let summary_has_focus = ui
                    .add_sized(
                        Vec2::new(ui.available_width(), 64.0),
                        egui::TextEdit::multiline(&mut self.boundary_creation_draft.summary)
                            .desired_rows(3),
                    )
                    .has_focus();
                ui.small("A boundary does not add membership or a relationship.");
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    create = ui.add(primary_button("Create boundary")).clicked()
                        || consume_command_enter_for_focused_input(
                            ui.ctx(),
                            title_has_focus || summary_has_focus,
                        );
                    cancel = ui.button("Cancel").clicked();
                });
            },
        );
        if cancel {
            self.side_pane_mode = SidePaneMode::Inspect;
        } else if create {
            if self.boundary_creation_draft.title.trim().is_empty() {
                self.set_failure("BOUNDARY NOT SAVED", "A boundary name cannot be empty.");
            } else {
                let draft = self.boundary_creation_draft.clone();
                self.create_visual_boundary(draft);
                self.boundary_creation_draft.title.clear();
                self.boundary_creation_draft.summary.clear();
            }
        }
    }

    fn create_version_pane(&mut self, ui: &mut egui::Ui) {
        let mut cancel = false;
        let mut create = false;
        let mut title = String::new();
        authoring_pane(
            ui,
            "Create version",
            "Make a complete copy of the selected version.",
            |ui| {
                inspector_field_label(ui, "Version name", None);
                let title_id = ui.make_persistent_id("create-version-name");
                let title_has_focus = ui
                    .add_sized(
                        Vec2::new(ui.available_width(), ui.spacing().interact_size.y),
                        egui::TextEdit::singleline(&mut self.version_creation_draft.title)
                            .id(title_id),
                    )
                    .has_focus();
                if self.pending_creation_name_focus == Some(CreationNameFocus::Version) {
                    ui.memory_mut(|memory| memory.request_focus(title_id));
                    self.pending_creation_name_focus = None;
                }
                title = self.version_creation_draft.title.trim().to_string();
                let title_is_empty = title.is_empty();
                if title_is_empty {
                    ui.colored_label(ATLAS_RECOVERY_RED, "Enter a version name.");
                }
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    create = ui
                        .add_enabled(!title_is_empty, primary_button("Create version"))
                        .clicked()
                        || (!title_is_empty
                            && consume_command_enter_for_focused_input(ui.ctx(), title_has_focus));
                    cancel = ui.button("Cancel").clicked();
                });
            },
        );
        if cancel {
            self.side_pane_mode = SidePaneMode::Inspect;
        } else if create {
            self.create_version_from_active(title);
            self.side_pane_mode = SidePaneMode::Inspect;
        }
    }

    fn create_lens_pane(&mut self, ui: &mut egui::Ui) {
        let mut cancel = false;
        let mut create = false;
        let mut title = String::new();
        let mut clone_current = false;
        authoring_pane(
            ui,
            "Create view",
            "Start with a fresh view, or copy the current presentation.",
            |ui| {
                inspector_field_label(ui, "View name", None);
                let title_id = ui.make_persistent_id("create-view-name");
                let title_has_focus = ui
                    .add_sized(
                        Vec2::new(ui.available_width(), ui.spacing().interact_size.y),
                        egui::TextEdit::singleline(&mut self.lens_creation_draft.title)
                            .id(title_id),
                    )
                    .has_focus();
                if self.pending_creation_name_focus == Some(CreationNameFocus::View) {
                    ui.memory_mut(|memory| memory.request_focus(title_id));
                    self.pending_creation_name_focus = None;
                }
                ui.add_space(8.0);
                ui.checkbox(
                    &mut self.lens_creation_draft.clone_current,
                    "Copy the current view",
                );
                if self.lens_creation_draft.clone_current {
                    ui.small("Copies layout, boundaries, filters, and zoom layers.");
                } else {
                    ui.small("Starts empty. Workspace facts are unchanged.");
                }
                title = self.lens_creation_draft.title.trim().to_string();
                let title_is_empty = title.is_empty();
                if title_is_empty {
                    ui.colored_label(ATLAS_RECOVERY_RED, "Enter a view name.");
                }
                clone_current = self.lens_creation_draft.clone_current;
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    create = ui
                        .add_enabled(!title_is_empty, primary_button("Create view"))
                        .clicked()
                        || (!title_is_empty
                            && consume_command_enter_for_focused_input(ui.ctx(), title_has_focus));
                    cancel = ui.button("Cancel").clicked();
                });
            },
        );
        if cancel {
            self.side_pane_mode = SidePaneMode::Inspect;
        } else if create {
            self.create_lens_from_active(title, clone_current);
            self.side_pane_mode = SidePaneMode::Inspect;
        }
    }

    /// Renders the creation form at its selected insertion slot.
    fn inline_layer_creation_form(&mut self, ui: &mut egui::Ui) -> bool {
        let reveal_form = self.pending_creation_name_focus == Some(CreationNameFocus::Layer);
        let form_top = ui.cursor().min;
        ui.add_space(8.0);
        let mut create_requested = false;
        let mut cancel_requested = false;
        authoring_pane(
            ui,
            "Create zoom layer",
            "Set the range where this level of detail is visible.",
            |ui| {
                inspector_field_label(ui, "Name", None);
                let title_id = ui.make_persistent_id("create-zoom-layer-name");
                let title_has_focus = ui
                    .add_sized(
                        Vec2::new(ui.available_width(), ui.spacing().interact_size.y),
                        egui::TextEdit::singleline(&mut self.layer_creation_title).id(title_id),
                    )
                    .has_focus();
                if self.pending_creation_name_focus == Some(CreationNameFocus::Layer) {
                    ui.memory_mut(|memory| memory.request_focus(title_id));
                    self.pending_creation_name_focus = None;
                }
                ui.add_space(8.0);
                ui.horizontal_wrapped(|ui| {
                    ui.label("Start");
                    ui.add(
                        egui::DragValue::new(&mut self.layer_creation_active_zoom_start_percent)
                            .update_while_editing(false)
                            .range(1.0..=409_600.0)
                            .speed(1.0)
                            .suffix("%"),
                    );
                    ui.label("End");
                    ui.add(
                        egui::DragValue::new(&mut self.layer_creation_active_zoom_end_percent)
                            .update_while_editing(false)
                            .range(1.0..=409_600.0)
                            .speed(1.0)
                            .suffix("%"),
                    );
                });
                ui.horizontal_wrapped(|ui| {
                    ui.label("Fade");
                    ui.add(
                        egui::DragValue::new(&mut self.layer_creation_fade_percent)
                            .update_while_editing(false)
                            .range(0.0..=409_599.0)
                            .speed(1.0)
                            .suffix(" pp"),
                    );
                });
                ui.add_space(10.0);
                ui.horizontal_wrapped(|ui| {
                    create_requested = ui.add(primary_button("Create layer")).clicked()
                        || consume_command_enter_for_focused_input(ui.ctx(), title_has_focus);
                    cancel_requested = ui.button("Cancel").clicked();
                });
            },
        );

        if reveal_form {
            ui.scroll_to_rect(Rect::from_min_max(form_top, ui.min_rect().max), None);
        }

        if cancel_requested {
            self.layer_cross_section_creating = false;
        }
        if create_requested {
            if self.layer_creation_title.trim().is_empty() {
                self.set_failure("LAYER NOT SAVED", "A layer name cannot be empty.");
            } else {
                let title = self.layer_creation_title.clone();
                let percent_settings = SemanticZoomLayerPercentSettings {
                    active_zoom_range: ZoomPercentRange {
                        start_percent: self.layer_creation_active_zoom_start_percent,
                        end_percent: self.layer_creation_active_zoom_end_percent,
                    },
                    fade_percent: self.layer_creation_fade_percent,
                };
                let creating_base_layer = self.layer_cross_section_drafts.is_empty();
                let insertion_index = self.layer_creation_index;
                let staged = self.stage_action(|session| {
                    if creating_base_layer {
                        session
                            .stage_create_semantic_zoom_base_layer_with_percent_settings(
                                title,
                                percent_settings,
                            )
                            .map_err(|error| error.to_string())
                    } else {
                        session
                            .stage_create_semantic_zoom_layer_at(
                                title,
                                percent_settings,
                                insertion_index,
                            )
                            .map_err(|error| error.to_string())
                    }
                });
                let layer_id = staged
                    .as_ref()
                    .ok()
                    .and_then(|staged| staged_semantic_zoom_layer_id_at(staged, insertion_index));
                self.commit_staged_action(staged, None, false, "Created a zoom layer.");
                self.layer_cross_section_creating = false;
                self.open_layer_cross_section();
                if let Some(layer_id) = layer_id {
                    self.select_semantic_zoom_layer(layer_id);
                }
                return true;
            }
        }
        false
    }

    fn semantic_zoom_layer_cross_section(&mut self, ui: &mut egui::Ui) {
        if self.layer_cross_section_drafts.is_empty() {
            self.open_layer_cross_section();
        }
        let active_layer_ids: BTreeSet<_> = self
            .semantic_zoom_frame()
            .map(|frame| {
                strongest_visible_layer_indices(&frame.layers)
                    .into_iter()
                    .filter_map(|index| frame.layers.get(index))
                    .map(|layer| layer.id.clone())
                    .collect()
            })
            .unwrap_or_default();
        let domain_end = layer_cross_section_domain_end(
            &self.layer_cross_section_drafts,
            self.interaction.camera().scale * 100.0,
        );

        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("Zoom layers")
                    .size(ATLAS_INSPECTOR_TITLE_SIZE)
                    .strong()
                    .color(ATLAS_PAPER_SIGNAL),
            );
        });
        ui.label("The cross-section shows where each saved layer sits on the zoom scale.");
        ui.small("Change Start or End to move a layer. Fade extends its transition on both sides.");
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            ui.label(format!("Scale shown: 0% to {:.0}%", domain_end));
        });
        ui.separator();

        egui::Frame::new()
            .fill(color_with_opacity(ATLAS_GROUND, 0.52))
            .stroke(Stroke::new(
                1.0,
                color_with_opacity(ATLAS_QUIET_GRAPHITE, 0.6),
            ))
            .inner_margin(egui::Margin::same(8))
            .show(ui, |ui| {
                ui.label(egui::RichText::new("Master context").strong());
                ui.small("0% and above. Lens context; not a saved layer.");
                let (track, _) =
                    ui.allocate_exact_size(Vec2::new(ui.available_width(), 18.0), Sense::hover());
                paint_layer_cross_section_master_track(ui.painter(), track);
            });

        if self.layer_cross_section_drafts.is_empty()
            && self.layer_cross_section_creating
            && self.inline_layer_creation_form(ui)
        {
            return;
        }

        if self.layer_cross_section_drafts.is_empty()
            && !self.layer_cross_section_creating
            && ui.button("+ Create first layer").clicked()
        {
            self.begin_inline_layer_creation();
        }
        if self.layer_cross_section_drafts.is_empty() {
            ui.add_space(12.0);
            ui.label("No saved zoom layers. Choose a position below to create one.");
            return;
        }

        ui.small("Typed changes save on Enter or when you leave the field.");
        let mut auto_save_drafts = BTreeMap::new();
        let mut reorder_request = None;
        let mut range_link_requests = Vec::new();
        for target_index in 0..self.layer_cross_section_drafts.len() {
            if ui.button("+ Create layer here").clicked() {
                let target = self.layer_cross_section_drafts[target_index].clone();
                self.begin_inline_layer_creation();
                self.layer_creation_index = target_index;
                let width =
                    (target.active_zoom_end_percent - target.active_zoom_start_percent).max(1.0);
                self.layer_creation_active_zoom_start_percent = target.active_zoom_start_percent;
                self.layer_creation_active_zoom_end_percent =
                    target.active_zoom_start_percent + width;
            }
            if self.layer_cross_section_creating
                && self.layer_creation_index == target_index
                && self.inline_layer_creation_form(ui)
            {
                return;
            }
            let previous_end_percent = target_index
                .checked_sub(1)
                .and_then(|index| self.layer_cross_section_drafts.get(index))
                .map(|draft| draft.active_zoom_end_percent);
            let next_start_percent = self
                .layer_cross_section_drafts
                .get(target_index + 1)
                .map(|draft| draft.active_zoom_start_percent);
            let draft = &mut self.layer_cross_section_drafts[target_index];
            let active = active_layer_ids.contains(&draft.layer_id);
            let layer_id = draft.layer_id.clone();
            let frame = egui::Frame::new()
                .fill(color_with_opacity(
                    if active {
                        ATLAS_ROUTE_BLUE
                    } else {
                        ATLAS_GROUND
                    },
                    if active { 0.17 } else { 0.44 },
                ))
                .stroke(Stroke::new(
                    if active { 1.5 } else { 1.0 },
                    color_with_opacity(
                        if active {
                            ATLAS_ROUTE_BLUE
                        } else {
                            ATLAS_QUIET_GRAPHITE
                        },
                        0.82,
                    ),
                ))
                .inner_margin(egui::Margin::same(8));
            let (_, dropped_layer_id) = ui.dnd_drop_zone::<String, _>(frame, |ui| {
                ui.horizontal(|ui| {
                    ui.dnd_drag_source(
                        ui.make_persistent_id(("zoom-layer-drag", &layer_id)),
                        layer_id.clone(),
                        |ui| {
                            ui.label("Drag");
                        },
                    );
                    let title_response = ui.add_sized(
                        Vec2::new((ui.available_width() * 0.58).max(120.0), 24.0),
                        egui::TextEdit::singleline(&mut draft.title),
                    );
                    if title_response.lost_focus() {
                        auto_save_drafts.insert(draft.layer_id.clone(), draft.clone());
                    }
                    if active {
                        ui.label(
                            egui::RichText::new("Current")
                                .small()
                                .color(ATLAS_ROUTE_BLUE),
                        );
                    }
                });
                ui.vertical(|ui| {
                    ui.small(format!(
                        "{}% to {}%. Fade: {} percentage points.",
                        draft.active_zoom_start_percent,
                        draft.active_zoom_end_percent,
                        draft.fade_percent,
                    ));
                    let (track, _) = ui
                        .allocate_exact_size(Vec2::new(ui.available_width(), 20.0), Sense::hover());
                    paint_layer_cross_section_track(
                        ui.painter(),
                        track,
                        draft.active_zoom_start_percent,
                        draft.active_zoom_end_percent,
                        draft.fade_percent,
                        domain_end,
                        active,
                    );
                    ui.horizontal_wrapped(|ui| {
                        ui.label("Start");
                        let start_changed = ui
                            .add(
                                egui::DragValue::new(&mut draft.active_zoom_start_percent)
                                    .update_while_editing(false)
                                    .range(1.0..=409_600.0)
                                    .speed(1.0)
                                    .suffix("%"),
                            )
                            .changed();
                        ui.label("End");
                        let end_changed = ui
                            .add(
                                egui::DragValue::new(&mut draft.active_zoom_end_percent)
                                    .update_while_editing(false)
                                    .range(1.0..=409_600.0)
                                    .speed(1.0)
                                    .suffix("%"),
                            )
                            .changed();
                        if start_changed {
                            draft.active_zoom_end_percent = draft
                                .active_zoom_end_percent
                                .max(draft.active_zoom_start_percent);
                        }
                        if end_changed {
                            draft.active_zoom_start_percent = draft
                                .active_zoom_start_percent
                                .min(draft.active_zoom_end_percent);
                        }
                        if start_changed || end_changed {
                            auto_save_drafts.insert(draft.layer_id.clone(), draft.clone());
                        }
                    });
                    ui.horizontal_wrapped(|ui| {
                        ui.label("Fade");
                        let fade_changed = ui
                            .add(
                                egui::DragValue::new(&mut draft.fade_percent)
                                    .update_while_editing(false)
                                    .range(0.0..=409_599.0)
                                    .speed(1.0)
                                    .suffix(" pp"),
                            )
                            .changed();
                        if fade_changed {
                            auto_save_drafts.insert(draft.layer_id.clone(), draft.clone());
                        }
                        ui.label("Card size");
                        let card_scale_changed = ui
                            .add(
                                egui::DragValue::new(&mut draft.card_scale)
                                    .update_while_editing(false)
                                    .range(0.1..=4.0)
                                    .speed(0.05),
                            )
                            .changed();
                        if card_scale_changed {
                            auto_save_drafts.insert(draft.layer_id.clone(), draft.clone());
                        }
                        if let Some(previous_end_percent) = previous_end_percent
                            && ui.button("Start = previous end").clicked()
                        {
                            range_link_requests.push((target_index, previous_end_percent, true));
                        }
                        if let Some(next_start_percent) = next_start_percent
                            && ui.button("End = next start").clicked()
                        {
                            range_link_requests.push((target_index, next_start_percent, false));
                        }
                    });
                });
            });
            if let Some(source_layer_id) = dropped_layer_id.as_deref()
                && source_layer_id != &layer_id
            {
                reorder_request = Some((source_layer_id.to_string(), target_index));
            }
            ui.add_space(6.0);
        }

        if ui.button("+ Create layer here").clicked() {
            let last = self.layer_cross_section_drafts.last().cloned();
            self.begin_inline_layer_creation();
            if let Some(last) = last {
                self.layer_creation_active_zoom_start_percent = last.active_zoom_end_percent;
                self.layer_creation_active_zoom_end_percent = last.active_zoom_end_percent
                    + (last.active_zoom_end_percent - last.active_zoom_start_percent).max(1.0);
            }
        }

        if self.layer_cross_section_creating
            && self.layer_creation_index == self.layer_cross_section_drafts.len()
            && self.inline_layer_creation_form(ui)
        {
            return;
        }

        for (target_index, boundary_percent, set_start) in range_link_requests {
            let Some(draft) = self.layer_cross_section_drafts.get_mut(target_index) else {
                continue;
            };
            if set_start {
                draft.active_zoom_start_percent =
                    boundary_percent.min(draft.active_zoom_end_percent);
            } else {
                draft.active_zoom_end_percent =
                    boundary_percent.max(draft.active_zoom_start_percent);
            }
            auto_save_drafts.insert(draft.layer_id.clone(), draft.clone());
        }

        for (_, draft) in auto_save_drafts {
            let layer_id = draft.layer_id.clone();
            let title = draft.title.clone();
            let percent_settings = SemanticZoomLayerPercentSettings {
                active_zoom_range: ZoomPercentRange {
                    start_percent: draft.active_zoom_start_percent,
                    end_percent: draft.active_zoom_end_percent,
                },
                fade_percent: draft.fade_percent,
            };
            self.commit_staged_action_with_notice(
                self.stage_action(|session| {
                    session
                        .stage_update_semantic_zoom_layer_percent_settings(
                            &layer_id,
                            title,
                            percent_settings,
                            draft.card_scale,
                        )
                        .map_err(|error| error.to_string())
                }),
                None,
                true,
                SuccessNotice::None,
            );
        }
        if let Some((layer_id, destination_index)) = reorder_request
            && self
                .layer_cross_section_drafts
                .iter()
                .position(|draft| draft.layer_id == layer_id)
                .is_some_and(|source_index| source_index != destination_index)
        {
            self.commit_staged_action(
                self.stage_action(|session| {
                    session
                        .stage_reorder_semantic_zoom_layer(&layer_id, destination_index)
                        .map_err(|error| error.to_string())
                }),
                None,
                true,
                "Reordered zoom layers.",
            );
            self.open_layer_cross_section();
        }
    }

    fn semantic_zoom_layer_section(&mut self, ui: &mut egui::Ui) {
        let layers = self
            .session()
            .and_then(|session| session.available_semantic_zoom_layers().ok())
            .unwrap_or_default();
        if self.semantic_zoom_layer_id.is_none()
            && let Some(layer) = layers.first()
        {
            self.select_semantic_zoom_layer(layer.id.clone());
        }

        let selected_concept_ids: Vec<_> = self.interaction.selected_card_ids().collect();
        let selected_relationship_ids = self
            .session()
            .and_then(|session| {
                let selected: BTreeSet<_> = selected_concept_ids.iter().copied().collect();
                session
                    .document()
                    .relationships_in_lens(session.active_version_id(), session.active_lens_id())
                    .ok()
                    .map(|relationships| {
                        relationships
                            .iter()
                            .filter(|relationship| {
                                selected.contains(&relationship.source_id)
                                    || selected.contains(&relationship.target_id)
                            })
                            .map(|relationship| relationship.id.clone())
                            .collect::<Vec<_>>()
                    })
            })
            .unwrap_or_default();

        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("Zoom layers")
                    .size(ATLAS_INSPECTOR_TITLE_SIZE)
                    .strong()
                    .color(ATLAS_PAPER_SIGNAL),
            );
            if ui.small_button("Show cross-section").clicked() {
                self.open_layer_cross_section();
            }
        });
        ui.label("A card has one base layer. It remains visible as context through deeper layers.");
        ui.small("A layer only changes presentation. It does not change the card or relationship meaning.");
        inspector_section_heading(ui, "Layer selection");

        if layers.is_empty() {
            ui.label("No zoom layers are defined for this Lens.");
            ui.label("Use Create > Zoom layer to add the first overview layer.");
        } else {
            ui.label("Use Create > Zoom layer to add another level of detail.");
        }

        let mut requested_layer_id = None;
        if !layers.is_empty() {
            let selected_layer_title = self
                .semantic_zoom_layer_id
                .as_ref()
                .and_then(|layer_id| layers.iter().find(|layer| &layer.id == layer_id))
                .map(|layer| layer.title.as_str())
                .unwrap_or("Select a layer");
            inspector_field_label(ui, "Edit layer", None);
            egui::ComboBox::from_id_salt("semantic-zoom-layer")
                .selected_text(selected_layer_title)
                .width(ui.available_width())
                .show_ui(ui, |ui| {
                    for layer in &layers {
                        if ui
                            .selectable_label(
                                self.semantic_zoom_layer_id.as_deref() == Some(&layer.id),
                                &layer.title,
                            )
                            .clicked()
                        {
                            requested_layer_id = Some(layer.id.clone());
                        }
                    }
                });
        }
        if let Some(layer_id) = requested_layer_id {
            self.select_semantic_zoom_layer(layer_id);
        }

        let mut save_layer = None;
        let mut delete_layer = None;
        let mut assign_relationships_to_layer = None;
        if let Some(draft) = self.semantic_zoom_layer_draft.as_mut() {
            inspector_section_heading(ui, "Layer settings");
            inspector_field_label(ui, "Name", None);
            ui.add_sized(
                Vec2::new(ui.available_width(), ui.spacing().interact_size.y),
                egui::TextEdit::singleline(&mut draft.title),
            );
            ui.horizontal(|ui| {
                ui.label("Active zoom range");
                ui.add(
                    egui::DragValue::new(&mut draft.active_zoom_start_percent)
                        .range(1.0..=409_600.0)
                        .speed(1.0)
                        .suffix("%"),
                );
                ui.label("to");
                ui.add(
                    egui::DragValue::new(&mut draft.active_zoom_end_percent)
                        .range(1.0..=409_600.0)
                        .speed(1.0)
                        .suffix("%"),
                );
            });
            ui.horizontal(|ui| {
                ui.label("Fade");
                ui.add(
                    egui::DragValue::new(&mut draft.fade_percent)
                        .range(0.0..=409_599.0)
                        .speed(1.0)
                        .suffix(" percentage points"),
                );
            });
            ui.small("The layer fades by this amount before and after its active range.");
            ui.horizontal(|ui| {
                ui.label("Displayed card size");
                ui.add(
                    egui::DragValue::new(&mut draft.card_scale)
                        .range(0.1..=4.0)
                        .speed(0.05),
                );
            });
            ui.horizontal(|ui| {
                if ui.button("Save layer").clicked() {
                    save_layer = Some(draft.clone());
                }
                if ui.button("Delete layer").clicked() {
                    delete_layer = Some(draft.layer_id.clone());
                }
            });
            ui.separator();
            if selected_concept_ids.is_empty() {
                ui.label("Select a card to set its layer policy in the card inspector.");
            } else {
                ui.label(format!("{} selected card(s)", selected_concept_ids.len()));
                if selected_relationship_ids.is_empty() {
                    ui.small("The selected cards have no relationships to assign.");
                } else if ui
                    .button("Set selected relationships to this layer")
                    .clicked()
                {
                    assign_relationships_to_layer = Some(draft.layer_id.clone());
                }
            }
        }

        if let Some(draft) = save_layer {
            let layer_id = draft.layer_id.clone();
            let title = draft.title.clone();
            let percent_settings = SemanticZoomLayerPercentSettings {
                active_zoom_range: ZoomPercentRange {
                    start_percent: draft.active_zoom_start_percent,
                    end_percent: draft.active_zoom_end_percent,
                },
                fade_percent: draft.fade_percent,
            };
            let card_scale = draft.card_scale;
            self.commit_staged_action(
                self.stage_action(|session| {
                    session
                        .stage_update_semantic_zoom_layer_percent_settings(
                            &layer_id,
                            title,
                            percent_settings,
                            card_scale,
                        )
                        .map_err(|error| error.to_string())
                }),
                None,
                true,
                "Saved zoom layer controls.",
            );
        }
        if let Some(layer_id) = assign_relationships_to_layer {
            self.commit_staged_action(
                self.stage_action(|session| {
                    session
                        .stage_set_semantic_zoom_layer_relationships(
                            &layer_id,
                            &selected_relationship_ids,
                            true,
                        )
                        .map_err(|error| error.to_string())
                }),
                None,
                true,
                "Set the selected relationships' base layer.",
            );
        }
        if let Some(layer_id) = delete_layer {
            self.commit_staged_action(
                self.stage_action(|session| {
                    session
                        .stage_delete_semantic_zoom_layer(&layer_id)
                        .map_err(|error| error.to_string())
                }),
                None,
                true,
                "Deleted zoom layer.",
            );
            self.semantic_zoom_layer_id = None;
            self.semantic_zoom_layer_draft = None;
        }
    }

    fn relationships_for_concept(&self, concept_id: u32) -> Vec<RelationshipView> {
        self.active_relationships()
            .into_iter()
            .filter(|relationship| {
                relationship.source_id == concept_id || relationship.target_id == concept_id
            })
            .collect()
    }

    fn relationship_by_id(&self, relationship_id: &str) -> Option<RelationshipView> {
        self.active_relationships()
            .into_iter()
            .find(|relationship| relationship.id == relationship_id)
    }

    fn relationship_kinds_for_endpoints(
        &self,
        source_id: u32,
        target_id: u32,
    ) -> Vec<RelationshipKind> {
        let Some(session) = self.session() else {
            return Vec::new();
        };
        let document = session.document();
        let Ok(source) = document.concept(session.active_version_id(), source_id) else {
            return Vec::new();
        };
        let Ok(target) = document.concept(session.active_version_id(), target_id) else {
            return Vec::new();
        };
        relationship_kinds()
            .into_iter()
            .filter(|kind| kind.supports_endpoints(&source.type_id, &target.type_id))
            .collect()
    }

    fn relationship_endpoints_share_containment_chain(
        &self,
        source_id: u32,
        target_id: u32,
    ) -> bool {
        cards_are_in_containment_chain(&self.dataset.card_parent_ids(), source_id, target_id)
    }

    fn relationship_inspector(&mut self, ui: &mut egui::Ui) {
        let Some(relationship_id) = self.selected_relationship_id.clone() else {
            return;
        };
        let Some(relationship) = self.relationship_by_id(&relationship_id) else {
            self.set_relationship_selection(None);
            return;
        };

        inspector_pane_header(
            ui,
            "Relationship inspector",
            "This saved connection is directional. Change its kind or remove it here.",
        );
        inspector_section_heading(ui, "Endpoints");
        inspector_field_label(ui, "Source", None);
        ui.strong(relationship.source_label());
        ui.add_space(4.0);
        inspector_field_label(ui, "Target", None);
        ui.strong(relationship.target_label());
        inspector_section_heading(ui, "Relationship kind");
        let mut kind = relationship.kind;
        let compatible_kinds =
            self.relationship_kinds_for_endpoints(relationship.source_id, relationship.target_id);
        egui::ComboBox::from_id_salt(("relationship-inspector-kind", &relationship.id))
            .selected_text(kind.label())
            .width(ui.available_width())
            .show_ui(ui, |ui| {
                for candidate in compatible_kinds {
                    ui.selectable_value(&mut kind, candidate, candidate.label());
                }
            });
        if kind != relationship.kind {
            self.change_relationship_kind(relationship.id.clone(), kind);
        }
        ui.small(format!("Connection ID: {}", relationship.id));
        ui.add_space(8.0);
        if ui
            .add(egui::Button::new("Remove relationship").fill(ATLAS_RECOVERY_RED))
            .clicked()
        {
            self.request_relationship_removal(relationship);
        }
    }

    fn card_layer_policy_controls(&mut self, ui: &mut egui::Ui, concept_id: u32) {
        let Some(session) = self.session() else {
            return;
        };
        let layers = session.available_semantic_zoom_layers().unwrap_or_default();
        if layers.is_empty() {
            return;
        }
        let current_mode = session
            .document()
            .semantic_zoom_card_layer_mode(
                session.active_version_id(),
                session.active_lens_id(),
                concept_id,
            )
            .unwrap_or(SemanticZoomCardLayerMode::Automatic);
        let visibility = session
            .document()
            .semantic_zoom_card_layer_visibility(
                session.active_version_id(),
                session.active_lens_id(),
                concept_id,
            )
            .ok();
        let mut requested_mode = None;
        let automatic = matches!(current_mode, SemanticZoomCardLayerMode::Automatic);
        let selected_layer_ids = card_layer_mode_selected_ids(&current_mode, &layers, &visibility);

        inspector_section_heading(ui, "Layer display");
        ui.horizontal(|ui| {
            if ui.selectable_label(automatic, "Automatic").clicked() {
                requested_mode = Some(SemanticZoomCardLayerMode::Automatic);
            }
            if ui.selectable_label(!automatic, "Manual").clicked() {
                let layer_ids = selected_layer_ids
                    .first()
                    .cloned()
                    .or_else(|| {
                        visibility
                            .as_ref()
                            .and_then(|value| value.base_layer_id.clone())
                    })
                    .map_or_else(|| vec![layers[0].id.clone()], |layer_id| vec![layer_id]);
                requested_mode = Some(SemanticZoomCardLayerMode::Manual { layer_ids });
            }
        });
        if automatic {
            ui.small("Card size selects exactly one layer.");
            egui::Frame::new()
                .fill(color_with_opacity(ATLAS_SURVEY_SLATE, 0.54))
                .stroke(Stroke::new(
                    1.0,
                    color_with_opacity(ATLAS_QUIET_GRAPHITE, 0.45),
                ))
                .inner_margin(egui::Margin::same(8))
                .show(ui, |ui| {
                    let layer_title = selected_layer_ids
                        .first()
                        .and_then(|layer_id| layers.iter().find(|layer| layer.id == *layer_id))
                        .map(|layer| layer.title.as_str())
                        .unwrap_or("No resolved layer");
                    ui.label(format!("Resolved layer: {layer_title}"));
                    ui.small("Resize the card to move it to a different layer.");
                });
        } else {
            ui.small("Select every layer where this card should appear.");
            egui::Frame::new()
                .fill(color_with_opacity(ATLAS_SURVEY_SLATE, 0.54))
                .stroke(Stroke::new(
                    1.0,
                    color_with_opacity(ATLAS_QUIET_GRAPHITE, 0.45),
                ))
                .inner_margin(egui::Margin::same(8))
                .show(ui, |ui| {
                    ui.label("Visible zoom layers");
                    for layer in &layers {
                        let mut selected = selected_layer_ids.contains(&layer.id);
                        let can_change = !selected || selected_layer_ids.len() > 1;
                        let response = ui.add_enabled(
                            can_change,
                            egui::Checkbox::new(&mut selected, &layer.title),
                        );
                        let response = if can_change {
                            response
                        } else {
                            response.on_disabled_hover_text(
                                "Select at least one layer for a manually displayed card.",
                            )
                        };
                        if response.changed() {
                            let mut layer_ids = selected_layer_ids.clone();
                            if selected {
                                layer_ids.push(layer.id.clone());
                            } else {
                                layer_ids.retain(|layer_id| layer_id != &layer.id);
                            }
                            requested_mode = Some(SemanticZoomCardLayerMode::Manual { layer_ids });
                        }
                    }
                });
        }

        if let Some(mode) = requested_mode.filter(|mode| *mode != current_mode) {
            self.commit_staged_action(
                self.stage_action(|session| {
                    session
                        .stage_set_semantic_zoom_card_layer_mode(concept_id, mode)
                        .map_err(|error| error.to_string())
                }),
                Some(concept_id),
                false,
                "Saved card layer display.",
            );
        }
    }

    fn relationship_creation_controls(&mut self, ui: &mut egui::Ui, primary_concept_id: u32) {
        let selected_concept_ids: Vec<_> = self.interaction.selected_card_ids().collect();
        if selected_concept_ids.len() < 2 {
            ui.small("Select a second card with Command-click, or drag from this card's right edge node to a target's left edge node.");
            return;
        }

        ui.separator();
        ui.heading("Create relationship");
        ui.label("Primary card");
        ui.strong(self.concept_label(primary_concept_id));
        ui.small("Choose a target, then choose the relationship type.");
        let mut create_target = None;
        let mut primary_change = None;
        for concept_id in selected_concept_ids {
            if concept_id == primary_concept_id {
                continue;
            }
            let target_title = self.concept_label(concept_id);
            let containment_blocks_creation =
                self.relationship_endpoints_share_containment_chain(primary_concept_id, concept_id);
            ui.group(|ui| {
                ui.label(target_title);
                if containment_blocks_creation {
                    ui.small("Unavailable: one card contains the other.");
                    if ui.button("Make primary").clicked() {
                        primary_change = Some(concept_id);
                    }
                } else {
                    ui.horizontal(|ui| {
                        if ui.button("Connect...").clicked() {
                            create_target = Some(concept_id);
                        }
                        if ui.button("Make primary").clicked() {
                            primary_change = Some(concept_id);
                        }
                    });
                }
            });
        }
        if let Some(concept_id) = primary_change {
            self.primary_relationship_concept_id = Some(concept_id);
            self.inspector_draft = None;
        }
        if let Some(target_id) = create_target {
            self.request_relationship_kind(
                primary_concept_id,
                target_id,
                ui.input(|input| input.pointer.hover_pos())
                    .unwrap_or(ui.next_widget_position()),
            );
        }
    }

    fn dialogs(&mut self, ctx: &egui::Context) {
        match self.dialog.take() {
            Some(EditorDialog::NewCanvas { mut draft }) => {
                let mut keep_open = true;
                egui::Window::new("New canvas")
                    .collapsible(false)
                    .resizable(false)
                    .default_width(540.0)
                    .show(ctx, |ui| {
                        #[cfg(not(target_arch = "wasm32"))]
                        ui.label("Create an empty canvas in its own local document.");
                        #[cfg(target_arch = "wasm32")]
                        ui.label("Create an empty canvas for this browser session.");
                        ui.add_space(10.0);
                        ui.label("Canvas name");
                        let _title_response = ui.add(
                            egui::TextEdit::singleline(&mut draft.title)
                                .desired_width(ui.available_width()),
                        );
                        #[cfg(not(target_arch = "wasm32"))]
                        if _title_response.changed() {
                            draft.update_suggested_path();
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            ui.add_space(8.0);
                            ui.horizontal(|ui| {
                                ui.label("Save location");
                                if ui.small_button("Use suggested location").clicked() {
                                    draft.save_path_tracks_title = true;
                                    draft.update_suggested_path();
                                }
                            });
                            let path_response = ui.add(
                                egui::TextEdit::singleline(&mut draft.save_path)
                                    .desired_width(ui.available_width()),
                            );
                            if path_response.changed() {
                                draft.save_path_tracks_title = false;
                            }
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        ui.small(
                            "The new document is saved before this window switches to it. Existing documents are not changed.",
                        );
                        #[cfg(target_arch = "wasm32")]
                        ui.small(
                            "The new canvas opens in this page. Use Save to download a local JSON file.",
                        );
                        ui.add_space(12.0);
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui.button("Cancel").clicked() {
                                keep_open = false;
                            }
                            if ui.button("Create canvas").clicked() {
                                match self.create_new_canvas(&draft) {
                                    Ok(()) => keep_open = false,
                                    Err(error) => self.set_failure("CANVAS NOT CREATED", error),
                                }
                            }
                        });
                    });
                if keep_open {
                    self.dialog = Some(EditorDialog::NewCanvas { draft });
                }
            }
            #[cfg(target_arch = "wasm32")]
            Some(EditorDialog::SaveAsBrowserCanvas { mut draft }) => {
                let mut keep_open = true;
                egui::Window::new("Save canvas as")
                    .collapsible(false)
                    .resizable(false)
                    .default_width(540.0)
                    .show(ctx, |ui| {
                        ui.label("Save a copy of this canvas as a local JSON download.");
                        ui.add_space(10.0);
                        ui.label("Canvas name");
                        ui.add(
                            egui::TextEdit::singleline(&mut draft.title)
                                .desired_width(ui.available_width()),
                        );
                        ui.small(
                            "The current canvas stays unchanged. Your browser chooses the download location.",
                        );
                        ui.add_space(12.0);
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui.button("Cancel").clicked() {
                                keep_open = false;
                            }
                            if ui.button("Save as").clicked() {
                                match self.save_as_browser_canvas(&draft) {
                                    Ok(()) => keep_open = false,
                                    Err(error) => self.set_failure("CANVAS NOT SAVED", error),
                                }
                            }
                        });
                    });
                if keep_open {
                    self.dialog = Some(EditorDialog::SaveAsBrowserCanvas { draft });
                }
            }
            Some(EditorDialog::ConfirmTypeChange {
                draft,
                staged,
                lens_title,
            }) => {
                let type_id = draft.type_id.clone();
                let mut staged = Some(staged);
                let mut keep_open = true;
                egui::Window::new("Confirm type change")
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.label(format!(
                            "The {type_id} type is not included in {lens_title}."
                        ));
                        ui.label("The concept stays in the workspace but is hidden in this Lens.");
                        ui.horizontal(|ui| {
                            if ui.button("Save inspector changes").clicked() {
                                self.confirm_inspector_save(
                                    draft.clone(),
                                    staged
                                        .take()
                                        .expect("the staged inspector save is available"),
                                );
                                keep_open = false;
                            }
                            if ui.button("Cancel").clicked() {
                                keep_open = false;
                            }
                        });
                    });
                if keep_open {
                    self.dialog = Some(EditorDialog::ConfirmTypeChange {
                        draft,
                        staged: staged.expect("the staged inspector save remains available"),
                        lens_title,
                    });
                }
            }
            Some(EditorDialog::RenameVersion {
                version_id,
                mut draft,
            }) => {
                let mut keep_open = true;
                egui::Window::new("Rename version")
                    .collapsible(false)
                    .resizable(false)
                    .default_width(420.0)
                    .show(ctx, |ui| {
                        ui.label("Version name");
                        ui.add(
                            egui::TextEdit::singleline(&mut draft.title)
                                .desired_width(ui.available_width()),
                        );
                        let title_is_empty = draft.title.trim().is_empty();
                        if title_is_empty {
                            ui.colored_label(ATLAS_RECOVERY_RED, "Enter a version name.");
                        }
                        ui.add_space(12.0);
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui.button("Cancel").clicked() {
                                keep_open = false;
                            }
                            if ui
                                .add_enabled(!title_is_empty, egui::Button::new("Save name"))
                                .clicked()
                            {
                                self.rename_version(
                                    version_id.clone(),
                                    draft.title.trim().to_string(),
                                );
                                keep_open = false;
                            }
                        });
                    });
                if keep_open {
                    self.dialog = Some(EditorDialog::RenameVersion { version_id, draft });
                }
            }
            Some(EditorDialog::RenameLens { lens_id, mut draft }) => {
                let mut keep_open = true;
                egui::Window::new("Rename view")
                    .collapsible(false)
                    .resizable(false)
                    .default_width(420.0)
                    .show(ctx, |ui| {
                        ui.label("View name");
                        ui.add(
                            egui::TextEdit::singleline(&mut draft.title)
                                .desired_width(ui.available_width()),
                        );
                        let title_is_empty = draft.title.trim().is_empty();
                        if title_is_empty {
                            ui.colored_label(ATLAS_RECOVERY_RED, "Enter a view name.");
                        }
                        ui.add_space(12.0);
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui.button("Cancel").clicked() {
                                keep_open = false;
                            }
                            if ui
                                .add_enabled(!title_is_empty, egui::Button::new("Save name"))
                                .clicked()
                            {
                                self.rename_lens(lens_id.clone(), draft.title.trim().to_string());
                                keep_open = false;
                            }
                        });
                    });
                if keep_open {
                    self.dialog = Some(EditorDialog::RenameLens { lens_id, draft });
                }
            }
            Some(EditorDialog::ChooseRelationshipKind {
                source_id,
                target_id,
                position,
            }) => {
                let mut keep_open = true;
                egui::Window::new("Relationship type")
                    .id(egui::Id::unique("choose-relationship-kind"))
                    .collapsible(false)
                    .resizable(false)
                    .fixed_pos(position)
                    .show(ctx, |ui| {
                        for kind in self.relationship_kinds_for_endpoints(source_id, target_id) {
                            if ui.button(kind.label()).clicked() {
                                self.create_relationship(source_id, target_id, kind);
                                keep_open = false;
                            }
                        }
                        ui.separator();
                        if ui.button("Cancel").clicked() {
                            keep_open = false;
                        }
                    });
                if keep_open {
                    self.dialog = Some(EditorDialog::ChooseRelationshipKind {
                        source_id,
                        target_id,
                        position,
                    });
                }
            }
            Some(EditorDialog::ConfirmRelationshipRemoval { relationship }) => {
                let mut keep_open = true;
                egui::Window::new("Remove relationship")
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.label("This removal is saved only after confirmation.");
                        ui.label(relationship_description(&relationship));
                        ui.horizontal(|ui| {
                            if ui.button("Remove and save").clicked() {
                                self.remove_relationship(relationship.id.clone());
                                keep_open = false;
                            }
                            if ui.button("Cancel").clicked() {
                                keep_open = false;
                            }
                        });
                    });
                if keep_open {
                    self.dialog = Some(EditorDialog::ConfirmRelationshipRemoval { relationship });
                }
            }
            None => {}
        }
    }

    fn concept_label(&self, concept_id: u32) -> String {
        self.session()
            .and_then(|session| {
                session
                    .document()
                    .concept(session.active_version_id(), concept_id)
                    .ok()
                    .map(|concept| concept.title.clone())
            })
            .map_or_else(
                || format!("Concept #{concept_id}"),
                |title| format!("{title} (#{concept_id})"),
            )
    }
}

impl eframe::App for InfiniteNoteCanvasApp {
    #[cfg(target_arch = "wasm32")]
    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        let preferences = self.stored_preferences();
        eframe::set_value(storage, PREFERENCES_STORAGE_KEY, &preferences);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.ui_time_seconds = ui.input(|input| input.time);
        #[cfg(not(target_arch = "wasm32"))]
        self.refresh_external_workspace(ui.ctx());
        self.process_native_menu_request();
        self.settings_window(ui.ctx());
        if ui.input(|input| input.key_pressed(egui::Key::Escape)) {
            self.dialog = None;
            if self.side_pane_mode != SidePaneMode::Inspect {
                self.side_pane_mode = SidePaneMode::Inspect;
            }
        }
        if ui.input(|input| input.key_pressed(egui::Key::Tab)) {
            self.cycle_lens();
        }
        if should_delete_canvas_selection(
            self.dialog.is_some(),
            ui.ctx().text_edit_focused(),
            ui.input(|input| input.key_pressed(egui::Key::Delete)),
            ui.input(|input| input.key_pressed(egui::Key::Backspace)),
        ) {
            self.delete_canvas_selection();
        }
        // Run these before any pane widgets process this frame's key events.
        // A selected canvas card therefore receives Cmd+C / Cmd+V directly
        // after the user returns focus to the canvas.
        self.handle_card_clipboard_shortcuts(ui.ctx());
        let content_rect = ui.available_rect_before_wrap();
        ui.scope_builder(
            egui::UiBuilder::new()
                .max_rect(content_rect)
                .layout(Layout::top_down(Align::Min)),
            |ui| {
                egui::Frame::new().fill(ATLAS_SURVEY_SLATE).show(ui, |ui| {
                    ui.vertical(|ui| {
                        self.top_bar(ui);
                        ui.separator();
                        let workspace_rect = ui.available_rect_before_wrap();
                        let inspector_is_visible =
                            !self.inspector_collapsed || !self.inspector_has_no_selection();
                        let (canvas_rect, inspector_rect) = canvas_and_inspector_rects(
                            workspace_rect,
                            inspector_is_visible,
                            inspector_panel_width(workspace_rect.width()),
                        );
                        if let Some(inspector_rect) = inspector_rect {
                            ui.painter()
                                .rect_filled(inspector_rect, 0.0, ATLAS_SURVEY_SLATE);
                            ui.painter().line_segment(
                                [
                                    Pos2::new(inspector_rect.left(), inspector_rect.top()),
                                    Pos2::new(inspector_rect.left(), inspector_rect.bottom()),
                                ],
                                Stroke::new(1.0, ATLAS_QUIET_GRAPHITE),
                            );
                        }
                        ui.scope_builder(
                            egui::UiBuilder::new()
                                .max_rect(canvas_rect)
                                .layout(Layout::top_down(Align::Min)),
                            |ui| {
                                ui.shrink_clip_rect(canvas_rect);
                                self.canvas(ui);
                            },
                        );
                        if let Some(inspector_rect) = inspector_rect {
                            ui.scope_builder(
                                egui::UiBuilder::new()
                                    .max_rect(inspector_rect)
                                    .layout(Layout::top_down(Align::Min)),
                                |ui| {
                                    ui.shrink_clip_rect(inspector_rect);
                                    self.inspector(ui);
                                },
                            );
                        }
                    });
                });
            },
        );
        self.dialogs(ui.ctx());
        self.handle_history_shortcuts(ui.ctx());
        self.publish_pending_card_clipboard(ui.ctx());
        self.toasts(ui.ctx());
    }
}

/// Includes authored layer scaling and the minimum painted shell at this zoom.
/// Hidden placements use their base layer, so temporary filtering cannot hide collisions.
fn paste_card_visual_bounds(
    document: &WorkspaceDocument,
    version_id: &str,
    lens_id: &str,
    camera_scale: f32,
    show_all: bool,
) -> BTreeMap<u32, CanvasRect> {
    let Some(lens) = document
        .version(version_id)
        .ok()
        .and_then(|version| version.lenses.iter().find(|lens| lens.id == lens_id))
    else {
        return BTreeMap::new();
    };
    let frame_bounds = if show_all {
        BTreeMap::new()
    } else {
        document
            .semantic_zoom_frame(version_id, lens_id, camera_scale)
            .map(|frame| {
                frame
                    .cards
                    .into_iter()
                    .map(|card| (card.card.id, card.card.bounds))
                    .collect()
            })
            .unwrap_or_default()
    };
    lens.placements
        .iter()
        .map(|placement| {
            let bounds = frame_bounds
                .get(&placement.concept_id)
                .copied()
                .unwrap_or_else(|| {
                    let scale = if show_all {
                        1.0
                    } else {
                        document
                            .semantic_zoom_card_layer_visibility(
                                version_id,
                                lens_id,
                                placement.concept_id,
                            )
                            .ok()
                            .and_then(|visibility| visibility.base_layer_id)
                            .and_then(|id| {
                                lens.semantic_zoom_layers
                                    .iter()
                                    .find(|layer| layer.id == id)
                            })
                            .map(|layer| layer.card_scale)
                            .unwrap_or(1.0)
                    };
                    let size = placement.bounds.size * scale;
                    CanvasRect::new(
                        placement.bounds.origin + (placement.bounds.size - size) * 0.5,
                        size,
                    )
                });
            let minimum = MINIMUM_VISIBLE_CARD_SIZE / camera_scale;
            let size = Point::new(bounds.size.x.max(minimum), bounds.size.y.max(minimum));
            let border = SELECTED_CARD_STROKE_WIDTH / camera_scale;
            (
                placement.concept_id,
                CanvasRect::new(
                    bounds.origin + (bounds.size - size) * 0.5 - Point::new(border, border),
                    size + Point::new(border * 2.0, border * 2.0),
                ),
            )
        })
        .collect()
}

/// Anchor the copied group at the cursor. If that would overlap an existing card,
/// shift the group just beyond the blocking card; never create accidental nesting.
fn card_paste_delta(source: &[CanvasRect], occupied: &[CanvasRect], anchor: Point) -> Point {
    let Some(first) = source.first() else {
        return Point::default();
    };
    let origin = source.iter().fold(first.origin, |point, bounds| {
        Point::new(point.x.min(bounds.origin.x), point.y.min(bounds.origin.y))
    });
    let mut delta = anchor - origin;
    loop {
        let mut next_x = delta.x;
        for bounds in source {
            let pasted = CanvasRect::new(bounds.origin + delta, bounds.size);
            for obstacle in occupied {
                if pasted.intersects(*obstacle) {
                    next_x = next_x.max(
                        obstacle.origin.x + obstacle.size.x + CARD_PASTE_OFFSET - bounds.origin.x,
                    );
                }
            }
        }
        if next_x == delta.x {
            return delta;
        }
        delta.x = next_x;
    }
}

/// Text editors always retain normal system copy/paste, including with a selected card.
fn card_clipboard_shortcuts_take_priority(
    dialog_is_open: bool,
    text_edit_is_focused: bool,
    _has_selected_cards: bool,
) -> bool {
    !dialog_is_open && !text_edit_is_focused
}

/// Native desktop integrations report Command+C and Command+V as semantic
/// Copy and Paste events. Retain the key shortcut fallback for integrations
/// that report physical key events instead. Removing the semantic event keeps
/// a focused text editor from also handling a selected-card command.
fn consume_card_clipboard_command(ctx: &egui::Context) -> Option<CardClipboardCommand> {
    ctx.input_mut(|input| {
        if let Some(command) = consume_card_clipboard_event(&mut input.events) {
            return Some(command);
        }

        let copy_shortcut = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::C);
        if input.consume_shortcut(&copy_shortcut) {
            return Some(CardClipboardCommand::Copy);
        }
        let paste_shortcut = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::V);
        input
            .consume_shortcut(&paste_shortcut)
            .then_some(CardClipboardCommand::Paste)
    })
}

fn consume_card_clipboard_event(events: &mut Vec<egui::Event>) -> Option<CardClipboardCommand> {
    let event_index = events
        .iter()
        .position(|event| matches!(event, egui::Event::Copy | egui::Event::Paste(_)))?;
    Some(match events.remove(event_index) {
        egui::Event::Copy => CardClipboardCommand::Copy,
        egui::Event::Paste(_) => CardClipboardCommand::Paste,
        _ => unreachable!("clipboard event position only selects copy or paste"),
    })
}

fn workspace_id_from_canvas_title(title: &str) -> String {
    let mut identifier = String::new();
    let mut previous_was_separator = false;
    for character in title.chars() {
        if character.is_ascii_alphanumeric() {
            identifier.push(character.to_ascii_lowercase());
            previous_was_separator = false;
        } else if !identifier.is_empty() && !previous_was_separator {
            identifier.push('-');
            previous_was_separator = true;
        }
    }
    if previous_was_separator {
        identifier.pop();
    }
    if identifier.is_empty() {
        "canvas".to_string()
    } else {
        identifier
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn default_new_canvas_directory() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        let home_directory = env::var_os("HOME")
            .map(PathBuf::from)
            .or_else(|| env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."));
        home_directory
            .join("Library")
            .join("Application Support")
            .join("Infinite Note Canvas")
            .join("Canvases")
    }
    #[cfg(target_os = "windows")]
    {
        let application_data = env::var_os("APPDATA")
            .map(PathBuf::from)
            .or_else(|| env::var_os("HOME").map(PathBuf::from))
            .or_else(|| env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."));
        application_data
            .join("Infinite Note Canvas")
            .join("Canvases")
    }
    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    {
        let home_directory = env::var_os("HOME")
            .map(PathBuf::from)
            .or_else(|| env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."));
        home_directory
            .join(".local")
            .join("share")
            .join("Infinite Note Canvas")
            .join("Canvases")
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn suggested_new_canvas_path(title: &str) -> PathBuf {
    suggested_new_canvas_path_in(&default_new_canvas_directory(), title)
}

#[cfg(not(target_arch = "wasm32"))]
fn suggested_new_canvas_path_in(directory: &Path, title: &str) -> PathBuf {
    let file_stem = workspace_id_from_canvas_title(title);
    let first_candidate = directory.join(format!("{file_stem}{NEW_CANVAS_FILE_SUFFIX}"));
    if !first_candidate.exists() {
        return first_candidate;
    }

    let mut sequence = 2_usize;
    loop {
        let candidate = directory.join(format!("{file_stem}-{sequence}{NEW_CANVAS_FILE_SUFFIX}"));
        if !candidate.exists() {
            return candidate;
        }
        sequence += 1;
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RelationshipView {
    id: String,
    source_id: u32,
    target_id: u32,
    kind: RelationshipKind,
    source_title: String,
    target_title: String,
}

impl RelationshipView {
    fn source_label(&self) -> String {
        format!("{} (#{})", self.source_title, self.source_id)
    }

    fn target_label(&self) -> String {
        format!("{} (#{})", self.target_title, self.target_id)
    }
}

/// Keeps input geometry aligned with semantic-zoom painting. A relationship
/// can have two context-visible endpoint cards while its own presentation rule
/// is inactive, so endpoint visibility is not sufficient to make its line
/// interactive.
fn relationship_views_in_semantic_frame(
    relationships: &[RelationshipView],
    frame: &SemanticZoomFrame,
) -> Vec<RelationshipView> {
    let rendered: BTreeMap<_, _> = frame
        .relationships
        .iter()
        .map(|relationship| (relationship.id.as_str(), relationship))
        .collect();
    relationships
        .iter()
        .filter_map(|relationship| {
            let projected = rendered.get(relationship.id.as_str())?;
            let mut view = relationship.clone();
            view.source_id = projected.source_id;
            view.target_id = projected.target_id;
            Some(view)
        })
        .collect()
}

fn relationship_description(relationship: &RelationshipView) -> String {
    format!(
        "{} --{}--> {}",
        relationship.source_label(),
        relationship.kind.label(),
        relationship.target_label()
    )
}

fn card_layer_mode_selected_ids(
    mode: &SemanticZoomCardLayerMode,
    layers: &[SemanticZoomLayerChoice],
    visibility: &Option<SemanticZoomCardLayerVisibility>,
) -> Vec<String> {
    match mode {
        SemanticZoomCardLayerMode::Automatic => visibility
            .as_ref()
            .map(|value| value.visible_layer_ids.clone())
            .unwrap_or_default(),
        SemanticZoomCardLayerMode::Manual { layer_ids } => layer_ids.clone(),
        SemanticZoomCardLayerMode::Dedicated { layer_id } => vec![layer_id.clone()],
        SemanticZoomCardLayerMode::AlwaysVisible => {
            layers.iter().map(|layer| layer.id.clone()).collect()
        }
    }
}

fn type_change_diagnostic_path(workspace_path: &Path) -> PathBuf {
    workspace_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(TYPE_CHANGE_DIAGNOSTIC_FILE_NAME)
}

#[cfg(not(target_arch = "wasm32"))]
fn startup_diagnostic_path(workspace_path: Option<PathBuf>) -> PathBuf {
    workspace_path
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."))
        .join(STARTUP_DIAGNOSTIC_FILE_NAME)
}

#[cfg(not(target_arch = "wasm32"))]
fn format_startup_failure_diagnostic(diagnostic_path: &Path, error: &str) -> String {
    format!(
        "event=eframe_startup_failure\ndiagnostic_path={}\nerror={error}\naction=Run ./start.sh with a writable external workspace and review this log.\n\n",
        diagnostic_path.display()
    )
}

fn format_type_change_failure_diagnostic(
    diagnostic_path: &Path,
    concept_id: u32,
    type_id: &str,
    error: &str,
) -> String {
    format!(
        "event=type_change_failure\ndiagnostic_path={}\nconcept_id={concept_id}\ntarget_type_id={type_id}\nerror={error}\naction=Review this log, correct the local workspace problem, and try the type change again.\n\n",
        diagnostic_path.display()
    )
}

fn inspector_save_failure_route(
    current_type_id: &str,
    requested_type_id: &str,
) -> InspectorSaveFailureRoute {
    if current_type_id == requested_type_id {
        InspectorSaveFailureRoute::OrdinaryNotice
    } else {
        InspectorSaveFailureRoute::TypeChangeToast
    }
}

fn append_diagnostic(path: &Path, contents: &str) -> std::io::Result<()> {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (path, contents);
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "browser sessions do not write local diagnostic logs",
        ))
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut file = OpenOptions::new().create(true).append(true).open(path)?;
        file.write_all(contents.as_bytes())?;
        file.sync_data()
    }
}

fn color_with_opacity(color: Color32, opacity: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(
        color.r(),
        color.g(),
        color.b(),
        (f32::from(color.a()) * opacity.clamp(0.0, 1.0)).round() as u8,
    )
}

fn semantic_zoom_dataset(frame: &SemanticZoomFrame) -> CanvasDataset {
    CanvasDataset::from_records(
        frame.cards.iter().map(|card| card.card.clone()).collect(),
        frame
            .relationships
            .iter()
            .map(|relationship| Relationship {
                source_id: relationship.source_id,
                target_id: relationship.target_id,
            })
            .collect(),
    )
}

fn canvas_card_matches_search(card: &ConceptCard, query: &str) -> bool {
    let lowercase_query = query.to_lowercase();
    card.title.to_lowercase().contains(&lowercase_query)
        || card.summary.to_lowercase().contains(&lowercase_query)
}

/// Advances a search result deterministically. If both requests arrive in one
/// frame, their effects cancel; this keeps keyboard and button input stable.
fn search_match_index_after_navigation(
    current_index: usize,
    match_count: usize,
    previous_requested: bool,
    next_requested: bool,
) -> usize {
    if match_count == 0 {
        return 0;
    }
    let current_index = current_index % match_count;
    match (previous_requested, next_requested) {
        (true, false) => (current_index + match_count - 1) % match_count,
        (false, true) => (current_index + 1) % match_count,
        _ => current_index,
    }
}

/// Centers a search result at a useful inspection scale. The minimum prevents
/// a result from remaining a dot after searching from an overview; a closer
/// manual zoom is respected.
fn camera_focused_on_card(
    current: Camera,
    card_bounds: CanvasRect,
    viewport_size: Point,
) -> Camera {
    let mut focused = Camera::new(current.offset, current.scale.max(0.8));
    focused.center_world_point_in_viewport(card_bounds.center(), viewport_size);
    focused
}

fn filtered_card_hit_test(
    dataset: &CanvasDataset,
    world_position: Point,
    include: impl Fn(&ConceptCard) -> bool,
) -> Option<&ConceptCard> {
    let candidates: Vec<_> = dataset
        .cards()
        .filter(|card| include(card) && card.bounds.contains_point(world_position))
        .collect();
    let parent_by_child = card_parent_ids(candidates.iter().map(|card| (card.id, card.bounds)));
    candidates.into_iter().max_by(|left, right| {
        filtered_card_containment_depth(left.id, &parent_by_child)
            .cmp(&filtered_card_containment_depth(right.id, &parent_by_child))
            .then(left.id.cmp(&right.id))
    })
}

fn filtered_card_containment_depth(card_id: u32, parent_by_child: &BTreeMap<u32, u32>) -> usize {
    let mut depth = 0;
    let mut current_id = card_id;
    while let Some(parent_id) = parent_by_child.get(&current_id) {
        depth += 1;
        current_id = *parent_id;
    }
    depth
}

fn semantic_zoom_layer_editor_draft(layer: &SemanticZoomLayer) -> SemanticZoomLayerEditorDraft {
    let percent_settings = layer.percent_settings.unwrap_or_else(|| {
        // Documents are migrated when loaded. This fallback keeps an
        // in-memory legacy document understandable while it is being edited.
        SemanticZoomLayerPercentSettings {
            active_zoom_range: ZoomPercentRange {
                start_percent: layer.fade_in.map_or(1.0, |range| range.end * 100.0),
                end_percent: layer
                    .fade_out
                    .map_or(409_600.0, |range| range.start * 100.0),
            },
            fade_percent: [layer.fade_in, layer.fade_out]
                .into_iter()
                .flatten()
                .map(|range| (range.end - range.start).abs() * 100.0)
                .fold(0.0, f32::max),
        }
    });
    SemanticZoomLayerEditorDraft {
        layer_id: layer.id.clone(),
        title: layer.title.clone(),
        active_zoom_start_percent: percent_settings.active_zoom_range.start_percent,
        active_zoom_end_percent: percent_settings.active_zoom_range.end_percent,
        fade_percent: percent_settings.fade_percent,
        card_scale: layer.card_scale,
    }
}

fn semantic_zoom_layer_range_draft(layer: &SemanticZoomLayer) -> LayerRangeDraft {
    let editor = semantic_zoom_layer_editor_draft(layer);
    LayerRangeDraft {
        layer_id: editor.layer_id,
        title: editor.title,
        active_zoom_start_percent: editor.active_zoom_start_percent,
        active_zoom_end_percent: editor.active_zoom_end_percent,
        fade_percent: editor.fade_percent,
        card_scale: editor.card_scale,
    }
}

fn layer_cross_section_domain_end(drafts: &[LayerRangeDraft], current_zoom_percent: f32) -> f32 {
    drafts
        .iter()
        .map(|draft| draft.active_zoom_end_percent + draft.fade_percent)
        .chain(std::iter::once(current_zoom_percent))
        .fold(200.0_f32, f32::max)
        .clamp(200.0, 409_600.0)
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct LayerCrossSectionTrack {
    fade_start: f32,
    active_start: f32,
    active_end: f32,
    fade_end: f32,
}

fn layer_cross_section_track(
    active_start_percent: f32,
    active_end_percent: f32,
    fade_percent: f32,
    domain_end_percent: f32,
) -> LayerCrossSectionTrack {
    let domain_end_percent = domain_end_percent.max(1.0);
    let active_start = active_start_percent
        .min(active_end_percent)
        .clamp(0.0, domain_end_percent);
    let active_end = active_start_percent
        .max(active_end_percent)
        .clamp(0.0, domain_end_percent);
    let fade = fade_percent.max(0.0);
    LayerCrossSectionTrack {
        fade_start: (active_start - fade).max(0.0) / domain_end_percent,
        active_start: active_start / domain_end_percent,
        active_end: active_end / domain_end_percent,
        fade_end: (active_end + fade).min(domain_end_percent) / domain_end_percent,
    }
}

fn paint_layer_cross_section_master_track(painter: &Painter, bounds: Rect) {
    painter.rect_filled(bounds, 3.0, color_with_opacity(ATLAS_GROUND, 0.8));
    painter.line_segment(
        [bounds.left_center(), bounds.right_center()],
        Stroke::new(2.0, color_with_opacity(ATLAS_QUIET_GRAPHITE, 0.8)),
    );
    painter.circle_filled(bounds.left_center(), 3.0, ATLAS_PAPER_SIGNAL);
    painter.circle_filled(bounds.right_center(), 3.0, ATLAS_PAPER_SIGNAL);
}

fn paint_layer_cross_section_track(
    painter: &Painter,
    bounds: Rect,
    active_start_percent: f32,
    active_end_percent: f32,
    fade_percent: f32,
    domain_end_percent: f32,
    active: bool,
) {
    let track = layer_cross_section_track(
        active_start_percent,
        active_end_percent,
        fade_percent,
        domain_end_percent,
    );
    painter.rect_filled(bounds, 3.0, color_with_opacity(ATLAS_GROUND, 0.86));
    let x_at = |fraction: f32| bounds.left() + bounds.width() * fraction;
    let fade_bounds = Rect::from_x_y_ranges(
        x_at(track.fade_start)..=x_at(track.fade_end),
        bounds.top()..=bounds.bottom(),
    );
    painter.rect_filled(
        fade_bounds,
        3.0,
        color_with_opacity(ATLAS_CONTROL_AMBER, if active { 0.32 } else { 0.2 }),
    );
    let active_bounds = Rect::from_x_y_ranges(
        x_at(track.active_start)..=x_at(track.active_end),
        bounds.top()..=bounds.bottom(),
    );
    painter.rect_filled(
        active_bounds,
        3.0,
        color_with_opacity(ATLAS_ROUTE_BLUE, if active { 0.84 } else { 0.62 }),
    );
    painter.rect_stroke(
        active_bounds,
        3.0,
        Stroke::new(
            if active { 1.5 } else { 1.0 },
            if active {
                ATLAS_PAPER_SIGNAL
            } else {
                ATLAS_ROUTE_BLUE
            },
        ),
        StrokeKind::Inside,
    );
}

fn semantic_zoom_layer_creation_title(existing_layer_count: usize) -> String {
    match existing_layer_count {
        0 => "Overview".to_string(),
        1 => "Working detail".to_string(),
        count => format!("Detail layer {count}"),
    }
}

fn semantic_zoom_layer_creation_percent_settings(
    existing_layer_count: usize,
) -> SemanticZoomLayerPercentSettings {
    let active_zoom_range = match existing_layer_count {
        0 => ZoomPercentRange {
            start_percent: 1.0,
            end_percent: 65.0,
        },
        1 => ZoomPercentRange {
            start_percent: 66.0,
            end_percent: 150.0,
        },
        _ => ZoomPercentRange {
            start_percent: 151.0,
            end_percent: 409_600.0,
        },
    };
    SemanticZoomLayerPercentSettings {
        active_zoom_range,
        fade_percent: 10.0,
    }
}

fn staged_semantic_zoom_layer_id_at(
    staged: &StagedWorkspaceMutation,
    index: usize,
) -> Option<String> {
    staged
        .document()
        .version(staged.active_version_id())
        .ok()?
        .lenses
        .iter()
        .find(|lens| lens.id == staged.active_lens_id())?
        .semantic_zoom_layers
        .get(index)
        .map(|layer| layer.id.clone())
}

fn consume_history_shortcut(ctx: &egui::Context) -> Option<HistoryCommand> {
    ctx.input_mut(|input| {
        let redo_shortcut = egui::KeyboardShortcut::new(
            egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
            egui::Key::Z,
        );
        if input.consume_shortcut(&redo_shortcut) {
            return Some(HistoryCommand::Redo);
        }

        let undo_shortcut = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::Z);
        input
            .consume_shortcut(&undo_shortcut)
            .then_some(HistoryCommand::Undo)
    })
}

fn consume_command_enter_for_focused_input(ctx: &egui::Context, input_has_focus: bool) -> bool {
    input_has_focus && {
        ctx.input_mut(|input| {
            input.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::Enter,
            ))
        })
    }
}

fn focused_input_draft_save_target(
    title_has_focus: bool,
    summary_has_focus: bool,
    details_has_focus: bool,
    assertion_has_focus: bool,
) -> bool {
    title_has_focus || summary_has_focus || details_has_focus || assertion_has_focus
}

fn history_command_label(command: HistoryCommand) -> &'static str {
    match command {
        HistoryCommand::Undo => "Undo",
        HistoryCommand::Redo => "Redo",
    }
}

fn history_command_route(
    command: HistoryCommand,
    has_unsaved_draft: bool,
    dialog_open: bool,
) -> HistoryCommandRoute {
    if has_unsaved_draft || dialog_open {
        HistoryCommandRoute::Consume
    } else {
        HistoryCommandRoute::Apply(command)
    }
}

fn inspector_draft_has_unsaved_changes(
    draft: &ConceptInspectorDraft,
    concept_id: u32,
    title: &str,
    type_id: &str,
    summary: &str,
    details: &str,
    assertion_attributes: &BTreeMap<String, String>,
) -> bool {
    draft.concept_id != concept_id
        || draft.title != title
        || draft.type_id != type_id
        || draft.summary != summary
        || draft.details != details
        || draft.assertion_attributes != *assertion_attributes
        || !draft.assertion_key.is_empty()
        || !draft.assertion_value.is_empty()
}

fn inspector_autosave_is_safe(draft: &ConceptInspectorDraft) -> bool {
    draft.assertion_key.is_empty()
        && draft.assertion_value.is_empty()
        && draft.assertion_error.is_none()
}

fn concept_inspector_header(ui: &mut egui::Ui) -> (egui::Response, egui::Response) {
    let save_button_size = concept_inspector_save_button_size(ui);
    let header_size = Vec2::new(ui.available_width(), save_button_size.y);
    ui.allocate_ui_with_layout(header_size, Layout::left_to_right(Align::Center), |ui| {
        let header_gap = ui.spacing().item_spacing.x + CONCEPT_INSPECTOR_HEADER_GAP;
        let heading_width = concept_inspector_heading_width(ui)
            .min((ui.available_width() - save_button_size.x - header_gap).max(0.0));
        let heading = ui.add_sized(
            Vec2::new(heading_width, save_button_size.y),
            egui::Label::new(
                egui::RichText::new("Concept inspector")
                    .size(ATLAS_INSPECTOR_TITLE_SIZE)
                    .strong()
                    .color(ATLAS_PAPER_SIGNAL),
            )
            .truncate(),
        );
        ui.add_space(CONCEPT_INSPECTOR_HEADER_GAP);
        let save_button = ui.add_sized(
            save_button_size,
            primary_button(CONCEPT_INSPECTOR_SAVE_LABEL),
        );
        (heading, save_button)
    })
    .inner
}

fn concept_inspector_heading_width(ui: &egui::Ui) -> f32 {
    let font_id = egui::TextStyle::Heading.resolve(ui.style());
    ui.painter()
        .layout_no_wrap(
            "Concept inspector".to_string(),
            font_id,
            ui.visuals().text_color(),
        )
        .size()
        .x
}

fn concept_inspector_save_button_size(ui: &egui::Ui) -> Vec2 {
    let font_id = egui::TextStyle::Button.resolve(ui.style());
    let text_size = ui
        .painter()
        .layout_no_wrap(
            CONCEPT_INSPECTOR_SAVE_LABEL.to_string(),
            font_id,
            ui.visuals().text_color(),
        )
        .size();
    let button_padding = ui.spacing().button_padding;
    Vec2::new(
        text_size.x + 2.0 * button_padding.x,
        text_size.y + 2.0 * button_padding.y,
    )
    .max(ui.spacing().interact_size)
}

fn history_restore_selection(
    saved_selection_ids: &[u32],
    selected_concept_id: Option<u32>,
    dataset: &CanvasDataset,
) -> Option<u32> {
    saved_selection_ids
        .iter()
        .copied()
        .find(|concept_id| dataset.card_by_id(*concept_id).is_some())
        .or_else(|| {
            selected_concept_id.filter(|concept_id| dataset.card_by_id(*concept_id).is_some())
        })
}

fn relationship_kinds() -> [RelationshipKind; 7] {
    [
        RelationshipKind::Contains,
        RelationshipKind::Owns,
        RelationshipKind::Operates,
        RelationshipKind::DependsOn,
        RelationshipKind::ImplementedIn,
        RelationshipKind::Delivers,
        RelationshipKind::DeployedTo,
    ]
}

fn supported_type_ids(session: &WorkspaceSession) -> Vec<String> {
    let mut type_ids: Vec<_> = ConceptKind::ALL
        .into_iter()
        .map(|kind| kind.type_id().to_string())
        .collect();
    type_ids.extend(
        session
            .document()
            .custom_types
            .iter()
            .map(|custom_type| custom_type.id.clone()),
    );
    type_ids.sort();
    type_ids.dedup();
    type_ids
}

fn toggle_created_card_type(selected_type_id: Option<String>, type_id: &str) -> Option<String> {
    (selected_type_id.as_deref() != Some(type_id)).then(|| type_id.to_string())
}

fn created_card_type_id(selected_type_id: Option<&str>, default_type_id: Option<&str>) -> String {
    selected_type_id
        .or(default_type_id)
        .unwrap_or(ConceptKind::Project.type_id())
        .to_string()
}

fn new_concept_size_from_preferences(preferences: &StoredPreferences) -> Point {
    Point::new(
        preferences.default_card_width.max(MINIMUM_CARD_SIZE.x),
        preferences.default_card_height.max(MINIMUM_CARD_SIZE.y),
    )
}

/// Preserves the user's preferred on-screen card dimensions at the current
/// camera scale and active layer scale. The minimum keeps its interaction
/// affordances on screen without becoming a huge canvas-space card.
fn new_concept_size_for_active_layer(
    preferences: &StoredPreferences,
    camera_scale: f32,
    layer_card_scale: f32,
) -> Point {
    let visible_scale = (camera_scale * layer_card_scale).max(f32::EPSILON);
    let preferred = new_concept_size_from_preferences(preferences);
    Point::new(
        (preferred.x / visible_scale).max(minimum_card_size_at_screen_scale(visible_scale).x),
        (preferred.y / visible_scale).max(minimum_card_size_at_screen_scale(visible_scale).y),
    )
}

fn new_boundary_size_from_preferences(preferences: &StoredPreferences) -> Point {
    Point::new(
        preferences
            .default_boundary_width
            .max(MINIMUM_BOUNDARY_SIZE.x),
        preferences
            .default_boundary_height
            .max(MINIMUM_BOUNDARY_SIZE.y),
    )
}

fn next_choice(choices: &[String], current: &str) -> Option<String> {
    let current_index = choices.iter().position(|choice| choice == current)?;
    choices.get((current_index + 1) % choices.len()).cloned()
}

fn inspector_draft_for<'draft>(
    draft: &'draft mut Option<ConceptInspectorDraft>,
    concept_id: u32,
    title: &str,
    type_id: &str,
    summary: &str,
    details: &str,
    assertion_attributes: &BTreeMap<String, String>,
) -> &'draft mut ConceptInspectorDraft {
    let should_replace = !matches!(draft, Some(draft) if draft.concept_id == concept_id);
    if should_replace {
        *draft = Some(ConceptInspectorDraft {
            concept_id,
            title: title.to_string(),
            type_id: type_id.to_string(),
            summary: summary.to_string(),
            details: details.to_string(),
            assertion_attributes: assertion_attributes.clone(),
            assertion_key: String::new(),
            assertion_value: String::new(),
            assertion_error: None,
        });
    }
    draft.as_mut().expect("an inspector draft was initialized")
}

fn assertion_save_label(
    assertion_attributes: &BTreeMap<String, String>,
    key: &str,
) -> &'static str {
    if assertion_attributes.contains_key(key) {
        "Replace assertion"
    } else {
        "Add assertion"
    }
}

fn concept_inspector_edit_draft(draft: &ConceptInspectorDraft) -> ConceptInspectorEditDraft {
    ConceptInspectorEditDraft {
        concept_id: draft.concept_id,
        title: draft.title.clone(),
        type_id: draft.type_id.clone(),
        summary: draft.summary.clone(),
        details: draft.details.clone(),
        assertion_attributes: draft.assertion_attributes.clone(),
    }
}

fn inspector_save_validation_error(draft: &ConceptInspectorEditDraft) -> Option<&'static str> {
    if draft.title.trim().is_empty() {
        Some("A concept name cannot be empty.")
    } else {
        None
    }
}

fn load_assertion_into_draft(draft: &mut ConceptInspectorDraft, key: &str, value: &str) {
    draft.assertion_key = key.to_string();
    draft.assertion_value = value.to_string();
    draft.assertion_error = None;
}

fn draft_assertion_error(draft: &ConceptInspectorDraft) -> Option<&'static str> {
    assertion_key_error(&draft.assertion_key)
}

fn save_pending_draft_assertion(draft: &mut ConceptInspectorDraft) -> Result<(), &'static str> {
    if draft.assertion_key.is_empty() && draft.assertion_value.is_empty() {
        return Ok(());
    }
    if let Some(error) = draft_assertion_error(draft) {
        return Err(error);
    }
    save_draft_assertion(draft);
    Ok(())
}

fn save_draft_assertion(draft: &mut ConceptInspectorDraft) {
    draft
        .assertion_attributes
        .insert(draft.assertion_key.clone(), draft.assertion_value.clone());
    draft.assertion_error = None;
    draft.assertion_key.clear();
    draft.assertion_value.clear();
}

fn remove_draft_assertion(draft: &mut ConceptInspectorDraft, key: &str) {
    draft.assertion_attributes.remove(key);
    if draft.assertion_key == key {
        draft.assertion_key.clear();
        draft.assertion_value.clear();
        draft.assertion_error = None;
    }
}

fn revert_draft_assertions(
    draft: &mut ConceptInspectorDraft,
    assertion_attributes: &BTreeMap<String, String>,
) {
    draft.assertion_attributes.clone_from(assertion_attributes);
    draft.assertion_key.clear();
    draft.assertion_value.clear();
    draft.assertion_error = None;
}

fn assertion_key_error(key: &str) -> Option<&'static str> {
    if key.trim().is_empty() {
        Some("Assertion key must not be empty.")
    } else if key.chars().any(char::is_control) {
        Some("Assertion key must not contain control characters.")
    } else {
        None
    }
}

fn camera_for_lens_activation(
    saved_camera: Option<Camera>,
    selected_card: Option<&ConceptCard>,
    viewport_size: Point,
) -> Camera {
    if let Some(saved_camera) = saved_camera {
        return saved_camera;
    }

    let mut camera = Camera::default();
    if viewport_size.x > 0.0
        && viewport_size.y > 0.0
        && let Some(selected_card) = selected_card
    {
        let bounds = selected_card.bounds.normalized();
        let center = bounds.origin + bounds.size * 0.5;
        camera.center_world_point_in_viewport(center, viewport_size);
    }
    camera
}

fn type_change_requires_lens_confirmation(concept_is_visible: bool) -> bool {
    !concept_is_visible
}

fn wheel_canvas_command(shift: bool, viewport_position: Point, wheel_delta: Vec2) -> CanvasCommand {
    if shift {
        CanvasCommand::PanBy {
            viewport_delta: Point::new(wheel_delta.x, wheel_delta.y),
        }
    } else {
        CanvasCommand::ZoomAt {
            viewport_position,
            scale_factor: (-wheel_delta.y * WHEEL_ZOOM_SENSITIVITY).exp(),
        }
    }
}

fn shift_hover_pan_command(
    previous: Option<Point>,
    current: Point,
    shift_held: bool,
) -> Option<CanvasCommand> {
    shift_held
        .then_some(previous)
        .flatten()
        .map(|previous| CanvasCommand::PanBy {
            viewport_delta: current - previous,
        })
}

fn card_detail_level(scale: f32) -> SemanticZoomLevel {
    match semantic_zoom_level(scale) {
        SemanticZoomLevel::Overview => SemanticZoomLevel::Card,
        level => level,
    }
}

fn rendered_card_count(visible_card_count: usize, safe_rendered_card_budget: usize) -> usize {
    visible_card_count.min(safe_rendered_card_budget)
}

fn point_in_viewport(position: Pos2, viewport: Rect) -> Point {
    Point::new(position.x - viewport.left(), position.y - viewport.top())
}

fn inspector_panel_width(available_width: f32) -> f32 {
    INSPECTOR_PREFERRED_WIDTH.min(available_width * INSPECTOR_MAXIMUM_WIDTH_FRACTION)
}

/// Splits the remaining application area once, before either child paints.
/// This prevents a canvas painter from extending underneath the inspector when
/// a narrow desktop window constrains both regions.
fn canvas_and_inspector_rects(
    available_rect: Rect,
    inspector_is_visible: bool,
    preferred_inspector_width: f32,
) -> (Rect, Option<Rect>) {
    if !inspector_is_visible || available_rect.width() <= 1.0 {
        return (available_rect, None);
    }
    let divider_width = 1.0;
    let inspector_width = preferred_inspector_width
        .max(0.0)
        .min((available_rect.width() - divider_width).max(0.0));
    let inspector_left = available_rect.right() - inspector_width;
    let inspector_rect = Rect::from_min_max(
        Pos2::new(inspector_left, available_rect.top()),
        available_rect.max,
    );
    let canvas_rect = Rect::from_min_max(
        available_rect.min,
        Pos2::new(
            (inspector_left - divider_width).max(available_rect.left()),
            available_rect.bottom(),
        ),
    );
    (canvas_rect, Some(inspector_rect))
}

fn connection_arrowhead_for_path(points: &[Pos2]) -> Option<[Pos2; 3]> {
    let (tip, unit_direction, path_length) = relationship_path_midpoint(points)?;
    let arrow_length = (path_length * 0.2).clamp(2.0, 8.0).min(path_length * 0.48);
    let base = tip - unit_direction * arrow_length;
    let arrow_half_width = (arrow_length * 0.5).max(1.0);
    let normal = Vec2::new(-unit_direction.y, unit_direction.x) * arrow_half_width;

    Some([tip, base + normal, base - normal])
}

fn connection_label_position_for_path(points: &[Pos2]) -> Option<Pos2> {
    let (midpoint, unit_direction, _) = relationship_path_midpoint(points)?;
    let normal = Vec2::new(-unit_direction.y, unit_direction.x) * 12.0;
    Some(midpoint + normal)
}

fn relationship_path_midpoint(points: &[Pos2]) -> Option<(Pos2, Vec2, f32)> {
    let path_length = points
        .windows(2)
        .map(|segment| segment[0].distance(segment[1]))
        .sum::<f32>();
    if !path_length.is_finite() || path_length <= f32::EPSILON {
        return None;
    }

    let midpoint_distance = path_length * 0.5;
    let mut traversed_distance = 0.0;
    for segment in points.windows(2) {
        let direction = segment[1] - segment[0];
        let segment_length = direction.length();
        if segment_length <= f32::EPSILON {
            continue;
        }
        if traversed_distance + segment_length >= midpoint_distance {
            let unit_direction = direction / segment_length;
            let midpoint = segment[0] + unit_direction * (midpoint_distance - traversed_distance);
            return Some((midpoint, unit_direction, path_length));
        }
        traversed_distance += segment_length;
    }

    None
}

fn relationship_port_hit_radius(camera: Camera) -> f32 {
    (10.0 / camera.scale).max(MINIMUM_POINTER_TARGET_WORLD_SIZE)
}

fn relationship_port_positions(bounds: CanvasRect) -> [(CardEdge, Point); 4] {
    let bounds = bounds.normalized();
    let center = bounds.center();
    [
        (CardEdge::Top, Point::new(center.x, bounds.origin.y)),
        (CardEdge::Right, Point::new(bounds.max_x(), center.y)),
        (CardEdge::Bottom, Point::new(center.x, bounds.max_y())),
        (CardEdge::Left, Point::new(bounds.origin.x, center.y)),
    ]
}

fn relationship_port_at(
    dataset: &CanvasDataset,
    world_position: Point,
    hit_radius: f32,
    selected_card_ids: Option<&BTreeSet<u32>>,
) -> Option<(u32, CardEdge)> {
    let search_bounds = CanvasRect::new(
        Point::new(world_position.x - hit_radius, world_position.y - hit_radius),
        Point::new(hit_radius * 2.0, hit_radius * 2.0),
    );
    dataset
        .cards_intersecting(search_bounds)
        .into_iter()
        .filter(|card| selected_card_ids.is_none_or(|selected| selected.contains(&card.id)))
        .flat_map(|card| {
            relationship_port_positions(card.bounds)
                .into_iter()
                .enumerate()
                .filter_map(move |(edge_index, (edge, port))| {
                    let delta = world_position - port;
                    let distance_squared = delta.x * delta.x + delta.y * delta.y;
                    (distance_squared <= hit_radius * hit_radius).then_some((
                        distance_squared,
                        card.id,
                        edge_index,
                        edge,
                    ))
                })
        })
        .min_by(
            |(left_distance, left_id, left_edge, _), (right_distance, right_id, right_edge, _)| {
                left_distance
                    .total_cmp(right_distance)
                    .then(left_id.cmp(right_id))
                    .then(left_edge.cmp(right_edge))
            },
        )
        .map(|(_, card_id, _, edge)| (card_id, edge))
}

fn relationship_source_port_at(
    dataset: &CanvasDataset,
    world_position: Point,
    hit_radius: f32,
) -> Option<(u32, CardEdge)> {
    relationship_port_at(dataset, world_position, hit_radius, None)
}

fn relationship_input_port_at(
    dataset: &CanvasDataset,
    world_position: Point,
    hit_radius: f32,
) -> Option<u32> {
    relationship_port_at(dataset, world_position, hit_radius, None).map(|(card_id, _)| card_id)
}

fn relationship_port_toward(bounds: CanvasRect, target: Point) -> (CardEdge, Point) {
    let bounds = bounds.normalized();
    let delta = target - bounds.center();
    let edge = if delta.x.abs() >= delta.y.abs() {
        if delta.x >= 0.0 {
            CardEdge::Right
        } else {
            CardEdge::Left
        }
    } else if delta.y >= 0.0 {
        CardEdge::Bottom
    } else {
        CardEdge::Top
    };
    let point = relationship_port_positions(bounds)
        .into_iter()
        .find_map(|(candidate, point)| (candidate == edge).then_some(point))
        .expect("every cardinal edge has a relationship port");
    (edge, point)
}

#[cfg(test)]
fn relationship_edge_endpoints(
    source_bounds: CanvasRect,
    target_bounds: CanvasRect,
) -> ((CardEdge, Point), (CardEdge, Point)) {
    (
        relationship_port_toward(source_bounds, target_bounds.center()),
        relationship_port_toward(target_bounds, source_bounds.center()),
    )
}

/// Resolves a relationship target from a source card's output-port drag.
///
/// An input port takes precedence, but a card body is also a valid target so
/// people do not need pixel-perfect placement. A source card is never its own
/// target; releasing there simply cancels the drag.
fn relationship_drop_target(
    dataset: &CanvasDataset,
    source_id: u32,
    world_position: Point,
    hit_radius: f32,
) -> Option<u32> {
    relationship_input_port_at(dataset, world_position, hit_radius)
        .filter(|target_id| *target_id != source_id)
        .or_else(|| dataset.hit_test(world_position).map(|card| card.id))
        .filter(|target_id| *target_id != source_id)
}

fn primary_selected_concept_id(
    primary_concept_id: Option<u32>,
    selected_concept_ids: impl Iterator<Item = u32>,
) -> Option<u32> {
    let selected_concept_ids: BTreeSet<_> = selected_concept_ids.collect();
    primary_concept_id
        .filter(|concept_id| selected_concept_ids.contains(concept_id))
        .or_else(|| selected_concept_ids.first().copied())
}

fn canvas_delete_target(
    selected_cards: impl Iterator<Item = u32>,
    selected_boundary_id: Option<&str>,
    selected_relationship_id: Option<&str>,
) -> Option<CanvasDeleteTarget> {
    if let Some(relationship_id) = selected_relationship_id {
        return Some(CanvasDeleteTarget::Relationship(
            relationship_id.to_string(),
        ));
    }
    if let Some(boundary_id) = selected_boundary_id {
        return Some(CanvasDeleteTarget::Boundary(boundary_id.to_string()));
    }
    let selected_cards = selected_cards.collect::<BTreeSet<_>>();
    (!selected_cards.is_empty())
        .then(|| CanvasDeleteTarget::Cards(selected_cards.into_iter().collect()))
}

fn should_delete_canvas_selection(
    dialog_is_open: bool,
    text_edit_focused: bool,
    delete_pressed: bool,
    backspace_pressed: bool,
) -> bool {
    !dialog_is_open && !text_edit_focused && (delete_pressed || backspace_pressed)
}

fn move_world_delta(
    camera: Camera,
    anchor_world_position: Point,
    viewport_position: Point,
) -> Point {
    camera.screen_to_world(viewport_position) - anchor_world_position
}

fn move_gesture_concept_ids(
    dataset: &CanvasDataset,
    dragged_concept_id: u32,
    selected_concept_ids: &[u32],
) -> Vec<u32> {
    let root_concept_ids = if selected_concept_ids.contains(&dragged_concept_id) {
        selected_concept_ids.to_vec()
    } else {
        vec![dragged_concept_id]
    };
    card_move_ids_with_contained_descendants(
        dataset.cards().map(|card| (card.id, card.bounds)),
        root_concept_ids,
    )
}

fn move_commit_route(concept_ids: &[u32]) -> Option<MoveCommitRoute> {
    match concept_ids {
        [] => None,
        [concept_id] => Some(MoveCommitRoute::Single(*concept_id)),
        [_, ..] => Some(MoveCommitRoute::Group),
    }
}

fn card_containment_notice_after_move(
    dataset: &CanvasDataset,
    concept_ids: &[u32],
    world_delta: Point,
) -> SuccessNotice {
    let [concept_id] = concept_ids else {
        return SuccessNotice::None;
    };
    let Some(card) = dataset.card_by_id(*concept_id) else {
        return SuccessNotice::None;
    };
    let before_parent_ids = dataset.card_parent_ids();
    let after_parent_ids = card_parent_ids(dataset.cards().map(|candidate| {
        let bounds = if candidate.id == *concept_id {
            CanvasRect::new(candidate.bounds.origin + world_delta, candidate.bounds.size)
        } else {
            candidate.bounds
        };
        (candidate.id, bounds)
    }));
    let before_parent_id = before_parent_ids.get(concept_id).copied();
    let after_parent_id = after_parent_ids.get(concept_id).copied();
    if before_parent_id == after_parent_id {
        return SuccessNotice::None;
    }
    let title_for = |card_id| {
        dataset
            .card_by_id(card_id)
            .map(|candidate| candidate.title.clone())
            .unwrap_or_else(|| "an unavailable card".to_string())
    };
    let change = match (before_parent_id, after_parent_id) {
        (None, Some(parent_id)) => CardContainmentChange::Entered {
            card_title: card.title.clone(),
            parent_title: title_for(parent_id),
        },
        (Some(parent_id), None) => CardContainmentChange::Exited {
            card_title: card.title.clone(),
            parent_title: title_for(parent_id),
        },
        (Some(previous_parent_id), Some(parent_id)) => CardContainmentChange::Moved {
            card_title: card.title.clone(),
            previous_parent_title: title_for(previous_parent_id),
            parent_title: title_for(parent_id),
        },
        (None, None) => return SuccessNotice::None,
    };
    SuccessNotice::Specific {
        title: "CONTAINMENT UPDATED".to_string(),
        detail: card_containment_change_detail(&change),
    }
}

fn visual_boundary_containment_notice_after_move(
    dataset: &CanvasDataset,
    boundaries: &[VisualBoundary],
    concept_ids: &[u32],
    world_delta: Point,
) -> SuccessNotice {
    let [concept_id] = concept_ids else {
        return SuccessNotice::None;
    };
    let Some(card) = dataset.card_by_id(*concept_id) else {
        return SuccessNotice::None;
    };
    let before_boundary = containing_visual_boundary_title(boundaries, card.bounds);
    let after_bounds = CanvasRect::new(card.bounds.origin + world_delta, card.bounds.size);
    let after_boundary = containing_visual_boundary_title(boundaries, after_bounds);
    if before_boundary == after_boundary {
        return SuccessNotice::None;
    }
    let change = match (before_boundary, after_boundary) {
        (None, Some(parent_title)) => CardContainmentChange::Entered {
            card_title: card.title.clone(),
            parent_title,
        },
        (Some(parent_title), None) => CardContainmentChange::Exited {
            card_title: card.title.clone(),
            parent_title,
        },
        (Some(previous_parent_title), Some(parent_title)) => CardContainmentChange::Moved {
            card_title: card.title.clone(),
            previous_parent_title,
            parent_title,
        },
        (None, None) => return SuccessNotice::None,
    };
    SuccessNotice::Specific {
        title: "BOUNDARY UPDATED".to_string(),
        detail: card_containment_change_detail(&change),
    }
}

fn containing_visual_boundary_title(
    boundaries: &[VisualBoundary],
    card_bounds: CanvasRect,
) -> Option<String> {
    boundaries
        .iter()
        .filter(|boundary| boundary.bounds.fully_contains(card_bounds))
        .min_by(|left, right| {
            rect_area(left.bounds)
                .total_cmp(&rect_area(right.bounds))
                .then(left.id.cmp(&right.id))
        })
        .map(|boundary| boundary.title.clone())
}

fn resize_layer_change_notice(
    card_title: &str,
    before_layer_title: &str,
    after_layer_title: &str,
) -> SuccessNotice {
    if before_layer_title == after_layer_title {
        SuccessNotice::None
    } else {
        SuccessNotice::Specific {
            title: "LAYER CHANGED".to_string(),
            detail: format!(
                "{card_title} now appears in {after_layer_title} instead of {before_layer_title}."
            ),
        }
    }
}

fn card_containment_change_detail(change: &CardContainmentChange) -> String {
    match change {
        CardContainmentChange::Entered {
            card_title,
            parent_title,
        } => format!("{card_title} is now inside {parent_title}."),
        CardContainmentChange::Exited {
            card_title,
            parent_title,
        } => format!("{card_title} is no longer inside {parent_title}."),
        CardContainmentChange::Moved {
            card_title,
            previous_parent_title,
            parent_title,
        } => format!("{card_title} moved from {previous_parent_title} to {parent_title}."),
    }
}

fn move_preview_bounds(
    bounds: CanvasRect,
    card_id: u32,
    gesture: Option<&PointerGesture>,
) -> CanvasRect {
    match gesture {
        Some(PointerGesture::Move {
            item_ids,
            preview_world_delta,
            ..
        }) if item_ids.contains(&CanvasItemId::Card(card_id)) => {
            CanvasRect::new(bounds.origin + *preview_world_delta, bounds.size)
        }
        Some(PointerGesture::ResizeCard {
            concept_id,
            preview_bounds,
            ..
        }) if *concept_id == card_id => *preview_bounds,
        _ => bounds,
    }
}

fn card_containment_depth(card_id: u32, parent_by_child: &BTreeMap<u32, u32>) -> usize {
    let mut depth = 0;
    let mut current_id = card_id;
    while let Some(parent_id) = parent_by_child.get(&current_id) {
        depth += 1;
        current_id = *parent_id;
    }
    depth
}

fn sort_cards_for_painting(cards: &mut [&ConceptCard], parent_by_child: &BTreeMap<u32, u32>) {
    cards.sort_by_key(|card| card_containment_depth(card.id, parent_by_child));
}

fn resize_anchor_for_card(card_id: u32, gesture: Option<&PointerGesture>) -> Option<bool> {
    match gesture {
        Some(PointerGesture::ResizeCard {
            concept_id,
            centered,
            ..
        }) if *concept_id == card_id => Some(*centered),
        _ => None,
    }
}

fn card_resize_handle_world_size(camera: Camera) -> f32 {
    (CARD_RESIZE_HANDLE_SCREEN_SIZE / camera.scale).max(MINIMUM_POINTER_TARGET_WORLD_SIZE)
}

fn card_resize_handle_screen_bounds(card_bounds: Rect) -> Rect {
    Rect::from_center_size(
        card_bounds.right_bottom(),
        Vec2::splat(CARD_RESIZE_HANDLE_SCREEN_SIZE),
    )
}

fn card_resize_target(
    dataset: &CanvasDataset,
    selected_card_ids: &BTreeSet<u32>,
    world_position: Point,
    handle_size: f32,
) -> Option<(u32, CanvasRect)> {
    selected_card_ids.iter().rev().find_map(|concept_id| {
        let card = dataset.card_by_id(*concept_id)?;
        let bounds = card.bounds.normalized();
        let handle_bounds = CanvasRect::new(
            Point::new(
                bounds.max_x() - handle_size * 0.5,
                bounds.max_y() - handle_size * 0.5,
            ),
            Point::new(handle_size, handle_size),
        );
        handle_bounds
            .contains_point(world_position)
            .then_some((card.id, card.bounds))
    })
}

fn resized_card_bounds(
    initial_bounds: CanvasRect,
    world_delta: Point,
    centered: bool,
    minimum_size: Point,
) -> CanvasRect {
    let size_delta = if centered {
        world_delta * 2.0
    } else {
        world_delta
    };
    let size = Point::new(
        (initial_bounds.size.x + size_delta.x).max(minimum_size.x),
        (initial_bounds.size.y + size_delta.y).max(minimum_size.y),
    );
    let origin = if centered {
        initial_bounds.center() - size * 0.5
    } else {
        initial_bounds.origin
    };
    CanvasRect::new(origin, size)
}

/// Card geometry lives in canvas units, but resize affordances live on screen.
/// This reciprocal minimum keeps a fully selected card usable at any zoom
/// level instead of growing to thousands of pixels at deep detail levels.
fn minimum_card_size_at_screen_scale(screen_scale: f32) -> Point {
    let screen_scale = screen_scale.max(f32::EPSILON);
    Point::new(
        (MINIMUM_CARD_INTERACTION_SCREEN_SIZE.x / screen_scale)
            .clamp(MINIMUM_CARD_PERSISTED_SIZE.x, MINIMUM_CARD_SIZE.x),
        (MINIMUM_CARD_INTERACTION_SCREEN_SIZE.y / screen_scale)
            .clamp(MINIMUM_CARD_PERSISTED_SIZE.y, MINIMUM_CARD_SIZE.y),
    )
}

#[derive(Clone, Debug, PartialEq)]
enum BoundaryPointerTarget {
    Move {
        boundary_id: String,
    },
    Resize {
        boundary_id: String,
        initial_bounds: CanvasRect,
    },
}

fn boundary_resize_handle_world_size(camera: Camera) -> f32 {
    (BOUNDARY_RESIZE_HANDLE_SCREEN_SIZE / camera.scale).max(MINIMUM_POINTER_TARGET_WORLD_SIZE)
}

fn interactive_visual_boundaries(
    boundaries: &[VisualBoundary],
    visible_world: CanvasRect,
) -> Vec<&VisualBoundary> {
    boundaries
        .iter()
        .filter(|boundary| !boundary_supplies_viewport_context(boundary.bounds, visible_world))
        .collect()
}

/// Returns true when a boundary gives meaningful navigational context. It
/// must span a complete visible axis and occupy at least half of the viewport.
/// Such a boundary remains visible, but its nested boundaries receive direct
/// pointer interaction.
fn boundary_supplies_viewport_context(
    boundary_bounds: CanvasRect,
    visible_world: CanvasRect,
) -> bool {
    let boundary_bounds = boundary_bounds.normalized();
    let visible_world = visible_world.normalized();
    !boundary_bounds.is_empty()
        && !visible_world.is_empty()
        && boundary_covers_viewport_on_any_axis(boundary_bounds, visible_world)
        && rect_viewport_coverage(boundary_bounds, visible_world) >= 0.5
}

fn boundary_covers_viewport_on_any_axis(
    boundary_bounds: CanvasRect,
    visible_world: CanvasRect,
) -> bool {
    (boundary_bounds.origin.x <= visible_world.origin.x
        && boundary_bounds.max_x() >= visible_world.max_x())
        || (boundary_bounds.origin.y <= visible_world.origin.y
            && boundary_bounds.max_y() >= visible_world.max_y())
}

fn rect_viewport_coverage(bounds: CanvasRect, viewport: CanvasRect) -> f32 {
    let bounds = bounds.normalized();
    let viewport = viewport.normalized();
    let intersection_width =
        (bounds.max_x().min(viewport.max_x()) - bounds.origin.x.max(viewport.origin.x)).max(0.0);
    let intersection_height =
        (bounds.max_y().min(viewport.max_y()) - bounds.origin.y.max(viewport.origin.y)).max(0.0);
    let viewport_area = rect_area(viewport);
    if viewport_area <= f32::EPSILON {
        0.0
    } else {
        intersection_width * intersection_height / viewport_area
    }
}

fn boundary_resize_handle_screen_bounds(boundary_bounds: Rect) -> Rect {
    Rect::from_center_size(
        boundary_bounds.right_bottom(),
        Vec2::splat(BOUNDARY_RESIZE_HANDLE_SCREEN_SIZE),
    )
}

fn boundary_pointer_target(
    boundaries: &[VisualBoundary],
    world_position: Point,
    handle_size: f32,
) -> Option<BoundaryPointerTarget> {
    let boundary = boundaries
        .iter()
        .enumerate()
        .filter(|(_, boundary)| {
            let bounds = boundary.bounds.normalized();
            let handle_bounds = CanvasRect::new(
                Point::new(bounds.max_x() - handle_size, bounds.max_y() - handle_size),
                Point::new(handle_size * 2.0, handle_size * 2.0),
            );
            bounds.contains_point(world_position) || handle_bounds.contains_point(world_position)
        })
        .min_by(|(left_index, left), (right_index, right)| {
            rect_area(left.bounds)
                .total_cmp(&rect_area(right.bounds))
                // Identical overlapping boundaries use the later record, which
                // matches their visual stacking order.
                .then_with(|| right_index.cmp(left_index))
        })?
        .1;
    let bounds = boundary.bounds.normalized();
    let handle_origin = Point::new(bounds.max_x() - handle_size, bounds.max_y() - handle_size);
    let handle_bounds = CanvasRect::new(
        handle_origin,
        Point::new(handle_size * 2.0, handle_size * 2.0),
    );
    if handle_bounds.contains_point(world_position) {
        return Some(BoundaryPointerTarget::Resize {
            boundary_id: boundary.id.clone(),
            initial_bounds: boundary.bounds,
        });
    }
    bounds
        .contains_point(world_position)
        .then(|| BoundaryPointerTarget::Move {
            boundary_id: boundary.id.clone(),
        })
}

fn resized_boundary_bounds(initial_bounds: CanvasRect, world_delta: Point) -> CanvasRect {
    CanvasRect::new(
        initial_bounds.origin,
        Point::new(
            (initial_bounds.size.x + world_delta.x).max(MINIMUM_BOUNDARY_SIZE.x),
            (initial_bounds.size.y + world_delta.y).max(MINIMUM_BOUNDARY_SIZE.y),
        ),
    )
}

fn boundary_preview_bounds(
    boundary: &VisualBoundary,
    gesture: Option<&PointerGesture>,
) -> CanvasRect {
    match gesture {
        Some(PointerGesture::Move {
            item_ids,
            preview_world_delta,
            ..
        }) if item_ids.contains(&CanvasItemId::Boundary(boundary.id.clone())) => CanvasRect::new(
            boundary.bounds.origin + *preview_world_delta,
            boundary.bounds.size,
        ),
        Some(PointerGesture::ResizeBoundary {
            boundary_id,
            preview_bounds,
            ..
        }) if boundary_id == &boundary.id => *preview_bounds,
        _ => boundary.bounds,
    }
}

fn boundary_inspector_draft(boundary: &VisualBoundary) -> BoundaryInspectorDraft {
    BoundaryInspectorDraft {
        boundary_id: boundary.id.clone(),
        title: boundary.title.clone(),
        summary: boundary.summary.clone(),
    }
}

#[cfg(test)]
fn relationship_overlay_placement_overrides(
    dataset: &CanvasDataset,
    gesture: Option<&PointerGesture>,
) -> RelationshipOverlayPlacementOverrides {
    let Some(PointerGesture::Move { item_ids, .. }) = gesture else {
        return RelationshipOverlayPlacementOverrides::default();
    };

    let mut placement_overrides = RelationshipOverlayPlacementOverrides::default();
    for concept_id in item_ids.iter().filter_map(|item_id| match item_id {
        CanvasItemId::Card(concept_id) => Some(*concept_id),
        CanvasItemId::Boundary(_) => None,
    }) {
        if let Some(card) = dataset.card_by_id(concept_id) {
            placement_overrides.insert(
                concept_id,
                move_preview_bounds(card.bounds, concept_id, gesture),
            );
        }
    }
    placement_overrides
}

fn canvas_rect_to_screen(bounds: CanvasRect, camera: Camera, viewport: Rect) -> Rect {
    let origin = camera.world_to_screen(bounds.origin);
    Rect::from_min_size(
        viewport.min + Vec2::new(origin.x, origin.y),
        Vec2::new(bounds.size.x * camera.scale, bounds.size.y * camera.scale),
    )
}

fn screen_rect_to_canvas_rect(rect: Rect) -> CanvasRect {
    CanvasRect::new(
        Point::new(rect.left(), rect.top()),
        Point::new(rect.width(), rect.height()),
    )
}

#[cfg(test)]
fn selected_relationship_paths(
    document: &WorkspaceDocument,
    version_id: &str,
    lens_id: &str,
    enabled: bool,
    selected_concept_id: Option<u32>,
    overlay_viewport: RelationshipOverlayViewport<'_>,
    placement_overrides: &RelationshipOverlayPlacementOverrides,
) -> Vec<Vec<Pos2>> {
    if !enabled {
        return Vec::new();
    }
    let Some(selected_concept_id) = selected_concept_id else {
        return Vec::new();
    };

    document
        .selected_concept_relationship_paths_with_displayed_bounds(
            version_id,
            lens_id,
            selected_concept_id,
            overlay_viewport.visible_world,
            placement_overrides,
            overlay_viewport.displayed_card_bounds_by_concept_id,
        )
        .map(relationship_paths_to_screen_positions)
        .unwrap_or_default()
}

#[cfg(test)]
fn relationship_paths_to_screen_positions(
    paths: Vec<SelectedConceptRelationshipOverlayPath>,
) -> Vec<Vec<Pos2>> {
    paths
        .into_iter()
        .map(|path| {
            path.points
                .into_iter()
                .map(|point| Pos2::new(point.x, point.y))
                .collect()
        })
        .collect()
}

#[derive(Clone, Debug, PartialEq)]
struct RelationshipPaintPath {
    id: String,
    kind: RelationshipKind,
    visible_segments: Vec<Vec<Pos2>>,
    crosses_boundary: bool,
    arrowhead: Option<[Pos2; 3]>,
    label_position: Option<Pos2>,
}

/// A circuit-style bridge at a relationship crossing. The later path crosses
/// over the earlier path without creating a semantic connection between them.
#[derive(Clone, Copy, Debug, PartialEq)]
struct RelationshipLineCrossing {
    over_path_index: usize,
    position: Pos2,
    direction: Vec2,
}

struct SemanticRelationshipPaintContext<'a> {
    relationship_paths: &'a [RelationshipPaintPath],
    boundary_rects: &'a [Rect],
    selected_relationship_id: Option<&'a str>,
    hover_target: Option<&'a CanvasHoverTarget>,
    show_relationship_labels: bool,
}

struct VisualBoundaryPaintContext<'a> {
    layer_opacities: &'a BTreeMap<String, f32>,
    filter_layers: bool,
    fade_bounds_by_id: &'a BTreeMap<String, CanvasRect>,
    selected_boundary_id: Option<&'a str>,
    hover_target: Option<&'a CanvasHoverTarget>,
    gesture: Option<&'a PointerGesture>,
    camera: Camera,
    viewport: Rect,
    visible_world: CanvasRect,
}

#[derive(Clone, Debug, PartialEq)]
struct CanvasContextEntry {
    entity: CanvasContextEntity,
    title: String,
    kind: CanvasContextKind,
    bounds: CanvasRect,
    depth: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum CanvasContextEntity {
    Boundary(String),
    Card(u32),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum CanvasContextKind {
    Boundary,
    Card { base_layer_title: Option<String> },
}

#[cfg(test)]
fn relationship_paint_paths(
    relationships: &[RelationshipView],
    endpoint_bounds_by_concept_id: &BTreeMap<u32, CanvasRect>,
    displayed_card_bounds_by_concept_id: &BTreeMap<u32, CanvasRect>,
    routing_boundary_rects: &[Rect],
    viewport: Rect,
) -> Vec<RelationshipPaintPath> {
    relationships
        .iter()
        .filter_map(|relationship| {
            let path = routed_relationship_points(
                relationship.source_id,
                relationship.target_id,
                endpoint_bounds_by_concept_id,
                displayed_card_bounds_by_concept_id,
                routing_boundary_rects,
            )?;
            relationship_paint_path(relationship, &path, routing_boundary_rects, viewport)
        })
        .collect()
}

fn relationship_paint_path(
    relationship: &RelationshipView,
    path: &[Pos2],
    boundary_rects: &[Rect],
    viewport: Rect,
) -> Option<RelationshipPaintPath> {
    let visible_segments = clipped_path_segments(path, viewport);
    (!visible_segments.is_empty()).then(|| RelationshipPaintPath {
        id: relationship.id.clone(),
        kind: relationship.kind,
        crosses_boundary: relationship_path_crosses_boundary(path, boundary_rects),
        arrowhead: connection_arrowhead_for_path(path)
            .filter(|arrowhead| viewport.contains(arrowhead[0])),
        label_position: should_paint_relationship_path_label(true, path)
            .then(|| connection_label_position_for_path(path))
            .flatten()
            .filter(|position| viewport.contains(*position)),
        visible_segments,
    })
}

#[cfg(test)]
fn routed_relationship_points(
    source_id: u32,
    target_id: u32,
    endpoint_bounds_by_concept_id: &BTreeMap<u32, CanvasRect>,
    displayed_card_bounds_by_concept_id: &BTreeMap<u32, CanvasRect>,
    routing_boundary_rects: &[Rect],
) -> Option<Vec<Pos2>> {
    let mut bounds = displayed_card_bounds_by_concept_id.clone();
    bounds.extend(
        endpoint_bounds_by_concept_id
            .iter()
            .map(|(id, bounds)| (*id, *bounds)),
    );
    let mut cache = RelationshipRouteCache::default();
    cache.update_layout(
        bounds,
        routing_boundary_rects
            .iter()
            .copied()
            .map(screen_rect_to_canvas_rect)
            .collect(),
    );
    Some(
        cache
            .route(source_id, target_id)?
            .iter()
            .map(|point| Pos2::new(point.x, point.y))
            .collect(),
    )
}

fn clipped_path_segments(points: &[Pos2], viewport: Rect) -> Vec<Vec<Pos2>> {
    let mut segments = Vec::new();
    let mut current_segment: Vec<Pos2> = Vec::new();
    for line in points.windows(2) {
        let Some((start, end)) = clip_line_segment_to_rect(line[0], line[1], viewport) else {
            if !current_segment.is_empty() {
                segments.push(std::mem::take(&mut current_segment));
            }
            continue;
        };
        if current_segment
            .last()
            .is_some_and(|last| last.distance_sq(start) > 0.0001)
        {
            segments.push(std::mem::take(&mut current_segment));
        }
        if current_segment.is_empty() {
            current_segment.push(start);
        }
        current_segment.push(end);
    }
    if !current_segment.is_empty() {
        segments.push(current_segment);
    }
    segments
}

fn clip_line_segment_to_rect(start: Pos2, end: Pos2, bounds: Rect) -> Option<(Pos2, Pos2)> {
    let delta = end - start;
    let mut enter = 0.0_f32;
    let mut exit = 1.0_f32;
    for (direction, distance) in [
        (-delta.x, start.x - bounds.left()),
        (delta.x, bounds.right() - start.x),
        (-delta.y, start.y - bounds.top()),
        (delta.y, bounds.bottom() - start.y),
    ] {
        if direction.abs() <= f32::EPSILON {
            if distance < 0.0 {
                return None;
            }
            continue;
        }
        let ratio = distance / direction;
        if direction < 0.0 {
            if ratio > exit {
                return None;
            }
            enter = enter.max(ratio);
        } else {
            if ratio < enter {
                return None;
            }
            exit = exit.min(ratio);
        }
    }
    (enter <= exit).then(|| (start + delta * enter, start + delta * exit))
}

fn boundary_route_obstacles(
    source_bounds: CanvasRect,
    target_bounds: CanvasRect,
    boundary_rects: &[Rect],
) -> Vec<CanvasRect> {
    boundary_rects
        .iter()
        .copied()
        .filter(|boundary| {
            !boundary_contains_card(*boundary, source_bounds)
                && !boundary_contains_card(*boundary, target_bounds)
        })
        .map(screen_rect_to_canvas_rect)
        .collect()
}

fn boundary_contains_card(boundary: Rect, card_bounds: CanvasRect) -> bool {
    let card_bounds = card_bounds.normalized();
    boundary.contains_rect(Rect::from_min_size(
        Pos2::new(card_bounds.origin.x, card_bounds.origin.y),
        Vec2::new(card_bounds.size.x, card_bounds.size.y),
    ))
}

fn relationship_path_at(
    relationship_paths: &[RelationshipPaintPath],
    position: Pos2,
    hit_radius: f32,
) -> Option<String> {
    relationship_paths
        .iter()
        .filter_map(|path| {
            let distance_squared = path
                .visible_segments
                .iter()
                .flat_map(|segment| segment.windows(2))
                .map(|segment| point_to_segment_distance_squared(position, segment[0], segment[1]))
                .min_by(f32::total_cmp)?;
            (distance_squared <= hit_radius * hit_radius)
                .then_some((distance_squared, path.id.clone()))
        })
        .min_by(|(left_distance, left_id), (right_distance, right_id)| {
            left_distance
                .total_cmp(right_distance)
                .then(left_id.cmp(right_id))
        })
        .map(|(_, id)| id)
}

fn point_to_segment_distance_squared(point: Pos2, start: Pos2, end: Pos2) -> f32 {
    let segment = end - start;
    let length_squared = segment.length_sq();
    if length_squared <= f32::EPSILON {
        return point.distance_sq(start);
    }
    let factor = ((point - start).dot(segment) / length_squared).clamp(0.0, 1.0);
    point.distance_sq(start + segment * factor)
}

#[cfg(test)]
fn relationship_paths_to_screen(
    paths: Vec<SelectedConceptRelationshipOverlayPath>,
    camera: Camera,
    viewport: Rect,
) -> Vec<Vec<Pos2>> {
    paths
        .into_iter()
        .map(|path| {
            path.points
                .into_iter()
                .map(|point| canvas_point_to_screen(point, camera, viewport))
                .collect()
        })
        .collect()
}

fn canvas_point_to_screen(point: Point, camera: Camera, viewport: Rect) -> Pos2 {
    let point = camera.world_to_screen(point);
    viewport.min + Vec2::new(point.x, point.y)
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct BoundaryAwareLineSegment {
    start: Pos2,
    end: Pos2,
    inside_boundary: bool,
}

fn boundary_aware_line_segments(
    start: Pos2,
    end: Pos2,
    boundary_rects: &[Rect],
) -> Vec<BoundaryAwareLineSegment> {
    let delta = end - start;
    let mut intersections = vec![0.0_f32, 1.0];
    for boundary in boundary_rects {
        for x in [boundary.left(), boundary.right()] {
            if delta.x.abs() > f32::EPSILON {
                let t = (x - start.x) / delta.x;
                let y = start.y + delta.y * t;
                if (0.0..=1.0).contains(&t)
                    && y >= boundary.top() - f32::EPSILON
                    && y <= boundary.bottom() + f32::EPSILON
                {
                    intersections.push(t);
                }
            }
        }
        for y in [boundary.top(), boundary.bottom()] {
            if delta.y.abs() > f32::EPSILON {
                let t = (y - start.y) / delta.y;
                let x = start.x + delta.x * t;
                if (0.0..=1.0).contains(&t)
                    && x >= boundary.left() - f32::EPSILON
                    && x <= boundary.right() + f32::EPSILON
                {
                    intersections.push(t);
                }
            }
        }
    }
    intersections.sort_by(f32::total_cmp);
    intersections.dedup_by(|left, right| (*left - *right).abs() < 0.0001);

    intersections
        .windows(2)
        .filter_map(|pair| {
            let start_t = pair[0];
            let end_t = pair[1];
            (end_t > start_t).then(|| {
                let segment_start = start + delta * start_t;
                let segment_end = start + delta * end_t;
                let midpoint = segment_start.lerp(segment_end, 0.5);
                BoundaryAwareLineSegment {
                    start: segment_start,
                    end: segment_end,
                    inside_boundary: boundary_rects
                        .iter()
                        .any(|boundary| boundary.contains(midpoint)),
                }
            })
        })
        .collect()
}

/// A boundary-crossing route has both interior and exterior portions. A route
/// that lives entirely inside a boundary keeps the ordinary relationship color.
fn relationship_path_crosses_boundary(path: &[Pos2], boundary_rects: &[Rect]) -> bool {
    let mut has_inside_segment = false;
    let mut has_outside_segment = false;
    for line in path.windows(2) {
        for segment in boundary_aware_line_segments(line[0], line[1], boundary_rects) {
            if segment.inside_boundary {
                has_inside_segment = true;
            } else {
                has_outside_segment = true;
            }
        }
    }
    has_inside_segment && has_outside_segment
}

fn boundary_context_color(
    route_crosses_boundary: bool,
    position: Pos2,
    boundary_rects: &[Rect],
) -> Color32 {
    let inside_boundary = boundary_rects
        .iter()
        .any(|boundary| boundary.contains(position));
    boundary_segment_color(route_crosses_boundary, inside_boundary)
}

fn boundary_segment_color(route_crosses_boundary: bool, inside_boundary: bool) -> Color32 {
    if route_crosses_boundary && inside_boundary {
        ATLAS_CONTROL_AMBER
    } else {
        ATLAS_ROUTE_BLUE
    }
}

fn relationship_line_crossings(
    relationship_paths: &[RelationshipPaintPath],
) -> Vec<RelationshipLineCrossing> {
    let mut crossings = Vec::new();
    for (under_path_index, under_path) in relationship_paths.iter().enumerate() {
        for (over_path_index, over_path) in relationship_paths
            .iter()
            .enumerate()
            .skip(under_path_index + 1)
        {
            for under_segment in under_path
                .visible_segments
                .iter()
                .flat_map(|segment| segment.windows(2))
            {
                for over_segment in over_path
                    .visible_segments
                    .iter()
                    .flat_map(|segment| segment.windows(2))
                {
                    let Some(position) = proper_line_segment_intersection(
                        under_segment[0],
                        under_segment[1],
                        over_segment[0],
                        over_segment[1],
                    ) else {
                        continue;
                    };
                    let direction = over_segment[1] - over_segment[0];
                    crossings.push(RelationshipLineCrossing {
                        over_path_index,
                        position,
                        direction,
                    });
                }
            }
        }
    }
    crossings
}

fn proper_line_segment_intersection(
    first_start: Pos2,
    first_end: Pos2,
    second_start: Pos2,
    second_end: Pos2,
) -> Option<Pos2> {
    let first_delta = first_end - first_start;
    let second_delta = second_end - second_start;
    let denominator = cross_product(first_delta, second_delta);
    if denominator.abs() <= f32::EPSILON {
        return None;
    }
    let offset = second_start - first_start;
    let first_progress = cross_product(offset, second_delta) / denominator;
    let second_progress = cross_product(offset, first_delta) / denominator;
    let first_clearance = RELATIONSHIP_LINE_JUMP_ENDPOINT_CLEARANCE / first_delta.length();
    let second_clearance = RELATIONSHIP_LINE_JUMP_ENDPOINT_CLEARANCE / second_delta.length();
    ((first_clearance..=(1.0 - first_clearance)).contains(&first_progress)
        && (second_clearance..=(1.0 - second_clearance)).contains(&second_progress))
    .then(|| first_start + first_delta * first_progress)
}

fn cross_product(left: Vec2, right: Vec2) -> f32 {
    left.x * right.y - left.y * right.x
}

fn relationship_line_jump_points(position: Pos2, direction: Vec2) -> Option<Vec<Pos2>> {
    let direction_length = direction.length();
    (direction_length >= RELATIONSHIP_LINE_JUMP_ENDPOINT_CLEARANCE * 2.0).then(|| {
        let unit_direction = direction / direction_length;
        let normal = Vec2::new(-unit_direction.y, unit_direction.x);
        (0..=8)
            .map(|index| {
                let progress = index as f32 / 8.0;
                let horizontal_offset = (progress * 2.0 - 1.0) * RELATIONSHIP_LINE_JUMP_RADIUS;
                let vertical_offset =
                    (progress * std::f32::consts::PI).sin() * RELATIONSHIP_LINE_JUMP_HEIGHT;
                position + unit_direction * horizontal_offset + normal * vertical_offset
            })
            .collect()
    })
}

fn paint_relationship_line_jump(
    painter: &Painter,
    crossing: RelationshipLineCrossing,
    route_crosses_boundary: bool,
    boundary_rects: &[Rect],
    width: f32,
    opacity: f32,
) {
    let Some(points) = relationship_line_jump_points(crossing.position, crossing.direction) else {
        return;
    };
    painter.circle_filled(
        crossing.position,
        RELATIONSHIP_LINE_JUMP_RADIUS + width + 1.0,
        ATLAS_GROUND,
    );
    let color = color_with_opacity(
        boundary_context_color(route_crosses_boundary, crossing.position, boundary_rects),
        opacity,
    );
    painter.line(points, Stroke::new(width, color));
}

fn paint_boundary_aware_path(
    painter: &Painter,
    path: &[Pos2],
    route_crosses_boundary: bool,
    boundary_rects: &[Rect],
    width: f32,
    opacity: f32,
) {
    for line in path.windows(2) {
        for segment in boundary_aware_line_segments(line[0], line[1], boundary_rects) {
            let color = boundary_segment_color(route_crosses_boundary, segment.inside_boundary);
            painter.line_segment(
                [segment.start, segment.end],
                Stroke::new(width, color_with_opacity(color, opacity)),
            );
        }
    }
}

fn relationship_path_stroke_width(selected: bool, hovered: bool) -> f32 {
    if selected {
        3.0
    } else if hovered {
        2.5
    } else {
        1.75
    }
}

fn paint_interactive_relationship_paths(
    painter: &Painter,
    relationship_paths: &[RelationshipPaintPath],
    boundary_rects: &[Rect],
    selected_relationship_id: Option<&str>,
    hover_target: Option<&CanvasHoverTarget>,
    show_relationship_labels: bool,
) {
    for relationship in relationship_paths {
        let selected = selected_relationship_id == Some(relationship.id.as_str());
        let hovered = matches!(hover_target, Some(CanvasHoverTarget::Relationship(id)) if id == &relationship.id);
        for segment in &relationship.visible_segments {
            paint_boundary_aware_path(
                painter,
                segment,
                relationship.crosses_boundary,
                boundary_rects,
                relationship_path_stroke_width(selected, hovered),
                1.0,
            );
        }
    }
    for crossing in relationship_line_crossings(relationship_paths) {
        let relationship = &relationship_paths[crossing.over_path_index];
        let selected = selected_relationship_id == Some(relationship.id.as_str());
        let hovered = matches!(hover_target, Some(CanvasHoverTarget::Relationship(id)) if id == &relationship.id);
        paint_relationship_line_jump(
            painter,
            crossing,
            relationship.crosses_boundary,
            boundary_rects,
            relationship_path_stroke_width(selected, hovered),
            1.0,
        );
    }
    for relationship in relationship_paths {
        let selected = selected_relationship_id == Some(relationship.id.as_str());
        let hovered = matches!(hover_target, Some(CanvasHoverTarget::Relationship(id)) if id == &relationship.id);
        if show_relationship_labels && let Some(label_position) = relationship.label_position {
            painter.text(
                label_position,
                Align2::LEFT_BOTTOM,
                relationship.kind.label(),
                FontId::proportional(ATLAS_RELATIONSHIP_LABEL_SIZE),
                if selected || hovered {
                    ATLAS_PAPER_SIGNAL
                } else {
                    ATLAS_QUIET_GRAPHITE
                },
            );
        }
    }
}

fn paint_semantic_zoom_relationships(
    painter: &Painter,
    frame: &SemanticZoomFrame,
    context: &SemanticRelationshipPaintContext<'_>,
) {
    let relationships_by_id: BTreeMap<_, _> = frame
        .relationships
        .iter()
        .map(|relationship| (relationship.id.as_str(), relationship))
        .collect();
    for path in context.relationship_paths {
        let Some(relationship) = relationships_by_id.get(path.id.as_str()) else {
            continue;
        };
        let selected = context.selected_relationship_id == Some(relationship.id.as_str());
        let hovered = matches!(context.hover_target, Some(CanvasHoverTarget::Relationship(id)) if id == &relationship.id);
        for segment in &path.visible_segments {
            paint_boundary_aware_path(
                painter,
                segment,
                path.crosses_boundary,
                context.boundary_rects,
                relationship_path_stroke_width(selected, hovered),
                relationship.opacity,
            );
        }
    }
    for crossing in relationship_line_crossings(context.relationship_paths) {
        let path = &context.relationship_paths[crossing.over_path_index];
        let Some(relationship) = relationships_by_id.get(path.id.as_str()) else {
            continue;
        };
        let selected = context.selected_relationship_id == Some(relationship.id.as_str());
        let hovered = matches!(context.hover_target, Some(CanvasHoverTarget::Relationship(id)) if id == &relationship.id);
        paint_relationship_line_jump(
            painter,
            crossing,
            path.crosses_boundary,
            context.boundary_rects,
            relationship_path_stroke_width(selected, hovered),
            relationship.opacity,
        );
    }
    for path in context.relationship_paths {
        let Some(relationship) = relationships_by_id.get(path.id.as_str()) else {
            continue;
        };
        let selected = context.selected_relationship_id == Some(relationship.id.as_str());
        let hovered = matches!(context.hover_target, Some(CanvasHoverTarget::Relationship(id)) if id == &relationship.id);
        if context.show_relationship_labels
            && let Some(label_position) = path.label_position
        {
            let label = if !relationship.label.trim().is_empty() {
                relationship.label.as_str()
            } else {
                relationship.kind.label()
            };
            painter.text(
                label_position,
                Align2::LEFT_BOTTOM,
                label,
                FontId::proportional(ATLAS_RELATIONSHIP_LABEL_SIZE),
                color_with_opacity(
                    if hovered || selected {
                        ATLAS_PAPER_SIGNAL
                    } else {
                        ATLAS_QUIET_GRAPHITE
                    },
                    relationship.opacity,
                ),
            );
        }
    }
}

/// Draws each relationship direction cue after cards so its direction remains
/// visible when a route crosses a card edge.
fn paint_relationship_direction_markers(
    painter: &Painter,
    relationship_paths: &[RelationshipPaintPath],
    boundary_rects: &[Rect],
    selected_relationship_id: Option<&str>,
    semantic_zoom_frame: Option<&SemanticZoomFrame>,
) {
    for path in relationship_paths {
        let opacity = match semantic_zoom_frame {
            Some(frame) => frame
                .relationships
                .iter()
                .find(|relationship| relationship.id == path.id)
                .map(|relationship| relationship.opacity),
            None => Some(1.0),
        };
        let Some(opacity) = opacity else {
            continue;
        };
        let Some([tip, left, right]) = path.arrowhead else {
            continue;
        };
        let color = color_with_opacity(
            boundary_context_color(path.crosses_boundary, tip, boundary_rects),
            opacity,
        );
        let width = if selected_relationship_id == Some(path.id.as_str()) {
            2.25
        } else {
            1.5
        };
        painter.line_segment([tip, left], Stroke::new(width, color));
        painter.line_segment([tip, right], Stroke::new(width, color));
    }
}

fn paint_relationship_drag_preview(
    painter: &Painter,
    dataset: &CanvasDataset,
    gesture: Option<&PointerGesture>,
    camera: Camera,
    viewport: Rect,
    boundary_rects: &[Rect],
) {
    let Some(PointerGesture::CreateRelationship {
        source_id,
        source_port,
        preview_world_position,
    }) = gesture
    else {
        return;
    };
    let Some(source_card) = dataset.card_by_id(*source_id) else {
        return;
    };
    let source_port = relationship_port_positions(source_card.bounds)
        .into_iter()
        .find_map(|(edge, point)| (edge == *source_port).then_some(point))
        .expect("the dragged relationship source has a cardinal edge port");
    let source = canvas_point_to_screen_for_painting(source_port, camera, viewport);
    let target = canvas_point_to_screen_for_painting(*preview_world_position, camera, viewport);
    let path = relationship_drag_preview_path(source, target, boundary_rects);
    let crosses_boundary = relationship_path_crosses_boundary(&path, boundary_rects);
    paint_boundary_aware_path(painter, &path, crosses_boundary, boundary_rects, 2.0, 0.86);
    if let Some([tip, left, right]) = connection_arrowhead_for_path(&path) {
        let color = color_with_opacity(
            boundary_context_color(crosses_boundary, tip, boundary_rects),
            0.86,
        );
        painter.line_segment([tip, left], Stroke::new(1.75, color));
        painter.line_segment([tip, right], Stroke::new(1.75, color));
    }
}

fn relationship_drag_preview_path(
    source: Pos2,
    target: Pos2,
    boundary_rects: &[Rect],
) -> Vec<Pos2> {
    let source_bounds = CanvasRect::new(
        Point::new(source.x - 0.5, source.y - 0.5),
        Point::new(1.0, 1.0),
    );
    let target_bounds = CanvasRect::new(
        Point::new(target.x - 0.5, target.y - 0.5),
        Point::new(1.0, 1.0),
    );
    let mut route = relationship_path_around_obstacles(
        source_bounds,
        target_bounds,
        &boundary_route_obstacles(source_bounds, target_bounds, boundary_rects),
    )
    .into_iter()
    .map(|point| Pos2::new(point.x, point.y))
    .collect::<Vec<_>>();
    if let Some(first) = route.first_mut() {
        *first = source;
    }
    if let Some(last) = route.last_mut() {
        *last = target;
    }
    route
}

#[cfg(test)]
fn should_paint_relationship_label(
    show_relationship_labels: bool,
    source: Pos2,
    target: Pos2,
) -> bool {
    should_paint_relationship_path_label(show_relationship_labels, &[source, target])
}

fn should_paint_relationship_path_label(show_relationship_labels: bool, points: &[Pos2]) -> bool {
    show_relationship_labels
        && points
            .windows(2)
            .map(|segment| segment[0].distance(segment[1]))
            .sum::<f32>()
            >= 90.0
}

fn canvas_point_to_screen_for_painting(point: Point, camera: Camera, viewport: Rect) -> Pos2 {
    let point = camera.world_to_screen(point);
    viewport.min + Vec2::new(point.x, point.y)
}

fn card_rect_for_painting(rect: Rect) -> Rect {
    if rect.width() >= MINIMUM_VISIBLE_CARD_SIZE && rect.height() >= MINIMUM_VISIBLE_CARD_SIZE {
        return rect;
    }

    Rect::from_center_size(
        rect.center(),
        Vec2::new(
            rect.width().max(MINIMUM_VISIBLE_CARD_SIZE),
            rect.height().max(MINIMUM_VISIBLE_CARD_SIZE),
        ),
    )
}

fn card_stroke_width(selected: bool) -> f32 {
    if selected {
        SELECTED_CARD_STROKE_WIDTH
    } else {
        CARD_STROKE_WIDTH
    }
}

fn card_paint_bounds(rect: Rect, selected: bool) -> Rect {
    card_rect_for_painting(rect).expand(card_stroke_width(selected))
}

fn displayed_card_world_bounds_by_concept_id(
    cards: &[&ConceptCard],
    gesture: Option<&PointerGesture>,
) -> BTreeMap<u32, CanvasRect> {
    cards
        .iter()
        .map(|card| (card.id, move_preview_bounds(card.bounds, card.id, gesture)))
        .collect()
}

/// The shared inspector surface owns the visual container. Authoring flows use
/// the same title, description, divider, and field rhythm as inspectors so a
/// change of task does not make the right pane feel like a different product.
fn authoring_pane<R>(
    ui: &mut egui::Ui,
    title: &str,
    description: &str,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    inspector_pane_header(ui, title, description);
    add_contents(ui)
}

/// Applies the same density to every right-pane flow. The canvas surface and
/// this pane use related slate tones, but their border and spacing make the
/// editing area unambiguous at a glance.
fn configure_inspector_ui(ui: &mut egui::Ui) {
    let spacing = ui.spacing_mut();
    spacing.item_spacing = Vec2::new(8.0, 9.0);
    spacing.button_padding = Vec2::new(8.0, 5.0);
    spacing.interact_size.y = spacing.interact_size.y.max(26.0);
}

fn inspector_pane_header(ui: &mut egui::Ui, title: &str, description: &str) {
    ui.label(
        egui::RichText::new(title)
            .size(ATLAS_INSPECTOR_TITLE_SIZE)
            .strong()
            .color(ATLAS_PAPER_SIGNAL),
    );
    ui.label(
        egui::RichText::new(description)
            .size(ATLAS_SUPPORTING_TEXT_SIZE)
            .color(ATLAS_QUIET_GRAPHITE),
    );
    ui.add_space(5.0);
    ui.separator();
    ui.add_space(3.0);
}

fn inspector_section_heading(ui: &mut egui::Ui, title: &str) {
    ui.add_space(8.0);
    ui.separator();
    ui.add_space(5.0);
    ui.label(egui::RichText::new(title).strong().color(ATLAS_ROUTE_BLUE));
    ui.add_space(1.0);
}

fn inspector_field_label(ui: &mut egui::Ui, label: &str, helper: Option<&str>) {
    ui.label(
        egui::RichText::new(label)
            .strong()
            .color(ATLAS_PAPER_SIGNAL),
    );
    if let Some(helper) = helper {
        ui.small(egui::RichText::new(helper).color(ATLAS_QUIET_GRAPHITE));
    }
}

/// Route blue is a light navigation and selection color. Primary actions use
/// the same fill with a dark, bold label so the action stays readable at the
/// compact inspector button size.
fn primary_button(label: &'static str) -> egui::Button<'static> {
    egui::Button::new(egui::RichText::new(label).strong().color(ATLAS_GROUND))
        .fill(ATLAS_ROUTE_BLUE)
}

fn canvas_hover_hint(target: &CanvasHoverTarget) -> &'static str {
    match target {
        CanvasHoverTarget::Card(_) => "Click to inspect. Drag to move.",
        CanvasHoverTarget::CardResize(_) => {
            "Drag the blue corner to resize. Hold Option for a center anchor."
        }
        CanvasHoverTarget::RelationshipPort { .. } => "Drag to create a relationship.",
        CanvasHoverTarget::Boundary(_) => "Click to inspect. Drag to move or resize.",
        CanvasHoverTarget::Relationship(_) => "Click to edit relationship",
        CanvasHoverTarget::ScaleLayer(_) => "Click to edit this zoom layer",
        CanvasHoverTarget::ZoomSlider => "Drag to set zoom",
    }
}

fn camera_scale_band(scale: f32) -> SemanticScaleBand {
    match semantic_zoom_level(scale) {
        SemanticZoomLevel::Overview => SemanticScaleBand::Broad,
        SemanticZoomLevel::Card => SemanticScaleBand::Working,
        SemanticZoomLevel::Detail => SemanticScaleBand::Detail,
    }
}

fn semantic_scale_band_for_layer(layer_index: usize, layer_count: usize) -> SemanticScaleBand {
    if layer_count <= 1 || layer_index == 0 {
        SemanticScaleBand::Broad
    } else if layer_index + 1 == layer_count {
        SemanticScaleBand::Detail
    } else {
        SemanticScaleBand::Working
    }
}

/// Saved layer presentation applies only when the Lens has layers and the
/// temporary view-all recovery mode is off.
fn semantic_layer_filtering_is_active(
    semantic_zoom_is_configured: bool,
    show_all_cards: bool,
) -> bool {
    semantic_zoom_is_configured && !show_all_cards
}

fn strongest_visible_layer_indices(layers: &[SemanticZoomLayerProjection]) -> Vec<usize> {
    let strongest_opacity = layers
        .iter()
        .map(|layer| layer.opacity)
        .filter(|opacity| *opacity > 0.0)
        .max_by(f32::total_cmp);
    let Some(strongest_opacity) = strongest_opacity else {
        return Vec::new();
    };
    layers
        .iter()
        .enumerate()
        .filter_map(|(index, layer)| {
            (layer.opacity.total_cmp(&strongest_opacity).is_eq()).then_some(index)
        })
        .collect()
}

fn semantic_scale_rail(
    viewport: Rect,
    scale: f32,
    layers: &[SemanticZoomLayerProjection],
) -> SemanticScaleRail {
    let active_layer_indices = strongest_visible_layer_indices(layers);
    let active_layer_index = active_layer_indices.last().copied();
    let (source, active_band, active_layer_title, labels, opacities, layer_ids) =
        match active_layer_index {
            Some(layer_index) => (
                SemanticScaleRailSource::LensLayer,
                Some(semantic_scale_band_for_layer(layer_index, layers.len())),
                Some(
                    active_layer_indices
                        .iter()
                        .map(|index| layers[*index].title.as_str())
                        .collect::<Vec<_>>()
                        .join(" / "),
                ),
                layers
                    .iter()
                    .map(|layer| layer.title.clone())
                    .collect::<Vec<_>>(),
                layers.iter().map(|layer| layer.opacity).collect::<Vec<_>>(),
                layers
                    .iter()
                    .map(|layer| Some(layer.id.clone()))
                    .collect::<Vec<_>>(),
            ),
            None if layers.is_empty() => {
                let active_band = camera_scale_band(scale);
                (
                    SemanticScaleRailSource::Camera,
                    Some(active_band),
                    None,
                    SemanticScaleBand::ALL
                        .into_iter()
                        .map(|band| band.label().to_string())
                        .collect(),
                    SemanticScaleBand::ALL
                        .into_iter()
                        .map(|band| if band == active_band { 1.0 } else { 0.52 })
                        .collect(),
                    SemanticScaleBand::ALL
                        .into_iter()
                        .map(|_| None)
                        .collect::<Vec<Option<String>>>(),
                )
            }
            None => (
                SemanticScaleRailSource::LensLayerGap,
                None,
                None,
                layers
                    .iter()
                    .map(|layer| layer.title.clone())
                    .collect::<Vec<_>>(),
                layers.iter().map(|layer| layer.opacity).collect::<Vec<_>>(),
                layers
                    .iter()
                    .map(|layer| Some(layer.id.clone()))
                    .collect::<Vec<_>>(),
            ),
        };
    let available_width = (viewport.width() - SEMANTIC_SCALE_RAIL_MARGIN * 2.0).max(0.0);
    let width = if available_width < SEMANTIC_SCALE_RAIL_MINIMUM_WIDTH {
        available_width
    } else {
        available_width.min(SEMANTIC_SCALE_RAIL_PREFERRED_WIDTH)
    };
    let description = match (source, active_band) {
        (SemanticScaleRailSource::Camera, Some(band)) => format!("Camera scale: {}", band.label()),
        (SemanticScaleRailSource::LensLayer, _) => format!(
            "Current layer: {}",
            active_layer_title.as_deref().unwrap_or("no active layer")
        ),
        (SemanticScaleRailSource::LensLayerGap, None) => {
            "Current layer: no active layer".to_string()
        }
        _ => "Current layer: unavailable".to_string(),
    };
    let header_lines = wrap_semantic_scale_rail_text(
        &description,
        (width - SEMANTIC_SCALE_RAIL_PADDING * 2.0).max(1.0),
    );
    let desired_height = SEMANTIC_SCALE_RAIL_PADDING * 2.0
        + header_lines.len() as f32 * SEMANTIC_SCALE_RAIL_HEADER_LINE_HEIGHT
        + 4.0
        + SEMANTIC_SCALE_RAIL_BAND_HEIGHT
        + 6.0
        + SEMANTIC_SCALE_RAIL_ZOOM_ROW_HEIGHT;
    let height = (viewport.height() - SEMANTIC_SCALE_RAIL_MARGIN * 2.0).clamp(0.0, desired_height);
    let bounds = Rect::from_min_size(
        Pos2::new(
            (viewport.left() + SEMANTIC_SCALE_RAIL_MARGIN).min(viewport.right()),
            (viewport.top() + SEMANTIC_SCALE_RAIL_MARGIN).min(viewport.bottom()),
        ),
        Vec2::new(width, height),
    );
    let band_count = labels.len().max(1) as f32;
    let band_width = ((bounds.width()
        - SEMANTIC_SCALE_RAIL_PADDING * 2.0
        - SEMANTIC_SCALE_RAIL_GAP * (band_count - 1.0))
        / band_count)
        .max(0.0);
    let band_top = bounds.top()
        + SEMANTIC_SCALE_RAIL_PADDING
        + header_lines.len() as f32 * SEMANTIC_SCALE_RAIL_HEADER_LINE_HEIGHT
        + 4.0;
    let band_height = SEMANTIC_SCALE_RAIL_BAND_HEIGHT.min(
        (bounds.bottom()
            - SEMANTIC_SCALE_RAIL_PADDING
            - SEMANTIC_SCALE_RAIL_ZOOM_ROW_HEIGHT
            - 6.0
            - band_top)
            .max(0.0),
    );
    let bands = labels
        .into_iter()
        .zip(opacities)
        .zip(layer_ids)
        .enumerate()
        .map(|(index, ((label, opacity), layer_id))| {
            let index = index as f32;
            SemanticScaleRailBand {
                layer_id,
                label,
                bounds: Rect::from_min_size(
                    Pos2::new(
                        bounds.left()
                            + SEMANTIC_SCALE_RAIL_PADDING
                            + index * (band_width + SEMANTIC_SCALE_RAIL_GAP),
                        band_top,
                    ),
                    Vec2::new(band_width, band_height),
                ),
                opacity,
                active: active_layer_indices.contains(&(index as usize))
                    || (layers.is_empty()
                        && active_band
                            .is_some_and(|band| SemanticScaleBand::ALL[index as usize] == band)),
            }
        })
        .collect();

    let zoom_slider_bounds = Rect::from_min_size(
        Pos2::new(
            bounds.left() + SEMANTIC_SCALE_RAIL_PADDING + SEMANTIC_SCALE_RAIL_ZOOM_LABEL_WIDTH,
            bounds.bottom()
                - SEMANTIC_SCALE_RAIL_PADDING
                - SEMANTIC_SCALE_RAIL_ZOOM_ROW_HEIGHT * 0.5
                - 2.0,
        ),
        Vec2::new(
            (bounds.width()
                - SEMANTIC_SCALE_RAIL_PADDING * 2.0
                - SEMANTIC_SCALE_RAIL_ZOOM_LABEL_WIDTH)
                .max(0.0),
            4.0,
        ),
    );
    SemanticScaleRail {
        bounds,
        source,
        active_band,
        active_layer_title,
        header_lines,
        bands,
        zoom_slider_bounds,
        zoom_slider_fraction: zoom_slider_fraction(scale),
        zoom_percent: scale * 100.0,
    }
}

fn semantic_scale_rail_layer_id_at(rail: &SemanticScaleRail, position: Pos2) -> Option<String> {
    if rail.zoom_slider_bounds.expand(8.0).contains(position) {
        return None;
    }
    if let Some(layer_id) = rail
        .bands
        .iter()
        .find(|band| band.bounds.contains(position))
        .and_then(|band| band.layer_id.clone())
    {
        return Some(layer_id);
    }
    rail.bounds
        .contains(position)
        .then(|| {
            rail.bands
                .iter()
                .find(|band| band.active)
                .and_then(|band| band.layer_id.clone())
        })
        .flatten()
}

fn zoom_slider_fraction(scale: f32) -> f32 {
    let minimum_log = MIN_SCALE.ln();
    let maximum_log = MAX_SCALE.ln();
    (scale.clamp(MIN_SCALE, MAX_SCALE).ln() - minimum_log) / (maximum_log - minimum_log)
}

fn zoom_scale_from_slider_fraction(fraction: f32) -> f32 {
    let minimum_log = MIN_SCALE.ln();
    let maximum_log = MAX_SCALE.ln();
    (minimum_log + fraction.clamp(0.0, 1.0) * (maximum_log - minimum_log)).exp()
}

fn camera_with_scale_at_viewport_center(
    camera: Camera,
    scale: f32,
    viewport_size: Point,
) -> Camera {
    let viewport_center = viewport_size * 0.5;
    let centered_world_point = camera.screen_to_world(viewport_center);
    let mut updated = Camera::new(camera.offset, scale);
    updated.center_world_point_in_viewport(centered_world_point, viewport_size);
    updated
}

/// Returns the smallest normalized rectangle that contains both inputs.
fn canvas_rect_union(first: CanvasRect, second: CanvasRect) -> CanvasRect {
    let first = first.normalized();
    let second = second.normalized();
    let left = first.origin.x.min(second.origin.x);
    let top = first.origin.y.min(second.origin.y);
    let right = first.max_x().max(second.max_x());
    let bottom = first.max_y().max(second.max_y());
    CanvasRect::new(
        Point::new(left, top),
        Point::new(right - left, bottom - top),
    )
}

/// Fits content with a quiet outer margin. It never zooms in: the context-menu
/// action is a recovery action for a view that has become too close or lost.
fn example_overview_camera(bounds: CanvasRect, viewport_size: Point) -> Camera {
    let top_inset = 100.0_f32.min(viewport_size.y * 0.25);
    let mut camera = camera_centered_to_fit_bounds(
        Camera::default(),
        bounds,
        Point::new(viewport_size.x, viewport_size.y - top_inset),
    );
    camera.offset.y += top_inset;
    camera
}

fn camera_centered_to_fit_bounds(
    current: Camera,
    bounds: CanvasRect,
    viewport_size: Point,
) -> Camera {
    let bounds = bounds.normalized();
    if bounds.is_empty() || viewport_size.x <= 0.0 || viewport_size.y <= 0.0 {
        return current;
    }
    let available_width = (viewport_size.x * 0.84).max(1.0);
    let available_height = (viewport_size.y * 0.84).max(1.0);
    let width_scale = available_width / bounds.size.x.max(1.0);
    let height_scale = available_height / bounds.size.y.max(1.0);
    let scale = current
        .scale
        .min(width_scale.min(height_scale))
        .clamp(MIN_SCALE, MAX_SCALE);
    let mut centered = Camera::new(current.offset, scale);
    centered.center_world_point_in_viewport(bounds.center(), viewport_size);
    centered
}

fn semantic_scale_rail_zoom_scale_at(rail: &SemanticScaleRail, position: Pos2) -> Option<f32> {
    rail.zoom_slider_bounds
        .expand(8.0)
        .contains(position)
        .then(|| semantic_scale_rail_zoom_scale_from_x(rail, position.x))
}

fn semantic_scale_rail_zoom_scale_from_x(rail: &SemanticScaleRail, x: f32) -> f32 {
    let fraction = if rail.zoom_slider_bounds.width() > 0.0 {
        (x - rail.zoom_slider_bounds.left()) / rail.zoom_slider_bounds.width()
    } else {
        0.0
    };
    zoom_scale_from_slider_fraction(fraction)
}

fn semantic_scale_instruction_position(viewport: Rect) -> Pos2 {
    viewport.right_bottom() - Vec2::new(SEMANTIC_SCALE_RAIL_MARGIN, SEMANTIC_SCALE_RAIL_MARGIN)
}

fn semantic_scale_rail_band_label(label: &str, width: f32) -> String {
    let maximum_characters = (width / 7.0).floor().max(1.0) as usize;
    let character_count = label.chars().count();
    if character_count <= maximum_characters {
        return label.to_string();
    }
    if maximum_characters == 1 {
        return ".".to_string();
    }
    format!(
        "{}...",
        label
            .chars()
            .take(maximum_characters.saturating_sub(1))
            .collect::<String>()
    )
}

fn wrap_semantic_scale_rail_text(text: &str, width: f32) -> Vec<String> {
    let maximum_characters = (width / 7.0).floor().max(1.0) as usize;
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        let required_characters =
            line.chars().count() + usize::from(!line.is_empty()) + word.chars().count();
        if !line.is_empty() && required_characters > maximum_characters {
            lines.push(std::mem::take(&mut line));
        }
        let mut remaining = word;
        while remaining.chars().count() > maximum_characters {
            let chunk = remaining
                .chars()
                .take(maximum_characters)
                .collect::<String>();
            if !line.is_empty() {
                lines.push(std::mem::take(&mut line));
            }
            lines.push(chunk);
            remaining = &remaining[remaining
                .char_indices()
                .nth(maximum_characters)
                .map_or(remaining.len(), |(index, _)| index)..];
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(remaining);
    }
    if !line.is_empty() {
        lines.push(line);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn paint_visual_boundaries(
    painter: &Painter,
    boundaries: &[VisualBoundary],
    context: &VisualBoundaryPaintContext<'_>,
) {
    for boundary in boundaries {
        let bounds = boundary_preview_bounds(boundary, context.gesture);
        let rect = canvas_rect_to_screen(bounds, context.camera, context.viewport);
        if !rect.intersects(context.viewport) {
            continue;
        }
        let selected = context.selected_boundary_id == Some(boundary.id.as_str());
        let hovered = matches!(context.hover_target, Some(CanvasHoverTarget::Boundary(id)) if id == &boundary.id);
        let opacity = context_entity_opacity(
            if context.filter_layers {
                boundary
                    .layer_id
                    .as_ref()
                    .and_then(|id| context.layer_opacities.get(id))
                    .copied()
                    .unwrap_or(1.0)
            } else {
                1.0
            },
            context.fade_bounds_by_id.get(&boundary.id).copied(),
            context.visible_world,
        );
        painter.rect_filled(
            rect,
            8.0,
            Color32::from_rgba_unmultiplied(
                ATLAS_ROUTE_BLUE.r(),
                ATLAS_ROUTE_BLUE.g(),
                ATLAS_ROUTE_BLUE.b(),
                (20.0 * opacity).round() as u8,
            ),
        );
        painter.rect_stroke(
            rect,
            8.0,
            Stroke::new(
                if selected { 2.5 } else { 1.0 },
                color_with_opacity(
                    if hovered {
                        ATLAS_PAPER_SIGNAL
                    } else {
                        ATLAS_ROUTE_BLUE
                    },
                    (if selected {
                        1.0
                    } else if hovered {
                        0.94
                    } else {
                        0.72
                    }) * opacity,
                ),
            ),
            StrokeKind::Inside,
        );
        let title_rect =
            Rect::from_min_size(rect.left_top(), Vec2::new(rect.width().min(260.0), 28.0));
        painter.rect_filled(
            title_rect,
            6.0,
            Color32::from_rgba_unmultiplied(
                ATLAS_SURVEY_SLATE.r(),
                ATLAS_SURVEY_SLATE.g(),
                ATLAS_SURVEY_SLATE.b(),
                (238.0 * opacity).round() as u8,
            ),
        );
        painter.text(
            title_rect.left_center() + Vec2::new(8.0, 0.0),
            Align2::LEFT_CENTER,
            &boundary.title,
            FontId::proportional(ATLAS_SUPPORTING_TEXT_SIZE),
            color_with_opacity(ATLAS_PAPER_SIGNAL, opacity),
        );
        if should_paint_boundary_summary(context.camera.scale, rect, &boundary.summary) {
            painter.text(
                rect.left_top() + Vec2::new(8.0, 38.0),
                Align2::LEFT_TOP,
                &boundary.summary,
                FontId::proportional(ATLAS_SUPPORTING_TEXT_SIZE),
                color_with_opacity(
                    ATLAS_QUIET_GRAPHITE,
                    opacity * summary_opacity_for_screen_bounds(rect),
                ),
            );
        }
        if selected {
            let handle = boundary_resize_handle_screen_bounds(rect);
            painter.rect_filled(handle, 3.0, color_with_opacity(ATLAS_ROUTE_BLUE, opacity));
            painter.rect_stroke(
                handle,
                3.0,
                Stroke::new(1.0, color_with_opacity(ATLAS_PAPER_SIGNAL, opacity)),
                StrokeKind::Inside,
            );
        }
    }
}

fn should_paint_boundary_summary(_scale: f32, rect: Rect, summary: &str) -> bool {
    !summary.trim().is_empty() && summary_opacity_for_screen_bounds(rect) > 0.0
}

/// Text follows the displayed card size, including its authored layer scale,
/// rather than the camera's global detail threshold.
fn summary_opacity_for_screen_bounds(rect: Rect) -> f32 {
    ((rect.width() - 120.0) / 60.0)
        .min((rect.height() - 52.0) / 28.0)
        .clamp(0.0, 1.0)
}

fn canvas_context_entries<'a>(
    boundaries: &[VisualBoundary],
    cards: impl IntoIterator<Item = &'a ConceptCard>,
    card_layer_titles: &BTreeMap<u32, String>,
    gesture: Option<&PointerGesture>,
    visible_world: CanvasRect,
) -> Vec<CanvasContextEntry> {
    let mut entries = boundaries
        .iter()
        .filter_map(|boundary| {
            let bounds = boundary_preview_bounds(boundary, gesture);
            // Keep the trail in lockstep with the interaction rule. A narrow
            // strip that crosses the view is still an editable boundary, not
            // the user's enclosing canvas context.
            boundary_supplies_viewport_context(bounds, visible_world).then(|| CanvasContextEntry {
                entity: CanvasContextEntity::Boundary(boundary.id.clone()),
                title: boundary.title.clone(),
                kind: CanvasContextKind::Boundary,
                bounds,
                depth: 0,
            })
        })
        .chain(cards.into_iter().filter_map(|card| {
            let bounds = move_preview_bounds(card.bounds, card.id, gesture);
            boundary_supplies_viewport_context(bounds, visible_world).then(|| CanvasContextEntry {
                entity: CanvasContextEntity::Card(card.id),
                title: card.title.clone(),
                kind: CanvasContextKind::Card {
                    base_layer_title: card_layer_titles.get(&card.id).cloned(),
                },
                bounds,
                depth: 0,
            })
        }))
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| {
        rect_area(right.bounds)
            .total_cmp(&rect_area(left.bounds))
            .then(left.title.cmp(&right.title))
    });
    for entry_index in 0..entries.len() {
        let bounds = entries[entry_index].bounds;
        entries[entry_index].depth = entries
            .iter()
            .enumerate()
            .filter(|(candidate_index, candidate)| {
                *candidate_index != entry_index
                    && rect_area(candidate.bounds) > rect_area(bounds)
                    && rect_encloses(candidate.bounds, bounds)
            })
            .count();
    }
    entries
}

/// Context fades according to how much larger its enclosing card or boundary
/// is than the visible canvas. The transition begins before it earns a
/// context-trail entry, so crossing that trail threshold never snaps opacity.
fn context_entity_opacity(
    base_opacity: f32,
    candidate_bounds: Option<CanvasRect>,
    visible_world: CanvasRect,
) -> f32 {
    let Some(candidate_bounds) = candidate_bounds else {
        return base_opacity;
    };
    let visible_area = rect_area(visible_world);
    let context_area_ratio = if visible_area > f32::EPSILON {
        (rect_area(candidate_bounds) / visible_area).max(1.0)
    } else {
        1.0
    };
    let context_opacity = CONTEXT_ENTITY_OPACITY_AT_DETAIL
        + (CONTEXT_ENTITY_OPACITY_AT_OVERVIEW - CONTEXT_ENTITY_OPACITY_AT_DETAIL)
            / context_area_ratio;
    let viewport_coverage = rect_viewport_coverage(candidate_bounds, visible_world);
    let transition = smoothstep(
        CONTEXT_FADE_START_VIEWPORT_COVERAGE,
        CONTEXT_FADE_COMPLETE_VIEWPORT_COVERAGE,
        viewport_coverage,
    );
    base_opacity * (1.0 + (context_opacity - 1.0) * transition)
}

fn smoothstep(start: f32, end: f32, value: f32) -> f32 {
    let range = (end - start).max(f32::EPSILON);
    let progress = ((value - start) / range).clamp(0.0, 1.0);
    progress * progress * (3.0 - 2.0 * progress)
}

fn rect_encloses(container: CanvasRect, contained: CanvasRect) -> bool {
    let container = container.normalized();
    let contained = contained.normalized();
    !container.is_empty()
        && !contained.is_empty()
        && container.origin.x <= contained.origin.x
        && container.origin.y <= contained.origin.y
        && container.max_x() >= contained.max_x()
        && container.max_y() >= contained.max_y()
}

fn rect_area(bounds: CanvasRect) -> f32 {
    let bounds = bounds.normalized();
    bounds.size.x * bounds.size.y
}

#[derive(Clone, Debug, PartialEq)]
struct ContextTrailLine {
    text: String,
    indent: f32,
}

fn context_trail_lines(entries: &[CanvasContextEntry], width: f32) -> Vec<ContextTrailLine> {
    entries
        .iter()
        .flat_map(|entry| {
            let indent = 12.0 * (entry.depth + 1) as f32;
            wrap_semantic_scale_rail_text(
                &canvas_context_entry_label(entry),
                (width - SEMANTIC_SCALE_RAIL_PADDING * 2.0 - indent).max(1.0),
            )
            .into_iter()
            .enumerate()
            .map(move |(line_index, text)| ContextTrailLine {
                text,
                indent: indent + if line_index == 0 { 0.0 } else { 8.0 },
            })
        })
        .collect()
}

fn paint_context_trail(
    painter: &Painter,
    viewport: Rect,
    rail: &SemanticScaleRail,
    context_entries: &[CanvasContextEntry],
    hover_hint: Option<&str>,
    resize_preview: Option<&str>,
) {
    let width = rail.bounds.width();
    let context_lines = context_trail_lines(context_entries, width);
    let feedback_lines = hover_hint
        .into_iter()
        .chain(resize_preview)
        .flat_map(|line| {
            wrap_semantic_scale_rail_text(
                line,
                (width - SEMANTIC_SCALE_RAIL_PADDING * 2.0).max(1.0),
            )
        })
        .collect::<Vec<_>>();
    if context_lines.is_empty() && feedback_lines.is_empty() {
        return;
    }
    let heading_height = if context_lines.is_empty() {
        0.0
    } else {
        CONTEXT_TRAIL_LINE_HEIGHT
    };
    let feedback_gap = if context_lines.is_empty() || feedback_lines.is_empty() {
        0.0
    } else {
        6.0
    };
    let desired_height = SEMANTIC_SCALE_RAIL_PADDING * 2.0
        + heading_height
        + context_lines.len() as f32 * CONTEXT_TRAIL_LINE_HEIGHT
        + feedback_gap
        + feedback_lines.len() as f32 * CONTEXT_TRAIL_LINE_HEIGHT;
    let top = (rail.bounds.bottom() + CONTEXT_TRAIL_GAP).min(viewport.bottom());
    let bounds = Rect::from_min_size(
        Pos2::new(rail.bounds.left(), top),
        Vec2::new(width, (viewport.bottom() - top).clamp(0.0, desired_height)),
    );
    painter.rect_filled(
        bounds,
        8.0,
        Color32::from_rgba_unmultiplied(
            ATLAS_SURVEY_SLATE.r(),
            ATLAS_SURVEY_SLATE.g(),
            ATLAS_SURVEY_SLATE.b(),
            238,
        ),
    );
    painter.rect_stroke(
        bounds,
        8.0,
        Stroke::new(1.0, color_with_opacity(ATLAS_QUIET_GRAPHITE, 0.66)),
        StrokeKind::Inside,
    );
    let mut position = bounds.left_top() + Vec2::splat(SEMANTIC_SCALE_RAIL_PADDING);
    if !context_lines.is_empty() {
        painter.text(
            position,
            Align2::LEFT_TOP,
            "Context trail",
            FontId::proportional(ATLAS_SUPPORTING_TEXT_SIZE),
            ATLAS_PAPER_SIGNAL,
        );
        position.y += CONTEXT_TRAIL_LINE_HEIGHT;
        for line in &context_lines {
            painter.text(
                position + Vec2::new(line.indent, 0.0),
                Align2::LEFT_TOP,
                &line.text,
                FontId::proportional(ATLAS_SUPPORTING_TEXT_SIZE),
                ATLAS_QUIET_GRAPHITE,
            );
            position.y += CONTEXT_TRAIL_LINE_HEIGHT;
        }
    }
    if !feedback_lines.is_empty() {
        position.y += feedback_gap;
    }
    for line in feedback_lines {
        painter.text(
            position,
            Align2::LEFT_TOP,
            line,
            FontId::proportional(ATLAS_SUPPORTING_TEXT_SIZE),
            if resize_preview.is_some() && hover_hint.is_none() {
                ATLAS_CONTROL_AMBER
            } else {
                ATLAS_PAPER_SIGNAL
            },
        );
        position.y += CONTEXT_TRAIL_LINE_HEIGHT;
    }
}

fn canvas_context_entry_label(entry: &CanvasContextEntry) -> String {
    match &entry.kind {
        CanvasContextKind::Boundary => format!("Boundary: {}", entry.title),
        CanvasContextKind::Card {
            base_layer_title: Some(layer_title),
        } => format!("{} ({layer_title})", entry.title),
        CanvasContextKind::Card {
            base_layer_title: None,
        } => entry.title.clone(),
    }
}

fn paint_semantic_scale_rail(
    painter: &Painter,
    rail: &SemanticScaleRail,
    hovered_layer_id: Option<&str>,
    zoom_slider_hovered: bool,
    viewport: Rect,
) {
    painter.rect_filled(
        rail.bounds,
        8.0,
        Color32::from_rgba_unmultiplied(
            ATLAS_SURVEY_SLATE.r(),
            ATLAS_SURVEY_SLATE.g(),
            ATLAS_SURVEY_SLATE.b(),
            238,
        ),
    );
    painter.rect_stroke(
        rail.bounds,
        8.0,
        Stroke::new(1.0, color_with_opacity(ATLAS_QUIET_GRAPHITE, 0.66)),
        StrokeKind::Inside,
    );
    for (line_index, line) in rail.header_lines.iter().enumerate() {
        painter.text(
            rail.bounds.left_top()
                + Vec2::new(
                    SEMANTIC_SCALE_RAIL_PADDING,
                    SEMANTIC_SCALE_RAIL_PADDING
                        + line_index as f32 * SEMANTIC_SCALE_RAIL_HEADER_LINE_HEIGHT,
                ),
            Align2::LEFT_TOP,
            line,
            FontId::proportional(ATLAS_SUPPORTING_TEXT_SIZE),
            ATLAS_PAPER_SIGNAL,
        );
    }
    for band in &rail.bands {
        let active = band.active;
        let hovered = band.layer_id.as_deref() == hovered_layer_id;
        painter.rect_filled(
            band.bounds,
            4.0,
            color_with_opacity(
                if active {
                    ATLAS_ROUTE_BLUE
                } else {
                    ATLAS_GROUND
                },
                if active {
                    0.28
                } else if hovered {
                    0.82
                } else {
                    0.64
                },
            ),
        );
        painter.rect_stroke(
            band.bounds,
            4.0,
            Stroke::new(
                1.0,
                color_with_opacity(
                    if hovered {
                        ATLAS_PAPER_SIGNAL
                    } else if active {
                        ATLAS_ROUTE_BLUE
                    } else {
                        ATLAS_QUIET_GRAPHITE
                    },
                    if hovered { 1.0 } else { band.opacity },
                ),
            ),
            StrokeKind::Inside,
        );
        painter.text(
            band.bounds.center(),
            Align2::CENTER_CENTER,
            semantic_scale_rail_band_label(&band.label, band.bounds.width()),
            FontId::proportional(ATLAS_SUPPORTING_TEXT_SIZE),
            color_with_opacity(
                if active || hovered {
                    ATLAS_PAPER_SIGNAL
                } else {
                    ATLAS_QUIET_GRAPHITE
                },
                if hovered { 1.0 } else { band.opacity },
            ),
        );
    }
    let track = rail.zoom_slider_bounds;
    let knob_x = track.left() + track.width() * rail.zoom_slider_fraction;
    let active_track = Rect::from_min_max(
        track.left_top(),
        Pos2::new(knob_x.max(track.left()), track.bottom()),
    );
    painter.text(
        Pos2::new(
            rail.bounds.left() + SEMANTIC_SCALE_RAIL_PADDING,
            track.center().y,
        ),
        Align2::LEFT_CENTER,
        format!("Zoom {:.0}%", rail.zoom_percent),
        FontId::proportional(ATLAS_SUPPORTING_TEXT_SIZE),
        ATLAS_QUIET_GRAPHITE,
    );
    painter.rect_filled(track, 2.0, color_with_opacity(ATLAS_GROUND, 0.82));
    painter.rect_filled(active_track, 2.0, ATLAS_ROUTE_BLUE);
    painter.circle_filled(
        Pos2::new(knob_x, track.center().y),
        if zoom_slider_hovered { 5.0 } else { 4.0 },
        if zoom_slider_hovered {
            ATLAS_PAPER_SIGNAL
        } else {
            ATLAS_ROUTE_BLUE
        },
    );
    painter.circle_stroke(
        Pos2::new(knob_x, track.center().y),
        if zoom_slider_hovered { 5.0 } else { 4.0 },
        Stroke::new(1.0, ATLAS_PAPER_SIGNAL),
    );
    if let Some(band) = rail
        .bands
        .iter()
        .find(|band| band.layer_id.as_deref() == hovered_layer_id)
    {
        paint_semantic_scale_rail_hover_label(painter, rail, band, viewport);
    }
}

fn paint_semantic_scale_rail_hover_label(
    painter: &Painter,
    rail: &SemanticScaleRail,
    band: &SemanticScaleRailBand,
    viewport: Rect,
) {
    let label = format!("Layer: {}", band.label);
    let lines = wrap_semantic_scale_rail_text(
        &label,
        (rail.bounds.width() - SEMANTIC_SCALE_RAIL_PADDING * 2.0).max(1.0),
    );
    let height = SEMANTIC_SCALE_RAIL_PADDING * 2.0
        + lines.len() as f32 * SEMANTIC_SCALE_RAIL_HEADER_LINE_HEIGHT;
    let left = if rail.bounds.right() + 8.0 + rail.bounds.width() <= viewport.right() {
        rail.bounds.right() + 8.0
    } else {
        rail.bounds.left()
    };
    let bounds = Rect::from_min_size(
        Pos2::new(left, rail.bounds.top().max(viewport.top() + 4.0)),
        Vec2::new(rail.bounds.width(), height),
    );
    painter.rect_filled(
        bounds,
        6.0,
        Color32::from_rgba_unmultiplied(
            ATLAS_SURVEY_SLATE.r(),
            ATLAS_SURVEY_SLATE.g(),
            ATLAS_SURVEY_SLATE.b(),
            250,
        ),
    );
    painter.rect_stroke(
        bounds,
        6.0,
        Stroke::new(1.0, ATLAS_PAPER_SIGNAL),
        StrokeKind::Inside,
    );
    for (line_index, line) in lines.iter().enumerate() {
        painter.text(
            bounds.left_top()
                + Vec2::new(
                    SEMANTIC_SCALE_RAIL_PADDING,
                    SEMANTIC_SCALE_RAIL_PADDING
                        + line_index as f32 * SEMANTIC_SCALE_RAIL_HEADER_LINE_HEIGHT,
                ),
            Align2::LEFT_TOP,
            line,
            FontId::proportional(ATLAS_SUPPORTING_TEXT_SIZE),
            ATLAS_PAPER_SIGNAL,
        );
    }
}

fn paint_grid(painter: &Painter, viewport: Rect, camera: Camera) {
    let grid_size = adaptive_grid_screen_spacing(camera.scale);
    let x_start = viewport.left() + camera.offset.x.rem_euclid(grid_size) - grid_size;
    let y_start = viewport.top() + camera.offset.y.rem_euclid(grid_size) - grid_size;
    let grid_color = Color32::from_rgba_unmultiplied(
        ATLAS_QUIET_GRAPHITE.r(),
        ATLAS_QUIET_GRAPHITE.g(),
        ATLAS_QUIET_GRAPHITE.b(),
        24,
    );
    let mut x = x_start;
    while x < viewport.right() {
        painter.line_segment(
            [
                Pos2::new(x, viewport.top()),
                Pos2::new(x, viewport.bottom()),
            ],
            Stroke::new(1.0, grid_color),
        );
        x += grid_size;
    }
    let mut y = y_start;
    while y < viewport.bottom() {
        painter.line_segment(
            [
                Pos2::new(viewport.left(), y),
                Pos2::new(viewport.right(), y),
            ],
            Stroke::new(1.0, grid_color),
        );
        y += grid_size;
    }
}

/// Keep grid cells legible at ordinary zoom, then halve their world spacing at
/// every higher zoom octave. The visible grid therefore gains useful smaller
/// subdivisions instead of becoming a few very large squares above 1000%.
fn adaptive_grid_screen_spacing(scale: f32) -> f32 {
    let scale = scale.max(f32::EPSILON);
    let high_zoom_octaves = scale.max(1.0).log2().floor().max(0.0);
    let world_spacing = 64.0 / 2.0_f32.powf(high_zoom_octaves);
    (world_spacing * scale).max(14.0)
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct CardPaintState {
    selected: bool,
    hovered: bool,
    relationship_port_hovered: Option<CardEdge>,
    resize_from_center: Option<bool>,
    layer_shifting: bool,
    detail_level: SemanticZoomLevel,
    opacity: f32,
}

fn paint_card(painter: &Painter, rect: Rect, card: &ConceptCard, state: CardPaintState) {
    let rect = card_rect_for_painting(rect);
    let color = card
        .accent_color
        .map(color_from_accent)
        .unwrap_or_else(|| color_for_kind(card.kind));
    let fill = color_with_opacity(
        Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 190),
        state.opacity,
    );
    painter.rect_filled(rect, 8.0, fill);
    painter.rect_stroke(
        rect,
        8.0,
        Stroke::new(
            card_stroke_width(state.selected),
            if state.layer_shifting {
                color_with_opacity(ATLAS_CONTROL_AMBER, state.opacity)
            } else if state.selected {
                color_with_opacity(ATLAS_ROUTE_BLUE, state.opacity)
            } else if state.hovered {
                color_with_opacity(ATLAS_PAPER_SIGNAL, state.opacity)
            } else {
                color_with_opacity(ATLAS_QUIET_GRAPHITE, state.opacity)
            },
        ),
        StrokeKind::Outside,
    );
    if rect.width() >= 42.0 && rect.height() >= 22.0 {
        let title_font = FontId::proportional(
            (ATLAS_CARD_TITLE_SIZE * (rect.width() / 240.0).clamp(0.75, 1.0)).round(),
        );
        let title_color = color_with_opacity(ATLAS_PAPER_SIGNAL, state.opacity);
        let title_clip_rect = Rect::from_min_max(
            rect.left_top() + Vec2::new(8.0, 7.0),
            Pos2::new(rect.right() - 8.0, rect.top() + 26.0),
        );
        let clipped_painter = painter.with_clip_rect(title_clip_rect);
        let title = ellipsized_text_for_width(
            painter,
            &card.title,
            title_font.clone(),
            title_color,
            title_clip_rect.width(),
        );
        clipped_painter.text(
            title_clip_rect.left_top(),
            Align2::LEFT_TOP,
            title,
            title_font,
            title_color,
        );
    }
    let summary_opacity = summary_opacity_for_screen_bounds(rect);
    if summary_opacity > 0.0 {
        let summary_color =
            color_with_opacity(ATLAS_QUIET_GRAPHITE, state.opacity * summary_opacity);
        let summary_clip_rect = Rect::from_min_max(
            rect.left_top() + Vec2::new(8.0, 30.0),
            rect.right_bottom() - Vec2::new(8.0, 7.0),
        );
        if summary_clip_rect.is_positive() {
            let clipped_painter = painter.with_clip_rect(summary_clip_rect);
            let summary = clipped_painter.layout(
                card.summary.clone(),
                FontId::proportional(
                    (ATLAS_DETAIL_BODY_SIZE * (rect.width() / 180.0).clamp(0.75, 1.0)).round(),
                ),
                summary_color,
                summary_clip_rect.width(),
            );
            clipped_painter.galley(summary_clip_rect.left_top(), summary, summary_color);
        }
    }
    let show_relationship_ports =
        state.selected || state.hovered || state.relationship_port_hovered.is_some();
    if show_relationship_ports && rect.width() >= 44.0 && rect.height() >= 28.0 {
        let card_bounds = screen_rect_to_canvas_rect(rect);
        let port_radius = 4.0;
        for (edge, point) in relationship_port_positions(card_bounds) {
            let port = Pos2::new(point.x, point.y);
            painter.circle_filled(
                port,
                if state.relationship_port_hovered == Some(edge) {
                    port_radius + 1.5
                } else {
                    port_radius
                },
                color_with_opacity(
                    if state.relationship_port_hovered == Some(edge) {
                        ATLAS_CONTROL_AMBER
                    } else {
                        ATLAS_ROUTE_BLUE
                    },
                    state.opacity,
                ),
            );
            painter.circle_stroke(
                port,
                if state.relationship_port_hovered == Some(edge) {
                    port_radius + 1.5
                } else {
                    port_radius
                },
                Stroke::new(1.0, color_with_opacity(ATLAS_PAPER_SIGNAL, state.opacity)),
            );
        }
        if state.selected {
            let handle = card_resize_handle_screen_bounds(rect);
            painter.rect_filled(
                handle,
                3.0,
                color_with_opacity(ATLAS_ROUTE_BLUE, state.opacity),
            );
            painter.rect_stroke(
                handle,
                3.0,
                Stroke::new(1.0, color_with_opacity(ATLAS_PAPER_SIGNAL, state.opacity)),
                StrokeKind::Inside,
            );
            if let Some(centered) = state.resize_from_center {
                let anchor = if centered {
                    rect.center()
                } else {
                    rect.left_top()
                };
                painter.circle_filled(
                    anchor,
                    4.0,
                    color_with_opacity(ATLAS_CONTROL_AMBER, state.opacity),
                );
                painter.circle_stroke(
                    anchor,
                    4.0,
                    Stroke::new(1.0, color_with_opacity(ATLAS_PAPER_SIGNAL, state.opacity)),
                );
            }
        }
    }
}

/// Fits a compact card title into its visual column. The canvas presents many
/// cards at once, so a short ellipsis is clearer than letting one name escape
/// its card or take over its neighbours.
fn ellipsized_text_for_width(
    painter: &Painter,
    text: &str,
    font_id: FontId,
    color: Color32,
    maximum_width: f32,
) -> String {
    if maximum_width <= 0.0 {
        return String::new();
    }
    if painter
        .layout_no_wrap(text.to_string(), font_id.clone(), color)
        .size()
        .x
        <= maximum_width
    {
        return text.to_string();
    }

    let ellipsis = "…";
    if painter
        .layout_no_wrap(ellipsis.to_string(), font_id.clone(), color)
        .size()
        .x
        > maximum_width
    {
        return String::new();
    }

    let mut fitted = String::new();
    for character in text.chars() {
        let mut candidate = fitted.clone();
        candidate.push(character);
        candidate.push_str(ellipsis);
        if painter
            .layout_no_wrap(candidate, font_id.clone(), color)
            .size()
            .x
            > maximum_width
        {
            break;
        }
        fitted.push(character);
    }
    fitted.push_str(ellipsis);
    fitted
}

fn color_for_kind(kind: ConceptKind) -> Color32 {
    match kind {
        ConceptKind::Organization => Color32::from_rgb(77, 94, 126),
        ConceptKind::Team => Color32::from_rgb(57, 111, 122),
        ConceptKind::Project => Color32::from_rgb(106, 85, 139),
        ConceptKind::Process => Color32::from_rgb(122, 94, 55),
        ConceptKind::Repository => Color32::from_rgb(74, 104, 92),
        ConceptKind::Service => Color32::from_rgb(55, 91, 145),
        ConceptKind::Environment => Color32::from_rgb(104, 89, 65),
        ConceptKind::Access => Color32::from_rgb(118, 68, 92),
    }
}

fn color_from_accent(accent: [f32; 4]) -> Color32 {
    Color32::from_rgba_premultiplied(
        (accent[0].clamp(0.0, 1.0) * 255.0) as u8,
        (accent[1].clamp(0.0, 1.0) * 255.0) as u8,
        (accent[2].clamp(0.0, 1.0) * 255.0) as u8,
        (accent[3].clamp(0.0, 1.0) * 255.0) as u8,
    )
}

fn toast_stack_routes(
    ordinary_notice_count: usize,
    has_type_change_failure: bool,
) -> Vec<ToastStackRoute> {
    let mut routes = (0..ordinary_notice_count)
        .map(ToastStackRoute::Ordinary)
        .collect::<Vec<_>>();
    if has_type_change_failure {
        routes.push(ToastStackRoute::TypeChangeFailure);
    }
    routes
}

fn remove_dismissed_notices(notices: &mut Vec<Notice>, now_seconds: f64) {
    notices.retain(|notice| {
        !matches!(
            notice.visual_state(now_seconds),
            OrdinaryNoticeVisualState::Dismissed
        )
    });
}

fn ordinary_notice_toast(ui: &mut egui::Ui, notice: &Notice, opacity: f32) -> egui::Response {
    let (fill, text_color, border) = if notice.failed {
        (
            color_with_opacity(Color32::from_rgb(63, 31, 36), opacity),
            color_with_opacity(Color32::from_rgb(255, 239, 239), opacity),
            color_with_opacity(ATLAS_RECOVERY_RED, opacity),
        )
    } else {
        (
            color_with_opacity(Color32::from_rgb(27, 57, 45), opacity),
            color_with_opacity(Color32::from_rgb(232, 255, 242), opacity),
            color_with_opacity(ATLAS_ROUTE_BLUE, opacity),
        )
    };
    egui::Frame::new()
        .fill(fill)
        .stroke(Stroke::new(1.0, border))
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(&notice.title)
                    .strong()
                    .color(text_color),
            );
            ui.add_space(4.0);
            ui.label(egui::RichText::new(&notice.detail).color(text_color));
        })
        .response
}

fn type_change_failure_toast(
    ui: &mut egui::Ui,
    error: &str,
    diagnostic_path: &Path,
    diagnostic_write_error: Option<&str>,
    opacity: f32,
) -> egui::Response {
    let fill = color_with_opacity(Color32::from_rgb(63, 31, 36), opacity);
    let text_color = color_with_opacity(Color32::from_rgb(255, 239, 239), opacity);
    let border = color_with_opacity(ATLAS_RECOVERY_RED, opacity);
    egui::Frame::new()
        .fill(fill)
        .stroke(Stroke::new(1.0, border))
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new("TYPE CHANGE NOT SAVED")
                    .strong()
                    .color(text_color),
            );
            ui.add_space(4.0);
            ui.label(egui::RichText::new(error).color(text_color));
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(format!("Diagnostic log: {}", diagnostic_path.display()))
                    .small()
                    .color(text_color),
            );
            if let Some(write_error) = diagnostic_write_error {
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(format!(
                        "The diagnostic log could not be written: {write_error}"
                    ))
                    .small()
                    .color(text_color),
                );
            }
        })
        .response
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        sync::atomic::{AtomicUsize, Ordering},
    };

    use super::*;

    static NEXT_DIAGNOSTIC_TEST_FILE: AtomicUsize = AtomicUsize::new(0);

    #[cfg(any())]
    mod removed_question_led_study_tests {
        use super::*;

        fn boundary_study_test_app() -> InfiniteNoteCanvasApp {
            InfiniteNoteCanvasApp {
                dataset: CanvasDataset::from_records(Vec::new(), Vec::new()),
                interaction: CanvasInteraction::new(Camera::default()),
                boundary_study: BoundaryStudy::for_scenario(
                    BoundaryStudyScenario::AccountResponsibility,
                ),
                boundary_study_scenario: BoundaryStudyScenario::AccountResponsibility,
                boundary_study_active: false,
                boundary_study_tool: BoundaryStudyTool::Navigate,
                boundary_study_primary_pointer_active: false,
                boundary_study_last_world_position: None,
                workspace_camera_before_study: None,
                workspace_state: WorkspaceState::ReadOnlyFailure("test workspace".to_string()),
                gesture: None,
                previous_shift_hover: None,
                link_mode: None,
                dialog: None,
                inspector_draft: None,
                semantic_zoom_layer_panel_open: false,
                semantic_zoom_layer_id: None,
                semantic_zoom_layer_draft: None,
                lens_cameras: std::collections::HashMap::new(),
                lens_canvas_controls: std::collections::HashMap::new(),
                canvas_viewport_size: Point::default(),
                fit_example_on_next_frame: false,
                relationship_route_cache: RefCell::default(),
                notices: Vec::new(),
                type_change_failure_toast: None,
                ui_time_seconds: 0.0,
            }
        }

        #[test]
        fn toast_stack_routes_keep_ordinary_notices_above_type_change_failure() {
            assert_eq!(
                toast_stack_routes(3, true),
                vec![
                    ToastStackRoute::Ordinary(0),
                    ToastStackRoute::Ordinary(1),
                    ToastStackRoute::Ordinary(2),
                    ToastStackRoute::TypeChangeFailure,
                ]
            );
        }

        #[test]
        fn toast_stack_routes_omit_type_change_failure_when_inactive() {
            assert_eq!(
                toast_stack_routes(2, false),
                vec![ToastStackRoute::Ordinary(0), ToastStackRoute::Ordinary(1)]
            );
            assert!(toast_stack_routes(0, false).is_empty());
        }

        #[test]
        fn plain_wheel_zooms_at_the_pointer() {
            let pointer = Point::new(120.0, 80.0);
            let command = wheel_canvas_command(false, pointer, Vec2::new(0.0, -40.0));

            assert!(matches!(
                command,
                CanvasCommand::ZoomAt {
                    viewport_position,
                    scale_factor,
                } if viewport_position == pointer && scale_factor > 1.0
            ));
        }

        #[test]
        fn shift_wheel_maps_to_viewport_pan() {
            assert_eq!(
                wheel_canvas_command(true, Point::new(30.0, 40.0), Vec2::new(12.0, -9.0)),
                CanvasCommand::PanBy {
                    viewport_delta: Point::new(12.0, -9.0),
                }
            );
        }

        #[test]
        fn shift_hover_maps_pointer_motion_to_pan() {
            assert_eq!(
                shift_hover_pan_command(Some(Point::new(5.0, 7.0)), Point::new(18.0, 11.0), true),
                Some(CanvasCommand::PanBy {
                    viewport_delta: Point::new(13.0, 4.0),
                })
            );
            assert_eq!(
                shift_hover_pan_command(Some(Point::default()), Point::new(1.0, 1.0), false),
                None
            );
        }

        #[test]
        fn boundary_study_primary_pointer_route_precedes_workspace_gestures() {
            assert_eq!(
                boundary_study_primary_pointer_route(false, true, true, false),
                (BoundaryStudyPrimaryPointerRoute::Down, true)
            );
            assert_eq!(
                boundary_study_primary_pointer_route(true, false, true, false),
                (BoundaryStudyPrimaryPointerRoute::Move, true)
            );
            assert_eq!(
                boundary_study_primary_pointer_route(true, false, false, true),
                (BoundaryStudyPrimaryPointerRoute::Up, false)
            );
            assert_eq!(
                boundary_study_primary_pointer_route(false, false, false, true),
                (BoundaryStudyPrimaryPointerRoute::None, false)
            );
        }

        #[test]
        fn boundary_study_toggle_preserves_in_progress_workspace_edits() {
            assert_eq!(
                boundary_study_toggle_block(true, false),
                Some(BoundaryStudyToggleBlock::Dialog)
            );
            assert_eq!(
                boundary_study_toggle_block(false, true),
                Some(BoundaryStudyToggleBlock::UnsavedInspectorDraft)
            );
            assert_eq!(boundary_study_toggle_block(false, false), None);
        }

        #[test]
        fn study_overview_camera_fits_the_broad_context_at_overview_scale() {
            let study = BoundaryStudy::representative_organization();
            let viewport_size = Point::new(620.0, 600.0);
            let camera = boundary_study_overview_camera(&study, viewport_size);
            let bounds = study.overview_world_bounds();
            let top_left = camera.world_to_screen(bounds.origin);
            let bottom_right = camera.world_to_screen(Point::new(bounds.max_x(), bounds.max_y()));

            assert_eq!(
                semantic_zoom_level(camera.scale),
                SemanticZoomLevel::Overview
            );
            assert!(top_left.x >= STUDY_OVERVIEW_PADDING - 0.01);
            assert!(top_left.y >= STUDY_OVERVIEW_PADDING - 0.01);
            assert!(bottom_right.x <= viewport_size.x - STUDY_OVERVIEW_PADDING + 0.01);
            assert!(bottom_right.y <= viewport_size.y - STUDY_OVERVIEW_PADDING + 0.01);
        }

        #[test]
        fn study_entry_starts_at_overview_and_restores_the_workspace_camera_on_exit() {
            let mut app = boundary_study_test_app();
            let workspace_camera = Camera::new(Point::new(-40.0, 90.0), 1.2);
            app.canvas_viewport_size = Point::new(620.0, 600.0);
            app.interaction.set_camera(workspace_camera);

            app.set_boundary_study_active(true);

            assert_eq!(
                semantic_zoom_level(app.interaction.camera().scale),
                SemanticZoomLevel::Overview
            );
            assert_eq!(app.workspace_camera_before_study, Some(workspace_camera));

            app.set_boundary_study_active(false);

            assert_eq!(app.interaction.camera(), workspace_camera);
            assert_eq!(app.workspace_camera_before_study, None);
        }

        #[test]
        fn boundary_study_mode_keeps_existing_dialog_and_inspector_draft_intact() {
            let mut app = boundary_study_test_app();
            app.dialog = Some(EditorDialog::Create(ConceptEditorDraft {
                title: "Unfinished concept".to_string(),
                type_id: Some("project".to_string()),
                summary: "Keep this draft.".to_string(),
            }));
            app.inspector_draft = Some(ConceptInspectorDraft {
                concept_id: 7,
                title: "Unsaved title".to_string(),
                type_id: "team".to_string(),
                summary: "Unsaved summary".to_string(),
                details: "Unsaved details".to_string(),
                assertion_attributes: BTreeMap::new(),
                assertion_key: String::new(),
                assertion_value: String::new(),
                assertion_error: None,
            });

            app.set_boundary_study_active(true);
            assert!(matches!(app.dialog, Some(EditorDialog::Create(_))));
            assert_eq!(
                app.inspector_draft
                    .as_ref()
                    .map(|draft| draft.title.as_str()),
                Some("Unsaved title")
            );

            app.set_boundary_study_active(false);
            assert!(matches!(app.dialog, Some(EditorDialog::Create(_))));
            assert_eq!(
                app.inspector_draft
                    .as_ref()
                    .map(|draft| draft.details.as_str()),
                Some("Unsaved details")
            );
        }

        #[test]
        fn reset_boundary_study_restores_the_deterministic_participant_fixture() {
            let scenario = BoundaryStudyScenario::DataReporting;
            let mut study = BoundaryStudy::for_scenario(scenario);
            let mut tool = BoundaryStudyTool::DrawVisualBoundary;
            let mut primary_pointer_active = true;
            let mut last_world_position = Some(Point::new(400.0, 320.0));
            study.apply(BoundaryStudyInput::SetTool(tool));
            study.apply(BoundaryStudyInput::PointerDown {
                world: Point::new(240.0, 180.0),
            });
            study.apply(BoundaryStudyInput::PointerUp {
                world: Point::new(520.0, 420.0),
            });
            let visible_world =
                CanvasRect::new(Point::new(-100.0, -100.0), Point::new(1_600.0, 1_200.0));
            assert_eq!(study.frame(visible_world).visual_boundaries.len(), 2);

            reset_boundary_study_state(
                &mut study,
                scenario,
                &mut tool,
                &mut primary_pointer_active,
                &mut last_world_position,
            );

            assert_eq!(tool, BoundaryStudyTool::Navigate);
            assert!(!primary_pointer_active);
            assert_eq!(last_world_position, None);
            assert_eq!(
                study.frame(visible_world),
                BoundaryStudy::for_scenario(scenario).frame(visible_world)
            );
        }

        #[test]
        fn account_lens_focus_style_takes_precedence_over_selection() {
            assert_eq!(
                boundary_study_node_paint_style(VisualNodeId(4), VisualNodeId(4), false),
                BoundaryStudyNodePaintStyle::Focused
            );
            assert_eq!(
                boundary_study_node_paint_style(VisualNodeId(4), VisualNodeId(4), true),
                BoundaryStudyNodePaintStyle::Focused
            );
            assert_eq!(
                boundary_study_node_paint_style(VisualNodeId(2), VisualNodeId(4), true),
                BoundaryStudyNodePaintStyle::Selected
            );
            assert_eq!(
                boundary_study_node_paint_style(VisualNodeId(2), VisualNodeId(4), false),
                BoundaryStudyNodePaintStyle::Ordinary
            );
        }

        #[test]
        fn study_legend_uses_frame_supplied_order_and_colors() {
            let frame = BoundaryStudy::for_scenario(BoundaryStudyScenario::DataReporting).frame(
                CanvasRect::new(Point::new(-100.0, -100.0), Point::new(1_600.0, 1_200.0)),
            );

            assert_eq!(
                frame
                    .legend_entries
                    .iter()
                    .map(|entry| entry.label.clone())
                    .collect::<Vec<_>>(),
                vec![
                    "Team".to_string(),
                    "Source service".to_string(),
                    "Pipeline".to_string(),
                    "Data product".to_string(),
                    "Reporting workspace".to_string(),
                ]
            );
            let source_service = frame
                .legend_entries
                .iter()
                .find(|entry| entry.label == "Source service")
                .expect("fixture supplies a source-service category");
            assert_eq!(
                study_legend_entry_color(source_service),
                Color32::from_rgb(73, 166, 210)
            );
        }

        #[test]
        fn study_key_and_focus_text_stay_inside_a_narrow_inspector() {
            let frame = BoundaryStudy::for_scenario(BoundaryStudyScenario::DataReporting).frame(
                CanvasRect::new(Point::new(-100.0, -100.0), Point::new(1_600.0, 1_200.0)),
            );
            let mut layout = None;
            egui::__run_test_ui(|ui| {
                ui.set_width(160.0);
                let inspector_boundary = ui.max_rect();
                let focus = study_wrapped_label(
                    ui,
                    egui::RichText::new(format!("Focus: {}", frame.focus.title)).strong(),
                );
                let key_entries = frame
                    .legend_entries
                    .iter()
                    .map(|entry| study_legend_entry(ui, entry).rect)
                    .collect::<Vec<_>>();
                layout = Some((inspector_boundary, focus.rect, key_entries));
            });

            let (inspector_boundary, focus_rect, key_entry_rects) =
                layout.expect("study layout is rendered");
            assert!(inspector_boundary.contains_rect(focus_rect));
            assert!(
                key_entry_rects
                    .iter()
                    .all(|rect| inspector_boundary.contains_rect(*rect)),
                "boundary: {inspector_boundary:?}, key entries: {key_entry_rects:?}"
            );
        }

        #[test]
        fn study_answer_sections_keep_each_fixture_order_title_and_rows() {
            let visible_world =
                CanvasRect::new(Point::new(-100.0, -100.0), Point::new(1_600.0, 1_200.0));
            let account_frame = BoundaryStudy::representative_organization().frame(visible_world);
            let reporting_frame = BoundaryStudy::for_scenario(BoundaryStudyScenario::DataReporting)
                .frame(visible_world);

            assert_eq!(
            study_answer_sections(&account_frame),
            vec![
                BoundaryStudyAnswerSection {
                    title: "Delivery path".to_string(),
                    rows: vec![
                        "Delivery Repository defines the work executed by Delivery Pipeline."
                            .to_string(),
                        "Delivery Pipeline delivers Deployment One from the repository definition."
                            .to_string(),
                        "Deployment One runs in Northstar AWS Account.".to_string(),
                    ],
                },
                BoundaryStudyAnswerSection {
                    title: "Teams with active roles".to_string(),
                    rows: vec![
                        "Development Services ART manages the delivery path through Delivery Pipeline."
                            .to_string(),
                        "Community DevSecOps validates Delivery Pipeline for security and compliance needs."
                            .to_string(),
                        "Ops and Infrastructure ART operates Northstar AWS Account for shared infrastructure."
                            .to_string(),
                    ],
                },
                BoundaryStudyAnswerSection {
                    title: "Affected context".to_string(),
                    rows: vec![
                        "Caspian Data ART depends on Northstar AWS Account for its data platform."
                            .to_string(),
                    ],
                },
            ]
        );
            assert_eq!(
                study_answer_sections(&reporting_frame),
                vec![
                    BoundaryStudyAnswerSection {
                        title: "Delivery path".to_string(),
                        rows: vec![
                        "Customer Events Service feeds customer events into Ingestion Pipeline."
                            .to_string(),
                        "Ingestion Pipeline transforms events into Curated Data Product."
                            .to_string(),
                        "Curated Data Product publishes data to Finance Reporting Workspace."
                            .to_string(),
                    ],
                    },
                    BoundaryStudyAnswerSection {
                        title: "Teams with active roles".to_string(),
                        rows: vec![
                            "Analytics Engineering Team manages Ingestion Pipeline.".to_string(),
                            "Data Platform Team operates Curated Data Product.".to_string(),
                            "Data Governance Team validates Curated Data Product.".to_string(),
                        ],
                    },
                    BoundaryStudyAnswerSection {
                        title: "Affected context".to_string(),
                        rows: vec![
                            "Finance Analytics depends on Finance Reporting Workspace.".to_string()
                        ],
                    },
                ]
            );
        }

        #[test]
        fn study_answer_sections_do_not_classify_fixture_supplied_titles() {
            let mut frame = BoundaryStudy::representative_organization().frame(CanvasRect::new(
                Point::new(-100.0, -100.0),
                Point::new(1_600.0, 1_200.0),
            ));
            frame.answer_sections[1].title = "Delivery stewards".to_string();

            assert_eq!(study_answer_sections(&frame)[1].title, "Delivery stewards");
        }

        #[test]
        fn study_hides_empty_answer_sections() {
            assert!(!study_answer_section_is_visible(&[]));
            assert!(study_answer_section_is_visible(&["One answer".to_string()]));
        }

        #[test]
        fn study_connection_labels_cross_fade_from_short_to_detailed_text() {
            assert_eq!(
                boundary_study_connection_labels(
                    SemanticZoomLevel::Overview,
                    0.0,
                    "depends on",
                    "Northstar AWS Account supports its data platform."
                ),
                Vec::<(String, f32)>::new()
            );
            assert_eq!(
                boundary_study_connection_labels(
                    SemanticZoomLevel::Card,
                    0.0,
                    "depends on",
                    "Northstar AWS Account supports its data platform."
                ),
                vec![("depends on".to_string(), 1.0)]
            );
            assert_eq!(
                boundary_study_connection_labels(
                    SemanticZoomLevel::Detail,
                    1.0,
                    "depends on",
                    "Northstar AWS Account supports its data platform."
                ),
                vec![(
                    "depends on. Northstar AWS Account supports its data platform.".to_string(),
                    1.0
                )]
            );
            assert_eq!(
                boundary_study_connection_labels(
                    SemanticZoomLevel::Detail,
                    0.5,
                    "depends on",
                    "Northstar AWS Account supports its data platform."
                ),
                vec![
                    ("depends on".to_string(), 0.5),
                    (
                        "depends on. Northstar AWS Account supports its data platform.".to_string(),
                        0.5
                    )
                ]
            );
        }

        #[test]
        fn study_connection_arrowheads_are_centered_and_point_from_source_to_target() {
            assert_eq!(
                boundary_study_connection_arrowhead(Pos2::new(0.0, 0.0), Pos2::new(100.0, 0.0)),
                Some([
                    Pos2::new(50.0, 0.0),
                    Pos2::new(42.0, 4.0),
                    Pos2::new(42.0, -4.0),
                ])
            );
            assert_eq!(
                boundary_study_connection_arrowhead(Pos2::new(0.0, 0.0), Pos2::new(0.0, 100.0)),
                Some([
                    Pos2::new(0.0, 50.0),
                    Pos2::new(-4.0, 42.0),
                    Pos2::new(4.0, 42.0),
                ])
            );
        }

        #[test]
        fn study_connection_arrowhead_handles_diagonal_short_and_degenerate_connections() {
            let arrowhead =
                boundary_study_connection_arrowhead(Pos2::new(0.0, 0.0), Pos2::new(100.0, 100.0))
                    .expect("diagonal connections have a directional arrowhead");
            let direction = Pos2::new(100.0, 100.0) - Pos2::new(0.0, 0.0);

            assert!((arrowhead[0].x - 50.0).abs() < 0.001);
            assert!((arrowhead[0].y - 50.0).abs() < 0.001);
            assert!(direction.dot(arrowhead[1] - arrowhead[0]) < 0.0);
            assert!(direction.dot(arrowhead[2] - arrowhead[0]) < 0.0);
            assert_eq!(
                boundary_study_connection_arrowhead(Pos2::new(0.0, 0.0), Pos2::new(12.0, 0.0)),
                Some([
                    Pos2::new(6.0, 0.0),
                    Pos2::new(3.6, 1.2),
                    Pos2::new(3.6, -1.2),
                ])
            );
            assert_eq!(
                boundary_study_connection_arrowhead(Pos2::new(12.0, 24.0), Pos2::new(12.0, 24.0)),
                None
            );
        }

        #[test]
        fn study_connection_label_position_stays_clear_of_the_centered_arrowhead() {
            let source = Pos2::new(0.0, 0.0);
            let target = Pos2::new(100.0, 0.0);
            let arrowhead = boundary_study_connection_arrowhead(source, target)
                .expect("non-degenerate connections have an arrowhead");
            let label_position = boundary_study_connection_label_position(source, target);

            assert_eq!(arrowhead[0], Pos2::new(50.0, 0.0));
            assert_eq!(
                label_position,
                Pos2::new(50.0, STUDY_CONNECTION_LABEL_OFFSET)
            );
            assert!(
                (label_position - arrowhead[0]).length() > STUDY_CONNECTION_ARROW_MAXIMUM_LENGTH
            );
        }

        #[test]
        fn study_crossing_cue_uses_connection_color_without_a_separate_marker() {
            assert_eq!(
                boundary_study_connection_color(false),
                STUDY_CONNECTION_COLOR
            );
            assert_eq!(
                boundary_study_connection_color(true),
                STUDY_CROSSING_CONNECTION_COLOR
            );
        }

        #[test]
        fn study_selector_exposes_temporary_scenarios_and_reset_without_draw_boundary_control() {
            assert_eq!(
                BoundaryStudyScenario::ALL
                    .into_iter()
                    .map(BoundaryStudyScenario::label)
                    .collect::<Vec<_>>(),
                vec![
                    "Account responsibility",
                    "Data reporting",
                    "Release controls",
                    "Workspace provisioning"
                ]
            );
            assert_eq!(STUDY_LENS_LABEL, "Question-led study");
            assert_eq!(STUDY_LENS_CONTROLS, ["Show overview", "Reset study"]);
            assert!(!STUDY_LENS_CONTROLS.contains(&"Draw visual boundary"));
        }

        #[test]
        #[test]
        fn selecting_a_study_scenario_rebuilds_only_temporary_study_state() {
            let mut app = boundary_study_test_app();
            app.boundary_study_tool = BoundaryStudyTool::DrawVisualBoundary;
            app.boundary_study_primary_pointer_active = true;
            app.boundary_study_last_world_position = Some(Point::new(400.0, 320.0));
            app.boundary_study.apply(BoundaryStudyInput::SetTool(
                BoundaryStudyTool::DrawVisualBoundary,
            ));
            app.boundary_study.apply(BoundaryStudyInput::PointerDown {
                world: Point::new(240.0, 180.0),
            });
            app.dialog = Some(EditorDialog::Create(ConceptEditorDraft {
                title: "Keep this workspace draft".to_string(),
                type_id: Some("project".to_string()),
                summary: "The study selector must not discard this.".to_string(),
            }));
            app.inspector_draft = Some(ConceptInspectorDraft {
                concept_id: 7,
                title: "Keep this inspector draft".to_string(),
                type_id: "team".to_string(),
                summary: "The selector must not discard this either.".to_string(),
                details: "Unsaved workspace detail".to_string(),
                assertion_attributes: BTreeMap::new(),
                assertion_key: String::new(),
                assertion_value: String::new(),
                assertion_error: None,
            });
            app.link_mode = Some(LinkMode {
                source_id: 9,
                kind: RelationshipKind::DependsOn,
                kind_query: "depends".to_string(),
            });

            app.select_boundary_study_scenario(BoundaryStudyScenario::ReleaseControls);

            assert_eq!(
                app.boundary_study_scenario,
                BoundaryStudyScenario::ReleaseControls
            );
            assert_eq!(
                app.boundary_study.scenario(),
                BoundaryStudyScenario::ReleaseControls
            );
            assert_eq!(app.boundary_study_tool, BoundaryStudyTool::Navigate);
            assert!(!app.boundary_study_primary_pointer_active);
            assert_eq!(app.boundary_study_last_world_position, None);
            let visible_world =
                CanvasRect::new(Point::new(-100.0, -100.0), Point::new(1_600.0, 1_200.0));
            let selected_frame = app.boundary_study.frame(visible_world);
            let core_frame = BoundaryStudy::for_scenario(BoundaryStudyScenario::ReleaseControls)
                .frame(visible_world);
            assert_eq!(selected_frame.scenario_title, "Release controls lens");
            assert_eq!(selected_frame.legend_entries, core_frame.legend_entries);
            assert_eq!(
                study_answer_sections(&selected_frame),
                study_answer_sections(&core_frame)
            );
            assert!(matches!(app.dialog, Some(EditorDialog::Create(_))));
            assert_eq!(
                app.inspector_draft
                    .as_ref()
                    .map(|draft| draft.details.as_str()),
                Some("Unsaved workspace detail")
            );
            assert_eq!(
                app.link_mode.as_ref().map(|link_mode| link_mode.source_id),
                Some(9)
            );
        }

        #[test]
        fn resetting_a_study_keeps_the_selected_scenario() {
            let mut app = boundary_study_test_app();
            app.select_boundary_study_scenario(BoundaryStudyScenario::DataReporting);
            app.boundary_study.apply(BoundaryStudyInput::PointerDown {
                world: Point::new(260.0, 180.0),
            });
            app.boundary_study.apply(BoundaryStudyInput::PointerMove {
                world: Point::new(320.0, 220.0),
            });

            app.reset_boundary_study();

            assert_eq!(
                app.boundary_study_scenario,
                BoundaryStudyScenario::DataReporting
            );
            assert_eq!(
                app.boundary_study.scenario(),
                BoundaryStudyScenario::DataReporting
            );
        }

        #[test]
        fn boundary_study_geometry_uses_the_existing_world_to_screen_transform() {
            let viewport = Rect::from_min_size(Pos2::new(32.0, 48.0), Vec2::new(800.0, 600.0));
            let camera = Camera::new(Point::new(80.0, 120.0), 1.5);
            let bounds = CanvasRect::new(Point::new(40.0, 60.0), Point::new(200.0, 80.0));

            assert_eq!(
                canvas_rect_to_screen(bounds, camera, viewport),
                Rect::from_min_size(Pos2::new(172.0, 258.0), Vec2::new(300.0, 120.0))
            );
            assert_eq!(
                canvas_point_to_screen(Point::new(40.0, 60.0), camera, viewport),
                Pos2::new(172.0, 258.0)
            );
        }
    }

    #[test]
    fn normal_navigation_never_switches_to_overview_mode() {
        assert_eq!(card_detail_level(0.01), SemanticZoomLevel::Card);
        assert_eq!(card_detail_level(0.2), SemanticZoomLevel::Card);
        assert_eq!(card_detail_level(1.0), SemanticZoomLevel::Card);
        assert_eq!(card_detail_level(2.0), SemanticZoomLevel::Detail);
    }

    #[test]
    fn boundary_pointer_target_prefers_the_resize_handle_and_keeps_visual_bounds() {
        let boundary = VisualBoundary {
            layer_id: None,
            id: "boundary-release-context".to_string(),
            title: "Release context".to_string(),
            summary: String::new(),
            bounds: CanvasRect::new(Point::new(100.0, 80.0), Point::new(400.0, 260.0)),
        };

        assert_eq!(
            boundary_pointer_target(
                std::slice::from_ref(&boundary),
                Point::new(498.0, 338.0),
                12.0
            ),
            Some(BoundaryPointerTarget::Resize {
                boundary_id: boundary.id.clone(),
                initial_bounds: boundary.bounds,
            })
        );
        assert_eq!(
            boundary_pointer_target(&[boundary], Point::new(140.0, 160.0), 12.0),
            Some(BoundaryPointerTarget::Move {
                boundary_id: "boundary-release-context".to_string(),
            })
        );
    }

    #[test]
    fn boundary_pointer_target_prefers_the_smallest_nested_boundary() {
        let child = VisualBoundary {
            layer_id: None,
            id: "boundary-child".to_string(),
            title: "Child".to_string(),
            summary: String::new(),
            bounds: CanvasRect::new(Point::new(160.0, 140.0), Point::new(180.0, 120.0)),
        };
        let parent = VisualBoundary {
            layer_id: None,
            id: "boundary-parent".to_string(),
            title: "Parent".to_string(),
            summary: String::new(),
            bounds: CanvasRect::new(Point::new(0.0, 0.0), Point::new(800.0, 600.0)),
        };

        assert_eq!(
            boundary_pointer_target(&[child.clone(), parent], Point::new(220.0, 180.0), 12.0),
            Some(BoundaryPointerTarget::Move {
                boundary_id: child.id,
            })
        );
    }

    #[test]
    fn enclosing_context_boundaries_do_not_receive_direct_manipulation() {
        let context = VisualBoundary {
            layer_id: None,
            id: "boundary-context".to_string(),
            title: "Organization".to_string(),
            summary: String::new(),
            bounds: CanvasRect::new(Point::new(0.0, 0.0), Point::new(1_000.0, 800.0)),
        };
        let child = VisualBoundary {
            layer_id: None,
            id: "boundary-child".to_string(),
            title: "Runtime".to_string(),
            summary: String::new(),
            bounds: CanvasRect::new(Point::new(120.0, 100.0), Point::new(220.0, 180.0)),
        };
        let visible_world = CanvasRect::new(Point::new(160.0, 140.0), Point::new(300.0, 250.0));

        assert_eq!(
            interactive_visual_boundaries(&[context, child.clone()], visible_world)
                .into_iter()
                .map(|boundary| boundary.id.as_str())
                .collect::<Vec<_>>(),
            vec![child.id.as_str()]
        );
    }

    #[test]
    fn axis_covering_context_boundaries_leave_nested_children_interactive() {
        let visible_world = CanvasRect::new(Point::new(160.0, 140.0), Point::new(300.0, 250.0));
        let horizontal_context = VisualBoundary {
            layer_id: None,
            id: "boundary-horizontal-context".to_string(),
            title: "Release lane".to_string(),
            summary: String::new(),
            bounds: CanvasRect::new(Point::new(0.0, 160.0), Point::new(800.0, 150.0)),
        };
        let horizontal_child = VisualBoundary {
            layer_id: None,
            id: "boundary-horizontal-child".to_string(),
            title: "Release gate".to_string(),
            summary: String::new(),
            bounds: CanvasRect::new(Point::new(180.0, 170.0), Point::new(100.0, 30.0)),
        };
        let vertical_context = VisualBoundary {
            layer_id: None,
            id: "boundary-vertical-context".to_string(),
            title: "Operations lane".to_string(),
            summary: String::new(),
            bounds: CanvasRect::new(Point::new(180.0, 0.0), Point::new(180.0, 800.0)),
        };
        let vertical_child = VisualBoundary {
            layer_id: None,
            id: "boundary-vertical-child".to_string(),
            title: "Operations gate".to_string(),
            summary: String::new(),
            bounds: CanvasRect::new(Point::new(190.0, 160.0), Point::new(20.0, 50.0)),
        };

        assert!(boundary_supplies_viewport_context(
            horizontal_context.bounds,
            visible_world
        ));
        assert!(boundary_supplies_viewport_context(
            vertical_context.bounds,
            visible_world
        ));
        assert_eq!(
            interactive_visual_boundaries(
                &[horizontal_context, horizontal_child.clone()],
                visible_world
            )
            .into_iter()
            .map(|boundary| boundary.id.as_str())
            .collect::<Vec<_>>(),
            vec![horizontal_child.id.as_str()]
        );
        let interactive_vertical = interactive_visual_boundaries(
            &[vertical_context, vertical_child.clone()],
            visible_world,
        )
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();
        assert_eq!(
            interactive_vertical
                .iter()
                .map(|boundary| boundary.id.as_str())
                .collect::<Vec<_>>(),
            vec![vertical_child.id.as_str()]
        );
        assert_eq!(
            boundary_pointer_target(&interactive_vertical, Point::new(200.0, 180.0), 12.0),
            Some(BoundaryPointerTarget::Move {
                boundary_id: vertical_child.id,
            })
        );
    }

    #[test]
    fn boundary_resize_preview_preserves_the_anchor_and_minimum_size() {
        let initial = CanvasRect::new(Point::new(100.0, 80.0), Point::new(400.0, 260.0));
        assert_eq!(
            resized_boundary_bounds(initial, Point::new(60.0, 40.0)),
            CanvasRect::new(Point::new(100.0, 80.0), Point::new(460.0, 300.0))
        );
        assert_eq!(
            resized_boundary_bounds(initial, Point::new(-900.0, -900.0)),
            CanvasRect::new(
                Point::new(100.0, 80.0),
                Point::new(MINIMUM_BOUNDARY_SIZE.x, MINIMUM_BOUNDARY_SIZE.y)
            )
        );
    }

    #[test]
    fn boundary_preview_uses_only_the_active_boundary_gesture() {
        let boundary = VisualBoundary {
            layer_id: None,
            id: "boundary-release-context".to_string(),
            title: "Release context".to_string(),
            summary: String::new(),
            bounds: CanvasRect::new(Point::new(100.0, 80.0), Point::new(400.0, 260.0)),
        };
        let gesture = PointerGesture::Move {
            item_ids: vec![CanvasItemId::Boundary(boundary.id.clone())],
            anchor_world_position: Point::new(100.0, 80.0),
            preview_world_delta: Point::new(24.0, -16.0),
        };
        assert_eq!(
            boundary_preview_bounds(&boundary, Some(&gesture)),
            CanvasRect::new(Point::new(124.0, 64.0), Point::new(400.0, 260.0))
        );
        let other = VisualBoundary {
            layer_id: None,
            id: "boundary-other".to_string(),
            ..boundary.clone()
        };
        assert_eq!(
            boundary_preview_bounds(&other, Some(&gesture)),
            other.bounds
        );
    }

    #[test]
    fn boundary_move_preview_moves_nested_boundaries_but_not_intersecting_siblings() {
        let outer = VisualBoundary {
            layer_id: None,
            id: "boundary-outer".to_string(),
            title: "Outer".to_string(),
            summary: String::new(),
            bounds: CanvasRect::new(Point::new(100.0, 80.0), Point::new(600.0, 420.0)),
        };
        let nested = VisualBoundary {
            layer_id: None,
            id: "boundary-nested".to_string(),
            title: "Nested".to_string(),
            summary: String::new(),
            bounds: CanvasRect::new(Point::new(180.0, 140.0), Point::new(220.0, 140.0)),
        };
        let intersecting = VisualBoundary {
            layer_id: None,
            id: "boundary-intersecting".to_string(),
            title: "Intersecting".to_string(),
            summary: String::new(),
            bounds: CanvasRect::new(Point::new(620.0, 140.0), Point::new(220.0, 140.0)),
        };
        let gesture = PointerGesture::Move {
            item_ids: vec![
                CanvasItemId::Boundary(outer.id.clone()),
                CanvasItemId::Boundary(nested.id.clone()),
            ],
            anchor_world_position: outer.bounds.origin,
            preview_world_delta: Point::new(32.0, -24.0),
        };

        assert_eq!(
            boundary_preview_bounds(&nested, Some(&gesture)),
            CanvasRect::new(Point::new(212.0, 116.0), nested.bounds.size)
        );
        assert_eq!(
            boundary_preview_bounds(&intersecting, Some(&gesture)),
            intersecting.bounds
        );
    }

    #[test]
    fn boundary_move_preview_moves_only_fully_enclosed_cards() {
        let contained_bounds = CanvasRect::new(Point::new(140.0, 120.0), Point::new(80.0, 48.0));
        let independent_bounds = CanvasRect::new(Point::new(360.0, 120.0), Point::new(80.0, 48.0));
        let gesture = PointerGesture::Move {
            item_ids: vec![
                CanvasItemId::Boundary("boundary-runtime".to_string()),
                CanvasItemId::Card(4),
            ],
            anchor_world_position: Point::new(120.0, 100.0),
            preview_world_delta: Point::new(32.0, -24.0),
        };

        assert_eq!(
            move_preview_bounds(contained_bounds, 4, Some(&gesture)),
            CanvasRect::new(Point::new(172.0, 96.0), contained_bounds.size)
        );
        assert_eq!(
            move_preview_bounds(independent_bounds, 5, Some(&gesture)),
            independent_bounds
        );
    }

    #[test]
    fn summaries_follow_display_size_independently_of_camera_scale() {
        let roomy_bounds = Rect::from_min_size(Pos2::new(40.0, 80.0), Vec2::new(240.0, 120.0));

        assert!(should_paint_boundary_summary(
            0.31,
            roomy_bounds,
            "Runtime core and infrastructure."
        ));
        assert!(should_paint_boundary_summary(
            1.49,
            roomy_bounds,
            "Runtime core and infrastructure."
        ));
        assert!(should_paint_boundary_summary(
            1.5,
            roomy_bounds,
            "Runtime core and infrastructure."
        ));
        assert!(!should_paint_boundary_summary(
            2.0,
            Rect::from_min_size(Pos2::new(40.0, 80.0), Vec2::new(120.0, 120.0)),
            "Runtime core and infrastructure."
        ));
    }

    #[test]
    fn summary_fade_uses_displayed_size_continuously() {
        let bounds = |width, height| Rect::from_min_size(Pos2::ZERO, Vec2::new(width, height));
        assert_eq!(summary_opacity_for_screen_bounds(bounds(120.0, 80.0)), 0.0);
        assert!((summary_opacity_for_screen_bounds(bounds(150.0, 80.0)) - 0.5).abs() < 0.001);
        assert_eq!(summary_opacity_for_screen_bounds(bounds(180.0, 80.0)), 1.0);
        assert!(summary_opacity_for_screen_bounds(bounds(180.0, 66.0)) < 1.0);
        assert_eq!(summary_opacity_for_screen_bounds(bounds(180.0, 52.0)), 0.0);
    }

    #[test]
    fn relationship_route_stays_visible_when_both_endpoint_cards_are_offscreen() {
        let endpoints = BTreeMap::from([
            (
                4,
                CanvasRect::new(Point::new(-240.0, 40.0), Point::new(120.0, 80.0)),
            ),
            (
                5,
                CanvasRect::new(Point::new(320.0, 40.0), Point::new(120.0, 80.0)),
            ),
        ]);
        let viewport = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 160.0));
        let relationships = vec![RelationshipView {
            id: "crosses-viewport".to_string(),
            source_id: 4,
            target_id: 5,
            kind: RelationshipKind::DependsOn,
            source_title: "Source".to_string(),
            target_title: "Target".to_string(),
        }];

        let paths =
            relationship_paint_paths(&relationships, &endpoints, &BTreeMap::new(), &[], viewport);

        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].visible_segments.len(), 1);
        assert_eq!(
            paths[0].visible_segments[0],
            vec![Pos2::new(0.0, 80.0), Pos2::new(200.0, 80.0)]
        );
    }

    #[test]
    fn parent_child_relationship_does_not_render_a_path() {
        let parent_bounds = CanvasRect::new(Point::new(0.0, 0.0), Point::new(400.0, 300.0));
        let child_bounds = CanvasRect::new(Point::new(220.0, 160.0), Point::new(120.0, 72.0));
        let endpoints = BTreeMap::from([(4, parent_bounds), (5, child_bounds)]);
        let relationships = vec![RelationshipView {
            id: "parent-to-child".to_string(),
            source_id: 4,
            target_id: 5,
            kind: RelationshipKind::DependsOn,
            source_title: "Parent".to_string(),
            target_title: "Child".to_string(),
        }];
        let viewport = Rect::from_min_size(Pos2::ZERO, Vec2::new(480.0, 360.0));

        let paths = relationship_paint_paths(&relationships, &endpoints, &endpoints, &[], viewport);

        assert!(paths.is_empty());
    }

    #[test]
    fn relationship_route_outside_the_viewport_has_no_painted_segment() {
        let endpoints = BTreeMap::from([
            (
                4,
                CanvasRect::new(Point::new(-440.0, 40.0), Point::new(120.0, 80.0)),
            ),
            (
                5,
                CanvasRect::new(Point::new(-240.0, 40.0), Point::new(120.0, 80.0)),
            ),
        ]);
        let viewport = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 160.0));
        let relationships = vec![RelationshipView {
            id: "outside-viewport".to_string(),
            source_id: 4,
            target_id: 5,
            kind: RelationshipKind::DependsOn,
            source_title: "Source".to_string(),
            target_title: "Target".to_string(),
        }];

        assert!(
            relationship_paint_paths(&relationships, &endpoints, &BTreeMap::new(), &[], viewport,)
                .is_empty()
        );
    }

    #[test]
    fn clipping_keeps_route_segments_separate_across_an_invisible_gap() {
        let viewport = Rect::from_min_size(Pos2::ZERO, Vec2::new(100.0, 100.0));
        let points = vec![
            Pos2::new(-50.0, 10.0),
            Pos2::new(10.0, 10.0),
            Pos2::new(-50.0, 10.0),
            Pos2::new(-50.0, 30.0),
            Pos2::new(10.0, 30.0),
        ];

        assert_eq!(
            clipped_path_segments(&points, viewport),
            vec![
                vec![
                    Pos2::new(0.0, 10.0),
                    Pos2::new(10.0, 10.0),
                    Pos2::new(0.0, 10.0)
                ],
                vec![Pos2::new(0.0, 30.0), Pos2::new(10.0, 30.0)],
            ]
        );
    }

    #[test]
    fn canvas_context_includes_enclosing_boundary_and_containing_card_in_order() {
        let boundaries = vec![VisualBoundary {
            layer_id: None,
            id: "boundary-outer".to_string(),
            title: "Platform landscape".to_string(),
            summary: String::new(),
            bounds: CanvasRect::new(Point::new(0.0, 0.0), Point::new(1_000.0, 1_000.0)),
        }];
        let mut card = test_card(Point::new(100.0, 100.0));
        card.title = "Delivery domain".to_string();
        card.bounds.size = Point::new(700.0, 700.0);
        let cards = [card];
        let visible_world = CanvasRect::new(Point::new(300.0, 300.0), Point::new(120.0, 100.0));

        let entries = canvas_context_entries(
            &boundaries,
            cards.iter(),
            &BTreeMap::from([(4, "Runtime detail".to_string())]),
            None,
            visible_world,
        );

        assert_eq!(
            entries
                .iter()
                .map(|entry| (canvas_context_entry_label(entry), entry.depth))
                .collect::<Vec<_>>(),
            vec![
                ("Boundary: Platform landscape".to_string(), 0),
                ("Delivery domain (Runtime detail)".to_string(), 1),
            ]
        );
    }

    #[test]
    fn canvas_context_excludes_containers_that_only_intersect_the_viewport() {
        let boundaries = vec![VisualBoundary {
            layer_id: None,
            id: "boundary-overlap".to_string(),
            title: "Partly visible boundary".to_string(),
            summary: String::new(),
            bounds: CanvasRect::new(Point::new(0.0, 0.0), Point::new(250.0, 270.0)),
        }];
        let mut card = test_card(Point::new(220.0, 220.0));
        card.title = "Partly visible card".to_string();
        card.bounds.size = Point::new(100.0, 100.0);
        let cards = [card];
        let visible_world = CanvasRect::new(Point::new(200.0, 200.0), Point::new(100.0, 100.0));

        assert!(
            canvas_context_entries(
                &boundaries,
                cards.iter(),
                &BTreeMap::new(),
                None,
                visible_world,
            )
            .is_empty()
        );
    }

    #[test]
    fn canvas_context_excludes_an_axis_covering_boundary_below_half_of_the_viewport() {
        let boundary = VisualBoundary {
            layer_id: None,
            id: "boundary-axis-context".to_string(),
            title: "Shared platform".to_string(),
            summary: String::new(),
            bounds: CanvasRect::new(Point::new(0.0, 220.0), Point::new(500.0, 40.0)),
        };
        let visible_world = CanvasRect::new(Point::new(200.0, 200.0), Point::new(100.0, 100.0));

        assert!(
            canvas_context_entries(
                &[boundary],
                std::iter::empty(),
                &BTreeMap::new(),
                None,
                visible_world,
            )
            .is_empty()
        );
    }

    #[test]
    fn canvas_context_includes_an_axis_covering_boundary_that_fills_half_the_viewport() {
        let boundary = VisualBoundary {
            layer_id: None,
            id: "boundary-half-context".to_string(),
            title: "Shared platform".to_string(),
            summary: String::new(),
            bounds: CanvasRect::new(Point::new(0.0, 220.0), Point::new(500.0, 50.0)),
        };
        let visible_world = CanvasRect::new(Point::new(200.0, 200.0), Point::new(100.0, 100.0));

        assert_eq!(
            canvas_context_entries(
                &[boundary],
                std::iter::empty(),
                &BTreeMap::new(),
                None,
                visible_world,
            )
            .into_iter()
            .map(|entry| entry.title)
            .collect::<Vec<_>>(),
            vec!["Shared platform"]
        );
    }

    #[test]
    fn canvas_context_treats_a_card_boundary_like_a_visual_boundary() {
        let mut card = test_card(Point::new(0.0, 220.0));
        card.title = "Delivery domain".to_string();
        card.bounds.size = Point::new(500.0, 50.0);
        let visible_world = CanvasRect::new(Point::new(200.0, 200.0), Point::new(100.0, 100.0));

        assert_eq!(
            canvas_context_entries(&[], [&card], &BTreeMap::new(), None, visible_world,)
                .into_iter()
                .map(|entry| canvas_context_entry_label(&entry))
                .collect::<Vec<_>>(),
            vec!["Delivery domain"]
        );
    }

    #[test]
    fn context_trail_cards_are_not_selectable_but_nested_cards_remain_interactive() {
        let mut app = test_app();
        let mut parent = test_card(Point::new(0.0, 0.0));
        parent.bounds.size = Point::new(600.0, 480.0);
        let mut child = test_card(Point::new(120.0, 100.0));
        child.id = 5;
        child.bounds.size = Point::new(120.0, 80.0);
        app.dataset = CanvasDataset::from_records(vec![parent, child], Vec::new());
        app.canvas_viewport_size = Point::new(300.0, 250.0);
        app.interaction
            .set_camera(Camera::new(Point::default(), 1.0));

        assert_eq!(app.context_trail_card_ids(), BTreeSet::from([4]));
        assert_eq!(app.filtered_canvas_hit_test(Point::new(40.0, 40.0)), None);
        assert_eq!(
            app.filtered_canvas_hit_test(Point::new(160.0, 140.0)),
            Some(5)
        );

        app.interaction.select_only_card_id(4);
        app.clear_noninteractive_canvas_selection();

        assert!(app.interaction.selected_card_ids().next().is_none());
    }

    #[test]
    fn context_entities_fade_by_their_area_ratio_to_the_visible_canvas() {
        let viewport = CanvasRect::new(Point::new(0.0, 0.0), Point::new(100.0, 100.0));
        let viewport_sized_parent = viewport;
        let four_viewport_parent = CanvasRect::new(Point::new(0.0, 0.0), Point::new(200.0, 200.0));
        let sixteen_viewport_parent =
            CanvasRect::new(Point::new(0.0, 0.0), Point::new(400.0, 400.0));

        assert_eq!(
            context_entity_opacity(1.0, Some(viewport_sized_parent), viewport),
            0.25
        );
        assert!(
            (context_entity_opacity(1.0, Some(four_viewport_parent), viewport) - 0.10).abs()
                < 0.000_1
        );
        assert!(
            (context_entity_opacity(1.0, Some(sixteen_viewport_parent), viewport) - 0.0625).abs()
                < 0.000_1
        );
        assert_eq!(context_entity_opacity(0.6, None, viewport), 0.6);
    }

    #[test]
    fn context_entity_opacity_is_continuous_before_the_trail_threshold() {
        let viewport = CanvasRect::new(Point::new(0.0, 0.0), Point::new(100.0, 100.0));
        let just_before = CanvasRect::new(Point::new(0.0, 0.0), Point::new(100.0, 49.0));
        let just_after = CanvasRect::new(Point::new(0.0, 0.0), Point::new(100.0, 51.0));

        let before_opacity = context_entity_opacity(1.0, Some(just_before), viewport);
        let after_opacity = context_entity_opacity(1.0, Some(just_after), viewport);

        assert!(before_opacity < 1.0);
        assert!(after_opacity < before_opacity);
        assert!((before_opacity - after_opacity) < 0.1);
    }

    #[test]
    fn canvas_context_accepts_a_container_equal_to_the_viewport() {
        let boundary = VisualBoundary {
            layer_id: None,
            id: "boundary-equal".to_string(),
            title: "Exact context".to_string(),
            summary: String::new(),
            bounds: CanvasRect::new(Point::new(200.0, 200.0), Point::new(100.0, 100.0)),
        };
        let visible_world = boundary.bounds;

        assert_eq!(
            canvas_context_entries(
                &[boundary],
                std::iter::empty(),
                &BTreeMap::new(),
                None,
                visible_world,
            )
            .into_iter()
            .map(|entry| entry.title)
            .collect::<Vec<_>>(),
            vec!["Exact context"]
        );
    }

    #[test]
    fn relationship_paths_route_around_unrelated_boundaries() {
        let displayed_card_bounds_by_concept_id = BTreeMap::from([
            (
                4,
                CanvasRect::new(Point::new(0.0, 0.0), Point::new(100.0, 60.0)),
            ),
            (
                5,
                CanvasRect::new(Point::new(500.0, 0.0), Point::new(100.0, 60.0)),
            ),
        ]);
        let unrelated_boundary =
            Rect::from_min_size(Pos2::new(220.0, -40.0), Vec2::new(160.0, 140.0));
        let route = routed_relationship_points(
            4,
            5,
            &displayed_card_bounds_by_concept_id,
            &displayed_card_bounds_by_concept_id,
            &[unrelated_boundary],
        )
        .unwrap();

        assert!(route.len() > 2);
        assert_path_stays_outside_painted_card(&route, unrelated_boundary);

        let target_boundary = Rect::from_min_size(Pos2::new(460.0, -20.0), Vec2::new(180.0, 100.0));
        let direct_route = routed_relationship_points(
            4,
            5,
            &displayed_card_bounds_by_concept_id,
            &displayed_card_bounds_by_concept_id,
            &[target_boundary],
        )
        .unwrap();

        assert_eq!(direct_route.len(), 2);
    }

    #[test]
    fn unlayered_semantic_scale_rail_uses_the_existing_camera_thresholds() {
        assert_eq!(camera_scale_band(0.01), SemanticScaleBand::Broad);
        assert_eq!(camera_scale_band(0.649), SemanticScaleBand::Broad);
        assert_eq!(camera_scale_band(0.65), SemanticScaleBand::Working);
        assert_eq!(camera_scale_band(1.499), SemanticScaleBand::Working);
        assert_eq!(camera_scale_band(1.5), SemanticScaleBand::Detail);
    }

    #[test]
    fn semantic_scale_rail_keeps_its_bounds_stable_across_active_bands() {
        let viewport = Rect::from_min_size(Pos2::new(24.0, 32.0), Vec2::new(800.0, 600.0));
        let broad = semantic_scale_rail(viewport, 0.64, &[]);
        let working = semantic_scale_rail(viewport, 0.65, &[]);
        let detail = semantic_scale_rail(viewport, 1.5, &[]);

        assert_eq!(broad.source, SemanticScaleRailSource::Camera);
        assert_eq!(broad.active_band, Some(SemanticScaleBand::Broad));
        assert_eq!(working.active_band, Some(SemanticScaleBand::Working));
        assert_eq!(detail.active_band, Some(SemanticScaleBand::Detail));
        assert_eq!(broad.bounds, working.bounds);
        assert_eq!(working.bounds, detail.bounds);
        assert!(viewport.contains_rect(detail.bounds));
        assert_eq!(
            broad
                .bands
                .iter()
                .map(|band| band.bounds)
                .collect::<Vec<_>>(),
            working
                .bands
                .iter()
                .map(|band| band.bounds)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            working
                .bands
                .iter()
                .map(|band| band.bounds)
                .collect::<Vec<_>>(),
            detail
                .bands
                .iter()
                .map(|band| band.bounds)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn zoom_slider_uses_the_full_logarithmic_camera_range_below_the_layer_bands() {
        let rail = semantic_scale_rail(
            Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(640.0, 480.0)),
            8.22,
            &[],
        );

        assert_eq!(zoom_slider_fraction(MIN_SCALE), 0.0);
        assert_eq!(zoom_slider_fraction(MAX_SCALE), 1.0);
        assert!((zoom_scale_from_slider_fraction(0.0) - MIN_SCALE).abs() < 0.000_1);
        assert!((zoom_scale_from_slider_fraction(1.0) - MAX_SCALE).abs() < 0.1);
        assert!(rail.zoom_slider_bounds.top() >= rail.bands[0].bounds.bottom());
        assert!((rail.zoom_percent - 822.0).abs() < 0.001);
        assert!(
            (semantic_scale_rail_zoom_scale_at(&rail, rail.zoom_slider_bounds.left_center())
                .expect("slider track accepts a pointer at its left edge")
                - MIN_SCALE)
                .abs()
                < 0.000_1
        );
        assert!(
            (semantic_scale_rail_zoom_scale_at(&rail, rail.zoom_slider_bounds.right_center())
                .expect("slider track accepts a pointer at its right edge")
                - MAX_SCALE)
                .abs()
                < 0.1
        );
    }

    #[test]
    fn zoom_slider_keeps_the_canvas_center_stable() {
        let viewport_size = Point::new(800.0, 600.0);
        let camera = Camera::new(Point::new(48.0, -32.0), 1.25);
        let center_before = camera.screen_to_world(viewport_size * 0.5);
        let updated = camera_with_scale_at_viewport_center(camera, 8.22, viewport_size);

        assert_eq!(updated.scale, 8.22);
        let center_after = updated.screen_to_world(viewport_size * 0.5);
        assert!((center_after.x - center_before.x).abs() < 0.000_1);
        assert!((center_after.y - center_before.y).abs() < 0.000_1);
    }

    #[test]
    fn context_trail_lines_keep_containment_indentation_inside_their_own_panel() {
        let bounds = CanvasRect::new(Point::new(0.0, 0.0), Point::new(800.0, 600.0));
        let entries = vec![
            CanvasContextEntry {
                entity: CanvasContextEntity::Boundary("boundary-jcwa".to_string()),
                title: "JCWA".to_string(),
                kind: CanvasContextKind::Boundary,
                bounds,
                depth: 0,
            },
            CanvasContextEntry {
                entity: CanvasContextEntity::Card(5),
                title: "Development and collaboration".to_string(),
                kind: CanvasContextKind::Card {
                    base_layer_title: Some("Architecture overview".to_string()),
                },
                bounds: CanvasRect::new(Point::new(80.0, 60.0), Point::new(480.0, 320.0)),
                depth: 1,
            },
        ];

        let lines = context_trail_lines(&entries, 360.0);

        assert_eq!(lines[0].text, "Boundary: JCWA");
        assert_eq!(lines[0].indent, 12.0);
        assert!(lines.iter().any(|line| {
            line.text.starts_with("Development") && (line.indent - 24.0).abs() < 0.001
        }));
    }

    #[test]
    fn view_all_mode_suspends_semantic_layer_filtering() {
        assert!(semantic_layer_filtering_is_active(true, false));
        assert!(!semantic_layer_filtering_is_active(true, true));
        assert!(!semantic_layer_filtering_is_active(false, false));
    }

    #[test]
    fn settings_window_defaults_to_non_destructive_display_controls() {
        let app = test_app();
        let preferences = app
            .preferences
            .lock()
            .unwrap_or_else(|error| error.into_inner());

        assert!(!preferences.open);
        assert_eq!(preferences.selected_tab, PreferencesTab::Canvas);
        assert!(preferences.stored.show_canvas_grid);
        assert!(preferences.stored.show_relationship_labels);
        assert_eq!(preferences.stored.last_opened_canvas_path, None);
        assert_eq!(
            preferences.stored,
            StoredPreferences {
                default_card_width: NEW_CONCEPT_SIZE.x,
                default_card_height: NEW_CONCEPT_SIZE.y,
                default_boundary_width: NEW_BOUNDARY_SIZE.x,
                default_boundary_height: NEW_BOUNDARY_SIZE.y,
                default_layer_start_percent: 1.0,
                default_layer_end_percent: 65.0,
                default_layer_fade_percent: 10.0,
                ..StoredPreferences::default()
            }
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn remembered_canvas_path_is_used_only_without_an_explicit_override() {
        let remembered = Path::new("/Users/tester/Maps/architecture.infinite-note.json");

        assert_eq!(
            remembered_workspace_path(Some(remembered), false),
            Some(remembered.to_path_buf())
        );
        assert_eq!(remembered_workspace_path(Some(remembered), true), None);
        assert_eq!(remembered_workspace_path(None, false), None);
    }

    #[test]
    fn authoring_preferences_supply_defaults_for_future_cards_and_layers() {
        let mut app = test_app();
        {
            let mut preferences = app
                .preferences
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            preferences.stored.default_card_width = 320.0;
            preferences.stored.default_card_height = 180.0;
            preferences.stored.default_card_type_id = Some("team".to_string());
            preferences.stored.default_boundary_width = 720.0;
            preferences.stored.default_boundary_height = 420.0;
            preferences.stored.default_layer_start_percent = 40.0;
            preferences.stored.default_layer_end_percent = 120.0;
            preferences.stored.default_layer_fade_percent = 15.0;
        }

        app.open_create_pane(SidePaneMode::CreateCard);
        assert_eq!(app.card_creation_draft.type_id.as_deref(), Some("team"));
        assert_eq!(
            new_concept_size_from_preferences(&app.stored_preferences()),
            Point::new(320.0, 180.0)
        );
        assert_eq!(
            new_boundary_size_from_preferences(&app.stored_preferences()),
            Point::new(720.0, 420.0)
        );
        assert_eq!(
            created_card_type_id(
                app.card_creation_draft.type_id.as_deref(),
                app.default_card_type_id().as_deref(),
            ),
            "team"
        );

        app.begin_inline_layer_creation();
        assert_eq!(app.layer_creation_active_zoom_start_percent, 40.0);
        assert_eq!(app.layer_creation_active_zoom_end_percent, 120.0);
        assert_eq!(app.layer_creation_fade_percent, 15.0);
    }

    #[test]
    fn explicit_card_type_overrides_the_saved_default() {
        assert_eq!(
            created_card_type_id(Some("service"), Some("team")),
            "service"
        );
        assert_eq!(created_card_type_id(None, Some("team")), "team");
        assert_eq!(created_card_type_id(None, None), "project");
    }

    #[test]
    fn new_card_size_defaults_never_fall_below_the_authoring_minimum() {
        let preferences = StoredPreferences {
            default_card_width: 2.0,
            default_card_height: 3.0,
            default_boundary_width: 4.0,
            default_boundary_height: 5.0,
            ..StoredPreferences::default()
        };

        assert_eq!(
            new_concept_size_from_preferences(&preferences),
            MINIMUM_CARD_SIZE
        );
        assert_eq!(
            new_boundary_size_from_preferences(&preferences),
            MINIMUM_BOUNDARY_SIZE
        );
    }

    #[test]
    fn new_card_size_preserves_the_preferred_screen_size_in_an_active_layer() {
        let preferences = StoredPreferences {
            default_card_width: 240.0,
            default_card_height: 120.0,
            ..StoredPreferences::default()
        };

        assert_eq!(
            new_concept_size_for_active_layer(&preferences, 2.0, 0.5),
            Point::new(240.0, 120.0)
        );
        assert_eq!(
            new_concept_size_for_active_layer(&preferences, 0.5, 0.5),
            Point::new(960.0, 480.0)
        );
        assert_eq!(
            new_concept_size_for_active_layer(&preferences, 16.0, 1.0),
            Point::new(240.0 / 16.0, MINIMUM_CARD_INTERACTION_SCREEN_SIZE.y / 16.0,)
        );
        let deepest_detail_card = new_concept_size_for_active_layer(&preferences, 4_096.0, 1.0);
        assert!((deepest_detail_card.x * 4_096.0 - 240.0).abs() < 0.000_1);
        assert!(
            (deepest_detail_card.y * 4_096.0 - MINIMUM_CARD_INTERACTION_SCREEN_SIZE.y).abs()
                < 0.000_1
        );
    }

    #[test]
    fn card_interaction_minimum_stays_small_in_canvas_units_at_high_zoom() {
        let minimum = minimum_card_size_at_screen_scale(12.16);

        assert!((minimum.x - (MINIMUM_CARD_INTERACTION_SCREEN_SIZE.x / 12.16)).abs() < 0.000_1);
        assert!((minimum.y - (MINIMUM_CARD_INTERACTION_SCREEN_SIZE.y / 12.16)).abs() < 0.000_1);
        assert!((minimum.x * 12.16 - MINIMUM_CARD_INTERACTION_SCREEN_SIZE.x).abs() < 0.000_1);
        assert!((minimum.y * 12.16 - MINIMUM_CARD_INTERACTION_SCREEN_SIZE.y).abs() < 0.000_1);
        assert_eq!(minimum_card_size_at_screen_scale(0.01), MINIMUM_CARD_SIZE);
        let deepest_detail_minimum = minimum_card_size_at_screen_scale(4_096.0);
        assert!(
            (deepest_detail_minimum.x * 4_096.0 - MINIMUM_CARD_INTERACTION_SCREEN_SIZE.x).abs()
                < 0.000_1
        );
        assert!(
            (deepest_detail_minimum.y * 4_096.0 - MINIMUM_CARD_INTERACTION_SCREEN_SIZE.y).abs()
                < 0.000_1
        );
    }

    #[test]
    fn adaptive_grid_adds_smaller_cells_at_high_zoom() {
        assert_eq!(adaptive_grid_screen_spacing(1.0), 64.0);
        assert_eq!(adaptive_grid_screen_spacing(2.0), 64.0);
        assert_eq!(adaptive_grid_screen_spacing(10.0), 80.0);
        assert_eq!(adaptive_grid_screen_spacing(1_000.0), 125.0);
    }

    #[test]
    fn settings_window_uses_the_platform_convention() {
        #[cfg(target_os = "macos")]
        assert_eq!(settings_window_title(), "Preferences");
        #[cfg(not(target_os = "macos"))]
        assert_eq!(settings_window_title(), "Settings");
    }

    #[test]
    fn relationship_label_preference_keeps_short_or_hidden_labels_off_canvas() {
        let source = Pos2::new(0.0, 0.0);
        let distant_target = Pos2::new(96.0, 0.0);
        let nearby_target = Pos2::new(80.0, 0.0);

        assert!(should_paint_relationship_label(
            true,
            source,
            distant_target
        ));
        assert!(!should_paint_relationship_label(
            false,
            source,
            distant_target
        ));
        assert!(!should_paint_relationship_label(
            true,
            source,
            nearby_target
        ));
    }

    #[test]
    fn semantic_scale_rail_changes_emphasis_without_positional_motion() {
        let viewport = Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(640.0, 480.0));
        let broad = semantic_scale_rail(viewport, 0.4, &[]);
        let detail = semantic_scale_rail(viewport, 2.0, &[]);

        assert_eq!(broad.bands[0].opacity, 1.0);
        assert_eq!(broad.bands[2].opacity, 0.52);
        assert_eq!(detail.bands[0].opacity, 0.52);
        assert_eq!(detail.bands[2].opacity, 1.0);
        assert_eq!(
            broad
                .bands
                .iter()
                .map(|band| band.bounds)
                .collect::<Vec<_>>(),
            detail
                .bands
                .iter()
                .map(|band| band.bounds)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn semantic_scale_rail_follows_resolved_custom_layer_fade_transitions() {
        let viewport = Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(640.0, 480.0));
        let layers = [
            SemanticZoomLayerProjection {
                id: "broad".to_string(),
                title: "Broad context".to_string(),
                opacity: 0.75,
            },
            SemanticZoomLayerProjection {
                id: "working".to_string(),
                title: "Working path".to_string(),
                opacity: 0.25,
            },
            SemanticZoomLayerProjection {
                id: "detail".to_string(),
                title: "Close detail".to_string(),
                opacity: 0.0,
            },
        ];

        let broad = semantic_scale_rail(viewport, 2.0, &layers);
        assert_eq!(broad.source, SemanticScaleRailSource::LensLayer);
        assert_eq!(broad.active_band, Some(SemanticScaleBand::Broad));
        assert_eq!(broad.description(), "Current layer: Broad context");

        let transitioned_layers = [
            SemanticZoomLayerProjection {
                opacity: 0.4,
                ..layers[0].clone()
            },
            SemanticZoomLayerProjection {
                opacity: 0.6,
                ..layers[1].clone()
            },
            SemanticZoomLayerProjection {
                opacity: 0.0,
                ..layers[2].clone()
            },
        ];
        let working = semantic_scale_rail(viewport, 0.1, &transitioned_layers);
        assert_eq!(working.active_band, Some(SemanticScaleBand::Working));

        let detail_layers = [
            SemanticZoomLayerProjection {
                opacity: 0.0,
                ..layers[0].clone()
            },
            SemanticZoomLayerProjection {
                opacity: 0.0,
                ..layers[1].clone()
            },
            SemanticZoomLayerProjection {
                opacity: 1.0,
                ..layers[2].clone()
            },
        ];
        let detail = semantic_scale_rail(viewport, 0.1, &detail_layers);
        assert_eq!(detail.active_band, Some(SemanticScaleBand::Detail));
        assert_eq!(
            broad
                .bands
                .iter()
                .map(|band| band.bounds)
                .collect::<Vec<_>>(),
            detail
                .bands
                .iter()
                .map(|band| band.bounds)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn semantic_scale_rail_names_an_empty_layer_transition_without_using_camera_scale() {
        let viewport = Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(640.0, 480.0));
        let layers = [SemanticZoomLayerProjection {
            id: "transition".to_string(),
            title: "Transition".to_string(),
            opacity: 0.0,
        }];

        let rail = semantic_scale_rail(viewport, 2.0, &layers);

        assert_eq!(rail.source, SemanticScaleRailSource::LensLayerGap);
        assert_eq!(rail.active_band, None);
        assert_eq!(rail.description(), "Current layer: no active layer");
        assert_eq!(
            rail.bands
                .iter()
                .map(|band| band.opacity)
                .collect::<Vec<_>>(),
            vec![0.0]
        );
    }

    #[test]
    fn authored_layer_rail_clicks_select_the_matching_layer_and_its_header_selects_active() {
        let viewport = Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(640.0, 480.0));
        let layers = [
            SemanticZoomLayerProjection {
                id: "overview".to_string(),
                title: "Overview".to_string(),
                opacity: 0.2,
            },
            SemanticZoomLayerProjection {
                id: "delivery".to_string(),
                title: "Delivery".to_string(),
                opacity: 1.0,
            },
        ];
        let rail = semantic_scale_rail(viewport, 1.0, &layers);

        assert_eq!(
            semantic_scale_rail_layer_id_at(&rail, rail.bands[0].bounds.center()),
            Some("overview".to_string())
        );
        assert_eq!(
            semantic_scale_rail_layer_id_at(&rail, rail.bounds.left_top()),
            Some("delivery".to_string())
        );
    }

    #[test]
    fn new_layer_defaults_expose_the_initial_percent_range_and_fade() {
        assert_eq!(
            semantic_zoom_layer_creation_percent_settings(0),
            SemanticZoomLayerPercentSettings {
                active_zoom_range: ZoomPercentRange {
                    start_percent: 1.0,
                    end_percent: 65.0,
                },
                fade_percent: 10.0,
            }
        );
        assert_eq!(
            semantic_zoom_layer_creation_percent_settings(1).active_zoom_range,
            ZoomPercentRange {
                start_percent: 66.0,
                end_percent: 150.0,
            }
        );
    }

    #[test]
    fn semantic_scale_rail_lists_each_authored_layer_and_names_the_active_one() {
        let viewport = Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(640.0, 480.0));
        let layers = [
            SemanticZoomLayerProjection {
                id: "overview".to_string(),
                title: "Organizational overview".to_string(),
                opacity: 0.0,
            },
            SemanticZoomLayerProjection {
                id: "delivery".to_string(),
                title: "Delivery path".to_string(),
                opacity: 0.25,
            },
            SemanticZoomLayerProjection {
                id: "controls".to_string(),
                title: "Shared controls".to_string(),
                opacity: 0.75,
            },
            SemanticZoomLayerProjection {
                id: "detail".to_string(),
                title: "Pipeline detail".to_string(),
                opacity: 0.0,
            },
        ];

        let rail = semantic_scale_rail(viewport, 1.0, &layers);

        assert_eq!(rail.bands.len(), layers.len());
        assert_eq!(
            rail.bands
                .iter()
                .map(|band| band.label.as_str())
                .collect::<Vec<_>>(),
            vec![
                "Organizational overview",
                "Delivery path",
                "Shared controls",
                "Pipeline detail",
            ]
        );
        assert_eq!(rail.description(), "Current layer: Shared controls");
        assert_eq!(rail.bands.iter().filter(|band| band.active).count(), 1);
        assert!(rail.bands[2].active);
    }

    #[test]
    fn semantic_scale_rail_names_each_layer_at_an_equal_opacity_transition() {
        let viewport = Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(640.0, 480.0));
        let layers = [
            SemanticZoomLayerProjection {
                id: "overview".to_string(),
                title: "Overview".to_string(),
                opacity: 0.0,
            },
            SemanticZoomLayerProjection {
                id: "delivery".to_string(),
                title: "Delivery".to_string(),
                opacity: 0.5,
            },
            SemanticZoomLayerProjection {
                id: "controls".to_string(),
                title: "Controls".to_string(),
                opacity: 0.5,
            },
        ];

        let rail = semantic_scale_rail(viewport, 1.0, &layers);

        assert_eq!(rail.description(), "Current layer: Delivery / Controls");
        assert!(!rail.bands[0].active);
        assert!(rail.bands[1].active);
        assert!(rail.bands[2].active);
    }

    #[test]
    fn semantic_scale_rail_wraps_the_full_current_layer_name() {
        let title = "Security assurance and deployment controls";
        let layers = [SemanticZoomLayerProjection {
            id: "controls".to_string(),
            title: title.to_string(),
            opacity: 1.0,
        }];
        let rail = semantic_scale_rail(
            Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(420.0, 300.0)),
            1.0,
            &layers,
        );

        assert_eq!(rail.description(), format!("Current layer: {title}"));
        assert_eq!(
            rail.header_lines
                .join("")
                .chars()
                .filter(|character| !character.is_whitespace())
                .collect::<String>(),
            rail.description()
                .chars()
                .filter(|character| !character.is_whitespace())
                .collect::<String>()
        );
    }

    #[test]
    fn layer_cross_section_keeps_range_and_fade_positions_on_one_scale() {
        let draft = LayerRangeDraft {
            layer_id: "delivery".to_string(),
            title: "Delivery path".to_string(),
            active_zoom_start_percent: 40.0,
            active_zoom_end_percent: 100.0,
            fade_percent: 10.0,
            card_scale: 1.0,
        };

        assert_eq!(
            layer_cross_section_domain_end(std::slice::from_ref(&draft), 75.0),
            200.0
        );
        assert_eq!(
            layer_cross_section_track(
                draft.active_zoom_start_percent,
                draft.active_zoom_end_percent,
                draft.fade_percent,
                200.0,
            ),
            LayerCrossSectionTrack {
                fade_start: 0.15,
                active_start: 0.2,
                active_end: 0.5,
                fade_end: 0.55,
            }
        );
    }

    #[test]
    fn semantic_zoom_layer_draft_uses_user_facing_percent_settings() {
        let layer = SemanticZoomLayer {
            id: "delivery".to_string(),
            title: "Delivery path".to_string(),
            percent_settings: Some(SemanticZoomLayerPercentSettings {
                active_zoom_range: ZoomPercentRange {
                    start_percent: 66.0,
                    end_percent: 150.0,
                },
                fade_percent: 11.0,
            }),
            fade_in: None,
            fade_out: None,
            card_scale: 0.8,
            concept_ids: Vec::new(),
            relationship_ids: Vec::new(),
        };

        let draft = semantic_zoom_layer_editor_draft(&layer);

        assert_eq!(draft.active_zoom_start_percent, 66.0);
        assert_eq!(draft.active_zoom_end_percent, 150.0);
        assert_eq!(draft.fade_percent, 11.0);
        assert_eq!(draft.card_scale, 0.8);
    }

    #[test]
    fn card_layer_policy_selection_keeps_automatic_and_manual_meaning_distinct() {
        let layers = vec![
            SemanticZoomLayerChoice {
                id: "overview".to_string(),
                title: "Organizational overview".to_string(),
            },
            SemanticZoomLayerChoice {
                id: "detail".to_string(),
                title: "Pipeline detail".to_string(),
            },
        ];

        assert_eq!(
            card_layer_mode_selected_ids(&SemanticZoomCardLayerMode::Automatic, &layers, &None),
            Vec::<String>::new()
        );
        assert_eq!(
            card_layer_mode_selected_ids(
                &SemanticZoomCardLayerMode::Manual {
                    layer_ids: vec!["overview".to_string(), "detail".to_string()],
                },
                &layers,
                &None,
            ),
            vec!["overview".to_string(), "detail".to_string()]
        );
        assert_eq!(
            card_layer_mode_selected_ids(&SemanticZoomCardLayerMode::AlwaysVisible, &layers, &None),
            vec!["overview".to_string(), "detail".to_string()]
        );
    }

    #[test]
    fn inspector_panel_keeps_a_readable_width_without_overtaking_the_canvas() {
        assert_eq!(inspector_panel_width(1_280.0), 380.0);
        assert_eq!(inspector_panel_width(800.0), 360.0);
    }

    #[test]
    fn canvas_and_inspector_use_non_overlapping_viewport_rectangles() {
        let available = Rect::from_min_size(Pos2::new(20.0, 60.0), Vec2::new(720.0, 540.0));
        let (canvas, inspector) = canvas_and_inspector_rects(available, true, 324.0);
        let inspector = inspector.expect("the inspector is visible");

        assert_eq!(canvas.left(), available.left());
        assert_eq!(inspector.right(), available.right());
        assert!(canvas.right() < inspector.left());
        assert_eq!(canvas.height(), available.height());
        assert_eq!(inspector.height(), available.height());
        assert_eq!(
            canvas_and_inspector_rects(available, false, 324.0),
            (available, None)
        );
    }

    #[test]
    fn center_view_only_zooms_out_and_keeps_all_bounds_inside_the_canvas() {
        let viewport = Point::new(800.0, 600.0);
        let bounds = CanvasRect::new(Point::new(-400.0, -250.0), Point::new(1_600.0, 1_000.0));
        let centered = camera_centered_to_fit_bounds(
            Camera::new(Point::new(32.0, 48.0), 2.0),
            bounds,
            viewport,
        );
        let screen_bounds = canvas_rect_to_screen(
            bounds,
            centered,
            Rect::from_min_size(Pos2::ZERO, Vec2::new(viewport.x, viewport.y)),
        );

        assert!(centered.scale <= 2.0);
        assert!(screen_bounds.left() >= 0.0);
        assert!(screen_bounds.right() <= viewport.x);
        assert!(screen_bounds.top() >= 0.0);
        assert!(screen_bounds.bottom() <= viewport.y);
    }

    #[test]
    fn search_navigation_wraps_for_enter_and_shift_enter_routes() {
        assert_eq!(search_match_index_after_navigation(0, 3, false, true), 1);
        assert_eq!(search_match_index_after_navigation(2, 3, false, true), 0);
        assert_eq!(search_match_index_after_navigation(0, 3, true, false), 2);
        assert_eq!(search_match_index_after_navigation(1, 3, true, true), 1);
        assert_eq!(search_match_index_after_navigation(2, 0, false, true), 0);
    }

    #[test]
    fn rendered_card_budget_is_bounded() {
        assert_eq!(rendered_card_count(4, MAX_RENDERED_CARDS), 4);
        assert_eq!(
            rendered_card_count(MAX_RENDERED_CARDS + 1, MAX_RENDERED_CARDS),
            MAX_RENDERED_CARDS
        );
    }

    #[test]
    fn card_visible_at_two_percent_stays_visible_at_one_percent() {
        let viewport = Rect::from_min_size(Pos2::new(32.0, 48.0), Vec2::new(800.0, 600.0));
        let bounds = CanvasRect::new(Point::new(400.0, 200.0), Point::new(400.0, 240.0));
        let two_percent_rect =
            canvas_rect_to_screen(bounds, Camera::new(Point::new(80.0, 120.0), 0.02), viewport);
        let one_percent_rect =
            canvas_rect_to_screen(bounds, Camera::new(Point::new(80.0, 120.0), 0.01), viewport);
        let one_percent_paint_rect = card_rect_for_painting(one_percent_rect);

        assert_eq!(card_rect_for_painting(two_percent_rect), two_percent_rect);
        assert_eq!(one_percent_paint_rect.center(), one_percent_rect.center());
        assert!((one_percent_paint_rect.width() - one_percent_rect.width()).abs() < f32::EPSILON);
        assert!((one_percent_paint_rect.height() - MINIMUM_VISIBLE_CARD_SIZE).abs() < f32::EPSILON);
    }

    #[test]
    fn selected_card_shell_keeps_a_visible_minimum_zoom_paint_bounds() {
        let viewport = Rect::from_min_size(Pos2::new(32.0, 48.0), Vec2::new(800.0, 600.0));
        let bounds = CanvasRect::new(Point::new(400.0, 200.0), Point::new(400.0, 240.0));
        let two_percent_rect =
            canvas_rect_to_screen(bounds, Camera::new(Point::new(80.0, 120.0), 0.02), viewport);
        let one_percent_rect =
            canvas_rect_to_screen(bounds, Camera::new(Point::new(80.0, 120.0), 0.01), viewport);
        let one_percent_card_rect = card_rect_for_painting(one_percent_rect);
        let one_percent_shell_bounds = card_paint_bounds(one_percent_rect, true);

        assert!(two_percent_rect.intersects(viewport));
        assert!(one_percent_rect.intersects(viewport));
        assert_eq!(one_percent_card_rect.center(), one_percent_rect.center());
        assert_eq!(one_percent_card_rect.width(), one_percent_rect.width());
        assert_eq!(one_percent_card_rect.height(), MINIMUM_VISIBLE_CARD_SIZE);
        assert_eq!(
            one_percent_shell_bounds,
            one_percent_card_rect.expand(SELECTED_CARD_STROKE_WIDTH)
        );
        assert!(one_percent_shell_bounds.intersects(viewport));
    }

    #[test]
    fn full_size_card_keeps_its_normal_paint_rect() {
        let screen_rect = Rect::from_min_size(Pos2::new(120.0, 80.0), Vec2::new(48.0, 32.0));

        assert_eq!(card_rect_for_painting(screen_rect), screen_rect);
    }

    #[test]
    fn disabled_selected_relationship_overlay_has_no_paths() {
        let document = WorkspaceDocument::sample();
        let displayed_card_bounds_by_concept_id = BTreeMap::new();
        let paths = selected_relationship_paths(
            &document,
            "current",
            "organization-landscape",
            false,
            Some(4),
            RelationshipOverlayViewport {
                visible_world: CanvasRect::new(Point::new(0.0, 0.0), Point::new(1_600.0, 800.0)),
                displayed_card_bounds_by_concept_id: &displayed_card_bounds_by_concept_id,
            },
            &RelationshipOverlayPlacementOverrides::default(),
        );

        assert!(paths.is_empty());
    }

    #[test]
    fn selected_relationship_paths_transform_every_route_point_for_the_viewport() {
        let paths = vec![SelectedConceptRelationshipOverlayPath {
            relationship_id: "route".to_string(),
            relationship_kind: RelationshipKind::DependsOn,
            source_id: 4,
            target_id: 5,
            points: vec![
                Point::new(-20.0, 5.0),
                Point::new(0.0, 30.0),
                Point::new(100.0, -10.0),
                Point::new(140.0, 70.0),
            ],
        }];
        let camera = Camera::new(Point::new(80.0, 120.0), 0.5);
        let viewport = Rect::from_min_size(Pos2::new(16.0, 24.0), Vec2::new(800.0, 400.0));

        assert_eq!(
            relationship_paths_to_screen(paths, camera, viewport),
            vec![vec![
                Pos2::new(86.0, 146.5),
                Pos2::new(96.0, 159.0),
                Pos2::new(146.0, 139.0),
                Pos2::new(166.0, 179.0),
            ]]
        );
    }

    #[test]
    fn enabled_selected_relationship_overlay_uses_the_routed_path_geometry() {
        let mut document = WorkspaceDocument::sample();
        document.current_state.relationships =
            vec![infinite_note_canvas_canvas::TypedRelationship {
                id: "visible".to_string(),
                kind: RelationshipKind::DependsOn,
                source_id: 7,
                target_id: 5,
                label: String::new(),
            }];
        document.current_state.lenses[0].placements = vec![
            infinite_note_canvas_canvas::ConceptPlacement {
                concept_id: 7,
                bounds: CanvasRect::new(Point::new(0.0, 0.0), Point::new(240.0, 132.0)),
            },
            infinite_note_canvas_canvas::ConceptPlacement {
                concept_id: 4,
                bounds: CanvasRect::new(Point::new(360.0, 0.0), Point::new(240.0, 132.0)),
            },
            infinite_note_canvas_canvas::ConceptPlacement {
                concept_id: 5,
                bounds: CanvasRect::new(Point::new(600.0, 0.0), Point::new(240.0, 132.0)),
            },
        ];
        let visible_world = CanvasRect::new(Point::new(0.0, -200.0), Point::new(1_200.0, 600.0));
        let camera = Camera::new(Point::new(80.0, 120.0), 0.5);
        let viewport = Rect::from_min_size(Pos2::new(16.0, 24.0), Vec2::new(800.0, 400.0));
        let pure_paths = document
            .selected_concept_relationship_paths_with_displayed_bounds(
                "current",
                "organization-landscape",
                7,
                visible_world,
                &RelationshipOverlayPlacementOverrides::default(),
                &displayed_test_card_bounds(
                    &[
                        (
                            7,
                            CanvasRect::new(Point::new(0.0, 0.0), Point::new(240.0, 132.0)),
                            true,
                        ),
                        (
                            4,
                            CanvasRect::new(Point::new(360.0, 0.0), Point::new(240.0, 132.0)),
                            false,
                        ),
                        (
                            5,
                            CanvasRect::new(Point::new(600.0, 0.0), Point::new(240.0, 132.0)),
                            false,
                        ),
                    ],
                    camera,
                    viewport,
                ),
            )
            .unwrap();
        let displayed_card_bounds_by_concept_id = displayed_test_card_bounds(
            &[
                (
                    7,
                    CanvasRect::new(Point::new(0.0, 0.0), Point::new(240.0, 132.0)),
                    true,
                ),
                (
                    4,
                    CanvasRect::new(Point::new(360.0, 0.0), Point::new(240.0, 132.0)),
                    false,
                ),
                (
                    5,
                    CanvasRect::new(Point::new(600.0, 0.0), Point::new(240.0, 132.0)),
                    false,
                ),
            ],
            camera,
            viewport,
        );

        let paths = selected_relationship_paths(
            &document,
            "current",
            "organization-landscape",
            true,
            Some(7),
            RelationshipOverlayViewport {
                visible_world,
                displayed_card_bounds_by_concept_id: &displayed_card_bounds_by_concept_id,
            },
            &RelationshipOverlayPlacementOverrides::default(),
        );

        assert_eq!(pure_paths.len(), 1);
        assert_eq!(pure_paths[0].points.len(), 4);
        assert_eq!(paths, relationship_paths_to_screen_positions(pure_paths));
    }

    #[test]
    fn rendered_relationship_paths_follow_painted_card_edges_and_preview_obstacles() {
        let camera = Camera::new(Point::new(80.0, 120.0), 0.5);
        let viewport = Rect::from_min_size(Pos2::new(16.0, 24.0), Vec2::new(800.0, 400.0));
        let visible_world = camera.visible_world_rect(Point::new(800.0, 400.0));

        let mut direct_document = WorkspaceDocument::sample();
        direct_document.current_state.relationships =
            vec![infinite_note_canvas_canvas::TypedRelationship {
                id: "direct".to_string(),
                kind: RelationshipKind::DependsOn,
                source_id: 7,
                target_id: 5,
                label: String::new(),
            }];
        direct_document.current_state.lenses[0].placements = vec![
            infinite_note_canvas_canvas::ConceptPlacement {
                concept_id: 7,
                bounds: CanvasRect::new(Point::new(0.0, 0.0), Point::new(240.0, 132.0)),
            },
            infinite_note_canvas_canvas::ConceptPlacement {
                concept_id: 5,
                bounds: CanvasRect::new(Point::new(600.0, 0.0), Point::new(240.0, 132.0)),
            },
        ];
        let direct_displayed_card_bounds_by_concept_id = displayed_test_card_bounds(
            &[
                (
                    7,
                    CanvasRect::new(Point::new(0.0, 0.0), Point::new(240.0, 132.0)),
                    true,
                ),
                (
                    5,
                    CanvasRect::new(Point::new(600.0, 0.0), Point::new(240.0, 132.0)),
                    false,
                ),
            ],
            camera,
            viewport,
        );
        let direct_paths = selected_relationship_paths(
            &direct_document,
            "current",
            "organization-landscape",
            true,
            Some(7),
            RelationshipOverlayViewport {
                visible_world,
                displayed_card_bounds_by_concept_id: &direct_displayed_card_bounds_by_concept_id,
            },
            &RelationshipOverlayPlacementOverrides::default(),
        );
        let direct_source =
            screen_rect_from_canvas_bounds(direct_displayed_card_bounds_by_concept_id[&7]);
        let direct_target =
            screen_rect_from_canvas_bounds(direct_displayed_card_bounds_by_concept_id[&5]);

        assert_eq!(
            direct_paths,
            vec![vec![
                direct_source.right_center(),
                direct_target.left_center()
            ]]
        );

        let mut obstacle_document = WorkspaceDocument::sample();
        obstacle_document.current_state.relationships = vec![
            infinite_note_canvas_canvas::TypedRelationship {
                id: "platform".to_string(),
                kind: RelationshipKind::DependsOn,
                source_id: 7,
                target_id: 4,
                label: String::new(),
            },
            infinite_note_canvas_canvas::TypedRelationship {
                id: "project".to_string(),
                kind: RelationshipKind::DependsOn,
                source_id: 7,
                target_id: 5,
                label: String::new(),
            },
        ];
        obstacle_document.current_state.lenses[0].placements = vec![
            infinite_note_canvas_canvas::ConceptPlacement {
                concept_id: 7,
                bounds: CanvasRect::new(Point::new(0.0, 0.0), Point::new(240.0, 132.0)),
            },
            infinite_note_canvas_canvas::ConceptPlacement {
                concept_id: 4,
                bounds: CanvasRect::new(Point::new(360.0, 0.0), Point::new(240.0, 132.0)),
            },
            infinite_note_canvas_canvas::ConceptPlacement {
                concept_id: 5,
                bounds: CanvasRect::new(Point::new(600.0, 0.0), Point::new(240.0, 132.0)),
            },
        ];
        let stored_displayed_card_bounds_by_concept_id = displayed_test_card_bounds(
            &[
                (
                    7,
                    CanvasRect::new(Point::new(0.0, 0.0), Point::new(240.0, 132.0)),
                    true,
                ),
                (
                    4,
                    CanvasRect::new(Point::new(360.0, 0.0), Point::new(240.0, 132.0)),
                    false,
                ),
                (
                    5,
                    CanvasRect::new(Point::new(600.0, 0.0), Point::new(240.0, 132.0)),
                    false,
                ),
            ],
            camera,
            viewport,
        );
        let stored_paths = selected_relationship_paths(
            &obstacle_document,
            "current",
            "organization-landscape",
            true,
            Some(7),
            RelationshipOverlayViewport {
                visible_world,
                displayed_card_bounds_by_concept_id: &stored_displayed_card_bounds_by_concept_id,
            },
            &RelationshipOverlayPlacementOverrides::default(),
        );
        let stored_platform_bounds =
            screen_rect_from_canvas_bounds(stored_displayed_card_bounds_by_concept_id[&4]);

        assert_eq!(stored_paths[1].len(), 4);
        assert_path_stays_outside_painted_card(&stored_paths[1], stored_platform_bounds);

        let mut preview_overrides = RelationshipOverlayPlacementOverrides::with_bounds(
            7,
            CanvasRect::new(Point::new(0.0, 200.0), Point::new(240.0, 132.0)),
        );
        preview_overrides.insert(
            4,
            CanvasRect::new(Point::new(360.0, 200.0), Point::new(240.0, 132.0)),
        );
        preview_overrides.insert(
            5,
            CanvasRect::new(Point::new(600.0, 200.0), Point::new(240.0, 132.0)),
        );
        let preview_displayed_card_bounds_by_concept_id = displayed_test_card_bounds(
            &[
                (
                    7,
                    CanvasRect::new(Point::new(0.0, 200.0), Point::new(240.0, 132.0)),
                    true,
                ),
                (
                    4,
                    CanvasRect::new(Point::new(360.0, 200.0), Point::new(240.0, 132.0)),
                    false,
                ),
                (
                    5,
                    CanvasRect::new(Point::new(600.0, 200.0), Point::new(240.0, 132.0)),
                    false,
                ),
            ],
            camera,
            viewport,
        );
        let preview_paths = selected_relationship_paths(
            &obstacle_document,
            "current",
            "organization-landscape",
            true,
            Some(7),
            RelationshipOverlayViewport {
                visible_world,
                displayed_card_bounds_by_concept_id: &preview_displayed_card_bounds_by_concept_id,
            },
            &preview_overrides,
        );
        let preview_source_bounds =
            screen_rect_from_canvas_bounds(preview_displayed_card_bounds_by_concept_id[&7]);
        let preview_platform_bounds =
            screen_rect_from_canvas_bounds(preview_displayed_card_bounds_by_concept_id[&4]);

        assert_ne!(preview_paths, stored_paths);
        assert!(point_on_rect_edge(
            preview_paths[1][0],
            preview_source_bounds
        ));
        assert_path_stays_outside_painted_card(&preview_paths[1], preview_platform_bounds);
        assert_eq!(
            obstacle_document.current_state.lenses[0].placements[0]
                .bounds
                .origin,
            Point::new(0.0, 0.0)
        );
    }

    #[test]
    fn rendered_relationship_paths_use_minimum_painted_card_edges() {
        let camera = Camera::new(Point::default(), 0.01);
        let viewport = Rect::from_min_size(Pos2::new(16.0, 24.0), Vec2::new(800.0, 400.0));
        let source_bounds = CanvasRect::new(Point::new(0.0, 0.0), Point::new(240.0, 132.0));
        let target_bounds = CanvasRect::new(Point::new(1_000.0, 0.0), Point::new(240.0, 132.0));
        let displayed_card_bounds_by_concept_id = displayed_test_card_bounds(
            &[(7, source_bounds, true), (5, target_bounds, false)],
            camera,
            viewport,
        );
        let mut document = WorkspaceDocument::sample();
        document.current_state.relationships =
            vec![infinite_note_canvas_canvas::TypedRelationship {
                id: "direct".to_string(),
                kind: RelationshipKind::DependsOn,
                source_id: 7,
                target_id: 5,
                label: String::new(),
            }];
        document.current_state.lenses[0].placements = vec![
            infinite_note_canvas_canvas::ConceptPlacement {
                concept_id: 7,
                bounds: source_bounds,
            },
            infinite_note_canvas_canvas::ConceptPlacement {
                concept_id: 5,
                bounds: target_bounds,
            },
        ];

        let paths = selected_relationship_paths(
            &document,
            "current",
            "organization-landscape",
            true,
            Some(7),
            RelationshipOverlayViewport {
                visible_world: camera.visible_world_rect(Point::new(800.0, 400.0)),
                displayed_card_bounds_by_concept_id: &displayed_card_bounds_by_concept_id,
            },
            &RelationshipOverlayPlacementOverrides::default(),
        );
        let source = screen_rect_from_canvas_bounds(displayed_card_bounds_by_concept_id[&7]);
        let target = screen_rect_from_canvas_bounds(displayed_card_bounds_by_concept_id[&5]);

        assert_eq!(paths.len(), 1);
        assert!(point_on_rect_edge(paths[0][0], source));
        assert!(point_on_rect_edge(paths[0][1], target));
        assert!(!point_is_in_rect_interior(paths[0][0], source));
        assert!(!point_is_in_rect_interior(paths[0][1], target));
    }

    #[test]
    fn unavailable_selected_relationship_overlay_endpoints_have_no_paths() {
        let document = WorkspaceDocument::sample();
        let displayed_card_bounds_by_concept_id = BTreeMap::new();
        let paths = selected_relationship_paths(
            &document,
            "current",
            "organization-landscape",
            true,
            Some(4),
            RelationshipOverlayViewport {
                visible_world: CanvasRect::new(Point::new(700.0, 200.0), Point::new(350.0, 250.0)),
                displayed_card_bounds_by_concept_id: &displayed_card_bounds_by_concept_id,
            },
            &RelationshipOverlayPlacementOverrides::default(),
        );

        assert!(paths.is_empty());
    }

    #[test]
    fn move_gesture_retains_the_full_selection_only_when_dragging_a_selected_card() {
        let mut second = test_card(Point::new(240.0, 0.0));
        second.id = 7;
        let mut third = test_card(Point::new(480.0, 0.0));
        third.id = 9;
        let mut unselected = test_card(Point::new(720.0, 0.0));
        unselected.id = 2;
        let dataset = CanvasDataset::from_records(
            vec![test_card(Point::new(0.0, 0.0)), second, third, unselected],
            Vec::new(),
        );

        assert_eq!(
            move_gesture_concept_ids(&dataset, 7, &[4, 7, 9]),
            vec![4, 7, 9]
        );
        assert_eq!(move_gesture_concept_ids(&dataset, 2, &[4, 7, 9]), vec![2]);
    }

    #[test]
    fn moving_a_card_container_previews_every_nested_card() {
        let mut parent = test_card(Point::new(0.0, 0.0));
        parent.id = 9;
        parent.bounds.size = Point::new(500.0, 360.0);
        let mut child = test_card(Point::new(80.0, 80.0));
        child.id = 7;
        child.bounds.size = Point::new(240.0, 180.0);
        let mut grandchild = test_card(Point::new(120.0, 120.0));
        grandchild.id = 4;
        let mut overlapping = test_card(Point::new(440.0, 80.0));
        overlapping.id = 2;
        let dataset =
            CanvasDataset::from_records(vec![parent, child, grandchild, overlapping], Vec::new());

        assert_eq!(move_gesture_concept_ids(&dataset, 9, &[]), vec![4, 7, 9]);
    }

    #[test]
    fn move_preview_bounds_places_every_moved_card_at_the_transient_position() {
        let first_bounds = CanvasRect::new(Point::new(40.0, 80.0), Point::new(120.0, 60.0));
        let second_bounds = CanvasRect::new(Point::new(240.0, 180.0), Point::new(80.0, 40.0));
        let gesture = PointerGesture::Move {
            item_ids: vec![CanvasItemId::Card(4), CanvasItemId::Card(9)],
            anchor_world_position: Point::new(60.0, 100.0),
            preview_world_delta: Point::new(90.0, -25.0),
        };

        assert_eq!(
            move_preview_bounds(first_bounds, 4, Some(&gesture)),
            CanvasRect::new(Point::new(130.0, 55.0), first_bounds.size)
        );
        assert_eq!(
            move_preview_bounds(second_bounds, 9, Some(&gesture)),
            CanvasRect::new(Point::new(330.0, 155.0), second_bounds.size)
        );
        assert_eq!(
            move_preview_bounds(first_bounds, 5, Some(&gesture)),
            first_bounds
        );
    }

    #[test]
    fn card_containment_uses_drag_preview_bounds_and_paints_parent_first() {
        let mut parent = test_card(Point::new(0.0, 0.0));
        parent.id = 9;
        parent.bounds.size = Point::new(300.0, 220.0);
        let child = test_card(Point::new(360.0, 80.0));
        let gesture = PointerGesture::Move {
            item_ids: vec![CanvasItemId::Card(child.id)],
            anchor_world_position: Point::new(400.0, 100.0),
            preview_world_delta: Point::new(-200.0, 0.0),
        };
        let cards = vec![&child, &parent];

        let preview_bounds = displayed_card_world_bounds_by_concept_id(&cards, Some(&gesture));
        let parent_by_child = card_parent_ids(
            preview_bounds
                .iter()
                .map(|(card_id, bounds)| (*card_id, *bounds)),
        );
        let mut paint_order = cards;
        sort_cards_for_painting(&mut paint_order, &parent_by_child);

        assert_eq!(parent_by_child, BTreeMap::from([(child.id, parent.id)]));
        assert_eq!(
            paint_order.iter().map(|card| card.id).collect::<Vec<_>>(),
            vec![parent.id, child.id]
        );
    }

    #[test]
    fn move_preview_collects_placement_overrides_for_every_moved_card() {
        let first_card = test_card(Point::new(40.0, 80.0));
        let mut second_card = test_card(Point::new(240.0, 180.0));
        second_card.id = 9;
        let dataset = CanvasDataset::from_records(vec![first_card, second_card], Vec::new());
        let gesture = PointerGesture::Move {
            item_ids: vec![CanvasItemId::Card(4), CanvasItemId::Card(9)],
            anchor_world_position: Point::new(60.0, 100.0),
            preview_world_delta: Point::new(90.0, -25.0),
        };

        let placement_overrides =
            relationship_overlay_placement_overrides(&dataset, Some(&gesture));

        assert_eq!(
            placement_overrides.bounds_by_concept_id,
            BTreeMap::from([
                (
                    4,
                    CanvasRect::new(Point::new(130.0, 55.0), Point::new(120.0, 80.0)),
                ),
                (
                    9,
                    CanvasRect::new(Point::new(330.0, 155.0), Point::new(120.0, 80.0)),
                ),
            ])
        );
    }

    #[test]
    fn move_commit_route_uses_the_single_or_group_stage_path() {
        assert_eq!(move_commit_route(&[]), None);
        assert_eq!(move_commit_route(&[4]), Some(MoveCommitRoute::Single(4)));
        assert_eq!(move_commit_route(&[4, 9]), Some(MoveCommitRoute::Group));
    }

    #[test]
    fn moving_a_card_into_another_card_reports_the_new_spatial_context() {
        let mut parent = test_card(Point::new(0.0, 0.0));
        parent.id = 9;
        parent.title = "Delivery domain".to_string();
        parent.bounds.size = Point::new(600.0, 400.0);
        let mut child = test_card(Point::new(700.0, 100.0));
        child.title = "Release pipeline".to_string();
        let dataset = CanvasDataset::from_records(vec![parent, child], Vec::new());

        assert_eq!(
            card_containment_notice_after_move(&dataset, &[4], Point::new(-300.0, 0.0)),
            SuccessNotice::Specific {
                title: "CONTAINMENT UPDATED".to_string(),
                detail: "Release pipeline is now inside Delivery domain.".to_string(),
            }
        );
        assert_eq!(
            card_containment_notice_after_move(&dataset, &[4], Point::new(80.0, 0.0)),
            SuccessNotice::None
        );
    }

    #[test]
    fn moving_a_card_into_or_out_of_a_visual_boundary_reports_the_change() {
        let mut card = test_card(Point::new(400.0, 80.0));
        card.title = "Release pipeline".to_string();
        let dataset = CanvasDataset::from_records(vec![card], Vec::new());
        let boundary = VisualBoundary {
            layer_id: None,
            id: "delivery-platform".to_string(),
            title: "Delivery platform".to_string(),
            summary: String::new(),
            bounds: CanvasRect::new(Point::new(0.0, 0.0), Point::new(500.0, 300.0)),
        };

        assert_eq!(
            visual_boundary_containment_notice_after_move(
                &dataset,
                std::slice::from_ref(&boundary),
                &[4],
                Point::new(-200.0, 0.0),
            ),
            SuccessNotice::Specific {
                title: "BOUNDARY UPDATED".to_string(),
                detail: "Release pipeline is now inside Delivery platform.".to_string(),
            }
        );
        let mut inside_card = test_card(Point::new(300.0, 80.0));
        inside_card.title = "Release pipeline".to_string();
        let inside_dataset = CanvasDataset::from_records(vec![inside_card], Vec::new());
        assert_eq!(
            visual_boundary_containment_notice_after_move(
                &inside_dataset,
                &[boundary],
                &[4],
                Point::new(120.0, 0.0),
            ),
            SuccessNotice::Specific {
                title: "BOUNDARY UPDATED".to_string(),
                detail: "Release pipeline is no longer inside Delivery platform.".to_string(),
            }
        );
    }

    #[test]
    fn resizing_a_card_reports_only_a_resolved_layer_change() {
        assert_eq!(
            resize_layer_change_notice("Release pipeline", "Overview", "Delivery detail"),
            SuccessNotice::Specific {
                title: "LAYER CHANGED".to_string(),
                detail: "Release pipeline now appears in Delivery detail instead of Overview."
                    .to_string(),
            }
        );
        assert_eq!(
            resize_layer_change_notice("Release pipeline", "Overview", "Overview"),
            SuccessNotice::None
        );
    }

    #[test]
    fn move_preview_and_release_share_the_same_pointer_derived_delta() {
        let camera = Camera::new(Point::new(24.0, -16.0), 2.0);
        let anchor_world_position = Point::new(50.0, 40.0);
        let viewport_position = Point::new(180.0, 96.0);
        let world_delta = move_world_delta(camera, anchor_world_position, viewport_position);
        let bounds = CanvasRect::new(Point::new(20.0, 30.0), Point::new(120.0, 60.0));
        let gesture = PointerGesture::Move {
            item_ids: vec![CanvasItemId::Card(4)],
            anchor_world_position,
            preview_world_delta: world_delta,
        };

        assert_eq!(world_delta, Point::new(28.0, 16.0));
        assert_eq!(
            move_preview_bounds(bounds, 4, Some(&gesture)).origin - bounds.origin,
            world_delta
        );
    }

    #[test]
    fn inspector_draft_keeps_unsaved_edits_for_the_selected_concept() {
        let mut draft = None;
        let assertion_attributes = BTreeMap::from([("priority".to_string(), "normal".to_string())]);
        let active_draft = inspector_draft_for(
            &mut draft,
            4,
            "Original name",
            "project",
            "Original summary",
            "Original details",
            &assertion_attributes,
        );
        active_draft.title = "Updated name".to_string();
        active_draft.type_id = "service".to_string();
        active_draft.summary = "Updated summary".to_string();
        active_draft.details = "Updated details".to_string();

        let draft = inspector_draft_for(
            &mut draft,
            4,
            "Original name",
            "project",
            "Original summary",
            "Original details",
            &assertion_attributes,
        );

        assert_eq!(draft.title, "Updated name");
        assert_eq!(draft.type_id, "service");
        assert_eq!(draft.summary, "Updated summary");
        assert_eq!(draft.details, "Updated details");
        assert_eq!(draft.assertion_attributes, assertion_attributes);
    }

    #[test]
    fn inspector_draft_resets_for_a_new_selected_concept() {
        let mut draft = Some(ConceptInspectorDraft {
            concept_id: 4,
            title: "Updated name".to_string(),
            type_id: "service".to_string(),
            summary: "Updated summary".to_string(),
            details: "Updated details".to_string(),
            assertion_attributes: BTreeMap::from([("priority".to_string(), "high".to_string())]),
            assertion_key: "priority".to_string(),
            assertion_value: "high".to_string(),
            assertion_error: Some("Assertion key must not be empty.".to_string()),
        });

        let draft = inspector_draft_for(
            &mut draft,
            8,
            "New concept",
            "project",
            "New summary",
            "New details",
            &BTreeMap::new(),
        );

        assert_eq!(draft.concept_id, 8);
        assert_eq!(draft.title, "New concept");
        assert_eq!(draft.type_id, "project");
        assert_eq!(draft.summary, "New summary");
        assert_eq!(draft.details, "New details");
        assert!(draft.assertion_attributes.is_empty());
        assert!(draft.assertion_key.is_empty());
        assert!(draft.assertion_value.is_empty());
        assert!(draft.assertion_error.is_none());
    }

    #[test]
    fn history_command_route_applies_undo_and_redo_without_an_unsaved_draft() {
        assert_eq!(
            history_command_route(HistoryCommand::Undo, false, false),
            HistoryCommandRoute::Apply(HistoryCommand::Undo)
        );
        assert_eq!(
            history_command_route(HistoryCommand::Redo, false, false),
            HistoryCommandRoute::Apply(HistoryCommand::Redo)
        );
    }

    #[test]
    fn history_command_labels_exclude_shortcut_text() {
        assert_eq!(history_command_label(HistoryCommand::Undo), "Undo");
        assert_eq!(history_command_label(HistoryCommand::Redo), "Redo");
    }

    #[test]
    fn command_enter_routes_each_focused_input_to_the_inspector_save() {
        assert!(focused_input_draft_save_target(true, false, false, false));
        assert!(focused_input_draft_save_target(false, true, false, false));
        assert!(focused_input_draft_save_target(false, false, true, false));
        assert!(focused_input_draft_save_target(false, false, false, true));
    }

    #[test]
    fn command_enter_does_not_route_without_a_focused_supported_input() {
        assert!(!focused_input_draft_save_target(false, false, false, false));
    }

    #[test]
    fn concept_inspector_header_left_aligns_the_save_action_with_a_visible_gap() {
        let mut header = None;
        egui::__run_test_ui(|ui| {
            ui.set_width(320.0);
            let inspector_boundary = ui.max_rect();
            let expected_save_button_size = concept_inspector_save_button_size(ui);
            let expected_header_gap = ui.spacing().item_spacing.x + CONCEPT_INSPECTOR_HEADER_GAP;
            let (heading, save_button) = concept_inspector_header(ui);
            header = Some((
                heading.rect,
                save_button.rect,
                inspector_boundary,
                expected_save_button_size,
                expected_header_gap,
            ));
        });

        let (
            heading_rect,
            save_button_rect,
            inspector_boundary,
            expected_save_button_size,
            expected_header_gap,
        ) = header.expect("the concept inspector header is rendered");
        assert_eq!(CONCEPT_INSPECTOR_SAVE_LABEL, "Save");
        assert_eq!(heading_rect.left(), inspector_boundary.left());
        assert_eq!(
            heading_rect.right() + expected_header_gap,
            save_button_rect.left()
        );
        assert!(save_button_rect.left() > inspector_boundary.left());
        assert!(
            inspector_boundary.contains_rect(save_button_rect),
            "boundary: {inspector_boundary:?}, heading: {heading_rect:?}, save button: {save_button_rect:?}"
        );
        assert!(save_button_rect.right() < inspector_boundary.right());
        assert_eq!(save_button_rect.size(), expected_save_button_size);
    }

    #[test]
    fn constrained_inspector_header_leaves_all_edit_fields_available() {
        let mut layout = None;
        egui::__run_test_ui(|ui| {
            ui.set_width(120.0);
            ui.set_height(280.0);
            let inspector_boundary = ui.max_rect();
            let expected_header_gap = ui.spacing().item_spacing.x + CONCEPT_INSPECTOR_HEADER_GAP;
            let (heading, save_button) = concept_inspector_header(ui);
            let mut title = "Selected concept".to_string();
            let mut type_id = "project".to_string();
            let mut summary = "Summary".to_string();
            let mut details = "Details".to_string();
            let mut assertion_key = "priority".to_string();
            let mut assertion_value = "high".to_string();

            let title_rect = ui.text_edit_singleline(&mut title).rect;
            let type_rect = egui::ComboBox::from_id_salt("constrained-inspector-type")
                .selected_text(&type_id)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut type_id, "project".to_string(), "project");
                })
                .response
                .rect;
            let summary_rect = ui.text_edit_multiline(&mut summary).rect;
            let details_rect = ui.text_edit_multiline(&mut details).rect;
            let assertion_key_rect = ui.text_edit_singleline(&mut assertion_key).rect;
            let assertion_value_rect = ui.text_edit_singleline(&mut assertion_value).rect;
            layout = Some((
                inspector_boundary,
                heading.rect,
                save_button.rect,
                expected_header_gap,
                [
                    title_rect,
                    type_rect,
                    summary_rect,
                    details_rect,
                    assertion_key_rect,
                    assertion_value_rect,
                ],
            ));
        });

        let (
            inspector_boundary,
            heading_rect,
            save_button_rect,
            expected_header_gap,
            edit_field_rects,
        ) = layout.expect("the constrained inspector layout is rendered");
        assert!(inspector_boundary.contains_rect(save_button_rect));
        assert_eq!(heading_rect.left(), inspector_boundary.left());
        assert_eq!(
            heading_rect.right() + expected_header_gap,
            save_button_rect.left()
        );
        assert!(heading_rect.bottom() <= edit_field_rects[0].top());
        assert!(
            edit_field_rects
                .into_iter()
                .all(|rect| inspector_boundary.contains_rect(rect)),
            "boundary: {inspector_boundary:?}, edit fields: {edit_field_rects:?}"
        );
    }

    #[test]
    fn history_command_route_consumes_shortcuts_for_unsaved_drafts_and_confirmations() {
        assert_eq!(
            history_command_route(HistoryCommand::Undo, true, false),
            HistoryCommandRoute::Consume
        );
        assert_eq!(
            history_command_route(HistoryCommand::Redo, false, true),
            HistoryCommandRoute::Consume
        );
    }

    #[test]
    fn saved_inspector_draft_permits_history_shortcuts_while_focused() {
        let draft = ConceptInspectorDraft {
            concept_id: 4,
            title: "Selected concept".to_string(),
            type_id: "project".to_string(),
            summary: "Saved summary".to_string(),
            details: "Saved details".to_string(),
            assertion_attributes: BTreeMap::from([("priority".to_string(), "high".to_string())]),
            assertion_key: String::new(),
            assertion_value: String::new(),
            assertion_error: None,
        };

        let has_unsaved_changes = inspector_draft_has_unsaved_changes(
            &draft,
            4,
            "Selected concept",
            "project",
            "Saved summary",
            "Saved details",
            &BTreeMap::from([("priority".to_string(), "high".to_string())]),
        );

        assert!(!has_unsaved_changes);
        assert_eq!(
            history_command_route(HistoryCommand::Undo, has_unsaved_changes, false),
            HistoryCommandRoute::Apply(HistoryCommand::Undo)
        );
    }

    #[test]
    fn changed_inspector_fields_consume_history_shortcuts() {
        let saved_draft = ConceptInspectorDraft {
            concept_id: 4,
            title: "Selected concept".to_string(),
            type_id: "project".to_string(),
            summary: "Saved summary".to_string(),
            details: "Saved details".to_string(),
            assertion_attributes: BTreeMap::new(),
            assertion_key: String::new(),
            assertion_value: String::new(),
            assertion_error: None,
        };
        let changed_drafts = [
            ConceptInspectorDraft {
                title: "Renamed concept".to_string(),
                ..saved_draft.clone()
            },
            ConceptInspectorDraft {
                type_id: "service".to_string(),
                ..saved_draft.clone()
            },
            ConceptInspectorDraft {
                summary: "Unsaved summary".to_string(),
                ..saved_draft.clone()
            },
            ConceptInspectorDraft {
                details: "Unsaved details".to_string(),
                ..saved_draft.clone()
            },
            ConceptInspectorDraft {
                assertion_key: "priority".to_string(),
                assertion_value: "high".to_string(),
                ..saved_draft
            },
        ];

        for draft in changed_drafts {
            let has_unsaved_changes = inspector_draft_has_unsaved_changes(
                &draft,
                4,
                "Selected concept",
                "project",
                "Saved summary",
                "Saved details",
                &BTreeMap::new(),
            );

            assert!(has_unsaved_changes);
            assert_eq!(
                history_command_route(HistoryCommand::Redo, has_unsaved_changes, false),
                HistoryCommandRoute::Consume
            );
        }
    }

    #[test]
    fn history_restoration_keeps_the_selected_concept_available_to_the_inspector() {
        let dataset =
            CanvasDataset::from_records(vec![test_card(Point::new(40.0, 80.0))], Vec::new());
        let selected_concept_id = history_restore_selection(&[], Some(4), &dataset);

        assert_eq!(selected_concept_id, Some(4));
        let mut draft = None;
        let inspector_draft = inspector_draft_for(
            &mut draft,
            selected_concept_id.expect("selected concept must remain available"),
            "Selected concept",
            "project",
            "Restored summary",
            "Restored details",
            &BTreeMap::new(),
        );

        assert_eq!(inspector_draft.concept_id, 4);
        assert_eq!(inspector_draft.summary, "Restored summary");
        assert_eq!(inspector_draft.details, "Restored details");
    }

    #[test]
    fn assertion_changes_stay_in_the_inspector_draft_until_save() {
        let mut draft = ConceptInspectorDraft {
            concept_id: 4,
            title: "Selected concept".to_string(),
            type_id: "project".to_string(),
            summary: "Saved summary".to_string(),
            details: "Saved details".to_string(),
            assertion_attributes: BTreeMap::from([("priority".to_string(), "high".to_string())]),
            assertion_key: "owner".to_string(),
            assertion_value: "team-a".to_string(),
            assertion_error: None,
        };

        assert_eq!(
            assertion_save_label(&draft.assertion_attributes, "owner"),
            "Add assertion"
        );
        assert_eq!(
            assertion_save_label(&draft.assertion_attributes, "priority"),
            "Replace assertion"
        );
        assert_eq!(save_pending_draft_assertion(&mut draft), Ok(()));
        remove_draft_assertion(&mut draft, "priority");

        assert_eq!(
            draft.assertion_attributes,
            BTreeMap::from([("owner".to_string(), "team-a".to_string())])
        );
    }

    #[test]
    fn inspector_save_routes_every_supported_field_through_one_aggregate_draft() {
        let draft = ConceptInspectorDraft {
            concept_id: 4,
            title: "Renamed concept".to_string(),
            type_id: "service".to_string(),
            summary: "Updated summary".to_string(),
            details: "Updated details".to_string(),
            assertion_attributes: BTreeMap::from([("priority".to_string(), "urgent".to_string())]),
            assertion_key: String::new(),
            assertion_value: String::new(),
            assertion_error: None,
        };

        assert_eq!(
            concept_inspector_edit_draft(&draft),
            ConceptInspectorEditDraft {
                concept_id: 4,
                title: "Renamed concept".to_string(),
                type_id: "service".to_string(),
                summary: "Updated summary".to_string(),
                details: "Updated details".to_string(),
                assertion_attributes: BTreeMap::from([(
                    "priority".to_string(),
                    "urgent".to_string(),
                )]),
            }
        );
    }

    #[test]
    fn inspector_save_keeps_invalid_drafts_available_for_correction() {
        let draft = ConceptInspectorEditDraft {
            concept_id: 4,
            title: " \t".to_string(),
            type_id: "project".to_string(),
            summary: "Saved summary".to_string(),
            details: "Saved details".to_string(),
            assertion_attributes: BTreeMap::new(),
        };

        assert_eq!(
            inspector_save_validation_error(&draft),
            Some("A concept name cannot be empty.")
        );
        assert_eq!(draft.title, " \t");
    }

    #[test]
    fn pending_invalid_assertion_blocks_an_inspector_save_without_losing_the_draft() {
        let mut draft = ConceptInspectorDraft {
            concept_id: 4,
            title: "Selected concept".to_string(),
            type_id: "project".to_string(),
            summary: "Saved summary".to_string(),
            details: "Saved details".to_string(),
            assertion_attributes: BTreeMap::new(),
            assertion_key: " \t".to_string(),
            assertion_value: "high".to_string(),
            assertion_error: None,
        };

        assert_eq!(
            save_pending_draft_assertion(&mut draft),
            Err("Assertion key must not be empty.")
        );
        assert_eq!(draft.assertion_key, " \t");
        assert_eq!(draft.assertion_value, "high");
        assert!(draft.assertion_attributes.is_empty());
    }

    #[test]
    fn inspector_autosave_waits_for_pending_assertion_input_to_be_resolved() {
        let mut draft = ConceptInspectorDraft {
            concept_id: 4,
            title: "Selected concept".to_string(),
            type_id: "project".to_string(),
            summary: "Saved summary".to_string(),
            details: "Saved details".to_string(),
            assertion_attributes: BTreeMap::new(),
            assertion_key: "priority".to_string(),
            assertion_value: "high".to_string(),
            assertion_error: None,
        };

        assert!(!inspector_autosave_is_safe(&draft));

        assert_eq!(save_pending_draft_assertion(&mut draft), Ok(()));
        assert!(inspector_autosave_is_safe(&draft));

        draft.assertion_error = Some("Assertion key must not be empty.".to_string());
        assert!(!inspector_autosave_is_safe(&draft));
    }

    #[test]
    fn loading_assertion_into_draft_replaces_values_and_clears_error() {
        let mut draft = ConceptInspectorDraft {
            concept_id: 4,
            title: "Selected concept".to_string(),
            type_id: "project".to_string(),
            summary: "Saved summary".to_string(),
            details: "Saved details".to_string(),
            assertion_attributes: BTreeMap::new(),
            assertion_key: "\t".to_string(),
            assertion_value: "Old value".to_string(),
            assertion_error: Some("Assertion key must not be empty.".to_string()),
        };

        load_assertion_into_draft(&mut draft, "priority", "high");

        assert_eq!(draft.assertion_key, "priority");
        assert_eq!(draft.assertion_value, "high");
        assert!(draft.assertion_error.is_none());
    }

    #[test]
    fn assertion_editor_rejects_invalid_keys_before_staging() {
        assert_eq!(
            assertion_key_error(" \t"),
            Some("Assertion key must not be empty.")
        );
        assert_eq!(
            assertion_key_error("priority\nlevel"),
            Some("Assertion key must not contain control characters.")
        );
        assert_eq!(assertion_key_error("priority"), None);
    }

    #[test]
    fn saved_lens_camera_takes_priority_over_selected_card_centering() {
        let saved_camera = Camera::new(Point::new(-40.0, 25.0), 1.4);

        assert_eq!(
            camera_for_lens_activation(
                Some(saved_camera),
                Some(&test_card(Point::new(200.0, 100.0))),
                Point::new(800.0, 600.0),
            ),
            saved_camera
        );
    }

    #[test]
    fn new_lens_camera_centers_the_retained_selected_card() {
        let card = test_card(Point::new(200.0, 100.0));
        let viewport_size = Point::new(800.0, 600.0);
        let camera = camera_for_lens_activation(None, Some(&card), viewport_size);
        let card_center = Point::new(260.0, 140.0);

        assert_eq!(
            camera.world_to_screen(card_center),
            Point::new(400.0, 300.0)
        );
    }

    #[test]
    fn lens_viewport_keys_are_independent_for_each_lens() {
        let mut cameras = HashMap::new();
        let ownership = LensViewportKey {
            version_id: "current".to_string(),
            lens_id: "ownership".to_string(),
        };
        let delivery = LensViewportKey {
            version_id: "current".to_string(),
            lens_id: "delivery".to_string(),
        };
        let ownership_camera = Camera::new(Point::new(15.0, 20.0), 1.25);
        let delivery_camera = Camera::new(Point::new(-40.0, 90.0), 0.8);
        cameras.insert(ownership.clone(), ownership_camera);
        cameras.insert(delivery.clone(), delivery_camera);

        assert_eq!(cameras[&ownership], ownership_camera);
        assert_eq!(cameras[&delivery], delivery_camera);
    }

    #[test]
    fn relationship_description_keeps_stored_source_kind_and_target() {
        let relationship = RelationshipView {
            id: "rel-1".to_string(),
            source_id: 7,
            target_id: 4,
            kind: RelationshipKind::DependsOn,
            source_title: "Release Pipeline".to_string(),
            target_title: "Portal API".to_string(),
        };

        let description = relationship_description(&relationship);

        assert_eq!(
            description,
            "Release Pipeline (#7) --depends on--> Portal API (#4)"
        );
        assert!(description.is_ascii());
    }

    #[test]
    fn type_change_confirmation_is_required_only_when_the_lens_hides_the_concept() {
        assert!(!type_change_requires_lens_confirmation(true));
        assert!(type_change_requires_lens_confirmation(false));
    }

    #[test]
    fn ordinary_notice_stays_opaque_then_fades_and_dismisses() {
        let notice = Notice::new("SAVED", "Saved concept.", false, 100.0);

        assert_eq!(
            notice.visual_state(109.999),
            OrdinaryNoticeVisualState::Opaque
        );
        assert_eq!(
            notice.visual_state(110.0),
            OrdinaryNoticeVisualState::Fading { opacity: 1.0 }
        );
        assert_eq!(
            notice.visual_state(110.25),
            OrdinaryNoticeVisualState::Fading { opacity: 0.5 }
        );
        assert_eq!(
            notice.visual_state(110.5),
            OrdinaryNoticeVisualState::Dismissed
        );
    }

    #[test]
    fn ordinary_notice_pauses_its_timer_while_hovered() {
        let mut notice = Notice::new("SAVED", "Saved concept.", false, 0.0);

        notice.set_hovered(true, 4.0);
        assert_eq!(notice.active_elapsed_seconds(20.0), 4.0);
        assert_eq!(notice.visual_state(20.0), OrdinaryNoticeVisualState::Opaque);

        notice.set_hovered(false, 20.0);
        assert_eq!(notice.active_elapsed_seconds(25.0), 9.0);
        assert_eq!(notice.visual_state(25.0), OrdinaryNoticeVisualState::Opaque);
        assert_eq!(
            notice.visual_state(26.0),
            OrdinaryNoticeVisualState::Fading { opacity: 1.0 }
        );
    }

    #[test]
    fn removing_dismissed_notices_keeps_later_notices_active() {
        let mut notices = vec![
            Notice::new("FIRST", "First saved concept.", false, 0.0),
            Notice::new("SECOND", "Second saved concept.", false, 1.0),
        ];

        remove_dismissed_notices(&mut notices, 10.5);

        assert_eq!(notices.len(), 1);
        assert_eq!(notices[0].title, "SECOND");
        assert_eq!(
            notices[0].visual_state(10.5),
            OrdinaryNoticeVisualState::Opaque
        );
    }

    #[test]
    fn type_change_failure_toast_stays_opaque_for_ten_active_seconds() {
        let toast = TypeChangeFailureToast::new(
            "type change failed".to_string(),
            PathBuf::from("diagnostic.log"),
            None,
            100.0,
        );

        assert_eq!(
            toast.visual_state(109.999),
            TypeChangeToastVisualState::Opaque
        );
        assert_eq!(
            toast.visual_state(110.0),
            TypeChangeToastVisualState::Fading { opacity: 1.0 }
        );
    }

    #[test]
    fn type_change_failure_toast_pauses_its_timer_while_hovered() {
        let mut toast = TypeChangeFailureToast::new(
            "type change failed".to_string(),
            PathBuf::from("diagnostic.log"),
            None,
            0.0,
        );

        toast.set_hovered(true, 4.0);
        assert_eq!(toast.active_elapsed_seconds(20.0), 4.0);
        assert_eq!(toast.visual_state(20.0), TypeChangeToastVisualState::Opaque);

        toast.set_hovered(false, 20.0);
        assert_eq!(toast.active_elapsed_seconds(25.0), 9.0);
        assert_eq!(toast.visual_state(25.0), TypeChangeToastVisualState::Opaque);
        assert_eq!(
            toast.visual_state(26.0),
            TypeChangeToastVisualState::Fading { opacity: 1.0 }
        );
    }

    #[test]
    fn type_change_failure_toast_fades_then_dismisses() {
        let toast = TypeChangeFailureToast::new(
            "type change failed".to_string(),
            PathBuf::from("diagnostic.log"),
            None,
            0.0,
        );

        assert_eq!(
            toast.visual_state(10.25),
            TypeChangeToastVisualState::Fading { opacity: 0.5 }
        );
        assert_eq!(
            toast.visual_state(10.5),
            TypeChangeToastVisualState::Dismissed
        );
    }

    #[test]
    fn type_change_failure_diagnostic_records_path_context_and_error() {
        let workspace_path = Path::new("/Users/tester/Workspace/team.infinite-note.json");
        let diagnostic_path = type_change_diagnostic_path(workspace_path);
        let diagnostic = format_type_change_failure_diagnostic(
            &diagnostic_path,
            42,
            "service",
            "candidate bytes did not match",
        );

        assert_eq!(
            diagnostic_path,
            PathBuf::from("/Users/tester/Workspace").join(TYPE_CHANGE_DIAGNOSTIC_FILE_NAME)
        );
        assert!(diagnostic.contains("event=type_change_failure\n"));
        assert!(diagnostic.contains("concept_id=42\n"));
        assert!(diagnostic.contains("target_type_id=service\n"));
        assert!(diagnostic.contains("error=candidate bytes did not match\n"));
        assert!(diagnostic.contains(&format!("diagnostic_path={}\n", diagnostic_path.display())));
        assert!(diagnostic.contains(
            "action=Review this log, correct the local workspace problem, and try the type change again.\n"
        ));
    }

    #[test]
    fn inspector_save_failure_routes_only_type_changes_to_the_type_change_toast() {
        assert_eq!(
            inspector_save_failure_route("team", "service"),
            InspectorSaveFailureRoute::TypeChangeToast
        );
        assert_eq!(
            inspector_save_failure_route("team", "team"),
            InspectorSaveFailureRoute::OrdinaryNotice
        );
    }

    #[test]
    fn startup_failure_diagnostic_uses_the_external_workspace_directory() {
        let diagnostic_path = startup_diagnostic_path(Some(PathBuf::from(
            "/Users/tester/Workspace/team.infinite-note.json",
        )));
        let diagnostic = format_startup_failure_diagnostic(
            &diagnostic_path,
            "the graphics context could not initialize",
        );

        assert_eq!(
            diagnostic_path,
            PathBuf::from("/Users/tester/Workspace").join(STARTUP_DIAGNOSTIC_FILE_NAME)
        );
        assert!(diagnostic.contains("event=eframe_startup_failure\n"));
        assert!(diagnostic.contains("error=the graphics context could not initialize\n"));
        assert!(diagnostic.contains(
            "action=Run ./start.sh with a writable external workspace and review this log.\n"
        ));
    }

    #[test]
    fn startup_failure_diagnostic_uses_the_working_directory_without_a_workspace() {
        assert_eq!(
            startup_diagnostic_path(None),
            PathBuf::from(".").join(STARTUP_DIAGNOSTIC_FILE_NAME)
        );
    }

    #[test]
    fn type_change_failure_diagnostic_appends_to_the_local_log() {
        let diagnostic_path = std::env::temp_dir().join(format!(
            "infinite-note-canvas-type-change-failure-{}-{}.log",
            std::process::id(),
            NEXT_DIAGNOSTIC_TEST_FILE.fetch_add(1, Ordering::Relaxed),
        ));
        let first_record =
            format_type_change_failure_diagnostic(&diagnostic_path, 42, "service", "first failure");
        let second_record = format_type_change_failure_diagnostic(
            &diagnostic_path,
            42,
            "repository",
            "second failure",
        );

        append_diagnostic(&diagnostic_path, &first_record).unwrap();
        append_diagnostic(&diagnostic_path, &second_record).unwrap();

        assert_eq!(
            fs::read_to_string(&diagnostic_path).unwrap(),
            first_record + &second_record
        );
        fs::remove_file(diagnostic_path).unwrap();
    }

    #[test]
    fn boundary_crossing_segments_use_control_color_only_inside_the_boundary() {
        let boundary = Rect::from_min_size(Pos2::new(50.0, 0.0), Vec2::new(100.0, 100.0));
        let segments =
            boundary_aware_line_segments(Pos2::new(0.0, 50.0), Pos2::new(200.0, 50.0), &[boundary]);

        assert_eq!(
            segments,
            vec![
                BoundaryAwareLineSegment {
                    start: Pos2::new(0.0, 50.0),
                    end: Pos2::new(50.0, 50.0),
                    inside_boundary: false,
                },
                BoundaryAwareLineSegment {
                    start: Pos2::new(50.0, 50.0),
                    end: Pos2::new(150.0, 50.0),
                    inside_boundary: true,
                },
                BoundaryAwareLineSegment {
                    start: Pos2::new(150.0, 50.0),
                    end: Pos2::new(200.0, 50.0),
                    inside_boundary: false,
                },
            ]
        );
        assert!(relationship_path_crosses_boundary(
            &[Pos2::new(0.0, 50.0), Pos2::new(200.0, 50.0)],
            &[boundary]
        ));
        assert_eq!(
            segments
                .iter()
                .map(|segment| boundary_segment_color(true, segment.inside_boundary))
                .collect::<Vec<_>>(),
            vec![ATLAS_ROUTE_BLUE, ATLAS_CONTROL_AMBER, ATLAS_ROUTE_BLUE]
        );
    }

    #[test]
    fn contained_relationship_segments_keep_the_relationship_route_color() {
        let boundary = Rect::from_min_size(Pos2::new(50.0, 0.0), Vec2::new(100.0, 100.0));
        let path = [Pos2::new(60.0, 50.0), Pos2::new(140.0, 50.0)];
        let segments = boundary_aware_line_segments(path[0], path[1], &[boundary]);

        assert!(!relationship_path_crosses_boundary(&path, &[boundary]));
        assert_eq!(
            segments
                .iter()
                .map(|segment| boundary_segment_color(false, segment.inside_boundary))
                .collect::<Vec<_>>(),
            vec![ATLAS_ROUTE_BLUE]
        );
    }

    #[test]
    fn overlapping_boundaries_keep_every_overlapped_line_segment_in_control_context() {
        let segments = boundary_aware_line_segments(
            Pos2::new(0.0, 50.0),
            Pos2::new(200.0, 50.0),
            &[
                Rect::from_min_size(Pos2::new(40.0, 0.0), Vec2::new(100.0, 100.0)),
                Rect::from_min_size(Pos2::new(90.0, 0.0), Vec2::new(70.0, 100.0)),
            ],
        );

        assert_eq!(
            segments
                .iter()
                .map(|segment| (segment.start.x, segment.end.x, segment.inside_boundary))
                .collect::<Vec<_>>(),
            vec![
                (0.0, 40.0, false),
                (40.0, 90.0, true),
                (90.0, 140.0, true),
                (140.0, 160.0, true),
                (160.0, 200.0, false),
            ]
        );
    }

    #[test]
    fn relationship_ports_resolve_a_source_and_target_at_card_edges() {
        let source = test_card(Point::new(0.0, 0.0));
        let mut target = test_card(Point::new(300.0, 0.0));
        target.id = 8;
        let dataset = CanvasDataset::from_records(vec![source, target], Vec::new());

        assert_eq!(
            relationship_port_positions(dataset.card_by_id(4).unwrap().bounds),
            [
                (CardEdge::Top, Point::new(60.0, 0.0)),
                (CardEdge::Right, Point::new(120.0, 40.0)),
                (CardEdge::Bottom, Point::new(60.0, 80.0)),
                (CardEdge::Left, Point::new(0.0, 40.0)),
            ]
        );
        assert_eq!(
            relationship_source_port_at(&dataset, Point::new(120.0, 40.0), 10.0),
            Some((4, CardEdge::Right))
        );
        assert_eq!(
            relationship_input_port_at(&dataset, Point::new(300.0, 40.0), 10.0),
            Some(8)
        );
    }

    #[test]
    fn relationship_port_drag_accepts_a_target_port_or_body_and_rejects_its_source() {
        let source = test_card(Point::new(0.0, 0.0));
        let mut target = test_card(Point::new(300.0, 0.0));
        target.id = 8;
        let dataset = CanvasDataset::from_records(vec![source, target], Vec::new());

        assert_eq!(
            relationship_drop_target(&dataset, 4, Point::new(300.0, 40.0), 10.0),
            Some(8)
        );
        assert_eq!(
            relationship_drop_target(&dataset, 4, Point::new(360.0, 40.0), 10.0),
            Some(8)
        );
        assert_eq!(
            relationship_drop_target(&dataset, 4, Point::new(60.0, 40.0), 10.0),
            None
        );
    }

    #[test]
    fn primary_card_stays_explicit_until_it_leaves_the_selection() {
        assert_eq!(
            primary_selected_concept_id(Some(8), [4, 8].into_iter()),
            Some(8)
        );
        assert_eq!(
            primary_selected_concept_id(Some(9), [8, 4].into_iter()),
            Some(4)
        );
        assert_eq!(
            primary_selected_concept_id(None, [8, 4].into_iter()),
            Some(4)
        );
        assert_eq!(primary_selected_concept_id(Some(4), [].into_iter()), None);
    }

    #[test]
    fn canvas_delete_target_prioritizes_relationship_then_boundary_then_cards() {
        assert_eq!(
            canvas_delete_target(
                [7, 5].into_iter(),
                Some("boundary-runtime"),
                Some("relationship-delivers"),
            ),
            Some(CanvasDeleteTarget::Relationship(
                "relationship-delivers".to_string()
            ))
        );
        assert_eq!(
            canvas_delete_target([7, 5].into_iter(), Some("boundary-runtime"), None),
            Some(CanvasDeleteTarget::Boundary("boundary-runtime".to_string()))
        );
        assert_eq!(
            canvas_delete_target([7, 5, 7].into_iter(), None, None),
            Some(CanvasDeleteTarget::Cards(vec![5, 7]))
        );
        assert_eq!(canvas_delete_target([].into_iter(), None, None), None);
    }

    #[test]
    fn canvas_delete_shortcut_respects_modal_and_text_edit_focus() {
        assert!(should_delete_canvas_selection(false, false, true, false));
        assert!(should_delete_canvas_selection(false, false, false, true));
        assert!(!should_delete_canvas_selection(true, false, true, false));
        assert!(!should_delete_canvas_selection(false, true, false, true));
        assert!(!should_delete_canvas_selection(false, false, false, false));
    }

    #[test]
    fn card_clipboard_shortcuts_respect_text_focus_even_with_selected_cards() {
        assert!(card_clipboard_shortcuts_take_priority(false, false, true));
        assert!(!card_clipboard_shortcuts_take_priority(false, true, true));
        assert!(!card_clipboard_shortcuts_take_priority(false, true, false));
        assert!(!card_clipboard_shortcuts_take_priority(true, false, true));
    }

    #[test]
    fn focused_text_editor_receives_paste_while_card_selection_remains() {
        let mut app = test_app();
        let path = std::env::temp_dir().join(format!(
            "reason-room-text-clipboard-{}.json",
            std::process::id()
        ));
        std::fs::write(&path, WorkspaceDocument::sample().to_pretty_json().unwrap()).unwrap();
        let session = WorkspaceSession::load_default(path.clone()).unwrap();
        app.dataset = session.dataset().clone();
        app.workspace_state = WorkspaceState::Loaded(Box::new(session));
        app.set_selection(Some(5));
        app.copy_selected_cards();
        let count = app.dataset.cards().count();
        let ctx = egui::Context::default();
        let mut summary = String::new();
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            ui.add(egui::TextEdit::singleline(&mut summary).id_salt("summary"))
                .request_focus();
        });
        output.textures_delta.clear();
        let mut output = ctx.run_ui(
            egui::RawInput {
                events: vec![egui::Event::Paste("Pasted summary text".into())],
                ..Default::default()
            },
            |ui| {
                assert!(ui.ctx().text_edit_focused());
                app.handle_card_clipboard_shortcuts(ui.ctx());
                ui.add(egui::TextEdit::singleline(&mut summary).id_salt("summary"));
            },
        );
        output.textures_delta.clear();
        assert_eq!(summary, "Pasted summary text");
        assert_eq!(app.dataset.cards().count(), count);
        assert_eq!(app.selected_concept_id(), Some(5));
        let mut output = ctx.run_ui(
            egui::RawInput {
                events: vec![
                    egui::Event::Key {
                        key: egui::Key::A,
                        physical_key: None,
                        pressed: true,
                        repeat: false,
                        modifiers: egui::Modifiers::COMMAND,
                    },
                    egui::Event::Copy,
                ],
                ..Default::default()
            },
            |ui| {
                app.handle_card_clipboard_shortcuts(ui.ctx());
                ui.add(egui::TextEdit::singleline(&mut summary).id_salt("summary"));
            },
        );
        assert!(output.platform_output.commands.iter().any(|command| matches!(command, egui::OutputCommand::CopyText(text) if text == "Pasted summary text")));
        output.textures_delta.clear();
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn platform_clipboard_copy_publishes_text_and_semantic_paste_duplicates_card() {
        let mut app = test_app();
        let path = std::env::temp_dir().join(format!(
            "reason-room-platform-clipboard-{}.json",
            std::process::id()
        ));
        std::fs::write(&path, WorkspaceDocument::sample().to_pretty_json().unwrap()).unwrap();
        let session = WorkspaceSession::load_default(path.clone()).unwrap();
        app.dataset = session.dataset().clone();
        app.workspace_state = WorkspaceState::Loaded(Box::new(session));
        app.set_selection(Some(5));
        let count = app.dataset.cards().count();
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(
            egui::RawInput {
                events: vec![egui::Event::Copy],
                ..Default::default()
            },
            |ui| {
                app.handle_card_clipboard_shortcuts(ui.ctx());
                app.publish_pending_card_clipboard(ui.ctx());
            },
        );
        assert!(output.platform_output.commands.iter().any(
            |command| matches!(command, egui::OutputCommand::CopyText(text) if !text.is_empty())
        ));
        output.textures_delta.clear();
        let mut output = ctx.run_ui(
            egui::RawInput {
                events: vec![egui::Event::Paste("Reason Room cards".into())],
                ..Default::default()
            },
            |ui| {
                app.handle_card_clipboard_shortcuts(ui.ctx());
            },
        );
        output.textures_delta.clear();
        assert_eq!(app.dataset.cards().count(), count + 1);
        let mut output = ctx.run_ui(
            egui::RawInput {
                events: vec![egui::Event::Key {
                    key: egui::Key::V,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::COMMAND,
                }],
                ..Default::default()
            },
            |ui| {
                app.handle_card_clipboard_shortcuts(ui.ctx());
            },
        );
        output.textures_delta.clear();
        assert_eq!(app.dataset.cards().count(), count + 2);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn views_retain_independent_temporary_filters_and_card_paste_keeps_shared_content() {
        let mut app = test_app();
        let path = std::env::temp_dir().join(format!(
            "reason-room-view-clipboard-{}.json",
            std::process::id()
        ));
        let mut document = WorkspaceDocument::sample();
        let view = document
            .create_empty_lens("current", "Clipboard destination".into())
            .unwrap();
        std::fs::write(&path, document.to_pretty_json().unwrap()).unwrap();
        let session = WorkspaceSession::load_default(path.clone()).unwrap();
        app.dataset = session.dataset().clone();
        app.workspace_state = WorkspaceState::Loaded(Box::new(session));
        app.set_selection(Some(5));
        app.copy_selected_cards();
        app.canvas_search_query = "source-only query".into();
        app.canvas_type_filter_ids.insert("service".into());
        app.show_all_cards = true;
        app.roll_up_relationships = true;
        app.select_lens(view.clone());
        assert!(app.canvas_search_query.is_empty());
        assert!(app.canvas_type_filter_ids.is_empty());
        assert!(!app.show_all_cards);
        assert!(!app.roll_up_relationships);
        assert!(app.can_paste_cards());
        app.paste_cards_at(Some(Point::new(5000.0, 5000.0)));
        let destination = app
            .session()
            .unwrap()
            .document()
            .version("current")
            .unwrap()
            .lenses
            .iter()
            .find(|lens| lens.id == view)
            .unwrap();
        assert_eq!(destination.placements[0].concept_id, 5);
        let bounds = paste_card_visual_bounds(
            app.session().unwrap().document(),
            "current",
            &view,
            app.interaction.camera().scale,
            false,
        );
        assert_eq!(bounds[&5].origin, Point::new(5000.0, 5000.0));
        app.select_lens("organization-landscape".into());
        assert_eq!(app.canvas_search_query, "source-only query");
        assert!(app.canvas_type_filter_ids.contains("service"));
        assert!(app.show_all_cards);
        assert!(app.roll_up_relationships);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn paste_collision_geometry_honors_layer_scaling_and_low_zoom_shells() {
        let mut document = WorkspaceDocument::sample();
        let lens_id = "organization-landscape";
        let layer = document
            .create_semantic_zoom_layer("current", lens_id, "Paste scale".into())
            .unwrap();
        document
            .update_semantic_zoom_layer_percent_settings(
                "current",
                lens_id,
                &layer,
                "Paste scale".into(),
                SemanticZoomLayerPercentSettings {
                    active_zoom_range: ZoomPercentRange {
                        start_percent: 1.0,
                        end_percent: 300.0,
                    },
                    fade_percent: 0.0,
                },
                2.0,
            )
            .unwrap();
        document
            .set_semantic_zoom_card_layer_mode(
                "current",
                lens_id,
                5,
                SemanticZoomCardLayerMode::Manual {
                    layer_ids: vec![layer],
                },
            )
            .unwrap();
        let raw = document.version("current").unwrap().lenses[0]
            .placements
            .iter()
            .find(|placement| placement.concept_id == 5)
            .unwrap()
            .bounds;
        for scale in [1.0, 0.001] {
            let projected = paste_card_visual_bounds(&document, "current", lens_id, scale, false);
            let bounds = projected[&5];
            assert!(bounds.size.x > raw.size.x * 2.0);
            assert!(bounds.size.y > raw.size.y * 2.0);
            assert!(bounds.size.x * scale >= MINIMUM_VISIBLE_CARD_SIZE);
            assert!(bounds.size.y * scale >= MINIMUM_VISIBLE_CARD_SIZE);
            let delta = card_paste_delta(&[bounds], &[bounds], bounds.origin);
            assert!(!CanvasRect::new(bounds.origin + delta, bounds.size).intersects(bounds));
        }
    }

    #[test]
    fn pasted_group_anchors_at_cursor_and_escapes_overlapping_cards() {
        let source = [
            CanvasRect::new(Point::new(10.0, 20.0), Point::new(100.0, 80.0)),
            CanvasRect::new(Point::new(30.0, 40.0), Point::new(20.0, 20.0)),
        ];
        let cursor = Point::new(400.0, 300.0);
        let delta = card_paste_delta(&source, &[], cursor);
        assert_eq!(source[0].origin + delta, cursor);
        let delta = card_paste_delta(&source, &source, source[0].origin);
        for bounds in source {
            let pasted = CanvasRect::new(bounds.origin + delta, bounds.size);
            assert!(source.iter().all(|origin| !pasted.intersects(*origin)));
        }
        // Existing children cannot become children of the pasted outer card.
        let obstacle = CanvasRect::new(Point::new(420.0, 320.0), Point::new(20.0, 20.0));
        let delta = card_paste_delta(&source, &[obstacle], cursor);
        assert!(!CanvasRect::new(source[0].origin + delta, source[0].size).intersects(obstacle));
    }

    #[test]
    fn clipboard_commands_consume_desktop_copy_and_paste_events_in_order() {
        let mut events = vec![
            egui::Event::Text("unrelated input".to_string()),
            egui::Event::Copy,
            egui::Event::Paste("system clipboard".to_string()),
        ];

        assert_eq!(
            consume_card_clipboard_event(&mut events),
            Some(CardClipboardCommand::Copy)
        );
        assert!(matches!(
            events.as_slice(),
            [egui::Event::Text(_), egui::Event::Paste(_)]
        ));
        assert_eq!(
            consume_card_clipboard_event(&mut events),
            Some(CardClipboardCommand::Paste)
        );
        assert!(matches!(events.as_slice(), [egui::Event::Text(_)]));
    }

    #[test]
    fn create_menu_modes_replace_the_shared_inspector_contents() {
        let mut app = test_app();

        app.open_create_pane(SidePaneMode::CreateCard);
        assert_eq!(app.side_pane_mode, SidePaneMode::CreateCard);
        app.open_create_pane(SidePaneMode::CreateBoundary);
        assert_eq!(app.side_pane_mode, SidePaneMode::CreateBoundary);
        app.begin_inline_layer_creation();
        assert_eq!(app.side_pane_mode, SidePaneMode::LayerCrossSection);
        assert!(app.layer_cross_section_creating);
        assert_eq!(app.layer_creation_title, "Overview");
        app.open_layer_cross_section();
        assert_eq!(app.side_pane_mode, SidePaneMode::LayerCrossSection);

        app.open_create_version_pane("Current".to_string());
        assert_eq!(app.side_pane_mode, SidePaneMode::CreateVersion);
        assert_eq!(app.version_creation_draft.title, "Current copy");

        app.open_create_lens_pane("Current".to_string());
        assert_eq!(app.side_pane_mode, SidePaneMode::CreateLens);
        assert_eq!(app.lens_creation_draft.title, "Current view");
        assert!(!app.lens_creation_draft.clone_current);
    }

    #[test]
    fn deselecting_the_last_canvas_item_collapses_the_inspector() {
        let mut app = test_app();
        app.interaction.select_only_card_id(4);
        app.inspector_collapsed = false;

        app.set_selection(None);

        assert!(app.inspector_has_no_selection());
        assert!(app.inspector_collapsed);
    }

    #[test]
    fn relationship_port_gesture_preserves_inspector_and_offers_type_at_drop() {
        let mut app = test_app();
        app.interaction
            .set_camera(Camera::new(Point::default(), 1.0));
        let source = test_card(Point::new(100.0, 300.0));
        let mut target = test_card(Point::new(500.0, 300.0));
        target.id = 8;
        app.dataset = CanvasDataset::from_records(vec![source, target], Vec::new());
        app.inspector_collapsed = true;
        let ctx = egui::Context::default();
        let viewport = Rect::from_min_size(Pos2::ZERO, Vec2::new(1000.0, 800.0));
        let viewport_size = Point::new(1000.0, 800.0);
        let source_port = Pos2::new(220.0, 340.0);
        let target_port = Pos2::new(500.0, 340.0);
        for (position, pressed) in [(source_port, true), (target_port, false)] {
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(viewport),
                    events: vec![
                        egui::Event::PointerMoved(position),
                        egui::Event::PointerButton {
                            pos: position,
                            button: PointerButton::Primary,
                            pressed,
                            modifiers: egui::Modifiers::default(),
                        },
                    ],
                    ..Default::default()
                },
                |ui| {
                    let response = ui.interact(
                        viewport,
                        egui::Id::unique("port-test-canvas"),
                        Sense::click_and_drag(),
                    );
                    app.handle_canvas_input(ui, &response, viewport, viewport_size, Rect::NOTHING);
                },
            );
            output.textures_delta.clear();
            assert!(app.inspector_collapsed);
            assert!(app.interaction.selected_card_ids().next().is_none());
        }
        assert!(app.gesture.is_none());
        assert!(
            matches!(app.dialog, Some(EditorDialog::ChooseRelationshipKind { source_id: 4, target_id: 8, position }) if position == target_port)
        );
    }

    #[test]
    fn preferences_open_as_an_embedded_window_without_native_viewports() {
        let app = test_app();
        app.open_settings_window();
        let ctx = egui::Context::default();
        ctx.set_embed_viewports(true);
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            app.settings_window(ui.ctx())
        });
        output.textures_delta.clear();
        assert!(!output.shapes.is_empty());
        assert!(app.preferences.lock().unwrap().open);
        assert!(!app.preferences.lock().unwrap().focus_requested);
        assert!(
            !output.viewport_output[&egui::ViewportId::ROOT]
                .commands
                .contains(&egui::ViewportCommand::Close)
        );
    }

    #[test]
    fn chrome_visuals_keep_text_and_controls_on_the_dark_palette() {
        let visuals = atlas_visuals();
        assert!(visuals.dark_mode);
        assert_eq!(visuals.text_color(), ATLAS_PAPER_SIGNAL);
        assert_eq!(visuals.panel_fill, ATLAS_GROUND);
        assert_eq!(visuals.window_fill, ATLAS_SURVEY_SLATE);
        // WCAG relative luminance contrast for the dimmest ordinary label
        // against the lightest product panel exceeds 4.5:1.
        let luminance = |color: Color32| {
            let linear = |value: u8| {
                let value = f64::from(value) / 255.0;
                if value <= 0.04045 {
                    value / 12.92
                } else {
                    ((value + 0.055) / 1.055).powf(2.4)
                }
            };
            0.2126 * linear(color.r()) + 0.7152 * linear(color.g()) + 0.0722 * linear(color.b())
        };
        assert!(
            (luminance(ATLAS_QUIET_GRAPHITE) + 0.05) / (luminance(ATLAS_SURVEY_SLATE) + 0.05) > 4.5
        );
    }

    #[test]
    fn relationship_creation_never_routes_through_a_confirmation_dialog() {
        let mut app = test_app();

        app.create_relationship(4, 8, RelationshipKind::DependsOn);

        assert!(app.dialog.is_none());
    }

    #[test]
    fn relationship_lines_use_cardinal_ports_nearest_to_the_other_card() {
        let source = CanvasRect::new(Point::new(100.0, 100.0), Point::new(120.0, 80.0));
        let right_target = CanvasRect::new(Point::new(400.0, 100.0), Point::new(120.0, 80.0));
        let top_target = CanvasRect::new(Point::new(100.0, -140.0), Point::new(120.0, 80.0));

        assert_eq!(
            relationship_edge_endpoints(source, right_target),
            (
                (CardEdge::Right, Point::new(220.0, 140.0)),
                (CardEdge::Left, Point::new(400.0, 140.0)),
            )
        );
        assert_eq!(
            relationship_edge_endpoints(source, top_target),
            (
                (CardEdge::Top, Point::new(160.0, 100.0)),
                (CardEdge::Bottom, Point::new(160.0, -60.0)),
            )
        );
    }

    #[test]
    fn relationship_source_ports_accept_an_unselected_card() {
        let dataset =
            CanvasDataset::from_records(vec![test_card(Point::new(0.0, 0.0))], Vec::new());

        assert_eq!(
            relationship_source_port_at(&dataset, Point::new(120.0, 40.0), 10.0),
            Some((4, CardEdge::Right))
        );
        assert_eq!(
            relationship_source_port_at(&dataset, Point::new(60.0, 0.0), 10.0),
            Some((4, CardEdge::Top))
        );
    }

    #[test]
    fn card_resize_keeps_the_top_left_or_center_anchor_as_requested() {
        let initial = CanvasRect::new(Point::new(100.0, 80.0), Point::new(120.0, 80.0));

        assert_eq!(
            resized_card_bounds(initial, Point::new(40.0, 20.0), false, MINIMUM_CARD_SIZE),
            CanvasRect::new(Point::new(100.0, 80.0), Point::new(160.0, 100.0))
        );
        assert_eq!(
            resized_card_bounds(initial, Point::new(40.0, 20.0), true, MINIMUM_CARD_SIZE),
            CanvasRect::new(Point::new(60.0, 60.0), Point::new(200.0, 120.0))
        );
        assert_eq!(
            resized_card_bounds(initial, Point::new(-900.0, -900.0), true, MINIMUM_CARD_SIZE,),
            CanvasRect::new(Point::new(120.0, 96.0), MINIMUM_CARD_SIZE)
        );
    }

    #[test]
    fn card_resize_hit_area_matches_the_visible_corner_handle() {
        let card = test_card(Point::new(100.0, 80.0));
        let dataset = CanvasDataset::from_records(vec![card], Vec::new());
        let selected = BTreeSet::from([4]);

        assert_eq!(
            card_resize_target(&dataset, &selected, Point::new(220.0, 160.0), 12.0),
            Some((
                4,
                CanvasRect::new(Point::new(100.0, 80.0), Point::new(120.0, 80.0))
            ))
        );
        assert_eq!(
            card_resize_target(&dataset, &selected, Point::new(210.0, 150.0), 12.0),
            None
        );
    }

    #[test]
    fn maximum_zoom_pointer_targets_stay_at_their_screen_sizes() {
        let camera = Camera::new(Point::default(), MAX_SCALE);
        assert!((card_resize_handle_world_size(camera) * MAX_SCALE - 12.0).abs() < 0.000_1);
        assert!((relationship_port_hit_radius(camera) * MAX_SCALE - 10.0).abs() < 0.000_1);
        assert!(
            (boundary_resize_handle_world_size(camera) * MAX_SCALE
                - BOUNDARY_RESIZE_HANDLE_SCREEN_SIZE)
                .abs()
                < 0.000_1
        );

        let mut card = test_card(Point::default());
        card.bounds.size = Point::new(240.0 / MAX_SCALE, 120.0 / MAX_SCALE);
        let dataset = CanvasDataset::from_records(vec![card.clone()], Vec::new());
        let selected = BTreeSet::from([card.id]);

        assert_eq!(
            card_resize_target(
                &dataset,
                &selected,
                card.bounds.center(),
                card_resize_handle_world_size(camera),
            ),
            None
        );
        assert_eq!(
            relationship_source_port_at(
                &dataset,
                card.bounds.center(),
                relationship_port_hit_radius(camera),
            ),
            None
        );
        assert!(
            card_resize_target(
                &dataset,
                &selected,
                Point::new(card.bounds.max_x(), card.bounds.max_y()),
                card_resize_handle_world_size(camera),
            )
            .is_some()
        );
    }

    #[test]
    fn resize_hover_and_hint_distinguish_the_corner_handle_from_move() {
        assert_eq!(
            canvas_hover_hint(&CanvasHoverTarget::CardResize(4)),
            "Drag the blue corner to resize. Hold Option for a center anchor."
        );
        assert_eq!(
            canvas_hover_hint(&CanvasHoverTarget::Card(4)),
            "Click to inspect. Drag to move."
        );
    }

    #[test]
    fn relationship_port_hover_has_a_connection_specific_hint() {
        assert_eq!(
            canvas_hover_hint(&CanvasHoverTarget::RelationshipPort {
                concept_id: 4,
                edge: CardEdge::Right,
            }),
            "Drag to create a relationship."
        );
    }

    #[test]
    fn resized_card_preview_uses_only_the_active_resize_gesture() {
        let initial = CanvasRect::new(Point::new(100.0, 80.0), Point::new(120.0, 80.0));
        let preview = CanvasRect::new(Point::new(100.0, 80.0), Point::new(180.0, 110.0));
        let gesture = PointerGesture::ResizeCard {
            concept_id: 4,
            initial_bounds: initial,
            anchor_world_position: Point::new(220.0, 160.0),
            centered: false,
            preview_bounds: preview,
        };

        assert_eq!(move_preview_bounds(initial, 4, Some(&gesture)), preview);
        assert_eq!(move_preview_bounds(initial, 8, Some(&gesture)), initial);
    }

    #[test]
    fn relationship_line_hit_test_uses_the_painted_path_and_nearest_line() {
        let paths = vec![
            RelationshipPaintPath {
                id: "far".to_string(),
                kind: RelationshipKind::DependsOn,
                visible_segments: vec![vec![Pos2::new(0.0, 20.0), Pos2::new(200.0, 20.0)]],
                crosses_boundary: false,
                arrowhead: None,
                label_position: None,
            },
            RelationshipPaintPath {
                id: "near".to_string(),
                kind: RelationshipKind::DependsOn,
                visible_segments: vec![vec![Pos2::new(0.0, 40.0), Pos2::new(200.0, 40.0)]],
                crosses_boundary: false,
                arrowhead: None,
                label_position: None,
            },
        ];

        assert_eq!(
            relationship_path_at(&paths, Pos2::new(80.0, 37.0), 7.0),
            Some("near".to_string())
        );
        assert_eq!(
            relationship_path_at(&paths, Pos2::new(80.0, 80.0), 7.0),
            None
        );
    }

    #[test]
    fn relationships_inside_cards_receive_hover_click_and_context_menu() {
        let mut app = test_app();
        let path = std::env::temp_dir().join(format!(
            "reason-room-relationship-hit-{}.json",
            std::process::id()
        ));
        let document = examples::ALL[1].document().unwrap();
        fs::write(&path, document.to_pretty_json().unwrap()).unwrap();
        let session = WorkspaceSession::load_default(path.clone()).unwrap();
        app.dataset = session.dataset().clone();
        app.workspace_state = WorkspaceState::Loaded(Box::new(session));
        app.show_all_cards = true;
        let viewport_size = Point::new(1600.0, 1000.0);
        let viewport = Rect::from_min_size(Pos2::ZERO, Vec2::new(1600.0, 1000.0));
        app.canvas_viewport_size = viewport_size;
        app.interaction
            .set_camera(Camera::new(Point::default(), 0.15));
        app.refresh_display_dataset();
        let camera = app.interaction.camera();
        // Use a painted segment inside an interactive enclosing card, away
        // from endpoint ports. This reproduces the card/line priority conflict.
        let position = app
            .visible_relationship_paths(viewport)
            .iter()
            .flat_map(|path| &path.visible_segments)
            .flat_map(|segment| segment.windows(2))
            .flat_map(|segment| {
                (1..10).map(move |step| segment[0].lerp(segment[1], step as f32 / 10.0))
            })
            .find(|position| {
                let world = camera.screen_to_world(Point::new(position.x, position.y));
                position.y > 300.0
                    && app.filtered_canvas_hit_test(world).is_some()
                    && relationship_source_port_at(
                        &app.dataset,
                        world,
                        relationship_port_hit_radius(camera),
                    )
                    .is_none()
            })
            .expect("manufacturing example has a relationship inside a card");
        let world = camera.screen_to_world(Point::new(position.x, position.y));
        let relationship_id = app.canvas_relationship_at(position, viewport).unwrap();
        assert_eq!(
            app.canvas_hover_target(world, position, viewport, viewport_size),
            Some(CanvasHoverTarget::Relationship(relationship_id.clone()))
        );
        assert_eq!(
            app.canvas_context_menu_target_at(world, position, viewport, viewport_size),
            CanvasContextMenuTarget::Relationship(relationship_id.clone())
        );

        let (card_position, card_id) = (-4..=4)
            .flat_map(|x| {
                (-4..=4).map(move |y| position + Vec2::new(x as f32 * 10.0, y as f32 * 10.0))
            })
            .find_map(|position| {
                let world = camera.screen_to_world(Point::new(position.x, position.y));
                match app.canvas_hover_target(world, position, viewport, viewport_size) {
                    Some(CanvasHoverTarget::Card(id)) => Some((position, id)),
                    _ => None,
                }
            })
            .expect("card surface away from the line stays interactive");
        assert_eq!(
            app.canvas_context_menu_target_at(
                camera.screen_to_world(Point::new(card_position.x, card_position.y)),
                card_position,
                viewport,
                viewport_size,
            ),
            CanvasContextMenuTarget::Card(card_id)
        );

        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(viewport),
                events: vec![
                    egui::Event::PointerMoved(position),
                    egui::Event::PointerButton {
                        pos: position,
                        button: PointerButton::Primary,
                        pressed: true,
                        modifiers: egui::Modifiers::default(),
                    },
                ],
                ..Default::default()
            },
            |ui| {
                let response = ui.interact(
                    viewport,
                    egui::Id::unique("test-canvas"),
                    Sense::click_and_drag(),
                );
                app.handle_canvas_input(ui, &response, viewport, viewport_size, Rect::NOTHING);
            },
        );
        output.textures_delta.clear();
        assert_eq!(app.selected_relationship_id, Some(relationship_id));
        assert!(app.interaction.selected_card_ids().next().is_none());
        assert!(
            app.gesture.is_none(),
            "a line click must not start moving the card"
        );
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn relationship_crossing_uses_one_stable_circuit_style_bridge() {
        let paths = vec![
            RelationshipPaintPath {
                id: "under".to_string(),
                kind: RelationshipKind::DependsOn,
                visible_segments: vec![vec![Pos2::new(0.0, 50.0), Pos2::new(100.0, 50.0)]],
                crosses_boundary: false,
                arrowhead: None,
                label_position: None,
            },
            RelationshipPaintPath {
                id: "over".to_string(),
                kind: RelationshipKind::DependsOn,
                visible_segments: vec![vec![Pos2::new(50.0, 0.0), Pos2::new(50.0, 100.0)]],
                crosses_boundary: false,
                arrowhead: None,
                label_position: None,
            },
        ];

        let crossings = relationship_line_crossings(&paths);

        assert_eq!(
            crossings,
            vec![RelationshipLineCrossing {
                over_path_index: 1,
                position: Pos2::new(50.0, 50.0),
                direction: Vec2::new(0.0, 100.0),
            }]
        );
        let bridge = relationship_line_jump_points(crossings[0].position, crossings[0].direction)
            .expect("a full line segment supports a bridge");
        assert_eq!(bridge.len(), 9);
        assert_eq!(bridge[0], Pos2::new(50.0, 42.0));
        assert_eq!(bridge[4], Pos2::new(45.0, 50.0));
        assert_eq!(bridge[8], Pos2::new(50.0, 58.0));
    }

    #[test]
    fn relationship_lines_that_meet_at_a_shared_endpoint_do_not_create_a_bridge() {
        let paths = vec![
            RelationshipPaintPath {
                id: "first".to_string(),
                kind: RelationshipKind::DependsOn,
                visible_segments: vec![vec![Pos2::new(0.0, 0.0), Pos2::new(50.0, 50.0)]],
                crosses_boundary: false,
                arrowhead: None,
                label_position: None,
            },
            RelationshipPaintPath {
                id: "second".to_string(),
                kind: RelationshipKind::DependsOn,
                visible_segments: vec![vec![Pos2::new(50.0, 50.0), Pos2::new(100.0, 0.0)]],
                crosses_boundary: false,
                arrowhead: None,
                label_position: None,
            },
        ];

        assert!(relationship_line_crossings(&paths).is_empty());
    }

    #[test]
    fn semantic_relationship_hit_geometry_excludes_hidden_relationships() {
        let relationships = vec![
            RelationshipView {
                id: "painted".to_string(),
                source_id: 4,
                target_id: 8,
                kind: RelationshipKind::DependsOn,
                source_title: "Source".to_string(),
                target_title: "Target".to_string(),
            },
            RelationshipView {
                id: "hidden".to_string(),
                source_id: 4,
                target_id: 9,
                kind: RelationshipKind::DependsOn,
                source_title: "Source".to_string(),
                target_title: "Context target".to_string(),
            },
        ];
        let frame = SemanticZoomFrame {
            cards: Vec::new(),
            relationships: vec![infinite_note_canvas_canvas::SemanticZoomRelationship {
                id: "painted".to_string(),
                kind: RelationshipKind::DependsOn,
                label: "depends on".to_string(),
                source_id: 4,
                target_id: 8,
                opacity: 1.0,
            }],
            layers: Vec::new(),
        };

        let interactive = relationship_views_in_semantic_frame(&relationships, &frame);

        assert_eq!(
            interactive
                .iter()
                .map(|relationship| relationship.id.as_str())
                .collect::<Vec<_>>(),
            vec!["painted"]
        );
    }

    #[test]
    fn selecting_a_relationship_clears_card_and_boundary_focus_for_the_shared_inspector() {
        let mut app = test_app();
        app.interaction.select_only_card_id(4);
        app.selected_boundary_id = Some("boundary-1".to_string());

        app.set_relationship_selection(Some("relationship-1".to_string()));

        assert!(app.interaction.selected_card_ids().next().is_none());
        assert!(app.selected_boundary_id.is_none());
        assert_eq!(
            app.selected_relationship_id.as_deref(),
            Some("relationship-1")
        );
    }

    fn test_card(origin: Point) -> ConceptCard {
        ConceptCard {
            id: 4,
            title: "Selected concept".to_string(),
            summary: String::new(),
            kind: ConceptKind::Project,
            accent_color: None,
            bounds: CanvasRect::new(origin, Point::new(120.0, 80.0)),
        }
    }

    #[test]
    fn canvas_search_matches_card_names_and_summaries_without_case() {
        let mut card = test_card(Point::new(0.0, 0.0));
        card.title = "Northstar delivery".to_string();
        card.summary = "Validated by the security gate.".to_string();

        assert!(canvas_card_matches_search(&card, "northstar"));
        assert!(canvas_card_matches_search(&card, "SECURITY"));
        assert!(!canvas_card_matches_search(&card, "unrelated"));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn external_workspace_refresh_waits_for_idle_canvas_authoring() {
        let mut app = test_app();

        assert!(app.external_workspace_refresh_is_safe());

        app.gesture = Some(PointerGesture::Pan);
        assert!(!app.external_workspace_refresh_is_safe());

        app.gesture = None;
        app.side_pane_mode = SidePaneMode::CreateCard;
        assert!(!app.external_workspace_refresh_is_safe());

        app.side_pane_mode = SidePaneMode::Inspect;
        app.layer_cross_section_creating = true;
        assert!(!app.external_workspace_refresh_is_safe());
    }

    #[test]
    fn search_focus_centers_a_card_and_never_keeps_an_overview_scale() {
        let bounds = CanvasRect::new(Point::new(240.0, 120.0), Point::new(120.0, 80.0));
        let focused = camera_focused_on_card(
            Camera::new(Point::new(10.0, 20.0), 0.25),
            bounds,
            Point::new(800.0, 600.0),
        );

        assert_eq!(focused.scale, 0.8);
        assert_eq!(
            focused.world_to_screen(bounds.center()),
            Point::new(400.0, 300.0)
        );
    }

    #[test]
    fn filtered_hit_test_ignores_hidden_cards_and_keeps_nested_card_priority() {
        let parent = test_card(Point::new(0.0, 0.0));
        let mut child = test_card(Point::new(20.0, 20.0));
        child.id = 5;
        child.bounds = CanvasRect::new(Point::new(20.0, 20.0), Point::new(60.0, 40.0));
        let dataset = CanvasDataset::from_records(vec![parent, child], Vec::new());

        assert_eq!(
            filtered_card_hit_test(&dataset, Point::new(40.0, 40.0), |_| true).map(|card| card.id),
            Some(5)
        );
        assert_eq!(
            filtered_card_hit_test(&dataset, Point::new(40.0, 40.0), |card| card.id == 4)
                .map(|card| card.id),
            Some(4)
        );
    }

    #[test]
    fn manufacturing_routes_stay_cached_across_zoom_pan_and_semantic_layers() {
        let mut app = test_app();
        let document = examples::ALL[1].document().unwrap();
        let path = std::env::temp_dir().join(format!(
            "reason-room-routing-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::write(&path, document.to_pretty_json().unwrap()).unwrap();
        let session = WorkspaceSession::load_default(path.clone()).unwrap();
        app.dataset = session.dataset().clone();
        app.workspace_state = WorkspaceState::Loaded(Box::new(session));
        let viewport = Rect::from_min_size(Pos2::new(0.0, 55.0), Vec2::new(1589.0, 810.0));
        let relationships = app.active_relationships();
        let mut baseline = BTreeMap::new();
        for scale in [0.42, 0.43, 0.439, 0.12, 1.2] {
            let mut camera = Camera::new(Point::new(-700.0, 300.0), scale);
            camera.center_world_point_in_viewport(
                Point::new(3000.0, 2300.0),
                Point::new(viewport.width(), viewport.height()),
            );
            // Pan as well as zoom; neither should change the world-space route.
            camera.offset.x += scale * 31.0;
            app.interaction.set_camera(camera);
            app.refresh_display_dataset();
            let paths = app.visible_relationship_paths(viewport);
            assert!(!paths.is_empty(), "missing painted routes at {scale}");
            let mut cache = app.relationship_route_cache.borrow_mut();
            for edge in &relationships {
                if let Some(route) = cache.route(edge.source_id, edge.target_id) {
                    if baseline.is_empty() && scale != 0.42 {
                        panic!("missing baseline");
                    }
                    if scale == 0.42 {
                        baseline.insert(edge.id.clone(), (route.to_vec(), route.as_ptr()));
                    } else if let Some((expected, pointer)) = baseline.get(&edge.id) {
                        assert_eq!(route, expected, "route changed for {} at {scale}", edge.id);
                        assert_eq!(route.as_ptr(), *pointer, "unnecessary recalculation");
                        for point in route {
                            let projected = canvas_point_to_screen(*point, camera, viewport);
                            let restored = camera.screen_to_world(Point::new(
                                projected.x - viewport.left(),
                                projected.y - viewport.top(),
                            ));
                            assert!(
                                (restored.x - point.x).abs() < 0.01
                                    && (restored.y - point.y).abs() < 0.01
                            );
                        }
                    }
                }
            }
        }
        assert!(!baseline.is_empty());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn example_switch_refuses_to_discard_a_card_edit() {
        let mut app = test_app();
        app.dataset = CanvasDataset::from_records(vec![test_card(Point::default())], Vec::new());
        let id = app.dataset.cards().next().unwrap().id;
        app.set_selection(Some(id));
        app.inspector_draft = Some(ConceptInspectorDraft {
            concept_id: id,
            title: "unsaved replacement".into(),
            type_id: "service".into(),
            summary: String::new(),
            details: String::new(),
            assertion_attributes: BTreeMap::new(),
            assertion_key: String::new(),
            assertion_value: String::new(),
            assertion_error: None,
        });
        let before = app.dataset.cards().count();
        app.load_example(examples::ALL[0]);
        assert_eq!(app.dataset.cards().count(), before);
        assert!(!app.fit_example_on_next_frame);
        assert_eq!(
            app.inspector_draft.as_ref().unwrap().title,
            "unsaved replacement"
        );
    }

    #[test]
    fn example_overview_fits_below_the_floating_controls() {
        let bounds = CanvasRect::new(Point::new(-100.0, -170.0), Point::new(6400.0, 4820.0));
        let viewport = Point::new(1280.0, 665.0);
        let camera = example_overview_camera(bounds, viewport);
        let top_left = camera.world_to_screen(bounds.origin);
        let bottom_right = camera.world_to_screen(bounds.origin + bounds.size);
        assert!(top_left.x >= 0.0 && top_left.y >= 100.0);
        assert!(bottom_right.x <= viewport.x && bottom_right.y <= viewport.y);
        assert!(camera.scale < 0.24);
    }

    fn authored_layers_test_app(suffix: &str) -> (InfiniteNoteCanvasApp, PathBuf) {
        let mut document = WorkspaceDocument::sample();
        for (title, start, end) in [("Overview", 50.0, 100.0), ("Detail", 150.0, 200.0)] {
            let id = document
                .create_semantic_zoom_layer("current", "organization-landscape", title.to_string())
                .unwrap();
            document
                .update_semantic_zoom_layer_percent_settings(
                    "current",
                    "organization-landscape",
                    &id,
                    title.to_string(),
                    SemanticZoomLayerPercentSettings {
                        active_zoom_range: ZoomPercentRange {
                            start_percent: start,
                            end_percent: end,
                        },
                        fade_percent: 10.0,
                    },
                    1.0,
                )
                .unwrap();
        }
        let path = std::env::temp_dir().join(format!(
            "reason-room-layer-{suffix}-{}.json",
            std::process::id()
        ));
        fs::write(&path, document.to_pretty_json().unwrap()).unwrap();
        let session = WorkspaceSession::load_default(path.clone()).unwrap();
        let mut app = test_app();
        app.dataset = session.dataset().clone();
        app.workspace_state = WorkspaceState::Loaded(Box::new(session));
        app.refresh_canvas_type_id_index();
        (app, path)
    }

    #[test]
    fn rollup_routes_hidden_child_edge_from_visible_parent_in_the_app() {
        let (mut app, path) = authored_layers_test_app("rollup-routing");
        let mut document = app.session().unwrap().document().clone();
        let layers = document.current_state.lenses[0]
            .semantic_zoom_layers
            .clone();
        document.current_state.lenses[0].placements = vec![
            infinite_note_canvas_canvas::ConceptPlacement {
                concept_id: 1,
                bounds: CanvasRect::new(Point::default(), Point::new(900.0, 800.0)),
            },
            infinite_note_canvas_canvas::ConceptPlacement {
                concept_id: 2,
                bounds: CanvasRect::new(Point::new(100.0, 100.0), Point::new(200.0, 100.0)),
            },
            infinite_note_canvas_canvas::ConceptPlacement {
                concept_id: 4,
                bounds: CanvasRect::new(Point::new(1100.0, 100.0), Point::new(200.0, 150.0)),
            },
        ];
        document.current_state.lenses[0].relationships =
            Some(vec![infinite_note_canvas_canvas::TypedRelationship {
                id: "rolled".to_string(),
                source_id: 2,
                target_id: 4,
                kind: RelationshipKind::DependsOn,
                label: "Depends on".to_string(),
            }]);
        for (id, index) in [(1, 0), (2, 1), (4, 0)] {
            document
                .set_semantic_zoom_card_layer_mode(
                    "current",
                    "organization-landscape",
                    id,
                    SemanticZoomCardLayerMode::Manual {
                        layer_ids: vec![layers[index].id.clone()],
                    },
                )
                .unwrap();
        }
        fs::write(&path, document.to_pretty_json().unwrap()).unwrap();
        let session = WorkspaceSession::load_default(path.clone()).unwrap();
        app.dataset = session.dataset().clone();
        app.workspace_state = WorkspaceState::Loaded(Box::new(session));
        app.interaction
            .set_camera(Camera::new(Point::default(), 0.75));
        app.roll_up_relationships = true;
        app.refresh_display_dataset();
        assert!(app.dataset.card_by_id(2).is_none());
        assert_eq!(
            app.active_relationships().len(),
            1,
            "Hidden child still supplies the original edge"
        );
        let viewport = Rect::from_min_size(Pos2::ZERO, Vec2::new(1400.0, 900.0));
        let paths = app.visible_relationship_paths(viewport);
        assert_eq!(paths.len(), 1, "Rollup reaches the routing and paint path");
        assert_eq!(paths[0].id, "rolled");
        let route = app
            .relationship_route_cache
            .borrow_mut()
            .route(1, 4)
            .unwrap()
            .to_vec();
        app.interaction
            .set_camera(Camera::new(Point::new(12.0, 6.0), 0.76));
        assert_eq!(app.visible_relationship_paths(viewport).len(), 1);
        assert_eq!(
            app.relationship_route_cache
                .borrow_mut()
                .route(1, 4)
                .unwrap(),
            route
        );
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn card_context_layer_action_saves_a_centered_resize_in_the_active_view() {
        let (mut app, path) = authored_layers_test_app("context-resize");
        let layer = app.session().unwrap().document().current_state.lenses[0].semantic_zoom_layers
            [0]
        .id
        .clone();
        let staged = app
            .session()
            .unwrap()
            .stage_set_semantic_zoom_card_layer_mode(
                1,
                SemanticZoomCardLayerMode::Manual {
                    layer_ids: vec![layer],
                },
            )
            .unwrap();
        app.commit_staged_action(Ok(staged), Some(1), false, "test");
        let before = app.session().unwrap().document().current_state.lenses[0]
            .placements
            .iter()
            .find(|placement| placement.concept_id == 1)
            .unwrap()
            .bounds;
        app.apply_canvas_context_menu_action(CanvasContextMenuAction::ShiftCardLayer(1, true));
        let session = app.session().unwrap();
        let after = session.document().current_state.lenses[0]
            .placements
            .iter()
            .find(|placement| placement.concept_id == 1)
            .unwrap()
            .bounds;
        assert_ne!(before.size, after.size);
        assert!((before.center().x - after.center().x).abs() < 0.001);
        assert_eq!(
            session
                .document()
                .semantic_zoom_card_layer_visibility(
                    session.active_version_id(),
                    session.active_lens_id(),
                    1
                )
                .unwrap()
                .base_layer_id
                .as_deref(),
            Some("layer-detail")
        );
        let saved = WorkspaceDocument::parse_json(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(&saved, session.document());
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn clicking_create_between_layers_saves_the_chosen_ordinal_and_range() {
        let (mut app, path) = authored_layers_test_app("insert-input");
        app.open_layer_cross_section();
        let ctx = egui::Context::default();
        let viewport = Rect::from_min_size(Pos2::ZERO, Vec2::new(600.0, 1600.0));
        let frame = |app: &mut InfiniteNoteCanvasApp, events| {
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(viewport),
                    events,
                    ..Default::default()
                },
                |ui| app.semantic_zoom_layer_cross_section(ui),
            );
            output.textures_delta.clear();
            output
        };
        let text_positions = |output: &egui::FullOutput, label: &str| {
            output
                .shapes
                .iter()
                .filter_map(|shape| {
                    if let egui::Shape::Text(text) = &shape.shape
                        && text.galley.job.text == label
                    {
                        return Some(text.pos + text.galley.size() * 0.5);
                    }
                    None
                })
                .collect::<Vec<_>>()
        };
        let initial = frame(&mut app, vec![]);
        let insert = text_positions(&initial, "+ Create layer here")[1];
        for pressed in [true, false] {
            let _ = frame(
                &mut app,
                vec![
                    egui::Event::PointerMoved(insert),
                    egui::Event::PointerButton {
                        pos: insert,
                        button: PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
        }
        assert_eq!(app.layer_creation_index, 1);
        assert_eq!(app.layer_creation_active_zoom_start_percent, 150.0);
        assert_eq!(app.layer_creation_active_zoom_end_percent, 200.0);
        app.layer_creation_title = "Inserted through UI".to_string();
        let form = frame(&mut app, vec![]);
        let create = text_positions(&form, "Create layer")[0];
        let overview = text_positions(&form, "50% to 100%. Fade: 10 percentage points.")[0];
        let detail = text_positions(&form, "150% to 200%. Fade: 10 percentage points.")[0];
        assert!(
            overview.y < create.y && create.y < detail.y,
            "creation form must stay between the selected neighboring rows"
        );
        for pressed in [true, false] {
            let _ = frame(
                &mut app,
                vec![
                    egui::Event::PointerMoved(create),
                    egui::Event::PointerButton {
                        pos: create,
                        button: PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
        }
        let saved = WorkspaceDocument::parse_json(&fs::read_to_string(&path).unwrap()).unwrap();
        let layers = &saved.current_state.lenses[0].semantic_zoom_layers;
        assert_eq!(layers.len(), 3);
        assert_eq!(layers[1].title, "Inserted through UI");
        assert_eq!(
            layers[1].percent_settings.unwrap().active_zoom_range,
            ZoomPercentRange {
                start_percent: 150.0,
                end_percent: 200.0
            }
        );
        assert_eq!(
            layers[2].percent_settings.unwrap().active_zoom_range,
            ZoomPercentRange {
                start_percent: 200.0,
                end_percent: 250.0
            }
        );
        fs::remove_file(path).unwrap();
    }

    fn layer_test_frame(
        ctx: &egui::Context,
        app: &mut InfiniteNoteCanvasApp,
        events: Vec<egui::Event>,
    ) -> egui::FullOutput {
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(600.0, 1600.0))),
                events,
                ..Default::default()
            },
            |ui| app.semantic_zoom_layer_cross_section(ui),
        );
        output.textures_delta.clear();
        output
    }

    fn layer_test_text_position(output: &egui::FullOutput, label: &str) -> Pos2 {
        output
            .shapes
            .iter()
            .find_map(|shape| {
                if let egui::Shape::Text(text) = &shape.shape
                    && text.galley.job.text == label
                {
                    Some(text.pos + text.galley.size() * 0.5)
                } else {
                    None
                }
            })
            .unwrap_or_else(|| panic!("missing layer text: {label}"))
    }

    fn layer_test_pointer(pos: Pos2, pressed: bool) -> egui::Event {
        egui::Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        }
    }

    #[test]
    fn layer_end_typing_saves_only_on_blur_or_enter() {
        for finish in [None, Some(egui::Key::Enter)] {
            let (mut app, path) = authored_layers_test_app(&format!("typing-{finish:?}"));
            app.open_layer_cross_section();
            let ctx = egui::Context::default();
            let initial = layer_test_frame(&ctx, &mut app, vec![]);
            let end = layer_test_text_position(&initial, "200");
            for pressed in [true, false] {
                layer_test_frame(
                    &ctx,
                    &mut app,
                    vec![
                        egui::Event::PointerMoved(end),
                        layer_test_pointer(end, pressed),
                    ],
                );
            }
            layer_test_frame(&ctx, &mut app, vec![]);
            let before = fs::read_to_string(&path).unwrap();
            // Incomplete "2" must neither save nor pull Start down from 150%.
            for text in ["2", "5", "0"] {
                layer_test_frame(&ctx, &mut app, vec![egui::Event::Text(text.to_string())]);
                assert_eq!(fs::read_to_string(&path).unwrap(), before);
                assert_eq!(
                    app.layer_cross_section_drafts[1].active_zoom_start_percent,
                    150.0
                );
            }
            if let Some(key) = finish {
                layer_test_frame(
                    &ctx,
                    &mut app,
                    vec![egui::Event::Key {
                        key,
                        physical_key: None,
                        pressed: true,
                        repeat: false,
                        modifiers: egui::Modifiers::NONE,
                    }],
                );
            } else {
                let blank = Pos2::new(550.0, 1500.0);
                for pressed in [true, false] {
                    layer_test_frame(
                        &ctx,
                        &mut app,
                        vec![
                            egui::Event::PointerMoved(blank),
                            layer_test_pointer(blank, pressed),
                        ],
                    );
                }
            }
            layer_test_frame(&ctx, &mut app, vec![]);
            let saved = WorkspaceDocument::parse_json(&fs::read_to_string(&path).unwrap()).unwrap();
            let range = saved.current_state.lenses[0].semantic_zoom_layers[1]
                .percent_settings
                .unwrap()
                .active_zoom_range;
            assert_eq!(range.start_percent, 150.0);
            assert_eq!(range.end_percent, 250.0);
            fs::remove_file(path).unwrap();
        }
    }

    #[test]
    fn dragging_layer_onto_first_row_persists_existing_range_slots() {
        let (mut app, path) = authored_layers_test_app("drag-first-row");
        app.open_layer_cross_section();
        let ctx = egui::Context::default();
        let initial = layer_test_frame(&ctx, &mut app, vec![]);
        let target = layer_test_text_position(&initial, "Drag");
        let source = initial
            .shapes
            .iter()
            .filter_map(|shape| {
                if let egui::Shape::Text(text) = &shape.shape
                    && text.galley.job.text == "Drag"
                {
                    Some(text.pos + text.galley.size() * 0.5)
                } else {
                    None
                }
            })
            .nth(1)
            .unwrap();
        layer_test_frame(
            &ctx,
            &mut app,
            vec![
                egui::Event::PointerMoved(source),
                layer_test_pointer(source, true),
            ],
        );
        layer_test_frame(
            &ctx,
            &mut app,
            vec![egui::Event::PointerMoved(source + Vec2::new(0.0, -12.0))],
        );
        layer_test_frame(&ctx, &mut app, vec![egui::Event::PointerMoved(target)]);
        layer_test_frame(&ctx, &mut app, vec![layer_test_pointer(target, false)]);
        let saved = WorkspaceDocument::parse_json(&fs::read_to_string(&path).unwrap()).unwrap();
        let layers = &saved.current_state.lenses[0].semantic_zoom_layers;
        assert_eq!(layers[0].title, "Detail");
        assert_eq!(
            layers[0].percent_settings.unwrap().active_zoom_range,
            ZoomPercentRange {
                start_percent: 50.0,
                end_percent: 100.0
            }
        );
        assert_eq!(layers[1].title, "Overview");
        assert_eq!(
            layers[1].percent_settings.unwrap().active_zoom_range,
            ZoomPercentRange {
                start_percent: 150.0,
                end_percent: 200.0
            }
        );
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn painted_summary_combines_size_and_layer_fade_and_shift_border_is_amber() {
        let ctx = egui::Context::default();
        let mut card = test_card(Point::default());
        card.summary = "Visible summary".to_string();
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            paint_card(
                ui.painter(),
                Rect::from_min_size(Pos2::new(20.0, 20.0), Vec2::new(150.0, 80.0)),
                &card,
                CardPaintState {
                    selected: true,
                    hovered: false,
                    relationship_port_hovered: None,
                    resize_from_center: None,
                    layer_shifting: true,
                    detail_level: SemanticZoomLevel::Card,
                    opacity: 0.8,
                },
            );
        });
        output.textures_delta.clear();
        assert!(output.shapes.iter().any(|shape| matches!(&shape.shape, egui::Shape::Rect(rect) if rect.stroke.color == color_with_opacity(ATLAS_CONTROL_AMBER, 0.8))));
        let summary = output
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::Shape::Text(text) if text.galley.job.text == "Visible summary" => Some(text),
                _ => None,
            })
            .expect("A roomy displayed card paints its summary even below global detail zoom");
        assert_eq!(
            summary.fallback_color,
            color_with_opacity(ATLAS_QUIET_GRAPHITE, 0.4)
        );
    }

    fn test_app() -> InfiniteNoteCanvasApp {
        InfiniteNoteCanvasApp {
            dataset: CanvasDataset::from_records(Vec::new(), Vec::new()),
            interaction: CanvasInteraction::new(Camera::default()),
            workspace_state: WorkspaceState::ReadOnlyFailure("test workspace".to_string()),
            gesture: None,
            previous_shift_hover: None,
            dialog: None,
            inspector_draft: None,
            card_clipboard: None,
            publish_card_clipboard: false,
            last_canvas_pointer: None,
            last_canvas_viewport: None,
            pending_concept_name_focus_id: None,
            pending_creation_name_focus: None,
            inspector_collapsed: false,
            side_pane_mode: SidePaneMode::Inspect,
            semantic_zoom_layer_id: None,
            semantic_zoom_layer_draft: None,
            preferences: Arc::new(Mutex::new(PreferencesWindowState::default())),
            settings_window_requested: Arc::new(AtomicBool::new(false)),
            new_canvas_requested: Arc::new(AtomicBool::new(false)),
            open_canvas_requested: Arc::new(AtomicBool::new(false)),
            save_canvas_requested: Arc::new(AtomicBool::new(false)),
            save_as_canvas_requested: Arc::new(AtomicBool::new(false)),
            example_requested: Arc::new(Mutex::new(None)),
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            _native_application_menu: None,
            show_all_cards: false,
            roll_up_relationships: false,
            card_creation_draft: ConceptEditorDraft {
                title: String::new(),
                type_id: None,
                summary: String::new(),
            },
            boundary_creation_draft: BoundaryEditorDraft {
                title: String::new(),
                summary: String::new(),
            },
            layer_creation_title: String::new(),
            layer_creation_active_zoom_start_percent: 1.0,
            layer_creation_active_zoom_end_percent: 65.0,
            layer_creation_fade_percent: 10.0,
            layer_cross_section_drafts: Vec::new(),
            layer_cross_section_creating: false,
            layer_creation_index: 0,
            canvas_search_query: String::new(),
            canvas_search_match_index: 0,
            canvas_type_filter_ids: BTreeSet::new(),
            canvas_type_id_by_concept_id: BTreeMap::new(),
            canvas_context_menu: None,
            pending_canvas_creation_position: None,
            version_creation_draft: WorkspaceTitleDraft {
                title: String::new(),
            },
            lens_creation_draft: LensCreationDraft {
                title: String::new(),
                clone_current: false,
            },
            selected_boundary_id: None,
            selected_relationship_id: None,
            primary_relationship_concept_id: None,
            boundary_inspector_draft: None,
            hover_target: None,
            lens_cameras: HashMap::new(),
            lens_canvas_controls: HashMap::new(),
            canvas_viewport_size: Point::default(),
            fit_example_on_next_frame: false,
            relationship_route_cache: RefCell::default(),
            notices: Vec::new(),
            type_change_failure_toast: None,
            ui_time_seconds: 0.0,
            #[cfg(not(target_arch = "wasm32"))]
            next_external_workspace_refresh_at: 0.0,
            #[cfg(not(target_arch = "wasm32"))]
            external_workspace_refresh_error: None,
        }
    }

    #[test]
    fn new_canvas_title_produces_a_stable_safe_workspace_identifier() {
        assert_eq!(
            workspace_id_from_canvas_title("Northstar AWS / Delivery"),
            "northstar-aws-delivery"
        );
        assert_eq!(workspace_id_from_canvas_title("..."), "canvas");
    }

    #[test]
    fn suggested_new_canvas_path_adds_a_sequence_after_a_collision() {
        let directory = std::env::temp_dir().join(format!(
            "infinite-note-canvas-new-path-test-{}",
            std::process::id()
        ));
        fs::create_dir_all(&directory).unwrap();
        let initial = directory.join("finance-reporting.infinite-note.json");
        fs::write(&initial, "existing document").unwrap();

        let suggested = suggested_new_canvas_path_in(&directory, "Finance reporting");

        assert_eq!(
            suggested,
            directory.join("finance-reporting-2.infinite-note.json")
        );
        fs::remove_file(initial).unwrap();
        fs::remove_dir(directory).unwrap();
    }

    fn displayed_test_card_bounds(
        cards: &[(u32, CanvasRect, bool)],
        camera: Camera,
        viewport: Rect,
    ) -> BTreeMap<u32, CanvasRect> {
        cards
            .iter()
            .map(|(concept_id, bounds, selected)| {
                let painted_bounds =
                    card_paint_bounds(canvas_rect_to_screen(*bounds, camera, viewport), *selected);
                (
                    *concept_id,
                    CanvasRect::new(
                        Point::new(painted_bounds.left(), painted_bounds.top()),
                        Point::new(painted_bounds.width(), painted_bounds.height()),
                    ),
                )
            })
            .collect()
    }

    fn screen_rect_from_canvas_bounds(bounds: CanvasRect) -> Rect {
        Rect::from_min_size(
            Pos2::new(bounds.origin.x, bounds.origin.y),
            Vec2::new(bounds.size.x, bounds.size.y),
        )
    }

    fn assert_path_stays_outside_painted_card(path: &[Pos2], bounds: Rect) {
        assert!(
            path.windows(2)
                .all(|segment| !screen_segment_crosses_rect_interior(
                    segment[0], segment[1], bounds
                )),
            "path {path:?} crosses painted card bounds {bounds:?}"
        );
    }

    fn point_on_rect_edge(point: Pos2, bounds: Rect) -> bool {
        const EPSILON: f32 = 0.0001;

        point.x >= bounds.left() - EPSILON
            && point.x <= bounds.right() + EPSILON
            && point.y >= bounds.top() - EPSILON
            && point.y <= bounds.bottom() + EPSILON
            && ((point.x - bounds.left()).abs() <= EPSILON
                || (point.x - bounds.right()).abs() <= EPSILON
                || (point.y - bounds.top()).abs() <= EPSILON
                || (point.y - bounds.bottom()).abs() <= EPSILON)
    }

    fn point_is_in_rect_interior(point: Pos2, bounds: Rect) -> bool {
        bounds.left() < point.x
            && point.x < bounds.right()
            && bounds.top() < point.y
            && point.y < bounds.bottom()
    }

    fn screen_segment_crosses_rect_interior(start: Pos2, end: Pos2, bounds: Rect) -> bool {
        let delta = end - start;
        let Some((x_start, x_end)) =
            open_screen_axis_interval(start.x, delta.x, bounds.left(), bounds.right())
        else {
            return false;
        };
        let Some((y_start, y_end)) =
            open_screen_axis_interval(start.y, delta.y, bounds.top(), bounds.bottom())
        else {
            return false;
        };

        x_start.max(y_start).max(0.0) < x_end.min(y_end).min(1.0)
    }

    fn open_screen_axis_interval(
        start: f32,
        delta: f32,
        minimum: f32,
        maximum: f32,
    ) -> Option<(f32, f32)> {
        if delta == 0.0 {
            return (minimum < start && start < maximum)
                .then_some((f32::NEG_INFINITY, f32::INFINITY));
        }

        let first = (minimum - start) / delta;
        let second = (maximum - start) / delta;
        Some((first.min(second), first.max(second)))
    }
}
