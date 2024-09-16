use std::collections::{hash_map, HashMap};
use std::path::Path;
use std::sync::Arc;

use floem::kurbo::{self, Point, Vec2};
use floem::text::TextLayout;
use floem::Renderer as FloemRenderer;
use arc_swap::access::Map;
use arc_swap::ArcSwap;
use floem::{prop_extractor, IntoView, View, ViewId};
use helix_core::{pos_at_coords, Position, Selection, Transaction};
use helix_view::graphics::Rect as HelixRect;
use helix_view::handlers::Handlers;
use helix_view::{doc_mut, handlers, theme, DocumentId};
use helix_view::{editor::Config, Editor};

use floem::style::{CustomStylable, FontProps, LineHeight, Style, TextColor};

prop_extractor! {
    Extractor {
        color: TextColor,
        line_height: LineHeight,
    }
}

pub fn helix_color_to_peniko(color: helix_view::graphics::Color) -> floem::peniko::Color {
    use floem::peniko::Color as PColor;
    use helix_view::graphics::Color as HColor;

    match color {
        HColor::Reset => PColor::TRANSPARENT,
        HColor::Black => PColor::BLACK,
        HColor::Red => PColor::RED,
        HColor::Green => PColor::GREEN,
        HColor::Yellow => PColor::YELLOW,
        HColor::Blue => PColor::BLUE,
        HColor::Magenta => PColor::MAGENTA,
        HColor::Cyan => PColor::CYAN,
        HColor::Gray => PColor::GRAY,
        HColor::LightRed => PColor { r: 255, g: 128, b: 128, a: 255 },
        HColor::LightGreen => PColor::LIGHT_GREEN,
        HColor::LightYellow => PColor::LIGHT_YELLOW,
        HColor::LightBlue => PColor::LIGHT_BLUE,
        HColor::LightMagenta => PColor{ r: 255, g: 128, b: 255, a: 255 },
        HColor::LightCyan => PColor::LIGHT_CYAN,
        HColor::LightGray => PColor::LIGHT_GRAY,
        HColor::White => PColor::WHITE,
        HColor::Rgb(r, g, b) => PColor::rgba8(r, g, b, 255),
        HColor::Indexed(index) => {
            // If you have an indexed color palette, convert index to an RGB color.
            // Here's a placeholder that maps the index to a predefined color.
            // You should replace this with your actual indexed color mapping logic.
            match index {
                0 => PColor::BLACK,
                1 => PColor::RED,
                2 => PColor::GREEN,
                3 => PColor::YELLOW,
                4 => PColor::BLUE,
                5 => PColor::MAGENTA,
                6 => PColor::CYAN,
                7 => PColor::WHITE,
                _ => PColor::GRAY, // Default fallback color
            }
        }
    }
}


pub struct HelixEditor {
    id: ViewId,
    editor: Editor,
    // doc_view_positions: std::collections::HashMap<DocumentId, HashMap<helix_view::ViewId, Vec2>>,
    text_layouts: HashMap<helix_view::ViewId, TextLayout>,
    size: floem::kurbo::Size,
    text_layout: TextLayout,
    font_props: FontProps,
    extractor: Extractor,
}
impl View for HelixEditor {
    fn id(&self) -> ViewId {
        self.id
    }

