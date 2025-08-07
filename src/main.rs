use cairo::Context;
use glib::ControlFlow;
use gtk4::gdk::Display;
use gtk4::gdk_pixbuf::PixbufLoader;
use gtk4::prelude::WidgetExtManual;
use gtk4::prelude::*;
use gtk4::{
    Align, Box as GtkBox, DrawingArea, EventControllerMotion, GestureClick, GestureDrag, Image,
    Label, Orientation, Overlay,
};
use gtk4::{CssProvider, StyleContext};
use libadwaita::prelude::*;
use libadwaita::{Application as AdwApplication, ApplicationWindow, HeaderBar};
use std::cell::RefCell;
use std::rc::Rc;

const ICON_DATA: &[u8] = include_bytes!("../icon.png");

// アイコンをバイナリに含める関数
// fn load_embedded_logo() -> Image {
//     let loader = PixbufLoader::new();
//     loader.write(ICON_DATA).unwrap();
//     loader.close().unwrap();

//     let pixbuf = loader.pixbuf().unwrap();
//     let image = Image::from_pixbuf(Some(&pixbuf));
//     image.set_pixel_size(24);
//     image
// }

// Apply css to label
fn apply_label_hover(label: &Label, hover_class: &str) {
    let motion = EventControllerMotion::new();

    let label_enter = label.clone();
    let hover_class_enter = hover_class.to_string(); // clone here
    motion.connect_enter(move |_, _, _| {
        label_enter.add_css_class(&hover_class_enter);
    });

    let label_leave = label.clone();
    let hover_class_leave = hover_class.to_string(); // clone again
    motion.connect_leave(move |_| {
        label_leave.remove_css_class(&hover_class_leave);
    });

    label.add_controller(motion);
}

// Function for menubar
fn draw_rounded_rectangle(cr: &Context, x: f64, y: f64, w: f64, h: f64, r: f64) {
    // 左上角
    cr.new_sub_path();
    cr.arc(
        x + r,
        y + r,
        r,
        std::f64::consts::PI,
        3.0 * std::f64::consts::PI / 2.0,
    );

    // 上辺
    cr.line_to(x + w - r, y);
    // 右上角
    cr.arc(x + w - r, y + r, r, 3.0 * std::f64::consts::PI / 2.0, 0.0);

    // 右辺
    cr.line_to(x + w, y + h - r);
    // 右下角
    cr.arc(x + w - r, y + h - r, r, 0.0, std::f64::consts::PI / 2.0);

    // 下辺
    cr.line_to(x + r, y + h);
    // 左下角
    cr.arc(
        x + r,
        y + h - r,
        r,
        std::f64::consts::PI / 2.0,
        std::f64::consts::PI,
    );
    cr.close_path();
}

// Apply css to the header
fn apply_hover_effects(label: &Label, is_clicked: Rc<RefCell<bool>>) {
    let label_hover = label.clone();
    let is_clicked_hover = is_clicked.clone();

    let motion_controller = EventControllerMotion::new();
    motion_controller.connect_enter(move |_, _, _| {
        if !*is_clicked_hover.borrow() {
            label_hover.add_css_class("header-label-hover");
        }
    });

    let label_leave = label.clone();
    let is_clicked_leave = is_clicked.clone();
    motion_controller.connect_leave(move |_| {
        if !*is_clicked_leave.borrow() {
            label_leave.remove_css_class("header-label-hover");
        }
    });

    label.add_controller(motion_controller);
}