    fn debug_name(&self) -> std::borrow::Cow<'static, str> {
        "Helix Editor".into()
    }

    fn view_style(&self) -> Option<floem::style::Style> {
        let theme = &self.editor.theme;
        let background = theme.get("ui.background").bg.map(|c| helix_color_to_peniko(c));
        Some(Style::new().apply_opt(background, Style::background))
        
    }

    fn style_pass(&mut self, cx: &mut floem::context::StyleCx<'_>) {
        if self.font_props.read(cx) | self.extractor.read(cx) {
            // self.text_layout = None;
            // self.available_text = None;
            // self.available_width = None;
            // self.available_text_layout = None;
            self.id.request_layout();
        }
        // if self.selection_style.read(cx) {
        //     self.id.request_paint();
        // }
    }

    fn event_before_children(&mut self, _cx: &mut floem::context::EventCx, event: &floem::event::Event) -> floem::event::EventPropagation {
        match event{
            // TODO: Handle these
            floem::event::Event::PointerDown(_) => {},
            floem::event::Event::PointerUp(_) => {},
            floem::event::Event::PointerMove(_) => {},
            floem::event::Event::PointerWheel(_) => {},
            floem::event::Event::DroppedFile(_) => {},
            floem::event::Event::KeyDown(ke) => {
                self.handle_key_event(ke);
            },
            floem::event::Event::FocusGained => {},
            floem::event::Event::FocusLost => {},
            _ => {}
        }
        floem::event::EventPropagation::Stop
    }


    fn compute_layout(&mut self, _cx: &mut floem::context::ComputeLayoutCx) -> Option<floem::kurbo::Rect> {
        let layout = self.id.get_layout().unwrap_or_default();
        let size = layout.size;
        self.size = (size.width as f64, size.height as f64).into();

        let helix_size = todo!("compute this");
        self.editor.resize(helix_size);

        for (view, focused) in self.editor.tree.views() {
            let id = view.id;
            let area = view.area;
            let pixel_size: kurbo::Size = todo!();
            match self.text_layouts.entry(id) {
                hash_map::Entry::Occupied(oe) => oe.get_mut().set_size(pixel_size.width as f32, pixel_size.height as f32),
                hash_map::Entry::Vacant(ve) => {
                    let doc = self.editor.document(view.id)
                },
            }
            view.area
        }

        self.text_layout.set_size(size.width, size.height);

        let text = self.editor.
        self.text_layout.set_text(, );
        None
    }


    fn paint(&mut self, cx: &mut floem::context::PaintCx) {
        
        // clear with background color
        let config = self.editor.config();

        // check if bufferline should be rendered
        use helix_view::editor::BufferLine;
        let use_bufferline = match config.bufferline {
            BufferLine::Always => true,
            BufferLine::Multiple if self.editor.documents.len() > 1 => true,
            _ => false,
        };

        // let cell_width = self.text_layout_engine.line_width() / self.text_layout_engine.total_characters_in_line();
        // let cell_height = self.text_layout_engine.line_height();

        // let x = (self.area.x0 / cell_width).floor() as u16;
        // let y = (self.area.y0 / cell_height).floor() as u16;
        // let width = (self.area.width() / cell_width).ceil() as u16;
        // let height = (self.area.height() / cell_height).ceil() as u16;

        let line_height = self.extractor.line_height().unwrap_or(floem::text::LineHeightValue::Normal(1.));
        let width 

        let area = HelixRect::new(0, 0, 300, 200);

        // -1 for commandline and -1 for bufferline
        let mut editor_area = area.clip_bottom(1);
        if use_bufferline {
            editor_area = editor_area.clip_top(1);
        }

        // if the terminal size suddenly changed, we need to trigger a resize
        self.editor.resize(editor_area);


        for (view, is_focused) in self.editor.tree.views() {
            let doc = self.editor.document(view.doc).unwrap();
            self.render_view(&self.editor, doc, view, area, cx, is_focused);
        }
    
    }
}
impl HelixEditor {

    fn point_to_editor_pos(self, point: Point) -> Position {
        self.text_layout.hit_point(point).
    }
    
    fn render_view<'a>(&self, editor: &Editor, doc: &helix_view::Document, view: &helix_view::View, area: HelixRect, cx: &mut floem::context::PaintCx<'a>, is_focused: bool) {

        let inner = view.inner_area(doc);
        let area = view.area;
        let theme = &editor.theme;
        let config = editor.config();

        let view_offset = self.doc_view_positions.get(&doc.id()).unwrap().get(&view.id).unwrap();
        // let view_offset = doc.view_offset(view.id);

        let text_annotations = view.text_annotations(doc, Some(theme));
        let mut decorations = DecorationManager::default();

        if is_focused && config.cursorline {
            decorations.add_decoration(Self::render_cursorline(doc, view, theme));
        }

        if is_focused && config.cursorcolumn {
            Self::highlight_cursorcolumn(doc, view, surface, theme, inner, &text_annotations);
        }

        // Set DAP highlights, if needed.
        if let Some(frame) = editor.current_stack_frame() {
            let dap_line = frame.line.saturating_sub(1);
            let style = theme.get("ui.highlight.frameline");
            let line_decoration = move |renderer: &mut TextRenderer, pos: LinePos| {
                if pos.doc_line != dap_line {
                    return;
                }
                renderer.set_style(Rect::new(inner.x, pos.visual_line, inner.width, 1), style);
            };

            decorations.add_decoration(line_decoration);
        }

        let syntax_highlights =
            Self::doc_syntax_highlights(doc, view_offset.anchor, inner.height, theme);

        let mut overlay_highlights =
            Self::empty_highlight_iter(doc, view_offset.anchor, inner.height);
        let overlay_syntax_highlights = Self::overlay_syntax_highlights(
            doc,
            view_offset.anchor,
            inner.height,
            &text_annotations,
        );
        if !overlay_syntax_highlights.is_empty() {
            overlay_highlights =
                Box::new(syntax::merge(overlay_highlights, overlay_syntax_highlights));
        }

        for diagnostic in Self::doc_diagnostics_highlights(doc, theme) {
            // Most of the `diagnostic` Vecs are empty most of the time. Skipping
            // a merge for any empty Vec saves a significant amount of work.
            if diagnostic.is_empty() {
                continue;
            }
            overlay_highlights = Box::new(syntax::merge(overlay_highlights, diagnostic));
        }

        if is_focused {
            let highlights = syntax::merge(
                overlay_highlights,
                Self::doc_selection_highlights(
                    editor.mode(),
                    doc,
                    view,
                    theme,
                    &config.cursor_shape,
                    self.terminal_focused,
                ),
            );
            let focused_view_elements = Self::highlight_focused_view_elements(view, doc, theme);
            if focused_view_elements.is_empty() {
                overlay_highlights = Box::new(highlights)
            } else {
                overlay_highlights = Box::new(syntax::merge(highlights, focused_view_elements))
            }
        }

        let gutter_overflow = view.gutter_offset(doc) == 0;
        if !gutter_overflow {
            Self::render_gutter(
                editor,
                doc,
                view,
                view.area,
                theme,
                is_focused & self.terminal_focused,
                &mut decorations,
            );
        }

        let primary_cursor = doc
            .selection(view.id)
            .primary()
            .cursor(doc.text().slice(..));
        if is_focused {
            decorations.add_decoration(text_decorations::Cursor {
                cache: &editor.cursor_cache,
                primary_cursor,
            });
        }
        let width = view.inner_width(doc);
        let config = doc.config.load();
        let enable_cursor_line = view
            .diagnostics_handler
            .show_cursorline_diagnostics(doc, view.id);
        let inline_diagnostic_config = config.inline_diagnostics.prepare(width, enable_cursor_line);
        decorations.add_decoration(InlineDiagnostics::new(
            doc,
            theme,
            primary_cursor,
            inline_diagnostic_config,
            config.end_of_line_diagnostics,
        ));
        render_document(
            surface,
            inner,
            doc,
            view_offset,
            &text_annotations,
            syntax_highlights,
            overlay_highlights,
            theme,
            decorations,
        );
        Self::render_rulers(editor, doc, view, inner, surface, theme);

        // if we're not at the edge of the screen, draw a right border
        if viewport.right() != view.area.right() {
            let x = area.right();
            let border_style = theme.get("ui.window");
            for y in area.top()..area.bottom() {
                surface[(x, y)]
                    .set_symbol(tui::symbols::line::VERTICAL)
                    //.set_symbol(" ")
                    .set_style(border_style);
            }
        }

        if config.inline_diagnostics.disabled()
            && config.end_of_line_diagnostics == DiagnosticFilter::Disable
        {
            Self::render_diagnostics(doc, view, inner, surface, theme);
        }

        let statusline_area = view
            .area
            .clip_top(view.area.height.saturating_sub(1))
            .clip_bottom(1); // -1 from bottom to remove commandline

        let mut context =
            statusline::RenderContext::new(editor, doc, view, is_focused, &self.spinners);

        statusline::render(&mut context, statusline_area, surface);
    
    }

    fn render_cursorline<'a>(doc: &helix_view::Document, view: &helix_view::View, theme: &theme::Theme, cx: &mut floem::context::PaintCx<'a>) {
        let text = doc.text().slice(..);
        // TODO only highlight the visual line that contains the cursor instead of the full visual line
        let primary_line = doc.selection(view.id).primary().cursor_line(text);

        // The secondary_lines do contain the primary_line, it doesn't matter
        // as the else-if clause in the loop later won't test for the
        // secondary_lines if primary_line == line.
        // It's used inside a loop so the collect isn't needless:
        // https://github.com/rust-lang/rust-clippy/issues/6164
        // #[allow(clippy::needless_collect)]
        let secondary_lines: Vec<_> = doc
            .selection(view.id)
            .iter()
            .map(|range| range.cursor_line(text))
            .collect();

        let primary_style = theme.get("ui.cursorline.primary");
        let secondary_style = theme.get("ui.cursorline.secondary");
        let viewport = view.area;

        cx.fill();

        move |renderer: &mut TextRenderer, pos: LinePos| {
            let area = Rect::new(viewport.x, pos.visual_line, viewport.width, 1);
            if primary_line == pos.doc_line {
                renderer.set_style(area, primary_style);
            } else if secondary_lines.binary_search(&pos.doc_line).is_ok() {
                renderer.set_style(area, secondary_style);
            }
        }

    }

    fn handle_key_event(&self, ke: &floem::keyboard::KeyEvent) -> _ {
        if ke.modifiers.is_empty() {
            let focused = self.editor.tree.focus;
            let view = self.editor.tree.get(focused);
            let current_doc = view.doc;
            Transaction::default()
                self.editor.document(current_doc).unwrap().apply(, )
            view.apply(, )
            view.apply( )

            
        }
    }
}