fn main() {
    // UI
    let preview_height = 430.0;
    // let preview_height_int = preview_height as i32;
    let playhead_position = Rc::new(RefCell::new((200.0, 150.0)));

    let mouse_position = Rc::new(RefCell::new((0.0, 0.0)));
    let show_rect = Rc::new(RefCell::new(false)); // ← フラグを作る

    // Build Adwaita Application
    let app = AdwApplication::builder()
        .application_id("com.Luvita.app")
        .build();

    app.connect_activate(move |app| {
        // CSS
        let css = "
            headerbar, .titlebar {
                min-height: 34px;
                /* padding-top: 0;
                padding-bottom: 0;
                 background-color:rgb(0, 110, 255); */
            }

            headerbar * {
                margin-top: 0;
                margin-bottom: 0;
                /* color: white;  ボタンや文字が見えるように */
            }

            .header-label {
                background-color: transparent;
                margin: 3px -3px 3px -3px;
                padding: 0 8px 0 8px;
                font-size: 13px;
            }

            .header-label-hover {
                background-color: rgba(151, 151, 151, 0.24); /* 半透明な青 */
                border-radius: 8px;
            }

            .header-label-clicked {
                background-color: rgba(0, 122, 204, 1.0); /* 濃い青 */
                color: white;
            }

            .menu-button {
                margin: 10px 0 0 75px;
                font-size: 13px;
                padding: 4px 10px 4px 10px;
                width: 100px;
            }

            .menu-button-hover {
                background-color: rgba(100, 100, 230, 1.0);
                border-radius: 6px;
                width: 100px;              /* ← 幅を揃える */
                display: inline-block;     /* ← ラベルに幅を持たせる */
            }

            .menu-button1 {
            margin: 10px 0 0 75px;
            font-size: 13px;
            padding: 4px 10px;
            min-width: 100px;
            max-width: 100px;
            }

            .menu-button-hover1 {
                background-color: rgba(100, 100, 230, 1.0);
                border-radius: 6px;
                width: 100px;              /* ← 幅を揃える */
                display: inline-block;     /* ← ラベルに幅を持たせる */
            }

            .transparent-button {
                background-color: transparent;
                border: none;
                box-shadow: none;
                font-size: 13px;
                font-weight: 400; /* 細い文字 */
                padding: 4px 1px0 4px 0;
            }

            .transparent-button:hover {
                background-color: rgba(100, 100, 230, 1.0); /* ホバー時だけ少し青く */
                border-radius: 6px;
            }
        ";

        let provider = CssProvider::new();
        provider.load_from_data(css);

        #[allow(deprecated)]
        StyleContext::add_provider_for_display(
            &Display::default().expect("Display not found"),
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        // Make Adwaita Window
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Luvita")
            .default_width(800)
            .default_height(600)
            .build();

        // Get window size
        let last_size = Rc::new(RefCell::new((0, 0)));
        let last_size_clone = last_size.clone();
        window.add_tick_callback(move |win, _| {
            let alloc = win.allocation();
            let (w, h) = (alloc.width(), alloc.height());
            let mut last = last_size_clone.borrow_mut();
            if w != last.0 || h != last.1 {
                //println!("Window resized: width = {}, height = {}", w, h);
                *last = (w, h);
            }
            ControlFlow::Continue
        });

        // DrawingArea
        let draw_area = DrawingArea::builder()
            .content_width(1133)
            .content_height(700)
            .build();

        {
            let playhead = playhead_position.clone();
            let mouse_position_clone = mouse_position.clone(); // ★追加
            let show_rect_clone = show_rect.clone(); // ← clone して中で使えるように

            draw_area.set_draw_func(move |drawing_area, cr, width, height| {
                //UI
                let separator_line_x = 800.0;

                // 🎯 [追加] マウスが (0,0)-(50,50) にあるときに赤い四角を表示
                let (mx, my) = *mouse_position_clone.borrow();
                if mx >= 0.0 && mx <= 50.0 && my >= 0.0 && my <= 50.0 {
                    cr.set_source_rgb(1.0, 0.0, 0.0); // 赤色
                    cr.rectangle(0.0, 0.0, 50.0, 50.0); // 四角形
                    cr.fill().unwrap();
                }

                // ✅ フラグが true のときだけ四角を描画
                if *show_rect_clone.borrow() {
                    cr.set_source_rgb(1.0, 0.0, 0.0); // 赤
                    cr.rectangle(0.0, 0.0, 50.0, 50.0);
                    cr.fill().unwrap();
                }

                // Draw background
                //cr.set_source_rgba(0.1, 0.1, 0.1, 0.0);
                //cr.paint().unwrap();

                // Draw layer
                let layer_height = 30.0;
                let label_area_width = 40.0; // 左のラベル描画幅
                cr.set_source_rgb(0.3, 0.3, 0.3);
                cr.set_line_width(1.0);

                let num_layers = (height as f64 / layer_height).ceil() as i32;
                let top_offset = &preview_height;
                for i in 0..num_layers {
                    let y = i as f64 * layer_height + top_offset;

                    // 横線
                    cr.move_to(0.0, y);
                    cr.line_to(width as f64, y);
                    cr.stroke().unwrap();

                    // レイヤー番号テキストを左に描画
                    let label = format!(" {}", i + 1);
                    cr.set_font_size(14.0);
                    cr.move_to(5.0, y + 14.0); // 5px右、14px下に調整（フォントサイズ考慮）
                    cr.set_source_rgb(0.8, 0.8, 0.8);
                    // cr.set_source_rgb(1.0, 1.0, 1.0);
                    cr.show_text(&label).unwrap();

                    // ラベルの右に縦線（縦棒）
                    cr.set_source_rgb(0.3, 0.3, 0.3);
                    cr.move_to(label_area_width, y);
                    cr.line_to(label_area_width, y + layer_height);
                    cr.stroke().unwrap();
                }

                // Get mouse position
                let (x, _y) = *playhead.borrow();

                // Draw playhead
                cr.set_source_rgb(1.0, 0.8, 0.2);
                cr.set_line_width(1.0);
                cr.move_to(x, preview_height);
                cr.line_to(x, preview_height + 300.0);
                let _ = cr.stroke();

                // マウスの動きに反応
                let motion = EventControllerMotion::new();
                let playhead_position_for_motion = playhead.clone();
                //let timeline_area_for_motion = drawing_area.clone();

                motion.connect_motion(move |_, x, y| {
                    *playhead_position_for_motion.borrow_mut() = (x, y); // マウスが動いているときの座標
                    // println!("{}, {}", x, y);
                    //timeline_area_for_motion.queue_draw();
                });

                drawing_area.add_controller(motion);
                //println!("{}, {}", playhead.borrow().0, playhead.borrow().1);

                // Menubar
                cr.set_source_rgb(0.2, 0.2, 0.2);
                let offset_x = 40.0;
                let offset_y = 1.0;
                let width = 270.0;
                let height = 123.0;
                let radius = 8.0;
                draw_rounded_rectangle(cr, offset_x, offset_y, width, height, radius);
                cr.fill().unwrap();

                // Drraw separator line
                cr.set_source_rgb(0.5, 0.5, 0.5);
                cr.set_line_width(1.0);
                cr.move_to(separator_line_x, 0.0);
                cr.line_to(separator_line_x, preview_height);
                cr.stroke().unwrap();
            });
        }
        // 🎯 ③マウスの動きを追跡する EventControllerMotion を設定
        let motion = EventControllerMotion::new();
        let mouse_position_for_motion = mouse_position.clone(); // Rc なので clone で共有
        let draw_area_clone = draw_area.clone(); // DrawingArea を再描画するための clone

        motion.connect_motion(move |_, x, y| {
            *mouse_position_for_motion.borrow_mut() = (x, y); // 座標を更新
            draw_area_clone.queue_draw(); // 描画エリアを再描画（四角が表示される）
        });

        // コントローラーを DrawingArea に追加
        draw_area.add_controller(motion);

        // Follow playhead while right-clicking
        let drag = GestureDrag::new();
        let drag_offset = Rc::new(RefCell::new((0.0, 0.0)));
        let playhead_position_for_begin = playhead_position.clone();
        let playhead_position_for_end = playhead_position.clone();
        let drag_offset_for_begin = drag_offset.clone();
        let drag_offset_for_end = drag_offset.clone();

        drag.connect_drag_begin(move |_, start_x, start_y| {
            let (cx, cy) = *playhead_position_for_begin.borrow();
            drag_offset_for_begin.borrow_mut().0 = start_x - cx;
            drag_offset_for_begin.borrow_mut().1 = start_y - cy;
        });

        drag.connect_drag_end(move |_, end_x, end_y| {
            let (cx, cy) = *playhead_position_for_end.borrow();
            drag_offset_for_end.borrow_mut().0 = end_x - cx;
            drag_offset_for_end.borrow_mut().1 = end_y - cy;
            println!("endpoint: {}", end_x) //end_xは移動距離だった
        });

        let playhead_position_for_update = playhead_position.clone();
        let drag_offset_for_update = drag_offset.clone();
        let drawing_area_for_update = draw_area.clone();

        drag.connect_drag_update(move |_, offset_x, offset_y| {
            let dx = offset_x - drag_offset_for_update.borrow().0;
            let dy = offset_y - drag_offset_for_update.borrow().1;
            *playhead_position_for_update.borrow_mut() = (dx, dy);
            drawing_area_for_update.queue_draw();
            // println!(
            //     "playhead_position: {}",
            //     playhead_position_for_update.borrow().0
            // ) //end_xは移動距離だった
        });
        draw_area.add_controller(drag);

        // Detect right click
        let click = GestureClick::builder().button(3).build(); //left:1 center:2 right:3
        click.connect_pressed(move |_, n_press, x, y| {
            println!("右クリック検出: ({}, {}) クリック回数: {}", x, y, n_press);
        });
        click.connect_released(move |_, n_press, x, y| {
            println!(
                "右クリックが離された！ 座標: ({}, {}), クリック回数: {}",
                x, y, n_press
            );
        });
        draw_area.add_controller(click);

        // Header
        let header_box = GtkBox::new(Orientation::Horizontal, 6);
        header_box.set_halign(Align::Start);

        let file_label = Label::new(Some("ファイル"));
        //file_label.set_margin_start(12); // left space
        file_label.set_focusable(true);
        file_label.add_css_class("header-label");

        let filter_label = Label::new(Some("フィルタ"));
        filter_label.set_focusable(true);
        filter_label.add_css_class("header-label");

        let setting_label = Label::new(Some("設定"));
        setting_label.set_focusable(true);
        setting_label.add_css_class("header-label");

        let edit_label = Label::new(Some("編集"));
        edit_label.set_focusable(true);
        edit_label.add_css_class("header-label");

        let profile_label = Label::new(Some("プロファイル"));
        profile_label.set_focusable(true);
        profile_label.add_css_class("header-label");

        let show_label = Label::new(Some("表示"));
        show_label.set_focusable(true);
        show_label.add_css_class("header-label");

        let other_label = Label::new(Some("その他"));
        other_label.set_focusable(true);
        other_label.add_css_class("header-label");

        let is_clicked = Rc::new(RefCell::new(false));

        // Detect mouseover
        let motion_controller = EventControllerMotion::new();
        let header_label_for_hover = file_label.clone();

        let is_clicked_clone = is_clicked.clone();
        motion_controller.connect_enter(move |_, _x, _y| {
            if !*is_clicked_clone.borrow() {
                header_label_for_hover.add_css_class("header-label-hover");
            }
        });
        let file_label_for_leave = file_label.clone();
        let is_clicked_for_leave = is_clicked.clone();

        motion_controller.connect_leave(move |_| {
            if !*is_clicked_for_leave.borrow() {
                file_label_for_leave.remove_css_class("header-label-hover");
            }
        });

        file_label.add_controller(motion_controller);

        // When header label clicked
        let gesture = GestureClick::builder().button(1).build();
        let file_label_for_click = file_label.clone();
        let is_clicked_for_click = is_clicked.clone();

        let show_rect_click = show_rect.clone(); ////
        let draw_area_for_click = draw_area.clone(); ////

        gesture.connect_pressed(move |_, _, _, _| {
            let mut clicked = is_clicked_for_click.borrow_mut();
            *clicked = !*clicked;

            if *clicked {
                file_label_for_click.remove_css_class("header-label-hover");
                file_label_for_click.add_css_class("header-label-clicked");
            } else {
                file_label_for_click.remove_css_class("header-label-clicked");
            }

            *show_rect_click.borrow_mut() = true; // ← 表示フラグON
            draw_area_for_click.queue_draw(); // ← 再描画
        });

        file_label.add_controller(gesture);
        //header_box.append(&file_label);

        // Detect clicks elsewhere
        let file_label_for_outside = file_label.clone();
        let is_clicked_for_outside = is_clicked.clone();

        // 既存の変数をクローンして使う
        let show_rect_for_outside = show_rect.clone();
        let draw_area_for_outside = draw_area.clone();

        let global_click = GestureClick::builder().button(1).build();
        global_click.connect_pressed(move |_, _, _, _| {
            if *is_clicked_for_outside.borrow() {
                file_label_for_outside.remove_css_class("header-label-clicked");
                *is_clicked_for_outside.borrow_mut() = false;
            }

            *show_rect_for_outside.borrow_mut() = false; ////
            draw_area_for_outside.queue_draw(); ////
        });
        window.add_controller(global_click);

        apply_hover_effects(&file_label, is_clicked.clone());
        apply_hover_effects(&filter_label, Rc::new(RefCell::new(false)));
        apply_hover_effects(&setting_label, Rc::new(RefCell::new(false)));
        apply_hover_effects(&edit_label, Rc::new(RefCell::new(false)));
        apply_hover_effects(&profile_label, Rc::new(RefCell::new(false)));
        apply_hover_effects(&show_label, Rc::new(RefCell::new(false)));
        apply_hover_effects(&other_label, Rc::new(RefCell::new(false)));

        // Icon
        let icon = Image::from_file("bitmap.png");
        icon.set_pixel_size(26);
        icon.set_margin_start(4);
        icon.set_margin_end(4);

        let header = HeaderBar::builder()
            .title_widget(&header_box)
            .show_end_title_buttons(true)
            .build();

        // Include icon in binary
        let loader = PixbufLoader::new();
        loader.write(ICON_DATA).unwrap();
        loader.close().unwrap();
        let pixbuf = loader.pixbuf().unwrap();
        let logo = Image::from_pixbuf(Some(&pixbuf));
        logo.set_pixel_size(24);

        // Set menu button in header
        header.pack_start(&icon);
        header.pack_start(&file_label);
        header.pack_start(&filter_label);
        header.pack_start(&setting_label);
        header.pack_start(&edit_label);
        header.pack_start(&profile_label);
        header.pack_start(&show_label);
        header.pack_start(&other_label);

        // menu button
        let overlay = Overlay::new();

        let label = Label::new(Some("開く"));
        label.set_halign(gtk4::Align::Start);
        label.set_valign(gtk4::Align::Start);
        label.set_css_classes(&["menu-button"]);

        let label1 = Label::new(Some("閉じる"));
        label1.set_halign(gtk4::Align::Start);
        label1.set_valign(gtk4::Align::Start);
        label1.set_margin_top(28);
        label1.set_css_classes(&["menu-button"]);

        let label2 = Label::new(Some("プロジェクトを開く"));
        label2.set_halign(gtk4::Align::Start);
        label2.set_valign(gtk4::Align::Start);
        label2.set_margin_top(28 * 2);
        label2.set_css_classes(&["menu-button"]);

        let label3 = Label::new(Some("終了"));
        label3.set_halign(gtk4::Align::Start);
        label3.set_valign(gtk4::Align::Start);
        label3.set_margin_top(28 * 3);
        label3.set_css_classes(&["menu-button"]);

        apply_label_hover(&label, "menu-button-hover");
        apply_label_hover(&label1, "menu-button-hover");
        apply_label_hover(&label2, "menu-button-hover");
        apply_label_hover(&label3, "menu-button-hover");

        // Add button to overlay
        overlay.set_child(Some(&draw_area));
        overlay.add_overlay(&label);
        overlay.add_overlay(&label1);
        overlay.add_overlay(&label2);
        overlay.add_overlay(&label3);

        // Assemble the UI
        let vbox = GtkBox::new(Orientation::Vertical, 0);
        vbox.append(&header);
        vbox.append(&draw_area);
        vbox.append(&overlay);
        window.set_content(Some(&vbox));
        window.present();
    });
    app.run();
}