pub fn test() -> impl IntoView {
    let lang_loader = helix_core::config::user_lang_loader().unwrap_or_else(|err| {
        eprintln!("{}", err);
        eprintln!("Press <ENTER> to continue with default language config");
        use std::io::Read;
        // This waits for an enter press.
        let _ = std::io::stdin().read(&mut []);
        helix_core::config::default_lang_loader()
    });

    let mut theme_parent_dirs = vec![helix_loader::config_dir()];
    theme_parent_dirs.extend(helix_loader::runtime_dirs().iter().cloned());
    let theme_loader = std::sync::Arc::new(theme::Loader::new(&theme_parent_dirs));

    let theme = theme_loader.default_theme(true);

    let syn_loader = Arc::new(ArcSwap::from_pointee(lang_loader));

    let size = HelixRect::new(0, 0, 100, 100);
    let config = Arc::new(arc_swap::access::Constant(Config::default()));
    let (completion_tx, completion_rx) = tokio::sync::mpsc::channel(10);
    let completion_stream = tokio_stream::wrappers::ReceiverStream::new(completion_rx);
    let completion_sig = floem::ext_event::create_signal_from_stream(
        handlers::lsp::CompletionEvent::Cancel,
        completion_stream,
    );

    let (signature_hints_tx, signature_hints_rx) = tokio::sync::mpsc::channel(10);
    let signature_hints_stream = tokio_stream::wrappers::ReceiverStream::new(signature_hints_rx);
    let signature_hints_sig = floem::ext_event::create_signal_from_stream(
        handlers::lsp::SignatureHelpEvent::Cancel,
        signature_hints_stream,
    );

    let (auto_save_tx, auto_save_rx) = tokio::sync::mpsc::channel(10);
    let auto_save_stream = tokio_stream::wrappers::ReceiverStream::new(auto_save_rx);
    let auto_save_sig = floem::ext_event::create_signal_from_stream(
        handlers::AutoSaveEvent::LeftInsertMode,
        auto_save_stream,
    );

    let handlers = Handlers {
        completions: completion_tx,
        signature_hints: signature_hints_tx,
        auto_save: auto_save_tx,
    };
    let mut editor = Editor::new(
        size,
        theme_loader.clone(),
        syn_loader.clone(),
        config.clone(),
        handlers,
    );

    editor.set_theme(theme);

    let _blank = editor.new_file(helix_view::editor::Action::VerticalSplit);

    let doc_id = editor
        .open(
            &Path::new("Cargo.toml"),
            helix_view::editor::Action::VerticalSplit,
        )
        .unwrap();
    let view_id = editor.tree.focus;
    let doc = doc_mut!(editor, &doc_id);
    let pos = Selection::point(pos_at_coords(
        doc.text().slice(..),
        Position::new(0, 0),
        true,
    ));
    doc.set_selection(view_id, pos);
    doc.versioned_identifier()


    "this"
}


